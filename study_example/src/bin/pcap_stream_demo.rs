//! # pcap 异步流式抓包演示
//!
//! 展示如何结合 pcap 库和 tokio 异步运行时实现流式网络数据包捕获。
//!
//! ## Rust 概念
//!
//! ### trait 实现 — `impl PacketCodec for SimpleDumpCodec`
//! pcap 库定义了一个 `PacketCodec` trait，要求实现 `decode` 方法。
//! `SimpleDumpCodec` 是一个空结构体（零大小类型），实现了该 trait。
//! 关联类型 `type Item = PacketOwned` 指定 decode 的返回类型。
//!
//! ### `async fn` — 异步函数
//! `start_new_stream` 和 `main` 都是异步函数，使用 `.await` 等待异步操作完成。
//! 异步函数在编译时被转换为状态机（编译器自动完成）。
//!
//! ### `#[tokio::main]` — 异步运行时入口
//! `#[tokio::main(flavor = "multi_thread", worker_threads = 10)]` 将 main 函数
//! 包装到 tokio 的多线程运行时中，包含 10 个工作线程。
//!
//! ### `StreamExt::for_each` — 流消费
//! `stream.for_each(move |s| { ... })` 对流中的每个元素执行闭包。
//! `futures::future::ready(())` 返回立即完成的 Future。
//!
//! ### `move` 闭包
//! `move |s| { ... }` 将环境变量（stream）的所有权移入闭包。
//!
//! ### `Box<[u8]>` — 堆分配的字节切片
//! 与 `Vec<u8>` 不同，`Box<[u8]>` 是固定长度的堆分配切片（类似 C 的 malloc），
//! 不能动态调整大小但内存开销更小。

use futures::StreamExt;
use pcap::{Active, Capture, Device, Error, PacketCodec, PacketStream};
use std::error;

/// 空的编解码器结构体
///
/// 零大小类型（ZST），不携带任何状态，仅用于实现 PacketCodec trait。
pub struct SimpleDumpCodec;

/// pcap 数据包的所有权版本
///
/// 将 pcap 的借用数据转换为拥有所有权的数据包。
/// `Box<[u8]>` 是堆分配的固定长度字节数组，
/// 通过 `pkt.data.into()` 从 `&[u8]` 转换而来。
#[derive(Debug, PartialEq, Eq)]
pub struct PacketOwned {
    pub header: pcap::PacketHeader,
    pub data: Box<[u8]>,
}

/// 为 SimpleDumpCodec 实现 PacketCodec trait
///
/// ## Rust 概念 — 关联类型
/// `type Item = PacketOwned;` 声明此 trait 实现的关联类型。
/// 每个 trait 实现只能有一个关联类型值。
impl PacketCodec for SimpleDumpCodec {
    type Item = PacketOwned;

    fn decode(&mut self, pkt: pcap::Packet) -> Self::Item {
        println!("SimplePacketCodec decode");
        PacketOwned {
            header: *pkt.header,          // * 解引用复制 PacketHeader (实现了 Copy)
            data: pkt.data.into(),         // &[u8] → Box<[u8]>, 拷贝数据到堆上
        }
    }
}

/// 异步启动抓包流
///
/// 调用同步函数 `new_stream` 并用 match 处理可能的错误。
/// 错误时调用 `std::process::exit(1)` 终止程序。
async fn start_new_stream(device: Device) -> PacketStream<Active, SimpleDumpCodec> {
    match new_stream(device) {
        Ok(stream) => stream,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);  // 非零退出码表示异常终止
        }
    }
}

/// 创建 pcap 抓包流
///
/// ## Rust 概念 — `?` 运算符
/// builder 模式中的每个 `?` 自动传播错误：
/// - `.immediate_mode(false)` — 关闭立即模式，使用批量读取
/// - `.promisc(true)` — 启用混杂模式，捕获所有经过网卡的数据包
/// - `.open()?` — 打开设备，失败则返回 Error
/// - `.setnonblock()?` — 设置为非阻塞模式（配合异步运行时）
///
/// ## Rust 概念 — `let _ =`
/// `cap.filter(..., true)` 的应用结果被忽略（_ 表示不需要返回值）。
fn new_stream(device: Device) -> Result<PacketStream<Active, SimpleDumpCodec>, Error> {
    println!("Using device {}", device.name);

    let mut cap = Capture::from_device(device)?
        .immediate_mode(false)
        .promisc(true)
        .open()?
        .setnonblock()?;
    let _ = cap.filter("ip host 211.149.224.47 and tcp", true);
    // cap.stream(codec) — 将 Capture 转换为异步 PacketStream
    cap.stream(SimpleDumpCodec {})
}

/// 主函数：异步流式抓包
///
/// ## Rust 概念 — `into_iter().find().unwrap()`
/// 迭代器链式调用：
/// 1. `Device::list()?` — 枚举所有网络设备
/// 2. `.into_iter()` — 转为所有权迭代器
/// 3. `.find(|d| d.name.contains(inter_name))` — 查找名称匹配的设备
/// 4. `.unwrap()` —   提取 Option 中的值（如果没找到则 panic）
///
/// ## Rust 概念 — `Box<dyn Error>`
/// trait 对象，可以容纳任何实现了 Error trait 的类型的引用。
/// 与具体类型不同，这是动态分发的。
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // 网络接口名称（macOS 的 UUID 格式）
    let inter_name = "F0F07CFD-D6FE-49C9-8993-8790AFB777A2";

    let device = Device::list()?
        .into_iter()
        .find(|d| d.name.contains(inter_name))
        .unwrap();
    let stream = start_new_stream(device).await;

    // 流式消费：对每个到达的数据包执行闭包
    // `futures::future::ready(())` — 返回立即完成的 Future
    let fut = stream.for_each(move |s| {
        println!("{:?}", s);
        futures::future::ready(())
    });
    fut.await;  // 阻塞等待流消费完成
    Ok(())
}
