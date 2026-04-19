use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Task};

use crate::features::theme;

#[derive(Default)]
pub struct NetScanner {
    target: String,
    results: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    TargetChanged(String),
    StartScan,
    Clear,
}

impl NetScanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::TargetChanged(s) => self.target = s,
            Msg::StartScan => {
                self.results
                    .push(format!("扫描: {}", self.target));
                self.results
                    .push("端口 22: 开放".to_string());
                self.results
                    .push("端口 80: 开放".to_string());
                self.results
                    .push("端口 443: 开放".to_string());
            }
            Msg::Clear => {
                self.results.clear();
                self.target.clear();
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let input = text_input("目标IP/网段", &self.target)
            .on_input(Msg::TargetChanged)
            .padding(theme::padding2(0.36, 1.0))
            .width(Length::Fill);

        let mut results_col = column![text("结果:").size(theme::font(1.0))].spacing(theme::size(0.57).0 as u32);
        for r in &self.results {
            results_col = results_col.push(text(r.as_str()).size(theme::font(1.0)));
        }

        let btn_row = row![
            button("扫描")
                .on_press(Msg::StartScan)
                .padding(theme::padding2(0.36, 0.5))
                .style(button::primary),
            button("清空")
                .on_press(Msg::Clear)
                .padding(theme::padding2(0.36, 0.5))
                .style(button::secondary),
        ]
        .spacing(theme::size(0.86).0 as u32);

        container(
            column![
                text("网络扫描").size(theme::font(1.0)),
                text("").size(theme::font(0.86)),
                input,
                text("").size(theme::font(0.57)),
                results_col,
                text("").size(theme::font(0.86)),
                btn_row,
            ]
            .spacing(theme::size(0.86).0 as u32)
            .padding(theme::padding(1.0)),
        )
        .padding(theme::padding(1.0))
        .style(container::transparent)
        .into()
    }
}
