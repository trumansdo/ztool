//! # 网络抓包状态管理与消息处理
//!
//! 定义抓包功能的数据模型（PacketCapture）、消息类型（Msg）和状态转换逻辑。
//!
//! ## Rust 概念 — struct 与 Default trait
//! `#[derive(Default)]` 自动生成 `Default::default()` 实现，
//! 等价于 `PacketCapture { interface: "", filter: "", ... }`。
//! 要求所有字段也实现了 Default（String 默认是空字符串，Vec 默认是空向量）。
//!
//! ## Rust 概念 — enum 与数据承载
//! `InterfaceChanged(String)` 不是简单的枚举值，而是携带数据的变体。
//! Rust 的 enum 是代数数据类型（ADT），每个变体可以携带不同类型和数量的数据，
//! 这是 Rust 相比 C/Java enum 最强大的特性之一。

use iced::Task;

use super::parser::{self, PacketInfo};

/// 网络抓包器的完整状态
///
/// ## Rust 概念 — pub 字段
/// `pub` 使字段对外部模块可访问。与 Java/C# 不同，Rust 中 pub 是字段级别的，
/// 结构体本身也要 pub 才能被外部使用。
#[derive(Default)]
pub struct PacketCapture {
    /// 网络接口名称（如 "eth0", "en0"），留空使用系统默认
    pub interface: String,
    /// BPF 过滤器表达式（Berkeley Packet Filter 语法）
    pub filter: String,
    /// 是否正在抓包中
    pub is_capturing: bool,
    /// 已捕获的数据包列表（最多保留 100 个）
    pub packets: Vec<PacketInfo>,
    /// 累计捕获总数（包括被丢弃的超出 100 个的部分）
    pub packet_count: usize,
}

impl PacketCapture {
    /// 将捕获的数据包格式化为可显示的字符串列表
    ///
    /// ## Rust 概念 — 不可变引用 `&self`
    /// 表示该方法借用 self 的只读引用，不修改结构体数据。
    /// 这是 Rust 所有权系统的核心 — 通过引用控制访问权限。
    ///
    /// ## Rust 概念 — 迭代器 enumerate
    /// `.iter().enumerate()` 返回 `(索引, 引用)` 的迭代器，
    /// 使用模式匹配 `for (i, packet) in ...` 解构元组。
    /// 类似 Python 的 `enumerate()`，但更类型安全。
    ///
    /// ## Rust 概念 — 显示限制
    /// `i >= 99` — 只显示前 100 个数据包（索引从 0 开始），
    /// 防止 UI 因数据过多而卡顿。
    /// `break` 提前退出循环。
    pub fn format_packets(&self) -> Vec<String> {
        // Vec::new() — 创建空向量，堆分配，容量从 0 开始
        let mut lines = Vec::new();
        
        for (i, packet) in self.packets.iter().enumerate() {
            lines.push(format!("[{}] {}", i + 1, parser::format_packet_info(packet)));
            if i >= 99 {
                lines.push(format!("... 共 {} 个数据包，仅显示前100个", self.packet_count));
                break;
            }
        }
        
        // 如果正在抓包但还没有数据包到达
        if lines.is_empty() && self.is_capturing {
            lines.push("正在捕获数据包...".to_string());
        }
        
        lines
    }
}

/// 抓包功能的所有可能消息
///
/// ## Rust 概念 — Clone trait
/// iced 要求消息实现 Clone，因为事件可能被复制到多个 widget。
/// 这里必须 #[derive(Clone)]，不像 JsonFmt 的 Msg 使用 thiserror 自动实现。
#[derive(Debug, Clone)]
pub enum Msg {
    /// 网络接口名称变更（携带新的接口名字符串）
    InterfaceChanged(String),
    /// BPF 过滤器变更（携带新的过滤器字符串）
    FilterChanged(String),
    /// 开始抓包
    StartCapture,
    /// 停止抓包
    StopCapture,
    /// 清空所有捕获数据
    Clear,
    /// 收到新数据包（携带解析后的数据包信息）
    NewPacket(PacketInfo),
}

/// 处理消息，更新状态
///
/// ## Rust 概念 — 可变引用 `&mut self`
/// `capture: &mut PacketCapture` 是对状态的独占可变引用。
/// Rust 的借用规则保证：同一时间只能有一个可变引用，
/// 从而在编译期杜绝数据竞争。
///
/// ## Rust 概念 — Task
/// `Task<Msg>` 是 iced 的命令类型，表示需要异步执行的操作。
/// `Task::none()` 表示没有异步命令。
///
/// ## Rust 概念 — 滑动窗口
/// `capture.packets.len() < 100` 实现了简单的滑动窗口：
/// 只保留最近的 100 个数据包，但 packet_count 继续累加以反映真实总数。
pub fn update(capture: &mut PacketCapture, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::InterfaceChanged(s) => {
            capture.interface = s;
            Task::none()
        }
        Msg::FilterChanged(s) => {
            capture.filter = s;
            Task::none()
        }
        Msg::StartCapture => {
            capture.is_capturing = true;
            // Vec::clear() — 清空向量但保留已分配的内存（capacity 不变）
            capture.packets.clear();
            capture.packet_count = 0;
            Task::none()
        }
        Msg::StopCapture => {
            capture.is_capturing = false;
            Task::none()
        }
        Msg::Clear => {
            capture.packets.clear();
            capture.packet_count = 0;
            capture.is_capturing = false;
            Task::none()
        }
        Msg::NewPacket(info) => {
            capture.packet_count += 1;  // += 1 自增运算符
            // 只保留最近 100 个包，避免内存无限增长
            if capture.packets.len() < 100 {
                capture.packets.push(info);
            }
            Task::none()
        }
    }
}
