use thiserror::Error;
use iced::widget::text_editor;

/// JSON格式化器状态存储
#[derive(Default)]
pub struct JsonFormatter {
    pub input: text_editor::Content,      // 用户输入的JSON字符串
    pub output: String,                   // 格式化后的输出
    pub error: Option<JsonFmtError>,       // 错误信息
}

impl JsonFormatter {
    /// 创建新的格式化器实例
    pub fn new() -> Self {
        Self::default()
    }
}

/// JSON格式化错误类型
#[derive(Debug, Clone, Error)]
pub enum JsonFmtError {
    #[error("JSON解析错误: {0}")]
    ParseError(String),
}

/// JSON格式化器消息
#[derive(Debug, Clone)]
pub enum Msg {
    InputChanged(text_editor::Action),  // 输入内容变化
    Format,                              // 执行格式化
    Clear,                               // 清空输入输出
}

/// 处理JSON格式化器消息，更新状态
pub fn update(formatter: &mut JsonFormatter, msg: Msg) -> Result<iced::Task<Msg>, JsonFmtError> {
    match msg {
        // 处理输入内容变化事件
        Msg::InputChanged(action) => {
            formatter.input.perform(action);
        }
        // 执行JSON格式化：将输入解析为JSON值并格式化为字符串
        Msg::Format => {
            let text = formatter.input.text().to_string();
            let value = serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|e| JsonFmtError::ParseError(e.to_string()))?;
            formatter.output = serde_json::to_string_pretty(&value)
                .map_err(|e| JsonFmtError::ParseError(e.to_string()))?;
            formatter.error = None;
        }
        // 清空：重置输入输出和错误状态
        Msg::Clear => {
            formatter.input = text_editor::Content::default();
            formatter.output.clear();
            formatter.error = None;
        }
    }
    Ok(iced::Task::none())
}