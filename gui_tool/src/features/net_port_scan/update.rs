use std::fmt;
use thiserror::Error;
use iced::Task;

use super::scan::{self, PortInfo};

/// 扫描错误类型
#[derive(Debug, Clone, Error)]
pub enum ScanError {
    #[error("无效的IP地址: {0}")]
    InvalidIp(String),
    #[error("无效的网段格式: {0}")]
    InvalidCidr(String),
}

/// 端口扫描模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScanMode {
    #[default]
    Common,    // 常用端口 (21,22,80等)
    Top100,    // Top 100端口
    All,       // 全部端口 (1-1024)
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
    /// 获取该模式对应的端口列表
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
    
    /// 获取模式显示标签
    pub fn label(&self) -> &'static str {
        match self {
            ScanMode::Common => "常用端口",
            ScanMode::Top100 => "Top 100",
            ScanMode::All => "全部端口",
        }
    }
}

/// 端口扫描器状态存储
#[derive(Default)]
pub struct NetScanner {
    pub target: String,                      // 目标IP/网段
    pub results: Vec<String>,               // 扫描结果文本
    pub logs: Vec<String>,                  // 扫描日志
    pub scan_mode: ScanMode,               // 扫描模式
    pub is_scanning: bool,                 // 是否正在扫描
    pub open_ports: Vec<(String, Vec<PortInfo>)>, // 发现的开放端口
}

impl NetScanner {
    /// 创建新的扫描器实例
    pub fn new() -> Self {
        Self::default()
    }
}

/// 端口扫描器消息
#[derive(Debug, Clone)]
pub enum Msg {
    TargetChanged(String),                         // 目标输入变化
    ScanModeChanged(ScanMode),                    // 扫描模式变化
    StartScan,                                    // 开始扫描
    Clear,                                       // 清空结果
    ScanProgress(String),                        // 扫描进度日志
    ScanResult(Vec<(String, Vec<PortInfo>)>, Vec<String>), // 扫描完成结果
    ScanError(String),                           // 扫描错误
}

/// 处理端口扫描器消息，更新状态
pub fn update(scanner: &mut NetScanner, msg: Msg) -> Task<Msg> {
    match msg {
        // 更新目标IP/网段
        Msg::TargetChanged(s) => {
            scanner.target = s;
            Task::none()
        }
        // 更新扫描模式
        Msg::ScanModeChanged(mode) => {
            scanner.scan_mode = mode;
            Task::none()
        }
        // 开始扫描：初始化扫描状态，记录日志
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
        // 处理扫描完成：格式化结果输出
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
        // 处理扫描错误
        Msg::ScanError(e) => {
            scanner.is_scanning = false;
            scanner.results.push(format!("扫描失败: {}", e));
            scanner.logs.push(format!("[x] 扫描失败: {}", e));
            Task::none()
        }
        // 更新扫描进度日志
        Msg::ScanProgress(log) => {
            scanner.logs.push(log);
            Task::none()
        }
        // 清空所有状态
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