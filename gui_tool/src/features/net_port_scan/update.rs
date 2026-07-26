//! 端口扫描 —— 状态管理和消息处理
//!
//! 处理用户交互、启动异步扫描任务、实时更新 UI 状态。
//!
//! # 异步架构（iced + tokio 最佳实践）
//! ```text
//! StartScan 消息
//!   └── Task::run(stream::channel)          // iced 异步流适配器
//!        └── async {                        // 在 tokio 运行时上执行
//!             ├── parse_ip_range()          // 纯计算，同步执行
//!             ├── syn_available?
//!             │    ├─ Y → spawn_blocking(syn_scan)  // 原始套接字阻塞
//!             │    └─ N → connect_scan_host_async() // 全异步 TCP Connect
//!             │           └─ tokio::spawn × N       // 协程级并发
//!             │              └─ tokio::time::timeout(500ms, TcpStream::connect)
//!             └── try_send(ScanComplete)
//!        }
//! ```
//!
//! # iced + tokio 集成模式
//!
//! ## 模式 1: 全异步操作（推荐）
//! 适合非阻塞 I/O（如 HTTP 请求、TCP 连接）：
//! ```text
//! stream::channel(buffer, |sender| async move {
//!     tokio::spawn(async { ... })  // 在 tokio 运行时上创建协程
//!     tokio::time::timeout(dur, fut).await
//! })
//! ```
//!
//! ## 模式 2: 阻塞操作（回调风格）
//! 适合 CPU 密集或原始套接字操作：
//! ```text
//! tokio::task::spawn_blocking(move || {
//!     // blocking code
//!     sender.try_send(msg);  // 通过 channel 回传结果
//! })
//! ```
//!
//! ## 对比: spawn_blocking vs tokio::spawn
//! - `spawn_blocking`: OS 线程级，适合 CPU 密集/原始套接字
//! - `tokio::spawn`: 协程级（无栈），适合 async I/O，数万并发无压力
//! - 本扫描器针对 Connect 模式使用后者，SYN 模式使用前者
//!
//! ## 通道设计
//! `futures::channel::mpsc::Sender` 是 iced 原生的跨线程通信方式：
//! - `try_send()` 非阻塞，缓冲区满时丢弃（防 UI 卡顿）
//! - `Sender: Clone`，可多生产者共享
//! - `iced::stream::channel` 内部将 mpsc::Receiver 包装为 Stream
//! - 消息通过 Stream 进入 iced 的 update 循环（主线程安全）

#[cfg(unix)]
use std::net::Ipv4Addr;

use futures::channel::mpsc;
use iced::{Task, stream};
use std::fmt;

use super::scan::{self, PortInfo, ScanMethod};

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
            let port_count = ports.len();
            scanner.total_ports = port_count;
            scanner.results.push(format!("开始扫描: {} ({})", target, mode.label()));
            scanner.logs.push(format!("[*] 开始扫描目标: {} (模式: {})", target, mode.label()));
            scanner.logs.push(format!("[*] 共 {} 个端口需要扫描", port_count));

            // iced + tokio 异步架构：
            // stream::channel 创建 iced 原生流适配器
            // 内部 async 闭包在 tokio 运行时上执行
            Task::run(
                stream::channel(100, move |mut sender: mpsc::Sender<Msg>| async move {
                    // ── 1. 解析 IP 范围（纯计算，直接在 async 上下文中执行） ──
                    let ips = match scan::parse_ip_range(&target) {
                        Ok(ips) => ips,
                        Err(e) => {
                            let _ = sender.try_send(Msg::ScanError(e.to_string()));
                            return;
                        }
                    };

                    let _ = sender.try_send(Msg::ScanLog(
                        format!("[*] 准备完成，共 {} 个主机", ips.len())
                    ));
                    let total = ips.len() * ports.len();
                    let _ = sender.try_send(Msg::ScanTotalPorts(total));

                    let mut all_results: Vec<(String, Vec<PortInfo>)> = Vec::new();

                    // ── 2. 遍历每个目标 IP ──
                    for ip in ips {
                        let _ = sender.try_send(Msg::ScanLog(
                            format!("[*] 扫描主机: {}", ip)
                        ));

                        // 自动选择扫描方法
                        let (open_ports, _method): (Vec<PortInfo>, ScanMethod) = {
                            // Ipv4Addr 是 Copy 类型，先复制一份供异步闭包捕获
                            let target_ip = ip;
                            let host_ports = ports.clone();

                            #[cfg(unix)]
                            if scan::check_syn_available() {
                                // ── 路径 A: SYN 半开扫描 ──
                                // 原始套接字操作 → 需 spawn_blocking
                                let mut s = sender.clone();
                                let src = scan::get_source_ip(target_ip)
                                    .unwrap_or(Ipv4Addr::new(0, 0, 0, 0));

                                let result = tokio::task::spawn_blocking(move || {
                                    scan::syn_scan_host(
                                        src, target_ip, &host_ports,
                                        move |info| {
                                            let _ = s.try_send(Msg::PortScanned {
                                                ip: target_ip.to_string(),
                                                port: info.port,
                                                is_open: info.is_open,
                                                service: info.service,
                                            });
                                        },
                                        std::time::Duration::from_secs(5),
                                    )
                                }).await;

                                match result {
                                    Ok(Ok((_total, _open, _closed, _filtered))) => {
                                        (Vec::new(), ScanMethod::Syn)
                                    }
                                    Ok(Err(e)) => {
                                        let _ = sender.try_send(Msg::ScanLog(
                                            format!("[!] SYN 失败 ({}), 回退 Connect", e)
                                        ));
                                        let mut s = sender.clone();
                                        let results = scan::connect_scan_host_async(
                                            target_ip, &host_ports, move |info| {
                                                let _ = s.try_send(Msg::PortScanned {
                                                    ip: target_ip.to_string(),
                                                    port: info.port,
                                                    is_open: info.is_open,
                                                    service: info.service,
                                                });
                                            },
                                            100,
                                        ).await;
                                        (results, ScanMethod::Connect)
                                    }
                                    Err(_) => (Vec::new(), ScanMethod::Syn),
                                }
                            } else {
                                // ── 路径 B: 全异步 TCP Connect 扫描 ──
                                let mut s = sender.clone();
                                let results = scan::connect_scan_host_async(
                                    target_ip, &host_ports, move |info| {
                                        let _ = s.try_send(Msg::PortScanned {
                                            ip: target_ip.to_string(),
                                            port: info.port,
                                            is_open: info.is_open,
                                            service: info.service,
                                        });
                                    },
                                    100,
                                ).await;
                                (results, ScanMethod::Connect)
                            }

                            // Windows: 直接走 Connect
                            #[cfg(windows)]
                            {
                                let mut s = sender.clone();
                                let results = scan::connect_scan_host_async(
                                    target_ip, &host_ports, move |info| {
                                        let _ = s.try_send(Msg::PortScanned {
                                            ip: target_ip.to_string(),
                                            port: info.port,
                                            is_open: info.is_open,
                                            service: info.service,
                                        });
                                    },
                                    100,
                                ).await;
                                (results, ScanMethod::Connect)
                            }
                        };

                        if !open_ports.is_empty() {
                            all_results.push((ip.to_string(), open_ports.clone()));
                            let _ = sender.try_send(Msg::HostScanComplete {
                                ip: ip.to_string(),
                                open_ports,
                            });
                        }
                    }

                    // ── 3. 扫描完成 ──
                    let _ = sender.try_send(Msg::ScanComplete {
                        results: all_results
                    });
                }),
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
