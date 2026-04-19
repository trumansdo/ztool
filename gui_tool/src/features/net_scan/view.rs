use crate::features::theme;
use iced::widget::{button, column, container, row, text, text_input};
use iced::Element;

use super::{Msg, NetScanner};

pub fn view(scanner: &NetScanner) -> Element<'_, Msg> {
    use iced::Length;

    let input = text_input("目标IP/网段", &scanner.target)
        .on_input(Msg::TargetChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    let mut results_col = column![text("结果:").size(theme::font(1.0))].spacing(theme::size(0.57).0 as u32);
    for r in &scanner.results {
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