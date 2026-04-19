use std::fmt;
use thiserror::Error;
use iced::Task;

use super::scan::{self, PortInfo};

#[derive(Debug, Clone, Error)]
pub enum ScanError {
    #[error("无效的IP地址: {0}")]
    InvalidIp(String),
    #[error("无效的网段格式: {0}")]
    InvalidCidr(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScanMode {
    #[default]
    Common,
    Top100,
    All,
}

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
    pub fn ports(&self) -> Vec<u16> {
        match self {
            ScanMode::Common => vec![
                21, 22, 23, 25, 53, 80, 110, 143, 443, 465, 587, 993, 995,
                3306, 3389, 5432, 5900, 6379, 8080, 8443, 27017,
            ],
            ScanMode::Top100 => (1..=100).collect(),
            ScanMode::All => (1..=1024).collect(),
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            ScanMode::Common => "常用端口",
            ScanMode::Top100 => "Top 100",
            ScanMode::All => "全部端口",
        }
    }
}

#[derive(Default)]
pub struct NetScanner {
    pub target: String,
    pub results: Vec<String>,
    pub logs: Vec<String>,
    pub scan_mode: ScanMode,
    pub is_scanning: bool,
    pub open_ports: Vec<(String, Vec<PortInfo>)>,
}

impl NetScanner {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    TargetChanged(String),
    ScanModeChanged(ScanMode),
    StartScan,
    Clear,
    ScanProgress(String),
    ScanResult(Vec<(String, Vec<PortInfo>)>, Vec<String>),
    ScanError(String),
}

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
            
            scanner.is_scanning = true;
            scanner.results.clear();
            scanner.logs.clear();
            scanner.open_ports.clear();
            scanner.results.push(format!("开始扫描: {} ({})", target, mode.label()));
            scanner.logs.push(format!("[*] 开始扫描目标: {} (模式: {})", target, mode.label()));
            
            let ports = mode.ports();
            let port_count = ports.len();
            scanner.logs.push(format!("[*] 共 {} 个端口需要扫描", port_count));
            
            Task::none()
        }
        Msg::ScanResult(ports, scan_logs) => {
            scanner.is_scanning = false;
            let host_count = ports.len();
            let total_open_ports: usize = ports.iter().map(|(_, p)| p.len()).sum();
            scanner.open_ports = ports;
            scanner.logs = scan_logs;
            scanner.logs.push(format!("[*] 扫描完成! 发现 {} 个主机, 共 {} 个开放端口", host_count, total_open_ports));
            
            if scanner.open_ports.is_empty() {
                scanner.results.push("未发现开放端口".to_string());
            } else {
                for (ip, port_infos) in &scanner.open_ports {
                    scanner.results.push(format!("\n{}:", ip));
                    for p in port_infos {
                        let service = p.service.unwrap_or("未知");
                        scanner.results.push(format!(
                            "  端口 {} 开放 - {}",
                            p.port, service
                        ));
                    }
                }
                scanner.results.push(format!("\n扫描完成，共发现 {} 个主机", host_count));
            }
            Task::none()
        }
        Msg::ScanError(e) => {
            scanner.is_scanning = false;
            scanner.results.push(format!("扫描失败: {}", e));
            scanner.logs.push(format!("[x] 扫描失败: {}", e));
            Task::none()
        }
        Msg::ScanProgress(log) => {
            scanner.logs.push(log);
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