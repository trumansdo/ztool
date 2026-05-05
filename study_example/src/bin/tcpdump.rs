//! # tcpdump — 网络数据包捕获与解析工具
//!
//! 使用 pnet 库实现类 tcpdump 的网络抓包功能：抓取数据链路层数据包，
//! 逐层解析以太网 → IPv4/IPv6 → TCP/UDP，并以十六进制+ASCII 格式打印负载。
//!
//! ## Rust 概念 — trait 多类型实现（多态）
//! `GettableEndPoints` trait 被 Ipv4Packet/Ipv6Packet/TcpPacket/UdpPacket 四个类型实现。
//! 通过 `&dyn GettableEndPoints` trait 对象实现动态分发，统一处理不同协议层。
//! 这是 Rust 中实现「接口多态」的主要方式（相对于泛型的静态分发）。
//!
//! ## Rust 概念 — `dyn Trait` trait 对象
//! `print_packet_info(l3: &dyn GettableEndPoints, l4: &dyn GettableEndPoints, ...)`
//! - `dyn` 关键字表示使用动态分发（运行时虚函数表查找）
//! - 相比泛型 `<T: Trait>`，性能略低但代码更简洁
//! - 可以接受任意实现了该 trait 的类型
//!
//! ## Rust 概念 — 内联子模块 `mod packets { }`
//! 在同一个源文件中使用花括号定义内联模块，无需单独文件。
//! 适合小型工具模块的定义。
//!
//! ## Rust 概念 — `const WIDTH` 编译期常量
//! `const WIDTH: usize = 20;` 定义编译期已知的常量。
//! 与 `let` 不同，const 会内联到使用位置（无运行时内存分配）。

use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

use packets::GettableEndPoints;

/// 十六进制转储的每行宽度（20 字节 = 32 字符十六进制 + ASCII 部分）
const WIDTH: usize = 20;

fn main() {
    // datalink::interfaces() — 获取所有网络接口列表
    let interfaces = datalink::interfaces();
    // .into_iter().nth(0) — 转换为所有权迭代器，取第一个元素
    // .unwrap_or_else(|| panic!(...)) — 提供闭包作为错误处理
    let interface = interfaces
        .into_iter()
        .nth(0)
        .unwrap_or_else(|| panic!("No such network interface:"));

    // datalink::channel() — 打开网络接口通道
    // Channel::Ethernet(tx, rx) — 模式匹配解构以太网通道
    // _tx: 发送端，用 _ 前缀表示不需要
    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Err(e) => {
            panic!("Failed to create datalink channel {}", e)
        }
        _ => panic!("Failed to create datalink channel"),
    };

    loop {
        match rx.next() {  // 阻塞等待下一个数据包
            Ok(frame) => {
                // EthernetPacket::new(frame) — 从原始字节构造以太网帧
                let frame = EthernetPacket::new(frame).unwrap();
                match frame.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        ipv4_handler(&frame);
                    }
                    EtherTypes::Ipv6 => {
                        ipv6_handler(&frame);
                    }
                    _ => {
                        println!("Not a ipv4 or ipv6");
                    }
                }
            }
            Err(e) => {
                panic!("Failed to read: {}", e);
            }
        }
    }
}

