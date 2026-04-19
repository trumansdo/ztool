use crate::features::theme;
use iced::widget::{button, column, container, row, text, text_input};
use iced::Element;

use super::{Msg, PacketCapture};

pub fn view(capture: &PacketCapture) -> Element<'_, Msg> {
    use iced::Length;

    let interface_input = text_input("网络接口", &capture.interface)
        .on_input(Msg::InterfaceChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    let filter_input = text_input("BPF过滤器", &capture.filter)
        .on_input(Msg::FilterChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    let mut results_col = column![text("捕获的数据包:").size(theme::font(1.0))].spacing(theme::size(0.57).0 as u32);
    for r in &capture.results {
        results_col = results_col.push(text(r.as_str()).size(theme::font(1.0)));
    }

    let btn_row = row![
        button("开始")
            .on_press(Msg::StartCapture)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::primary),
        button("停止")
            .on_press(Msg::StopCapture)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::danger),
    ]
    .spacing(theme::size(0.86).0 as u32);

    container(
        column![
            text("网络抓包").size(theme::font(1.0)),
            text("").size(theme::font(0.86)),
            interface_input,
            filter_input,
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