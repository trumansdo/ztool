use thiserror::Error;
use iced::widget::text_editor;

#[derive(Default)]
pub struct JsonFormatter {
    pub input: text_editor::Content,
    pub output: String,
    pub error: Option<JsonFmtError>,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Error)]
pub enum JsonFmtError {
    #[error("JSON解析错误: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone)]
pub enum Msg {
    InputChanged(text_editor::Action),
    Format,
    Clear,
}

pub fn update(formatter: &mut JsonFormatter, msg: Msg) -> Result<iced::Task<Msg>, JsonFmtError> {
    match msg {
        Msg::InputChanged(action) => {
            formatter.input.perform(action);
        }
        Msg::Format => {
            let text = formatter.input.text().to_string();
            let value = serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|e| JsonFmtError::ParseError(e.to_string()))?;
            formatter.output = serde_json::to_string_pretty(&value)
                .map_err(|e| JsonFmtError::ParseError(e.to_string()))?;
            formatter.error = None;
        }
        Msg::Clear => {
            formatter.input = text_editor::Content::default();
            formatter.output.clear();
            formatter.error = None;
        }
    }
    Ok(iced::Task::none())
}