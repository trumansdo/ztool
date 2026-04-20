use crate::features::theme;
use iced::widget::{button, column, container, row, text, text_input};
use iced::Element;
use iced::Length;

use super::{Msg, PacketCapture};

/// 渲染网络抓包的视图 UI
pub fn view(capture: &PacketCapture) -> Element<'_, Msg> {
    // 网络接口输入框
    let interface_input = text_input("网络接口 (留空使用默认)", &capture.interface)
        .on_input(Msg::InterfaceChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    // BPF过滤器输入框
    let filter_input = text_input("BPF过滤器 (如: tcp, udp, port 80)", &capture.filter)
        .on_input(Msg::FilterChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    // 格式化数据包显示文本
    let packet_lines = capture.format_packets();
    let results_text = if packet_lines.is_empty() {
        "点击\"开始\"启动抓包".to_string()
    } else {
        packet_lines.join("\n\n")
    };

    // 捕获结果区域
    let results_col = column![
        text(format!("捕获的数据包 (共 {} 个):", capture.packet_count)).size(theme::font(1.0)),
        container(
            text(results_text)
                .size(theme::font(0.85))
        )
        .height(Length::Fill)
        .padding(theme::padding(0.5))
    ];

    // 操作按钮行：根据捕获状态切换开始/停止
    let btn_row = row![
        if capture.is_capturing {
            button("停止")
                .on_press(Msg::StopCapture)
                .padding(theme::padding2(0.36, 0.5))
                .style(button::danger)
        } else {
            button("开始")
                .on_press(Msg::StartCapture)
                .padding(theme::padding2(0.36, 0.5))
                .style(button::primary)
        },
        button("清空")
            .on_press(Msg::Clear)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::secondary),
    ]
    .spacing(theme::size(0.86).0 as u32);

    // 组装主面板
    container(
        column![
            text("网络抓包").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),
            interface_input,
            filter_input,
            text("").size(theme::font(0.3)),
            btn_row,
            text("").size(theme::font(0.5)),
            results_col,
        ]
        .spacing(theme::size(0.5).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0))
    .height(Length::Fill)
    .into()
}