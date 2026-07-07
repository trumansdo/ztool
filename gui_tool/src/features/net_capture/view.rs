//! # 网络抓包视图模块

use crate::features::theme;
use iced::widget::{button, column, container, row, text, text_input};
use iced::Element;
use iced::Length;

use super::{Msg, PacketCapture};

pub fn view(capture: &PacketCapture) -> Element<'_, Msg> {
    // text_input 是 iced 的文本输入组件
    // .on_input(Msg::InterfaceChanged) — 将输入变化事件映射为消息
    // .width(Length::Fill) — 宽度填满父容器
    let interface_input = text_input("网络接口 (留空使用默认)", &capture.interface)
        .on_input(Msg::InterfaceChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    // BPF (Berkeley Packet Filter) 过滤器，用于筛选特定协议/端口的数据包
    // 例如 "tcp port 80" 只捕获 HTTP 流量
    let filter_input = text_input("BPF过滤器 (如: tcp, udp, port 80)", &capture.filter)
        .on_input(Msg::FilterChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    // 格式化数据包列表为字符串向量
    // Vec<String> 是一个堆分配的动态数组，每个元素是可变的 String
    let packet_lines = capture.format_packets();
    // if/else 表达式 — Rust 中 if 是表达式，可以直接赋值
    let results_text = if packet_lines.is_empty() {
        "点击\"开始\"启动抓包".to_string()
    } else {
        // join("\n\n") — 用双换行连接所有字符串
        packet_lines.join("\n\n")
    };

    // column! 宏创建垂直布局
    // format! 宏进行字符串插值（类似 C 的 sprintf / Python 的 f-string）
    let results_col = column![
        text(format!("捕获的数据包 (共 {} 个):", capture.packet_count)).size(theme::font(1.0)),
        container(
            text(results_text)
                .size(theme::font(0.85))
        )
        .height(Length::Fill)  // 高度填满剩余空间
        .padding(theme::padding(0.5))
    ];

    // row! 宏创建水平布局
    // 条件按钮渲染：根据 is_capturing 状态切换按钮文本和样式
    // button::danger — 危险操作样式（红色）
    // button::primary — 主要操作样式（蓝色）
    // button::secondary — 次要操作样式（灰色）
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
    .spacing(theme::size(0.86).0 as u32);  // as u32 — 类型强制转换，f32 → u32

    // 主容器：把标题、输入框、按钮、结果区域组合在一起
    let content = container(
        column![
            text("网络抓包").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),  // 用空文本作为间隔
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
    .into();  // .into() 将 container 转换为 Element（利用 From trait）

    content
}
