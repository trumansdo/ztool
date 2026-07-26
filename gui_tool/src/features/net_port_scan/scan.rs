//! 端口扫描核心引擎
//!
//! 策略:
//! - **Unix (Linux/Mac)**: 真 SYN 半开扫描 (`pnet::transport` 原始套接字)
//! - **Windows**: 自动回退 TCP Connect 扫描
//!
//! # SYN 扫描原理 (RFC 793)
//! 发送原始 SYN → 收到 SYN+ACK=开放 / RST=关闭 / 无响应=filtered
//! 不完成三次握手，无应用层日志。

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::Mutex;
#[cfg(unix)]
use std::time::Instant;
use std::time::Duration;
use rayon::prelude::*;

// ─── 端口信息 ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortInfo {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<&'static str>,
}

impl PortInfo {
    fn open(port: u16) -> Self {
        PortInfo { port, is_open: true, service: get_service_name(port) }
    }
    fn closed(port: u16) -> Self {
        PortInfo { port, is_open: false, service: None }
    }
}

pub const COMMON_PORTS: &[(u16, &str)] = &[
    (21, "FTP"), (22, "SSH"), (23, "Telnet"), (25, "SMTP"),
    (53, "DNS"), (80, "HTTP"), (110, "POP3"), (143, "IMAP"),
    (443, "HTTPS"), (465, "SMTPS"), (587, "SMTP"),
    (993, "IMAPS"), (995, "POP3S"),
    (3306, "MySQL"), (3389, "RDP"), (5432, "PostgreSQL"),
    (5900, "VNC"), (6379, "Redis"),
    (8080, "HTTP-Alt"), (8443, "HTTPS-Alt"), (27017, "MongoDB"),
];

pub fn get_service_name(port: u16) -> Option<&'static str> {
    COMMON_PORTS.iter().find(|(p, _)| *p == port).map(|(_, n)| *n)
}

// ─── 扫描方法 ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScanMethod { Syn, Connect }

// ─── 错误 ──────────────────────────────────────────────

#[derive(Debug)]
pub enum ScanError {
    InvalidIp(String), InvalidCidr(String),
}
impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::InvalidIp(s)  => write!(f, "无效的IP地址: {}", s),
            ScanError::InvalidCidr(s)=> write!(f, "无效的网段格式: {}", s),
        }
    }
}
impl std::error::Error for ScanError {}

// ─── IP 解析 ────────────────────────────────────────────

pub fn parse_ip_range(input: &str) -> Result<Vec<Ipv4Addr>, ScanError> {
    let input = input.trim();
    if input.contains('/') {
        let p: Vec<&str> = input.split('/').collect();
        if p.len() != 2 { return Err(ScanError::InvalidCidr(input.into())); }
        let ip: Ipv4Addr = p[0].parse().map_err(|_| ScanError::InvalidIp(p[0].into()))?;
        let prefix: u8 = p[1].parse().map_err(|_| ScanError::InvalidCidr(input.into()))?;
        if prefix > 32 { return Err(ScanError::InvalidCidr(input.into())); }
        let mask = !((1u32 << (32 - prefix)) - 1);
        let net = u32::from(ip) & mask;
        let bc = net | !mask;
        Ok(((net + 1)..bc).map(Ipv4Addr::from).collect())
    } else if input.contains('-') {
        let r: Vec<&str> = input.split('-').collect();
        if r.len() != 2 { return Err(ScanError::InvalidIp(input.into())); }
        let s: Ipv4Addr = r[0].parse().map_err(|_| ScanError::InvalidIp(r[0].into()))?;
        let es = if r[1].contains('.') { r[1].to_string() }
                 else { let mut parts: Vec<&str> = r[0].split('.').collect();
                        if parts.len() == 4 { parts[3] = r[1]; parts.join(".") }
                        else { r[1].to_string() }};
        let e: Ipv4Addr = es.parse().map_err(|_| ScanError::InvalidIp(r[1].into()))?;
        let (su, eu) = (u32::from(s), u32::from(e));
        if su > eu { return Err(ScanError::InvalidIp(input.into())); }
        Ok((su..=eu).map(Ipv4Addr::from).collect())
    } else {
        let ip: Ipv4Addr = input.parse().map_err(|_| ScanError::InvalidIp(input.into()))?;
        Ok(vec![ip])
    }
}

// ─── 简易 LCG 随机数 ───────────────────────────────────

struct Lcg(u64);
impl Lcg {
    fn new() -> Self {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
        Lcg(t)
    }
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 32) as u32
    }
    fn port(&mut self) -> u16 {
        let p = self.next_u32() as u16;
        if p < 1024 { p + 1024 } else { p }
    }
}

