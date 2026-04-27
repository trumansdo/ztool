//! 端口扫描核心引擎
//!
//! 提供 IP 范围解析、端口检测、并行扫描等功能。
//!
//! # 核心算法
//! 1. **IP 范围解析**: 支持单 IP、IP 范围 (1-10)、CIDR 网段 (0/24)
//! 2. **端口扫描**: TCP connect 方式探测端口开放状态
//! 3. **并行化**: 使用 rayon 并行扫描所有端口，自动利用多核 CPU
//!
//! # Rust 语法要点
//!
//! ## `use std::net::{IpAddr, Ipv4Addr, ...}` —— 嵌套导入
//! 从 `std::net` 模块一次性导入多个类型。等价于：
//! ```text
//! use std::net::IpAddr;
//! use std::net::Ipv4Addr;
//! ...
//! ```
//!
//! ## `TcpStream::connect_timeout` —— TCP 连接
//! `TcpStream` 是 Rust 标准库的 TCP 套接字包装。
//! `connect_timeout(&addr, timeout)` 尝试在指定超时时间内建立 TCP 连接，
//! 成功返回 `Ok(stream)`，失败（端口关闭/超时）返回 `Err`。
//!
//! ## std::sync::Mutex —— 互斥锁
//! ```text
//! let callback = Mutex::new(callback);
//! // ...
//! if let Ok(mut cb) = callback.lock() { cb(result); }
//! ```
//! `Mutex` 提供内部可变性 + 线程安全。`lock()` 获取锁，
//! 返回 `LockResult<MutexGuard<T>>` —— 即可以解引用访问内部值的智能指针。
//! Rust 的 Mutex 是"有毒的" (poisoned) —— 如果持有锁的线程 panic，
//! 后续 lock() 返回 Err，防止不一致状态被读取。
//!
//! ## rayon::par_iter() —— 数据并行
//! `ports.par_iter()` 将串行迭代器转换为并行迭代器。
//! rayon 内部使用 work-stealing 调度，比手动 `thread::spawn` 更高效：
//! - 自动负载均衡
//! - 避免创建过多线程的开销
//! - 适合 CPU 密集型任务（端口扫描主要是 I/O wait，但 TCP 连接超时短，并行仍有价值）

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;
use std::sync::Mutex;
use rayon::prelude::*;

/// 端口扫描结果
///
/// # Rust: `#[derive(Clone, Copy)]`
/// `Copy` 意味着这个类型是"按值复制"语义 —— 赋值后原变量仍可用。
/// 因为所有字段都是 Copy 类型（u16, bool, Option<&str>）。
/// Copy 类型不能包含 String/Vec 等堆分配类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortInfo {
    /// 端口号 (0~65535)
    pub port: u16,
    /// 端口是否开放（TCP 连接成功 = 开放）
    pub is_open: bool,
    /// 服务名称（仅有匹配的常用端口才有值）
    pub service: Option<&'static str>,
}

/// 常用端口与服务名称映射表
///
/// # Rust: `&[(u16, &str)]` —— 切片引用
/// `&[(u16, &str)]` 是对 `[(u16, &str)]` 切片的引用。
/// 切片是"胖指针"：包含数据指针 + 长度。
/// 与数组 `[T; N]` 不同，切片的大小在编译时不固定。
///
/// # Rust: `const` + `&`
/// `pub const COMMON_PORTS: &[(u16, &str)] = &[...];`
/// 这里 `&[...]` 创建了一个对静态数组的引用。
/// const 的值在编译时完全确定，存储在二进制只读数据段。
pub const COMMON_PORTS: &[(u16, &str)] = &[
    (21, "FTP"),
    (22, "SSH"),
    (23, "Telnet"),
    (25, "SMTP"),
    (53, "DNS"),
    (80, "HTTP"),
    (110, "POP3"),
    (143, "IMAP"),
    (443, "HTTPS"),
    (465, "SMTPS"),
    (587, "SMTP"),
    (993, "IMAPS"),
    (995, "POP3S"),
    (3306, "MySQL"),
    (3389, "RDP"),
    (5432, "PostgreSQL"),
    (5900, "VNC"),
    (6379, "Redis"),
    (8080, "HTTP-Alt"),
    (8443, "HTTPS-Alt"),
    (27017, "MongoDB"),
];

