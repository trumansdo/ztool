use iced::widget::{button, column, container, text, text_input, Row};
use iced::{Element, Length, Task};

use crate::features::theme;

#[derive(Default)]
pub struct JsonFormatter {
    input: String,
    output: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    InputChanged(String),
    Format,
    Clear,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::InputChanged(s) => self.input = s,
            Msg::Format => match serde_json::from_str::<serde_json::Value>(&self.input) {
                Ok(v) => {
                    self.output = serde_json::to_string_pretty(&v).unwrap_or_default();
                    self.error = None;
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                    self.output.clear();
                }
            },
            Msg::Clear => {
                self.input.clear();
                self.output.clear();
                self.error = None;
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let input_field = text_input("输入JSON...", &self.input)
            .on_input(Msg::InputChanged)
            .padding(theme::padding2(0.5, 1.0))
            .width(Length::Fill)
            .size(theme::font(1.0));

        let error_text = if let Some(ref e) = self.error {
            text(e)
                .color([1.0, 0.3, 0.3])
                .size(theme::font(0.9))
        } else {
            text("").size(theme::font(0.9))
        };

        let output_field = text_input("输出结果", &self.output)
            .padding(theme::padding2(0.5, 1.0))
            .width(Length::Fill)
            .size(theme::font(1.0));

        let btn_row = Row::with_children(vec![
            button("格式化")
                .on_press(Msg::Format)
                .padding(theme::padding2(0.36, 0.5))
                .style(button::primary)
                .into(),
            text("")
                .size(theme::font(1.0))
                .into(),
            button("清空")
                .on_press(Msg::Clear)
                .padding(theme::padding2(0.36, 0.5))
                .style(button::secondary)
                .into(),
        ])
        .spacing(theme::size(0.5).0 as u32);

        let sp = |size| text("").size(size);

        container(
            column![
                text("JSON格式化").size(theme::font(1.1)),
                sp(theme::font(0.3)),
                input_field,
                sp(theme::font(0.3)),
                error_text,
                sp(theme::font(0.3)),
                output_field,
                sp(theme::font(0.5)),
                btn_row,
            ]
            .spacing(theme::size(0.2).0 as u32)
            .padding(theme::padding(1.0)),
        )
        .padding(theme::padding(1.0))
        .style(container::transparent)
        .into()
    }
}
