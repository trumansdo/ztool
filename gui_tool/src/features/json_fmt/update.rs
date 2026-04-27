//! JSON 格式化 —— 状态管理和消息处理
//!
//! 处理 JSON 格式化相关的所有消息：输入变化、格式化、复制、清空。
//!
//! # Rust 语法要点
//!
//! ## `thiserror` 派生宏
//! `#[derive(Error)]` 是 `thiserror` crate 提供的派生宏，
//! 自动为枚举生成 `Display` 和 `Error` trait 的实现。
//! `#[error("JSON解析错误: {0}")]` 定义 Display 格式，`{0}` 引用第 0 个字段。
//!
//! ## `serde_json` 序列化
//! - `serde_json::from_str::<Value>(&text)`: 反序列化 JSON 字符串为通用 Value 类型
//! - `serde_json::to_string_pretty(&value)`: 将 Value 序列化为美化格式的 JSON 字符串
//! - `::<Value>` 是 turbofish 语法，显式指定泛型参数的类型
//!
//! ## `iced::Task` —— 异步任务抽象
//! `Task::none()`: 不执行任何操作的"空任务"
//! `Task::perform(future, mapper)`: 执行异步操作（如 tokio 定时器），完成后发送消息
//! `Task::run(stream_factory, mapper)`: 创建异步流（用于实时进度报告）
//!
//! ## `clipboard::write(text)` —— 剪贴板操作
//! iced 提供的剪贴板写入 Task，将文本复制到系统剪贴板。
//!
//! ## `tokio::time::sleep` —— 异步休眠
//! 非阻塞的休眠，3 秒后通过 Task 发送 AutoDismiss 消息来消除 Toast。
//! 与 `std::thread::sleep` 不同，tokio::time::sleep 不阻塞线程。

use crate::ui::widgets::toast::{ToastItem, ToastLevel, ToastPosition};
use thiserror::Error;
use iced::widget::text_editor;

/// JSON 格式化器状态
///
/// # Rust: `#[derive(Default)]`
/// 自动实现 Default trait，所有字段用类型的默认值初始化。
/// - `text_editor::Content::default()` 创建空文本内容
/// - `Option<JsonFmtError>::default()` 返回 None
/// - `Vec<ToastItem>::default()` 返回空 Vec
/// - `u64::default()` 返回 0
///
/// # 字段可见性
/// `pub` 字段可从外部模块读写（view 函数需要读取，update 函数需要写入）。
/// 这是 iced 的常见模式 —— 状态结构体的字段通常是 pub 的。
#[derive(Default)]
pub struct JsonFormatter {
    /// 输入编辑器的文本内容（iced text_editor 专用类型）
    pub input: text_editor::Content,
    /// 格式化输出编辑器的文本内容
    pub output: text_editor::Content,
    /// JSON 解析错误信息，None 表示无错误
    pub error: Option<JsonFmtError>,
    /// 当前显示的 Toast 列表
    pub toasts: Vec<ToastItem>,
    /// Toast ID 自增计数器（私有，外部不需要直接访问）
    next_toast_id: u64,
}

impl JsonFormatter {
    /// 添加一个 Toast 并返回其 ID
    ///
    /// # Rust: `&mut self` 可变借用
    /// 方法可以修改结构体字段。`self` 的不同形式：
    /// - `&self` —— 不可变借用（只读）
    /// - `&mut self` —— 可变借用（可修改）
    /// - `self` —— 获取所有权（消耗自身）
    ///
    /// # Rust: `u64` 溢出
    /// Rust 在 debug 模式下检查整数溢出并 panic，
    /// 在 release 模式下 wrap around（循环）。这里 ID 自增不太可能溢出。
    fn push_toast(&mut self, level: ToastLevel, text: String, position: ToastPosition) -> u64 {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toasts.push(ToastItem {
            id,
            level,
            text,
            position,
        });
        id
    }
}

