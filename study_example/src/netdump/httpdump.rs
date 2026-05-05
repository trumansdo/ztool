//! # HTTP 流量抓包与 OSI 四层解析
//!
//! 逐层解析以太网 → IPv4 → TCP → HTTP，提取完整的 HTTP 头部信息。
//! 演示了 OSI 模型在实际代码中的体现。
//!
//! ## OSI 七层模型与 TCP/IP 四层模型
//! ```
//! OSI 七层            TCP/IP 四层         pnet 实现
//! ─────────          ──────────         ─────────
//! 应用层(HTTP...) ─┐
//! 表示层(SSL...)   ├── 应用层 ────────  手动解析 HTTP
//! 会话层(RPC...)  ─┘
//! 传输层(TCP/UDP) ─── 传输层 ────────  TcpPacket / UdpPacket
//! 网络层(IP...)   ─── 网络层 ────────  Ipv4Packet / Ipv6Packet
//! 链路层(Ethernet) ─ 网络接口层 ──────  EthernetPacket
//! 物理层                (硬件)
//! ```
//!
//! ## Rust 概念 — 生命周期标注 `<'a>`
//! `TcpHolisticPacket<'a>` 持有对 pnet 解析结果的引用，
//! 需要标注生命周期 'a 表示所有引用必须与原始数据包一样长。
//!
//! ## Rust 概念 — `Display` trait 手动实现
//! 为 `TcpHolisticPacket` 实现 `Display`，可以用在 `println!("{}", pkt)` 等格式宏中。
//! 也可以直接用 `{}` 格式化而不需要 `{:?}`。
//!
//! ## Rust 概念 — `trait` 为外部类型实现
//! `impl<'a> PacketSummary for pcap::Packet<'a>` — 为第三方 crate 的类型实现自定义 trait。
//! Rust 允许在定义 trait 的 crate 中为任意类型实现（孤儿规则）。
//!
//! ## Rust 概念 — itertools `tuple_windows`
//! `tuple_windows::<(_, _, _, _)>()` 返回连续 4 个元素的滑动窗口。
//! 用于查找 HTTP 头/体分隔标记 `\r\n\r\n`（4 个字节）。

use std::{error::Error, fmt::Display, io::Cursor, sync::mpsc, thread};

use byteorder::{BigEndian, ReadBytesExt};
use itertools::Itertools;
use pcap::{Capture, Device};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::{Ipv4Flags, Ipv4Packet},
    tcp::{TcpFlags, TcpOptionNumbers, TcpPacket}, Packet,
};

/// 四层数据包的全景视图
///
/// 保存以太网、IPv4、TCP 各层解析结果的引用。
///
/// ## Rust 概念 — 生命周期 `'a`
/// 此结构体的所有字段都是对原始数据包数据的借用引用。
/// `'a` 生命周期参数确保这些引用不会比原始数据活得更久（防止悬垂指针）。
/// 编译器在编译时检查生命周期约束。
#[derive(Debug, PartialEq)]
pub struct TcpHolisticPacket<'a> {
    identification: u16,
    frame_packet: &'a pcap::Packet<'a>,
    eth_packet: &'a EthernetPacket<'a>,
    ipv4_packet: &'a Ipv4Packet<'a>,
    tcp_packet: &'a TcpPacket<'a>,
}

/// 数据包摘要 trait — 为各层实现统一的信息格式化接口
///
/// ## Rust 概念 — trait 定义
/// `pub trait PacketSummary` 声明了一个公共接口，只要实现了 `summary()` 方法，
/// 任何类型都可以被格式化为摘要字符串。
/// 下面是四种实现：pcap::Packet（帧层）、EthernetPacket（链路层）、
/// Ipv4Packet（网络层）、TcpPacket（传输层）。
///
/// ## Rust 概念 — 孤儿规则的应用
/// 虽然 `pcap::Packet` 是外部 crate 的类型，但因为 `PacketSummary` 定义在本 crate 中，
/// 所以可以合法地为它实现 trait。
pub trait PacketSummary {
    fn summary(&self) -> String;
}