// ═══════════════════════════════════════════════════════════
//  TCP CONNECT 扫描（跨平台回退方案）
// ═══════════════════════════════════════════════════════════

fn connect_port(ip: Ipv4Addr, port: u16, timeout: Duration) -> PortInfo {
    match TcpStream::connect_timeout(&SocketAddr::new(IpAddr::V4(ip), port), timeout) {
        Ok(_) => PortInfo::open(port),
        Err(_) => PortInfo::closed(port),
    }
}

pub fn connect_scan_host<F>(ip: Ipv4Addr, ports: &[u16], callback: F) -> Vec<PortInfo>
where F: FnMut(PortInfo) + Send + Sync {
    let timeout = Duration::from_millis(500);
    let cb = Mutex::new(callback);
    ports.par_iter().map(|&port| {
        let r = connect_port(ip, port, timeout);
        if let Ok(mut c) = cb.lock() { c(r); }
        r
    }).filter(|p| p.is_open).collect()
}

/// Async TCP Connect 扫描 — 基于 tokio 的全异步实现
///
/// 使用 `tokio::net::TcpStream` + `tokio::time::timeout` 实现非阻塞连接，
/// `tokio::sync::Semaphore` 控制并发度。
/// 相比 `connect_scan_host`（rayon 线程池），async 模式：
/// - 无 OS 线程开销（协程级并发）
/// - 精确的并发控制（semaphore）
/// - 可取消（配合 tokio::select!）
pub async fn connect_scan_host_async<F>(
    ip: Ipv4Addr,
    ports: &[u16],
    callback: F,
    concurrency: usize,
) -> Vec<PortInfo>
where
    F: FnMut(PortInfo) + Send + Sync + 'static,
{
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let sem = Arc::new(Semaphore::new(concurrency));
    let cb = Arc::new(Mutex::new(callback));
    let conn_timeout = Duration::from_millis(500);
    let mut handles = Vec::with_capacity(ports.len());

    for &port in ports {
        let sem = sem.clone();
        let cb = cb.clone();
        let addr = SocketAddr::new(IpAddr::V4(ip), port);

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let is_open = tokio::time::timeout(conn_timeout, tokio::net::TcpStream::connect(addr))
                .await
                .is_ok();

            let info = if is_open { PortInfo::open(port) } else { PortInfo::closed(port) };
            if let Ok(mut c) = cb.lock() { c(info); }
            info
        }));
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(info) = handle.await {
            if info.is_open {
                results.push(info);
            }
        }
    }
    results
}

// ═══════════════════════════════════════════════════════════
//  SYN 扫描（Unix: 原始套接字）
// ═══════════════════════════════════════════════════════════

/// 检查原始套接字是否可用
pub fn check_syn_available() -> bool {
    #[cfg(unix)] {
        use pnet::transport::{transport_channel, TransportChannelType, TransportProtocol};
        use pnet::packet::ip::IpNextHeaderProtocols;
        transport_channel(4096, TransportChannelType::Layer4(
            TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)
        )).is_ok()
    }
    #[cfg(windows)] { false }  // Windows 不支持原始 TCP 套接字
}

