//! 端口扫描 —— 状态管理和消息处理
//!
//! 处理用户交互、启动异步扫描任务、实时更新 UI 状态。
//!
//! # 异步架构
//! ```text
//! StartScan 消息
//!   └── Task::run(stream::channel)        // 创建 iced 异步通道
//!        └── tokio::spawn_blocking(...)    // 在阻塞线程池中运行
//!             ├── scan::parse_ip_range()   // 解析 IP 范围
//!             ├── scan::scan_host()        // 并行扫描（rayon）
//!             │    └── 每个端口 → try_send(PortScanned)
//!             └── try_send(ScanComplete)
//! ```
//!
//! # Rust 语法要点
//!
//! ## `futures::channel::mpsc` —— 多生产者单消费者通道
//! `mpsc::Sender` 可以从多个线程发送消息到同一个接收端。
//! `try_send()` 非阻塞发送，如果缓冲区满则返回错误（不会死锁）。
//! 设置 buffer 大小 100，缓冲区满了丢弃消息（扫描结果太快时防止 UI 卡顿）。
//!
//! ## `iced::stream::channel` —— iced 的异步流适配
//! `stream::channel(buffer, |sender| async move { ... })` 创建一个 iced 流：
//! 1. iced 分配一个 mpsc 通道
//! 2. 将 receiver 包装为 iced Stream
//! 3. 调用闭包，传入 sender，在异步上下文中执行
//! 4. 闭包通过 sender 发送的消息自动进入 iced 的消息循环
//!
//! ## `tokio::task::spawn_blocking` —— 阻塞任务
//! 在 tokio 的阻塞线程池上运行 CPU 密集/阻塞代码。
//! 不同于 `tokio::spawn`（适合异步代码），`spawn_blocking` 不会阻塞异步运行时，
//! 因为它运行在独立的线程池中。
//!
//! ## `move` 闭包
//! `move || { ... }` 和 `move |sender| async move { ... }`
//! `move` 关键字强制闭包获取所捕获变量的所有权（而非借用）。
//! 这是必需的，因为闭包会被发送到另一个线程执行，
//! 无法保证原始变量的生命周期覆盖闭包的生命周期。
//!
//! ## `let _ = expr;` —— 忽略 Result
//! `let _ = sender.try_send(msg);` 忽略 try_send 的返回值。
//! 如果改为 `sender.try_send(msg).unwrap()`，缓冲区满时会 panic。
//! `let _` 是显式忽略值的惯用写法，比不接收返回值更清晰地表达意图。

use futures::channel::mpsc;
use iced::{Task, stream};
use std::fmt;

use super::scan::{self, PortInfo};

/// 扫描模式枚举
///
/// 决定扫描的端口范围。每种模式对应不同数量的端口。
/// `#[default]` 标记 Common 为默认模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScanMode {
    /// 常用端口（约 21 个）
    #[default]
    Common,
    /// 前 100 个端口（1-100）
    Top100,
    /// 全部端口（1-65535，约需 30 分钟）
    All,
}

/// 为 ScanMode 实现 Display trait —— 用于下拉列表和日志显示
///
/// # Rust: `impl fmt::Display for ScanMode`
/// `Formatter<'_>` 是带生命周期的泛型，`'_` 表示编译器自动推断。
impl fmt::Display for ScanMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanMode::Common => write!(f, "常用端口"),
            ScanMode::Top100 => write!(f, "Top 100"),
            ScanMode::All => write!(f, "全部端口"),
        }
    }
}

impl ScanMode {
    /// 获取该模式下的端口列表
    ///
    /// # Rust: `vec![]` 宏
    /// 创建 Vec，元素类型从上下文推断。
    /// `(1..=100).collect()` 从 Range 创建 Vec。
    /// `(1..=65535).collect()` —— 65535 个端口，约 128KB 内存。
    pub fn ports(&self) -> Vec<u16> {
        match self {
            ScanMode::Common => vec![
                21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995, 3306, 3389, 5432, 5900, 6379, 8080, 8443,
                27017,
            ],
            ScanMode::Top100 => (1..=100).collect(),
            ScanMode::All => (1..=65535).collect(),
        }
    }

    /// 获取模式显示名称
    pub fn label(&self) -> &'static str {
        match self {
            ScanMode::Common => "常用端口",
            ScanMode::Top100 => "Top 100",
            ScanMode::All => "全部端口",
        }
    }
}

