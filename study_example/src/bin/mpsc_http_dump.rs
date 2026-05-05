//! # mpsc 通道演示 — 多线程 HTTP 抓包
//!
//! ## Rust 概念 — MPSC 通道 (多生产者，单消费者)
//! ```rust
//! let (sender, receiver) = mpsc::channel();
//! ```
//! - **mpsc** = Multiple Producer, Single Consumer
//! - `sender` — 发送端（可.clone() 给多个线程）
//! - `receiver` — 接收端（只有一个，不可 clone）
//! - 这是 Rust 标准库中**线程间通信的最基础方式**，类似 Go 的 channel。
//!
//! ## Rust 概念 — `thread::spawn` 线程创建
//! ```rust
//! thread::spawn(move || { ... });
//! ```
//! - `thread::spawn()` 创建新的操作系统线程
//! - `move ||` — move 闭包，捕获的变量所有权移入闭包（receiver 被移入新线程）
//! - 生成线程后主线程继续执行（非阻塞）
//!
//! ## Rust 概念 — `Box<[u8]>` vs `Vec<u8>`
//! `Box<[u8]>` 是不可变长度的堆分配字节切片，比 Vec 更紧凑（少了 capacity 字段）。
//! 当数据大小固定后不需要再增长时，用 Box<[u8]> 可节省 8 字节内存。
//!
//! ## Rust 概念 — pcap 非阻塞模式
//! ```rust
//! Capture::from_device(device)?
//!     .immediate_mode(true)    // 立即模式：包到达即刻返回
//!     .promisc(true)           // 混杂模式：捕获所有包（不只是发给本机的）
//!     .open()?
//!     .setnonblock()?;         // 非阻塞：没有包时立即返回 Err（而非阻塞等待）
//! ```
//!
//! ## Rust 概念 — Deref 解引用 `*packet.header`
//! pcap::PacketHeader 可能被 Copy trait 实现，`*` 解引用后可直接复制。

use std::sync::mpsc;
use std::{error::Error, thread};

use pcap::{Capture, Device};

fn main() -> Result<(), Box<dyn Error>> {
    let _ = run();
    Ok(())
}

/// 拥有所有权的网络数据包
/// 
/// ## Rust 概念 — Derive 宏
/// `#[derive(Debug, PartialEq, Eq)]` 自动生成：
/// - `Debug` — 格式化调试输出（`{:?}`）
/// - `PartialEq` / `Eq` — `==` / `!=` 比较运算
#[derive(Debug, PartialEq, Eq)]
struct PacketOwned {
    pub header: pcap::PacketHeader,
    pub data: Box<[u8]>,
}

fn run() -> Result<(), Box<dyn Error>> {
    // http://www.zhengbang.com/static/js/view/getWidth.js
    // http://www.zhengbang.com/static/images/prev.png  大概是1.8k  限制在1414byte一个数据包就可以分包了
    // curl -o  http://www.zhengbang.com/static/uploads/20220823/385e649fe80db74d9e46063f68b5b490.jpg
    println!("start capture");
    let inter_name = "F0F07CFD-D6FE-49C9-8993-8790AFB777A2";

    // Device::list()? — `?` 自动传播 pcap 错误
    // .into_iter().find(...) — 迭代器查找第一个匹配
    let device = Device::list()?
        .into_iter()
        .find(|d| d.name.contains(inter_name))
        .unwrap();  // unwrap 断言一定能找到设备（开发阶段可用，生产应处理 None）

    let mut cap = Capture::from_device(device)?
        .immediate_mode(true)
        .promisc(true)
        .open()?
        .setnonblock()?;
    // BPF 过滤器: "ip host 211.149.224.47 and tcp"
    cap.filter("ip host 211.149.224.47 and tcp", true)?;

    // 创建 mpsc 通道
    let (sender, recevier) = mpsc::channel();
    // 创建子线程接收并打印数据包
    thread::spawn(move || loop {
        let x: PacketOwned = recevier.recv().unwrap();
        println!("receive packet: {:?}", x.header);
    });

    // 主线程循环抓包并通过通道发送
    loop {
        match cap.next_packet() {
            Ok(packet) => {
                let _ = sender.send(PacketOwned {
                    header: *packet.header,       // Deref 解引用复制
                    data: Box::from(packet.data), // &[u8] → Box<[u8]>
                });
            }
            Err(_) => {}  // 非阻塞模式下的"暂无数据"错误，静默忽略
        }
    }
}