/// JSON 格式化错误类型
///
/// # Rust: `#[derive(Error)]` —— thiserror 派生宏
/// thiserror 简化了自定义错误类型的实现。
/// `#[error("JSON解析错误: {0}")]` 生成 Display 实现，
/// 使用 `{0}` 引用第 0 个字段的值（类似 format! 语法）。
#[derive(Debug, Clone, Error)]
pub enum JsonFmtError {
    /// JSON 解析失败，携带错误描述
    #[error("JSON解析错误: {0}")]
    ParseError(String),
}

/// JSON 格式化模块的消息类型
///
/// # Rust: 携带数据的枚举变体
/// `InputChanged(text_editor::Action)` —— 携带 text_editor 的 Action 类型。
/// Action 是 iced 的文本编辑操作（插入、删除、选择等）的枚举。
#[derive(Debug, Clone)]
pub enum Msg {
    /// 用户编辑输入框时触发，携带编辑操作
    InputChanged(text_editor::Action),
    /// 输出框文本变化（通常不需要处理，但 iced 要求绑定）
    OutputChanged(text_editor::Action),
    /// 格式化按钮点击 → 解析 JSON 并美化输出
    Format,
    /// 清空按钮点击 → 重置所有内容
    Clear,
    /// 复制结果按钮点击 → 写入剪贴板 + 显示 Toast
    CopyOutput,
    /// 自动关闭 Toast，携带要关闭的 Toast ID
    AutoDismiss(u64),
}

/// 处理消息，更新 JSON 格式化器状态
///
/// # Rust: match 的控制流
/// 每个分支处理一种消息类型，最后用 `Task::none()` 表示同步操作完成。
/// `CopyOutput` 分支返回非空 Task（剪贴板写入 + 定时器），
/// 除此之外都是同步操作，不需要异步 Task。
pub fn update(formatter: &mut JsonFormatter, msg: Msg) -> iced::Task<Msg> {
    match msg {
        // `action` 绑定到 InputChanged 携带的 Action
        Msg::InputChanged(action) => {
            // `content.perform(action)` 执行编辑操作
            formatter.input.perform(action);
            iced::Task::none()
        }
        Msg::OutputChanged(action) => {
            formatter.output.perform(action);
            iced::Task::none()
        }
        Msg::Format => {
            // `content.text()` 获取编辑器中的原始文本引用
            let text = formatter.input.text().to_string();
            // serde_json 解析：
            // `::<serde_json::Value>` turbofish 语法显式指定解析目标类型
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(value) => {
                    // 反序列化成功，美化输出
                    let pretty = serde_json::to_string_pretty(&value).unwrap_or_default();
                    // `Content::with_text(&str)` 创建带初始文本的内容
                    formatter.output = text_editor::Content::with_text(&pretty);
                    formatter.error = None;
                }
                Err(e) => {
                    // 反序列化失败，显示错误信息
                    formatter.error = Some(JsonFmtError::ParseError(e.to_string()));
                    formatter.output = text_editor::Content::default();
                }
            }
            iced::Task::none()
        }
        Msg::Clear => {
            // `Content::default()` 创建空内容
            formatter.input = text_editor::Content::default();
            formatter.output = text_editor::Content::default();
            formatter.error = None;
            iced::Task::none()
        }
        Msg::CopyOutput => {
            let text = formatter.output.text().to_string();
            // 显示成功 Toast（右上角，3 秒后自动消失）
            let id = formatter.push_toast(ToastLevel::Success, "已复制到剪贴板".to_string(), ToastPosition::TopRight);
            // Task 链式组合:
            // 1. clipboard::write(text) → 写入剪贴板
            // 2. chain → 写入完成后执行后续 Task
            // 3. Task::perform(sleep(3s), |_| AutoDismiss(id)) → 3秒后发送消除消息
            iced::clipboard::write(text)
                .chain(iced::Task::perform(
                    tokio::time::sleep(std::time::Duration::from_secs(3)),
                    move |_| Msg::AutoDismiss(id),
                ))
        }
        Msg::AutoDismiss(id) => {
            // `Vec::retain(|t| t.id != id)` 保留 id 不等于指定值的元素
            // 即移除指定 id 的 Toast
            formatter.toasts.retain(|t| t.id != id);
            iced::Task::none()
        }
    }
}
