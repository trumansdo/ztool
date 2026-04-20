use crate::features::theme;
use iced::widget::{button, column, container, text, text_editor, Row};
use iced::Pixels;
use iced::Element;

use super::{JsonFormatter, Msg};

/// 渲染JSON格式化器的视图 UI
pub fn view(formatter: &JsonFormatter) -> Element<'_, Msg> {
    // 输入编辑器：接收用户输入的JSON字符串
    let input_editor = text_editor(&formatter.input)
        .on_action(Msg::InputChanged)
        .padding(theme::padding2(0.5, 1.0))
        .width(Pixels(400.0))
        .height(Pixels(150.0))
        .placeholder("输入JSON...");

    // 错误提示：显示解析错误信息
    let error_text = if let Some(ref e) = formatter.error {
        text(e.to_string())
            .color([1.0, 0.3, 0.3])
            .size(theme::font(0.9))
    } else {
        text("").size(theme::font(0.9))
    };

    // 输出标签
    let output_field = text("输出结果")
        .size(theme::font(0.9));

    // 按钮行：格式化/清空按钮
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

    // 间距辅助函数
    let sp = |size| text("").size(size);

    // 输出显示区域：显示格式化后的JSON
    let output_display = container(
        text(&formatter.output)
            .size(theme::font(0.9))
    )
    .padding(theme::padding2(0.5, 1.0))
    .width(Pixels(400.0))
    .height(Pixels(150.0));

    // 组装主面板：垂直布局所有元素
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