/// 打印数据包信息的十六进制+ASCII转储
///
/// ## Rust 概念 — `&dyn GettableEndPoints` vs 泛型
/// 使用 trait 对象而非泛型 `<T: GettableEndPoints>`，因为参数可以是不相关的不同类型。
/// 泛型要求同一函数签名的所有使用指向单一类型；
/// trait 对象数组允许不同具体类型同时使用。
///
/// ## Rust 概念 — 十六进制+ASCII转储算法
/// 类似 Wireshark 的十六进制显示：每行 WIDTH 个字节，
/// 左侧显示十六进制值，右侧显示对应的 ASCII 字符（不可打印字符显示为 `.`）。
/// `payload[i] as char` — 将 u8 转换为 char（仅对 ASCII 有效）。
fn print_packet_info(l3: &dyn GettableEndPoints, l4: &dyn GettableEndPoints, proto: &str) {
    println!(
        "Captured a {} packet from {}|{} to {}|{}\n",
        proto,
        l3.get_source(),
        l4.get_source(),
        l3.get_destination(),
        l4.get_destination()
    );
    let payload = l4.get_payload();
    let len = payload.len();

    // 十六进制部分
    for i in 0..len {
        // {:<02X} — 左对齐，2位，十六进制大写，0填充
        print!("{:<02X} ", payload[i]);
        // 每 WIDTH 个字节或最后一行换行
        if i % WIDTH == WIDTH - 1 || i == len - 1 {
            // 补齐空格（最后一行可能不满 WIDTH）
            for _j in 0..WIDTH - 1 - (i % (WIDTH)) {
                print!("   ");
            }
            // ASCII 部分：分隔符
            print!("| ");
            // 打印对应的 ASCII 字符
            for j in i - i % WIDTH..i + 1 {
                if payload[j].is_ascii_alphabetic() {
                    print!("{}", payload[j] as char);
                } else {
                    print!(".");  // 不可打印字符显示为点
                }
            }
            print!("\n");
        }
    }
    // "=".repeat(WIDTH * 3) — 字符串重复，打印分隔线
    println!("{}", "=".repeat(WIDTH * 3));
    print!("\n");
}

fn ipv4_handler(ethernet: &EthernetPacket) {
    // if let 解构 Option：仅当 Ipv4Packet 解析成功时执行
    if let Some(packet) = Ipv4Packet::new(ethernet.payload()) {
        match packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet);
            }
            IpNextHeaderProtocols::Udp => {
                udp_handler(&packet);
            }
            _ => {
                println!("Not a tcp or a udp packet");
            }
        }
    }
}

fn ipv6_handler(ethernet: &EthernetPacket) {
    if let Some(packet) = Ipv6Packet::new(ethernet.payload()) {
        // IPv6 使用 get_next_header() 而非 get_next_level_protocol()
        match packet.get_next_header() {
            IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet);
            }
            IpNextHeaderProtocols::Udp => {
                udp_handler(&packet);
            }
            _ => {
                println!("Not a tcp or a udp packet");
            }
        }
    }
}

/// TCP 处理器：通过 trait 对象传递，兼容 IPv4 和 IPv6
///
/// ## Rust 概念 — trait 对象向下传递
/// `packet: &dyn GettableEndPoints` 可以是 Ipv4Packet 也可以是 Ipv6Packet。
/// 调用 `packet.get_payload()` 时通过虚表查找到对应具体类型的实现。
fn tcp_handler(packet: &dyn GettableEndPoints) {
    let tcp = TcpPacket::new(packet.get_payload());
    if let Some(tcp) = tcp {
        print_packet_info(packet, &tcp, "TCP");
    }
}

fn udp_handler(packet: &dyn GettableEndPoints) {
    let udp = UdpPacket::new(packet.get_payload());
    if let Some(udp) = udp {
        print_packet_info(packet, &udp, "UDP");
    }
}

/// 内联子模块 — 在同一个文件中定义模块
///
/// ## Rust 概念 — 模块内的模块
/// `mod packets { ... }` 在主文件内创建私有子模块。
/// 可以用 `use packets::GettableEndPoints;` 在上层引用。
///
/// ## Rust 概念 — trait 定义
/// `pub trait GettableEndPoints` 定义一个公共接口：
/// - `get_source()` → 源端点字符串
/// - `get_destination()` → 目标端点字符串
/// - `get_payload()` → 负载字节切片
///
/// ## Rust 概念 — trait 的多类型实现
/// 同一个 trait 为不同协议包类型各实现一次。
/// 每种协议包返回的信息格式不同，但都满足相同的接口契约。
mod packets {
    use pnet::packet::ipv4::Ipv4Packet;
    use pnet::packet::ipv6::Ipv6Packet;
    use pnet::packet::tcp::TcpPacket;
    use pnet::packet::udp::UdpPacket;
    use pnet::packet::Packet;

    pub trait GettableEndPoints {
        fn get_source(&self) -> String;
        fn get_destination(&self) -> String;
        fn get_payload(&self) -> &[u8];
    }

    impl<'a> GettableEndPoints for Ipv4Packet<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }

    impl<'a> GettableEndPoints for Ipv6Packet<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }

    impl<'a> GettableEndPoints for TcpPacket<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }

    impl<'a> GettableEndPoints for UdpPacket<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }
}