/// SYN 扫描主机（Unix 专用）
#[cfg(unix)]
pub fn syn_scan_host<F>(
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
    ports: &[u16],
    callback: F,
    timeout: Duration,
) -> Result<(usize, usize, usize, usize), String>
where F: FnMut(PortInfo) + Send + 'static {
    use pnet::packet::ip::IpNextHeaderProtocols;
    use pnet::packet::tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags};
    use pnet::transport::{tcp_packet_iter, transport_channel, TransportChannelType, TransportProtocol};
    use std::collections::HashMap;

    let (mut sender, mut receiver) = transport_channel(
        65536,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
    ).map_err(|e| format!("打开原始套接字失败 (需root权限): {}", e))?;

    // 每端口分配唯一源端口+序列号
    let mut rng = Lcg::new();
    let mut probe: HashMap<u16, (u16, u32, u8)> = HashMap::with_capacity(ports.len());
    // status: 0=未响应, 1=开放, 2=关闭
    for &port in ports {
        let sp = rng.port();
        let seq = rng.next_u32();
        probe.insert(sp, (port, seq, 0u8));
    }

    // ── 批量发送 SYN ──
    let send_start = Instant::now();
    for (&sp, &(dp, seq, _)) in &probe {
        let mut buf = [0u8; 20];
        if let Some(mut tcp) = MutableTcpPacket::new(&mut buf[..]) {
            tcp.set_source(sp);
            tcp.set_destination(dp);
            tcp.set_sequence(seq);
            tcp.set_acknowledgement(0);
            tcp.set_data_offset(5);
            tcp.set_flags(TcpFlags::SYN);
            tcp.set_window(65535);
            tcp.set_checksum(0);
            tcp.set_urgent_ptr(0);
            let ck = ipv4_checksum(&tcp.to_immutable(), &source_ip, &target_ip);
            tcp.set_checksum(ck);
            let _ = sender.send_to(&tcp.to_immutable(), IpAddr::V4(target_ip));
        }
    }

    // ── 收集响应 ──
    let send_elapsed = send_start.elapsed();
    let deadline = Instant::now()
        + if timeout > send_elapsed { timeout - send_elapsed } else { Duration::from_millis(500) };

    // 设置套接字接收超时
    pnet_sys::set_socket_receive_timeout(receiver.socket.fd, Duration::from_millis(200)).ok();

    let mut iter = tcp_packet_iter(&mut receiver);
    loop {
        if Instant::now() >= deadline { break; }
        match iter.next() {
            Ok((pkt, src)) => {
                if src != IpAddr::V4(target_ip) { continue; }
                let flags = pkt.get_flags();
                let dst_port = pkt.get_destination();
                let ack_seq = pkt.get_acknowledgement();
                if let Some(entry) = probe.get_mut(&dst_port) {
                    if entry.2 != 0 { continue; }
                    let (tport, sseq, _) = *entry;
                    if (flags & TcpFlags::SYN != 0 && flags & TcpFlags::ACK != 0
                        && ack_seq == sseq.wrapping_add(1))
                    {
                        entry.2 = 1;
                        callback(PortInfo::open(tport));
                    } else if flags & TcpFlags::RST != 0 {
                        entry.2 = 2;
                        callback(PortInfo::closed(tport));
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(_) => break,
        }
    }

    // filtered 端口回调
    let (mut oc, mut cc, mut fc) = (0usize, 0usize, 0usize);
    for (_, (port, _, st)) in &probe {
        match st { 1 => oc += 1, 2 => cc += 1, _ => { fc += 1; callback(PortInfo::closed(*port)); } }
    }
    Ok((ports.len(), oc, cc, fc))
}

/// Windows 占位 — 始终回退 Connect 扫描
#[cfg(windows)]
pub fn syn_scan_host<F>(
    _source_ip: Ipv4Addr, _target_ip: Ipv4Addr,
    _ports: &[u16], _callback: F,
    _timeout: Duration,
) -> Result<(usize, usize, usize, usize), String>
where F: FnMut(PortInfo) + Send + 'static {
    Err("Windows 不支持原始 TCP 套接字, 请使用 Connect 扫描".into())
}

/// 获取本机 IP（到目标的 UDP/TCP 连接确定）
pub fn get_source_ip(target: Ipv4Addr) -> Option<Ipv4Addr> {
    let sock = TcpStream::connect_timeout(
        &SocketAddr::new(IpAddr::V4(target), 80), Duration::from_millis(50));
    match sock {
        Ok(s) => s.local_addr().ok().and_then(|a| match a {
            SocketAddr::V4(v4) => Some(*v4.ip()), _ => None
        }),
        Err(_) => {
            // 枚举本机所有非回环 IP
            for iface in pnet::datalink::interfaces() {
                for ip in iface.ips {
                    if let IpAddr::V4(v4) = ip.ip() {
                        if !v4.is_loopback() { return Some(v4); }
                    }
                }
            }
            None
        }
    }
}

/// 智能扫描入口 — 自动选择最优方法
pub fn scan_host<F>(ip: Ipv4Addr, ports: &[u16], callback: F) -> (Vec<PortInfo>, ScanMethod)
where F: FnMut(PortInfo) + Send + Sync + 'static {
    #[cfg(unix)]
    if check_syn_available() {
        let src = get_source_ip(ip).unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let cb = Mutex::new(&mut callback);
        match syn_scan_host(src, ip, ports, |info| {
            if let Ok(mut c) = cb.lock() { c(info); }
        }, Duration::from_secs(5)) {
            Ok((_total, _open, _closed, _filtered)) => {
                return (Vec::new(), ScanMethod::Syn);
            }
            Err(_e) => { /* fallback to connect */ }
        }
    }

    let results = connect_scan_host(ip, ports, callback);
    (results, ScanMethod::Connect)
}
