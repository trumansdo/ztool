use crate::features::theme;
use iced::widget::{button, column, container, text, text_editor, Row};
use iced::Pixels;
use iced::Element;

use super::{JsonFormatter, Msg};

pub fn view(formatter: &JsonFormatter) -> Element<'_, Msg> {
    let input_editor = text_editor(&formatter.input)
        .on_action(Msg::InputChanged)
        .padding(theme::padding2(0.5, 1.0))
        .width(Pixels(400.0))
        .height(Pixels(150.0))
        .placeholder("输入JSON...");

    let error_text = if let Some(ref e) = formatter.error {
        text(e.to_string())
            .color([1.0, 0.3, 0.3])
            .size(theme::font(0.9))
    } else {
        text("").size(theme::font(0.9))
    };

    let output_field = text("输出结果")
        .size(theme::font(0.9));

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

    let output_display = container(
        text(&formatter.output)
            .size(theme::font(0.9))
    )
    .padding(theme::padding2(0.5, 1.0))
    .width(Pixels(400.0))
    .height(Pixels(150.0));

    container(
        column![
            text("JSON格式化").size(theme::font(1.1)),
            sp(theme::font(0.3)),
            input_editor,
            sp(theme::font(0.3)),
            error_text,
            sp(theme::font(0.3)),
            output_field,
            output_display,
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