use iced::Task;

use super::parser::{self, PacketInfo};

#[derive(Default)]
pub struct PacketCapture {
    pub interface: String,
    pub filter: String,
    pub is_capturing: bool,
    pub packets: Vec<PacketInfo>,
    pub packet_count: usize,
}

impl PacketCapture {
    pub fn new() -> Self {
        Self::default()
    }
    
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

#[derive(Debug, Clone)]
pub enum Msg {
    InterfaceChanged(String),
    FilterChanged(String),
    StartCapture,
    StopCapture,
    Clear,
    NewPacket(PacketInfo),
}

pub fn update(capture: &mut PacketCapture, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::InterfaceChanged(s) => capture.interface = s,
        Msg::FilterChanged(s) => capture.filter = s,
        Msg::StartCapture => {
            capture.is_capturing = true;
            capture.packets.clear();
            capture.packet_count = 0;
        }
        Msg::StopCapture => {
            capture.is_capturing = false;
        }
        Msg::Clear => {
            capture.packets.clear();
            capture.packet_count = 0;
            capture.is_capturing = false;
        }
        Msg::NewPacket(info) => {
            capture.packet_count += 1;
            if capture.packets.len() < 100 {
                capture.packets.push(info);
            }
        }
    }
    Task::none()
}