/// 为 pcap 的帧层实现摘要
///
/// ## Rust 概念 — `as` 类型转换
/// `self.header.ts.tv_sec as u64` — 将 i64/long 安全转换为 u64。
/// `as` 是 Rust 的强制转换关键字，不同于 C 的括号转换。
impl<'a> PacketSummary for pcap::Packet<'a> {
    fn summary(&self) -> String {
        let sec = self.header.ts.tv_sec as u64;
        let usec = self.header.ts.tv_usec as u64;
        format!("Frame[Time:{} Len:{} CapLen:{}]", (sec * 1000000 + usec), self.header.len, self.header.caplen)
    }
}

/// 为以太网层实现摘要
impl<'a> PacketSummary for EthernetPacket<'a> {
    fn summary(&self) -> String {
        format!(
            "Ethernet[EtherType:{} Src:{}  Dst:{} PacketLen:{} PayloadLen:{}]",
            self.get_ethertype(),
            self.get_source(),
            self.get_destination(),
            self.packet().len(),
            self.payload().len()
        )
    }
}

/// 为 IPv4 层实现摘要
///
/// ## Rust 概念 — 位运算提取标志位
/// `((Ipv4Flags::DontFragment & ipv4_packet_flags) >> 1)` 用位掩码和右移提取单个标志位。
/// Ipv4Flags 是位标志位，用 `&` 掩码取出特定位，`>> n` 右移得到 0 或 1。
impl<'a> PacketSummary for Ipv4Packet<'a> {
    fn summary(&self) -> String {
        let ipv4_packet_flags = self.get_flags();
        format!(
            "NetWork[IPv4 Src:{} Dst:{} HeaderLen:{} Ident:{} Flags:[{} DF:{} MF:{}] ...]",
            self.get_source(),
            self.get_destination(),
            self.get_header_length(),
            self.get_identification(),
            0,
            ((Ipv4Flags::DontFragment & ipv4_packet_flags) >> 1),   // DF 位
            (Ipv4Flags::MoreFragments & ipv4_packet_flags),          // MF 位
        )
    }
}

/// 为 TCP 层实现摘要
///
/// ## Rust 概念 — 迭代器方法链 + Cursor
/// TCP 选项解析：`.into_iter().map(|x| {...})` 遍历 TCP 选项，
/// 用 `Cursor` 从字节数据中读取大端整数。
///
/// `Cursor::new(x.data)` — 为字节切片创建可读游标
/// `read_uint::<BigEndian>(len)` — 以大端字节序读取无符号整数
/// `TcpOptionNumbers::MSS` 等枚举值匹配 — 识别 TCP 选项类型
impl<'a> PacketSummary for TcpPacket<'a> {
    fn summary(&self) -> String {
        let tcp_packet_flags = self.get_flags();
        format!(
            "Transport[TCP SrcPort:{} DstPort:{} Seq:{} Ack:{} Flags[...SYN:{} FIN:{}] WindowSize:{} ...]",
            self.get_source(),
            self.get_destination(),
            self.get_sequence(),
            self.get_acknowledgement(),
            ((TcpFlags::SYN & tcp_packet_flags) >> 1),
            (TcpFlags::FIN & tcp_packet_flags),
            self.get_window(),
        )
    }
}

/// 为 TcpHolisticPacket 实现 Display trait
///
/// ## Rust 概念 — `Display` trait
/// Display 用于面向用户的格式化（{} 格式），Debug 用于调试（{:?} 格式）。
/// `write!` / `writeln!` 宏将格式化字符串写入 Formatter。
///
/// ## Rust 概念 — `let _ = writeln!(...)`
/// 忽略写入结果（总是成功因为写入字符串不会失败）。
/// `_` 前缀静默抑制未使用结果的编译警告。
impl<'a> Display for TcpHolisticPacket<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "FrameLayer -> {}", self.frame_packet.summary());
        let _ = writeln!(f, "EthernetLayer -> {}", self.eth_packet.summary());
        let _ = writeln!(f, "NetWorkLayer -> {}", self.ipv4_packet.summary());
        let _ = writeln!(f, "TransportLayer -> {}", self.tcp_packet.summary());
        writeln!(f, "-----------------------------------------------------")
    }
}

