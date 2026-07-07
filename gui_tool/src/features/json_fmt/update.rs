//! JSON 格式化 —— 状态管理和消息处理
//!
//! 处理 JSON 格式化相关的所有消息：输入变化、格式化、复制、清空。

use crate::ui::widgets::toast::{Toast, ToastLevel, ToastPosition};
use thiserror::Error;
use iced::widget::text_editor;

#[derive(Default)]
pub struct JsonFormatter {
    pub input: text_editor::Content,
    pub output: text_editor::Content,
    pub error: Option<JsonFmtError>,
    pub toasts: Vec<Toast>,
}

#[derive(Debug, Clone, Error)]
pub enum JsonFmtError {
    #[error("JSON解析错误: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub enum Msg {
    InputChanged(text_editor::Action),
    OutputChanged(text_editor::Action),
    Format,
    Clear,
    CopyOutput,
    CloseToast(usize),
}

pub fn update(formatter: &mut JsonFormatter, msg: Msg) -> iced::Task<Msg> {
    match msg {
        Msg::InputChanged(action) => {
            formatter.input.perform(action);
            iced::Task::none()
        }
        Msg::OutputChanged(action) => {
            formatter.output.perform(action);
            iced::Task::none()
        }
        Msg::Format => {
            let text = formatter.input.text().to_string();
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(value) => {
                    let pretty = serde_json::to_string_pretty(&value).unwrap_or_default();
                    formatter.output = text_editor::Content::with_text(&pretty);
                    formatter.error = None;
                }
                Err(e) => {
                    formatter.error = Some(JsonFmtError::ParseError(e.to_string()));
                    formatter.output = text_editor::Content::default();
                }
            }
            iced::Task::none()
        }
        Msg::Clear => {
            formatter.input = text_editor::Content::default();
            formatter.output = text_editor::Content::default();
            formatter.error = None;
            iced::Task::none()
        }
        Msg::CopyOutput => {
            let text = formatter.output.text().to_string();
            formatter.toasts.push(Toast {
                level: ToastLevel::Success,
                text: "已复制到剪贴板".to_string(),
                position: ToastPosition::TopRight,
            });
            iced::clipboard::write(text)
        }
        Msg::CloseToast(index) => {
            formatter.toasts.remove(index);
            iced::Task::none()
        }
    }
}