/// 根据端口号获取服务名称
///
/// # Rust: 迭代器链
/// `COMMON_PORTS.iter().find(|(p, _)| *p == port).map(|(_, name)| *name)`
/// - `.iter()`: 创建不可变引用迭代器，产生 `&(u16, &str)`
/// - `.find(|(p, _)| ...)`: 找到第一个满足条件的元素
/// - `|(p, _)| *p == port`: 模式解构闭包参数，`(p, _)` 解构元组引用
/// - `.map(|(_, name)| *name)`: 提取服务名称
///
/// # 返回值
/// `Option<&'static str>` —— 找到则返回静态字符串引用，找不到则 None。
pub fn get_service_name(port: u16) -> Option<&'static str> {
    COMMON_PORTS.iter().find(|(p, _)| *p == port).map(|(_, name)| *name)
}

/// IP 解析错误类型
///
/// # Rust: 自定义错误类型
/// 实现三个 trait 使错误可用：
/// 1. `Debug`: 通过 `#[derive(Debug)]` 自动获得
/// 2. `Display`: 手动实现 `fmt::Display`（人类可读的错误信息）
/// 3. `Error`: 通过 `impl std::error::Error for ScanError {}` 空实现标记
///    因为 Display 已提供所有必要信息，不需要 error source
#[derive(Debug)]
pub enum ScanError {
    /// 无效的 IP 地址格式
    InvalidIp(String),
    /// 无效的 CIDR 网段格式
    InvalidCidr(String),
}

/// Display 实现 —— 格式化打印
///
/// # Rust: `impl std::fmt::Display for ScanError`
/// `Formatter<'_>` 的 `'_` 是生命周期省略。
/// `write!(f, "...", s)` 是格式化写入宏，类似 `format!` 但写入到 Formatter。
impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::InvalidIp(s) => write!(f, "无效的IP地址: {}", s),
            ScanError::InvalidCidr(s) => write!(f, "无效的网段格式: {}", s),
        }
    }
}

/// 空实现 Error trait —— Display 已提供足够信息
impl std::error::Error for ScanError {}

/// 解析 IP 范围字符串
///
/// # 支持格式
/// - 单个 IP: `192.168.1.1`
/// - IP 范围: `192.168.1.1-10`
/// - CIDR 网段: `192.168.1.0/24`
///
/// # Rust: `Result<T, E>` —— 可能失败的操作
/// `Result<Vec<Ipv4Addr>, ScanError>` 表示要么成功返回 IP 列表，
/// 要么失败返回 ScanError。
/// 这与异常机制不同 —— Result 是值，由调用方显式处理。
///
/// # Rust: `str::parse()` —— 字符串解析
/// `parts[0].parse()` 返回 `Result<Ipv4Addr, AddrParseError>`。
/// 因为 Ipv4Addr 实现了 `FromStr` trait，所以可以调用 `.parse()`。
/// `map_err(|_| ScanError::InvalidIp(...))` 将解析错误转换为自定义错误类型。
pub fn parse_ip_range(input: &str) -> Result<Vec<Ipv4Addr>, ScanError> {
    let input = input.trim();

    // CIDR 格式: 192.168.1.0/24
    if input.contains('/') {
        // `split('/')` 返回迭代器，`collect()` 收集为 Vec<&str>
        let parts: Vec<&str> = input.split('/').collect();
        if parts.len() != 2 {
            return Err(ScanError::InvalidCidr(input.to_string()));
        }

        // `parse()` 尝试将 &str 解析为目标类型
        // `map_err(|_| ...)` 忽略具体错误，转为自定义错误
        let ip: Ipv4Addr = parts[0].parse()
            .map_err(|_| ScanError::InvalidIp(parts[0].to_string()))?;

        let prefix: u8 = parts[1].parse()
            .map_err(|_| ScanError::InvalidCidr(input.to_string()))?;

        if prefix > 32 {
            return Err(ScanError::InvalidCidr(input.to_string()));
        }

        // 计算子网掩码和地址范围
        // `1u32 << (32 - prefix)` — 位运算: 1 左移 (32-prefix) 位
        // 例如 prefix=24 → 1u32 << 8 = 256，mask = !255 = 0xFFFFFF00
        let mask = !((1u32 << (32 - prefix)) - 1);
        // `u32::from(ip)` — 将 Ipv4Addr 转为 u32（大端字节序的网络字节序）
        let network = u32::from(ip) & mask;
        let broadcast = network | !mask;

        // 排除网络地址和广播地址
        let mut results = Vec::new();
        // `(network + 1)..broadcast` — 创建整数 Range
        // 1..5 表示 [1, 2, 3, 4]（不包含上界）
        for addr in (network + 1)..broadcast {
            // `Ipv4Addr::from(u32)` — 将 u32 转回 Ipv4Addr
            results.push(Ipv4Addr::from(addr));
        }

        Ok(results)
    }
    // 范围格式: 192.168.1.1-10
    else if input.contains('-') {
        let range: Vec<&str> = input.split('-').collect();
        if range.len() != 2 {
            return Err(ScanError::InvalidIp(input.to_string()));
        }

        let start: Ipv4Addr = range[0].parse()
            .map_err(|_| ScanError::InvalidIp(range[0].to_string()))?;
        // 补全 IP: 如果用户输入了 "192.168.1.1-10"，
        // end 部分只有 "10"，需要补全前几段
        let end_str = if range[1].contains('.') {
            range[1].to_string()
        } else {
            let mut parts: Vec<&str> = range[0].split('.').collect();
            if parts.len() == 4 {
                parts[3] = range[1];
                parts.join(".")
            } else {
                range[1].to_string()
            }
        };
        let end: Ipv4Addr = end_str.parse()
            .map_err(|_| ScanError::InvalidIp(range[1].to_string()))?;

        let start_u32 = u32::from(start);
        let end_u32 = u32::from(end);

        if start_u32 > end_u32 {
            return Err(ScanError::InvalidIp(input.to_string()));
        }

        let mut results = Vec::new();
        // `..=` 是包含上界的 Range
        for addr in start_u32..=end_u32 {
            results.push(Ipv4Addr::from(addr));
        }

        Ok(results)
    }
    // 单个 IP
    else {
        let ip: Ipv4Addr = input.parse()
            .map_err(|_| ScanError::InvalidIp(input.to_string()))?;
        Ok(vec![ip])
    }
}

