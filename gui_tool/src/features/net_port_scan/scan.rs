use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortInfo {
    pub port: u16,
    pub is_open: bool,
    pub service: Option<&'static str>,
}

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

pub fn get_service_name(port: u16) -> Option<&'static str> {
    COMMON_PORTS.iter().find(|(p, _)| *p == port).map(|(_, name)| *name)
}

#[derive(Debug)]
pub enum ScanError {
    InvalidIp(String),
    InvalidCidr(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::InvalidIp(s) => write!(f, "无效的IP地址: {}", s),
            ScanError::InvalidCidr(s) => write!(f, "无效的网段格式: {}", s),
        }
    }
}

impl std::error::Error for ScanError {}

pub fn parse_ip_range(input: &str) -> Result<Vec<Ipv4Addr>, ScanError> {
    let input = input.trim();
    
    if input.contains('/') {
        let cidr: Vec<&str> = input.split('/').collect();
        if cidr.len() != 2 {
            return Err(ScanError::InvalidCidr(input.to_string()));
        }
        
        let ip: Ipv4Addr = cidr[0].parse()
            .map_err(|_| ScanError::InvalidIp(cidr[0].to_string()))?;
        
        let prefix: u8 = cidr[1].parse()
            .map_err(|_| ScanError::InvalidCidr(input.to_string()))?;
        
        if prefix > 32 {
            return Err(ScanError::InvalidCidr(input.to_string()));
        }
        
        let mask = !((1u32 << (32 - prefix)) - 1);
        let network = u32::from(ip) & mask;
        let broadcast = network | !mask;
        
        let mut results = Vec::new();
        for addr in (network + 1)..broadcast {
            results.push(Ipv4Addr::from(addr));
        }
        
        Ok(results)
    } else if input.contains('-') {
        let range: Vec<&str> = input.split('-').collect();
        if range.len() != 2 {
            return Err(ScanError::InvalidIp(input.to_string()));
        }
        
        let start: Ipv4Addr = range[0].parse()
            .map_err(|_| ScanError::InvalidIp(range[0].to_string()))?;
        
        let end: Ipv4Addr = range[1].parse()
            .map_err(|_| ScanError::InvalidIp(range[1].to_string()))?;
        
        let start_u32 = u32::from(start);
        let end_u32 = u32::from(end);
        
        if start_u32 > end_u32 {
            return Err(ScanError::InvalidIp(input.to_string()));
        }
        
        let mut results = Vec::new();
        for addr in start_u32..=end_u32 {
            results.push(Ipv4Addr::from(addr));
        }
        
        Ok(results)
    } else {
        let ip: Ipv4Addr = input.parse()
            .map_err(|_| ScanError::InvalidIp(input.to_string()))?;
        Ok(vec![ip])
    }
}

pub fn scan_port(ip: Ipv4Addr, port: u16, timeout: Duration) -> PortInfo {
    let addr = SocketAddr::new(IpAddr::V4(ip), port);
    let connect_start = std::time::Instant::now();
    let is_open = TcpStream::connect_timeout(&addr, timeout).is_ok();
    let elapsed = connect_start.elapsed();
    
    if is_open {
        eprintln!("[debug] port {} open, took {:?}", port, elapsed);
    }
    
    PortInfo {
        port,
        is_open,
        service: if is_open { get_service_name(port) } else { None },
    }
}

pub fn scan_host(ip: Ipv4Addr, ports: &[u16]) -> Vec<PortInfo> {
    let timeout = Duration::from_millis(500);
    ports.iter()
        .map(|&port| scan_port(ip, port, timeout))
        .filter(|p| p.is_open)
        .collect()
}