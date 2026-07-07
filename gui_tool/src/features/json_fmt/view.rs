//! JSON 格式化 —— 视图层
//!
//! 渲染 JSON 格式化工具的 UI 界面。

use crate::features::theme;
use crate::ui::widgets::Toaster;
use iced::widget::{button, column, container, text, text_editor, Row};
use iced::Element;
use iced::Length;

use super::{JsonFormatter, Msg};

pub fn view(formatter: &JsonFormatter) -> Element<'_, Msg> {
    let input_editor = container(
        text_editor(&formatter.input)
            .on_action(Msg::InputChanged)
            .padding(theme::padding2(0.5, 1.0))
            .height(Length::Fixed(150.0))
            .placeholder("输入JSON..."),
    )
    .width(Length::Fill);

    let error_text = if let Some(ref e) = formatter.error {
        text(e.to_string())
            .color([1.0, 0.3, 0.3])
            .size(theme::font(0.9))
    } else {
        text("").size(theme::font(0.9))
    };

    let output_editor = container(
        text_editor(&formatter.output)
            .on_action(Msg::OutputChanged)
            .padding(theme::padding2(0.5, 1.0))
            .height(Length::Fixed(200.0)),
    )
    .width(Length::Fill);

    let btn_row = Row::with_children(vec![
        button("格式化")
            .on_press(Msg::Format)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::primary)
            .into(),
        text("")
            .size(theme::font(1.0))
            .into(),
        button("复制结果")
            .on_press(Msg::CopyOutput)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::secondary)
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

    let content = container(
        column![
            text("JSON格式化").size(theme::font(1.1)),
            sp(theme::font(0.3)),
            input_editor,
            sp(theme::font(0.3)),
            error_text,
            sp(theme::font(0.3)),
            text("输出结果").size(theme::font(0.9)),
            output_editor,
            sp(theme::font(0.5)),
            btn_row,
        ]
        .spacing(theme::size(0.2).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0))
    .style(container::transparent);

    Toaster::new(content, &formatter.toasts, Msg::CloseToast).into()
}