/// 端口扫描器状态
///
/// # Rust: `#[derive(Default)]`
/// 所有字段都用默认值初始化（String=""、Vec=[]、bool=false、usize=0）。
#[derive(Default)]
pub struct NetScanner {
    /// 目标 IP/网段输入
    pub target: String,
    /// 扫描结果摘要（格式化文本，显示在"扫描结果"区）
    pub results: Vec<String>,
    /// 扫描日志（实时输出，显示在"扫描日志"区）
    pub logs: Vec<String>,
    /// 当前扫描模式
    pub scan_mode: ScanMode,
    /// 是否正在扫描
    pub is_scanning: bool,
    /// 已完成的开放端口汇总（IP → 端口列表）
    pub open_ports: Vec<(String, Vec<PortInfo>)>,
    /// 已扫描端口计数
    pub scanned_count: usize,
    /// 计划扫描端口总数（主机数 × 每主机端口数）
    pub total_ports: usize,
}

impl NetScanner {
    // 目前没有额外方法，所有操作通过 update 函数完成
}

/// 消息类型枚举
///
/// 涵盖用户交互和扫描过程中的异步事件。
/// `#[derive(Debug, Clone)]` —— Clone 是因为 Msg 在 iced 内部可能被复制。
#[derive(Debug, Clone)]
pub enum Msg {
    /// 目标输入框文本改变
    TargetChanged(String),
    /// 扫描模式下拉列表改变
    ScanModeChanged(ScanMode),
    /// 开始扫描按钮点击
    StartScan,
    /// 清空按钮点击
    Clear,
    /// 单个端口扫描完成（来自 rayon 线程的实时进度）
    PortScanned {
        ip: String,
        port: u16,
        is_open: bool,
        service: Option<&'static str>,
    },
    /// 单个主机扫描完成
    HostScanComplete { ip: String, open_ports: Vec<PortInfo> },
    /// 全部扫描完成
    ScanComplete { results: Vec<(String, Vec<PortInfo>)> },
    /// 扫描进度更新 —— 已知主机数后设置总端口数
    ScanTotalPorts(usize),
    /// 扫描日志消息
    ScanLog(String),
    /// 扫描错误消息
    ScanError(String),
}

