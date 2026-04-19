use crate::features::theme;
use iced::widget::{button, column, container, text, text_input, Row};
use iced::{Element, Length, Task};

#[derive(Default)]
pub struct JsonFormatter {
    pub input: String,
    pub output: String,
    pub error: Option<String>,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    InputChanged(String),
    Format,
    Clear,
}

pub fn update(formatter: &mut JsonFormatter, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::InputChanged(s) => formatter.input = s,
        Msg::Format => match serde_json::from_str::<serde_json::Value>(&formatter.input) {
            Ok(v) => {
                formatter.output = serde_json::to_string_pretty(&v).unwrap_or_default();
                formatter.error = None;
            }
            Err(e) => {
                formatter.error = Some(e.to_string());
                formatter.output.clear();
            }
        },
        Msg::Clear => {
            formatter.input.clear();
            formatter.output.clear();
            formatter.error = None;
        }
    }
    Task::none()
}