/// 处理单个数据包：从以太网帧开始逐层解析
///
/// ## Rust 概念 — 链式迭代器查找 HTTP 头结束标记
/// ```
/// tcp_payload.into_iter()
///     .tuple_windows::<(_, _, _, _)>()     // 创建 4 字节滑动窗口
///     .position(|x| x == (&13u8, &10u8, &13u8, &10u8))  // 查找 \r\n\r\n
///     .map(|i| &tcp_payload[0..i])         // 提取 HTTP 头部
/// ```
/// - `tuple_windows` 是 itertools 提供的组合器（非标准库）
/// - `13u8` 是 CR (Carriage Return), `10u8` 是 LF (Line Feed)
/// - `position()` 返回 Option<usize>，找到则 Some(位置)
/// - `&[0..i]` 切片语法提取前 i 个字节
///
/// ## Rust 概念 — `ok_or()` 错误转换
/// `Ipv4Packet::new(...).ok_or("msg")?` — 将 Option 转为 Result，
/// None 时返回自定义错误信息。
fn handle_packet(frame_packet: PacketOwned) -> Result<(), Box<dyn Error>> {
    let fr_pck = pcap::Packet {data: &frame_packet.data,header: &frame_packet.header};

    let eth_packet = EthernetPacket::new(fr_pck.data).unwrap();

    match eth_packet.get_ethertype() {
        EtherTypes::Ipv4 => {
            let ipv4_packet = Ipv4Packet::new(eth_packet.payload()).ok_or("Ipv4Packet::new failed")?;
            match ipv4_packet.get_next_level_protocol() {
                IpNextHeaderProtocols::Tcp => {
                    let tcp_packet = TcpPacket::new(ipv4_packet.payload()).ok_or("TcpPacket::new failed")?;
                    let tcp_payload = tcp_packet.payload();
                    // 在 TCP 负载中查找 HTTP 头部（\r\n\r\n 结尾标志）
                    let http_header = tcp_payload
                        .into_iter()
                        .tuple_windows::<(_, _, _, _)>()
                        .position(|x| x == (&13u8, &10u8, &13u8, &10u8))
                        .map(|i| &tcp_payload[0..i])
                        .map(|x| String::from_utf8(x.to_vec()))
                        .ok_or("no CRLF found");
                    // 打印四层信息
                    println!(
                        "{}",
                        TcpHolisticPacket {
                            identification: ipv4_packet.get_identification(),
                            frame_packet: &fr_pck,
                            eth_packet: &eth_packet,
                            ipv4_packet: &ipv4_packet,
                            tcp_packet: &tcp_packet,
                        }
                    );
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

/// pcap 数据包的所有权版本
#[derive(Debug, PartialEq, Eq)]
pub struct PacketOwned {
    pub header: pcap::PacketHeader,
    pub data: Box<[u8]>,
}

/// 启动 HTTP 抓包
///
/// ## Rust 概念 — mpsc 多生产者单消费者通道
/// `mpsc::channel()` 创建无界通道：
/// - `sender` → 发送端（可克隆为多个生产者）
/// - `receiver` → 接收端（只有一个消费者）
///
/// ## Rust 概念 — `thread::spawn(move || { ... })`
/// 创建系统线程。`move` 闭包将 receiver 所有权移入新线程。
/// 新线程独立运行，从通道接收数据包并处理。
///
/// ## Rust 概念 — 主线程与工作线程分离
/// 主线程负责抓包（阻塞在 cap.next_packet()），
/// 工作线程负责解析和打印（阻塞在 receiver.recv()）。
/// 这是一种经典的生产者-消费者模式。
///
/// ## Rust 概念 — `Box::from(slice)`
/// 将 &[u8] 切片复制到堆上，返回 Box<[u8]>（固定大小的堆数据）。
/// 与 Vec 不同，Box<[u8]> 不能动态调整大小但占用更少内存。
pub fn run() -> Result<(), Box<dyn Error>> {
    println!("start capture");
    let inter_name = "F0F07CFD-D6FE-49C9-8993-8790AFB777A2";

    let device = Device::list()?
        .into_iter()
        .find(|d| d.name.contains(inter_name))
        .unwrap();

    let mut cap = Capture::from_device(device)?
        .immediate_mode(true)   // 立即模式：尽快交付数据包
        .promisc(true)          // 混杂模式：捕获所有经过的数据包
        .open()?
        .setnonblock()?;        // 非阻塞模式
    cap.filter("ip host 211.149.224.47 and tcp", true)?;

    // 创建 mpsc 通道并启动工作线程
    let (sender, recevier) = mpsc::channel();
    thread::spawn(move || loop {
        match recevier.recv() {
            Ok(packet) => {
                let _ = handle_packet(packet);
            }
            Err(_) => {}  // 通道关闭时退出循环
        }
    });

    // 主线程循环：抓包并发送到工作线程
    loop {
        match cap.next_packet() {
            Ok(packet) => {
                // 将抓到的数据包拷贝到堆上（原始数据是临时借用的）
                let _ = sender.send(PacketOwned {
                    header: *packet.header,
                    data: Box::from(packet.data),
                });
            }
            Err(_) => {}
        }
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use std::time::Duration;
    use pnet::{datalink, packet::tcp::TcpOptionNumbers};

    #[test]
    fn it_works() {
        // 枚举所有网络接口并打印
        datalink::interfaces()
            .into_iter()
            .for_each(|d| println!("{:?}", d));
    }

    #[test]
    fn tcp_options() {
        println!("{:?}", TcpOptionNumbers::EOL);
        println!("{:?}", TcpOptionNumbers::NOP);
        println!("{:?}", TcpOptionNumbers::MSS);
        println!("{:?}", TcpOptionNumbers::WSCALE);
        println!("{:?}", TcpOptionNumbers::SACK_PERMITTED);
        println!("{:?}", TcpOptionNumbers::SACK);
        println!("{:?}", TcpOptionNumbers::TIMESTAMPS);
    }

    /// 异步测试 — tokio 多线程运行时
    ///
    /// ## Rust 概念 — `#[tokio::test]`
    /// tokio 提供的异步测试宏，支持在测试中使用 async/await。
    /// `flavor = "multi_thread"` — 使用多线程运行时
    /// `worker_threads = 10` — 10 个工作线程
    #[tokio::test(flavor = "multi_thread", worker_threads = 10)]
    async fn tokio_test() {
        use std::time::Instant;
        use tokio::time::{sleep, Duration};

        let start = Instant::now();
        println!("main thread id: {:?}", std::thread::current().id());
        // tokio::spawn 并发启动多个异步任务
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());

        println!("All tasks completed in {:?} {:?}", start.elapsed(), std::thread::current().id());
    }

    /// 异步测试辅助函数
    ///
    /// ## Rust 概念 — `.await` 暂停与恢复
    /// `sleep(Duration::from_secs(1)).await` — 异步等待 1 秒，不阻塞当前线程。
    /// 注意：末尾没有分号？实际上有分号。`.await` 是一个后缀操作。
    async fn async_test() {
        use std::time::Instant;
        use tokio::time::{sleep, Duration};
        println!("async_test started on thread {:?} at time {:?}", std::thread::current().id(), Instant::now());
        sleep(Duration::from_secs(1)).await;
        println!("async_test finished on thread {:?} at time {:?}", std::thread::current().id(), Instant::now());
    }
}