/// 处理消息，更新端口扫描器状态
///
/// StartScan 是最复杂的消息处理，涉及异步任务创建、跨线程通信和状态同步。
///
/// # Rust: `Task::run(stream::channel(...))` 内部运作
/// 1. `stream::channel(100, |sender| async move { ... })` 创建 iced 流
/// 2. iced 在异步运行时上调用闭包
/// 3. 闭包中 `spawn_blocking` 将阻塞代码提交到专用线程池
/// 4. 在阻塞线程中通过 `sender.try_send()` 向 iced 发送消息
/// 5. iced 接收到消息后调用本 update 函数
pub fn update(scanner: &mut NetScanner, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::TargetChanged(s) => {
            scanner.target = s;
            Task::none()
        }

        Msg::ScanModeChanged(mode) => {
            scanner.scan_mode = mode;
            Task::none()
        }

        Msg::StartScan => {
            // `clone()` 复制数据，因为闭包需要获取所有权
            // String 的 clone 会分配新的堆内存
            let target = scanner.target.clone();
            let mode = scanner.scan_mode;

            if target.is_empty() {
                scanner.results.push("请输入目标IP或网段".to_string());
                return Task::none();
            }

            // 初始化扫描状态
            scanner.is_scanning = true;
            scanner.results.clear();
            scanner.logs.clear();
            scanner.open_ports.clear();
            scanner.scanned_count = 0;
            let ports = mode.ports();
            scanner.total_ports = ports.len();
            scanner.results.push(format!("开始扫描: {} ({})", target, mode.label()));
            scanner.logs.push(format!("[*] 开始扫描目标: {} (模式: {})", target, mode.label()));

            let port_count = ports.len();
            scanner.logs.push(format!("[*] 共 {} 个端口需要扫描", port_count));

            // 创建异步扫描任务
            //
            // `Task::run(stream, mapper)` 签名：
            // - stream: impl Stream<Item = Msg>
            // - mapper: Fn(Msg) -> Msg（这里直接返回自身）
            //
            // `stream::channel(100, factory)`:
            // - 100: 缓冲区大小
            // - factory: FnOnce(Sender) -> impl Future
            Task::run(
                stream::channel(100, move |mut sender: mpsc::Sender<Msg>| async move {
                    // `spawn_blocking` 在 tokio 阻塞线程池中运行闭包
                    tokio::task::spawn_blocking(move || {
                        match scan::parse_ip_range(&target) {
                            Ok(ips) => {
                                let _ = sender.try_send(Msg::ScanLog(
                                    format!("[*] 准备完成，共 {} 个主机", ips.len())
                                ));
                                let total = ips.len() * ports.len();
                                let _ = sender.try_send(Msg::ScanTotalPorts(total));

                                let mut all_results = Vec::new();

                                // 遍历每个 IP
                                for ip in ips {
                                    let _ = sender.try_send(Msg::ScanLog(
                                        format!("[*] 扫描主机: {}", ip)
                                    ));

                                    // 克隆 sender 传给回调（clone 后两个 sender 共享同一个通道）
                                    let mut s = sender.clone();
                                    // rayon 并行扫描当前主机的所有端口
                                    let open_ports = scan::scan_host(ip, &ports, move |port_info| {
                                        let _ = s.try_send(Msg::PortScanned {
                                            ip: ip.to_string(),
                                            port: port_info.port,
                                            is_open: port_info.is_open,
                                            service: port_info.service,
                                        });
                                    });

                                    if !open_ports.is_empty() {
                                        all_results.push((ip.to_string(), open_ports.clone()));
                                        let _ = sender.try_send(Msg::HostScanComplete {
                                            ip: ip.to_string(),
                                            open_ports,
                                        });
                                    }
                                }

                                let _ = sender.try_send(Msg::ScanComplete {
                                    results: all_results
                                });
                            }
                            Err(e) => {
                                let _ = sender.try_send(Msg::ScanError(e.to_string()));
                            }
                        }
                    })
                    .await
                    .unwrap();  // spawn_blocking 几乎不会失败
                }),
                // mapper: 直接返回消息自身（类型已经是 Msg，无需映射）
                |msg| msg,
            )
        }

        Msg::PortScanned { ip: _, port, is_open, service } => {
            scanner.scanned_count += 1;
            if is_open {
                let service_name = service.unwrap_or("");
                let log = format!("[+] {}:{} 开放 - {}", scanner.target, port, service_name);
                scanner.logs.push(log);
                // 防止日志无限增长 —— 最多保留 500 条
                // `drain(..excess)` 移除并丢弃前 excess 条
                if scanner.logs.len() > 500 {
                    let excess = scanner.logs.len() - 500;
                    scanner.logs.drain(..excess);
                }
            }
            Task::none()
        }

        Msg::HostScanComplete { ip, open_ports } => {
            if !open_ports.is_empty() {
                scanner.logs.push(format!(
                    "[+] {} 开放端口: {:?}",
                    ip,
                    open_ports.iter().map(|p| p.port).collect::<Vec<_>>()
                ));
            }
            Task::none()
        }

        Msg::ScanComplete { results } => {
            scanner.is_scanning = false;
            scanner.open_ports = results;

            // `iter().map().sum()` —— 迭代器组合子链
            let host_count = scanner.open_ports.len();
            let total_open_ports: usize = scanner.open_ports.iter()
                .map(|(_, p)| p.len())
                .sum();

            scanner.logs.push(format!(
                "[*] 扫描完成! 发现 {} 个主机, 共 {} 个开放端口",
                host_count, total_open_ports
            ));

            // 格式化最终结果
            if scanner.open_ports.is_empty() {
                scanner.results.push("未发现开放端口".to_string());
            } else {
                for (ip, port_infos) in &scanner.open_ports {
                    scanner.results.push(format!("\n{}:", ip));
                    for p in port_infos {
                        let service = p.service.unwrap_or("未知");
                        scanner.results.push(format!("  端口 {} 开放 - {}", p.port, service));
                    }
                }
                scanner.results.push(format!("\n扫描完成，共发现 {} 个主机", host_count));
            }
            Task::none()
        }

        Msg::ScanTotalPorts(total) => {
            scanner.total_ports = total;
            scanner.logs.push(format!("[*] 共 {} 个端口需要扫描", total));
            Task::none()
        }

        Msg::ScanLog(e) => {
            scanner.logs.push(e);
            Task::none()
        }

        Msg::ScanError(e) => {
            scanner.is_scanning = false;
            scanner.results.push(format!("扫描失败: {}", e));
            scanner.logs.push(format!("[x] 扫描失败: {}", e));
            Task::none()
        }

        Msg::Clear => {
            scanner.results.clear();
            scanner.logs.clear();
            scanner.open_ports.clear();
            scanner.target.clear();
            scanner.is_scanning = false;
            Task::none()
        }
    }
}
