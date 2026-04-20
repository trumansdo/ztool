use iced::Task;

use super::parser::{self, PacketInfo};

/// 网络数据包捕获器状态存储
#[derive(Default)]
pub struct PacketCapture {
    pub interface: String,               // 网络接口名称
    pub filter: String,                   // BPF过滤器
    pub is_capturing: bool,               // 是否正在捕获
    pub packets: Vec<PacketInfo>,          // 捕获的数据包列表
    pub packet_count: usize,              // 总捕获数量
}

impl PacketCapture {
    /// 创建新的捕获器实例
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 格式化数据包为显示文本
    pub fn format_packets(&self) -> Vec<String> {
        let mut lines = Vec::new();
        
        for (i, packet) in self.packets.iter().enumerate() {
            lines.push(format!("[{}] {}", i + 1, parser::format_packet_info(packet)));
            if i >= 99 {
                lines.push(format!("... 共 {} 个数据包，仅显示前100个", self.packet_count));
                break;
            }
        }
        
        if lines.is_empty() && self.is_capturing {
            lines.push("正在捕获数据包...".to_string());
        }
        
        lines
    }
}

/// 网络抓包消息
#[derive(Debug, Clone)]
pub enum Msg {
    InterfaceChanged(String),      // 网络接口变化
    FilterChanged(String),      // 过滤器变化
    StartCapture,              // 开始捕获
    StopCapture,              // 停止捕获
    Clear,                   // 清空
    NewPacket(PacketInfo),     // 新捕获数据包
}

/// 处理网络抓包消息，更新状态
pub fn update(capture: &mut PacketCapture, msg: Msg) -> Task<Msg> {
    match msg {
        // 更新网络接口
        Msg::InterfaceChanged(s) => capture.interface = s,
        // 更新过滤器
        Msg::FilterChanged(s) => capture.filter = s,
        // 开始捕获：初始化捕获状态
        Msg::StartCapture => {
            capture.is_capturing = true;
            capture.packets.clear();
            capture.packet_count = 0;
        }
        // 停止捕获
        Msg::StopCapture => {
            capture.is_capturing = false;
        }
        // 清空：重置所有状态
        Msg::Clear => {
            capture.packets.clear();
            capture.packet_count = 0;
            capture.is_capturing = false;
        }
        // 添加新捕获的数据包（仅保留前100个）
        Msg::NewPacket(info) => {
            capture.packet_count += 1;
            if capture.packets.len() < 100 {
                capture.packets.push(info);
            }
        }
    }
    Task::none()
}