/// 扫描单个端口
///
/// 使用 TCP 三次握手探测端口开放状态：
/// - 发送 SYN 包
/// - 开放端口返回 SYN+ACK（连接成功）
/// - 关闭端口返回 RST 或超时
///
/// # Rust: `SocketAddr` 构造
/// `SocketAddr::new(IpAddr::V4(ip), port)` 创建 IPv4 套接字地址。
/// 需要 `IpAddr::V4(ip)` 因为 SocketAddr 接受 `IpAddr` 枚举（通用类型），
/// 而 `ip` 是 `Ipv4Addr` （具体类型）。
pub fn scan_port(ip: Ipv4Addr, port: u16, timeout: Duration) -> PortInfo {
    let addr = SocketAddr::new(IpAddr::V4(ip), port);
    // `is_ok()` 将 Result 转为 bool —— Ok 表示连接成功（端口开放）
    let is_open = TcpStream::connect_timeout(&addr, timeout).is_ok();
    PortInfo {
        port,
        is_open,
        // 只在端口开放时查询服务名，关闭的端口没必要查询
        service: if is_open { get_service_name(port) } else { None },
    }
}

/// 扫描主机的多个端口（并行）
///
/// # Rust: 泛型 + trait 约束
/// ```text
/// pub fn scan_host<F>(ip: Ipv4Addr, ports: &[u16], callback: F) -> Vec<PortInfo>
/// where
///     F: FnMut(PortInfo) + Send + Sync,
/// ```
/// - `<F>`: 泛型参数，接受任意回调类型
/// - `FnMut(PortInfo)`: 可被多次调用，可能修改自身状态（闭包捕获的外部变量）
/// - `Send`: 可在多线程间传递所有权
/// - `Sync`: 共享引用可跨线程使用
/// `Mutex` 将非 Sync 的 F 包装为 Sync，因为 Mutex<T> 实现了 Sync（当 T: Send）。
///
/// # rayon 并行扫描原理
/// `ports.par_iter()` 将切片分割成多个 chunk，分发给线程池中的工作线程。
/// 每个线程处理自己的 chunk，互不干扰。通过 Mutex 保护回调实现线程间通信。
///
/// # Rust: `Vec::collect()` 的自动类型推断
/// `.filter(|p| p.is_open).collect()`
/// 编译器从函数返回类型 `Vec<PortInfo>` 推断 collect 的目标集合类型。
/// 无需像 C++ 那样显式指定 `collect<Vec<_>>()`（但可以写）。
pub fn scan_host<F>(ip: Ipv4Addr, ports: &[u16], callback: F) -> Vec<PortInfo>
where
    F: FnMut(PortInfo) + Send + Sync,
{
    let timeout = Duration::from_millis(500);
    // Mutex::new() 将 callback 包装在互斥锁中
    let callback = Mutex::new(callback);

    ports.par_iter()
        .map(|&port| {
            let result = scan_port(ip, port, timeout);
            // lock() 获取锁 —— 同一时刻只有一个线程能执行这里的代码
            // 但如果扫描很快（连接超时 500ms），锁竞争不会太严重
            // `if let Ok(mut cb) = callback.lock()` —— 处理 Mutex 中毒情况
            if let Ok(mut cb) = callback.lock() {
                cb(result);
            }
            result
        })
        .filter(|p| p.is_open)  // 只保留开放端口
        .collect()
}
