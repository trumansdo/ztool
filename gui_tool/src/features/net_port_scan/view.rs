use crate::features::theme;
use crate::features::net_port_scan::{ScanMode, Msg, NetScanner};
use iced::widget::{button, column, container, row, text, text_input, pick_list};
use iced::Element;
use iced::Length;

pub fn view(scanner: &NetScanner) -> Element<'_, Msg> {
    let input = text_input("目标IP/网段 (如: 192.168.1.1, 192.168.1.1-10, 192.168.1.0/24)", &scanner.target)
        .on_input(Msg::TargetChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    let modes: Vec<ScanMode> = vec![ScanMode::Common, ScanMode::Top100, ScanMode::All];
    let mode_list = pick_list(
        modes,
        Some(scanner.scan_mode),
        Msg::ScanModeChanged,
    );

    let scan_btn = if scanner.is_scanning {
        button("扫描中...")
            .padding(theme::padding2(0.36, 0.5))
            .style(button::secondary)
    } else {
        button("扫描")
            .on_press(Msg::StartScan)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::primary)
    };

    let results_text = if scanner.results.is_empty() {
        "暂无结果".to_string()
    } else {
        scanner.results.join("\n")
    };

    let results_col = column![
        text("扫描结果:").size(theme::font(1.0)),
        container(
            text(results_text)
                .size(theme::font(0.9))
        )
        .height(Length::Fill)
        .padding(theme::padding(0.5))
    ];

    let logs_text = if scanner.logs.is_empty() {
        "等待扫描...".to_string()
    } else {
        scanner.logs.join("\n")
    };

    let logs_col = column![
        text("扫描日志:").size(theme::font(1.0)),
        container(
            text(logs_text)
                .size(theme::font(0.8))
        )
        .height(Length::Fixed(150.0))
        .padding(theme::padding(0.5))
    ];

    let btn_row = row![
        scan_btn,
        button("清空")
            .on_press(Msg::Clear)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::secondary),
    ]
    .spacing(theme::size(0.86).0 as u32);

    container(
        column![
            text("端口扫描").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),
            input,
            text("").size(theme::font(0.3)),
            row![mode_list, btn_row]
                .spacing(theme::size(0.86).0 as u32),
            text("").size(theme::font(0.5)),
            results_col,
            text("").size(theme::font(0.3)),
            logs_col,
        ]
        .spacing(theme::size(0.5).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0))
    .height(Length::Fill)
    .into()
}