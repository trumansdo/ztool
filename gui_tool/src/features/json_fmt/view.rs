//! JSON 格式化 —— 视图层
//!
//! 渲染 JSON 格式化工具的 UI 界面，包括输入框、错误提示、输出框和操作按钮。
//!
//! # iced view 函数模式
//! 每个功能模块的 view 函数返回 `(Element, Vec<Layered>)`：
//! - `Element`: 主要内容（输入框、按钮等）
//! - `Vec<Layered>`: 叠加层（Toast 通知等浮动元素）
//! 这种设计让每个模块可以独立管理自己的 Toast，无需顶层介入。

use crate::features::theme;
use crate::ui::widgets::{toast, overlay::Layered};
use iced::widget::{button, column, container, text, text_editor, Row};
use iced::Element;
use iced::Length;

use super::{JsonFormatter, Msg};

/// 渲染 JSON 格式化界面
///
/// # Rust: 函数返回元组
/// `-> (Element<'_, Msg>, Vec<Layered<'_, Msg>>)`
/// 返回一个二元组。Rust 的元组用 `(type1, type2, ...)` 表示，
/// 可以有 0~12 个元素（超过12个需要用 struct）。
///
/// # Rust: 生命周期省略 (elision)
/// `Element<'_, Msg>` 中的 `'_` 是匿名生命周期。
/// 编译器根据"生命周期省略规则"自动推断：
/// 输入引用 `&formatter` 的生命周期绑定到输出的 Element 生命周期。
/// 相当于说"返回的 Element 借用了 formatter 的数据"。
pub fn view(formatter: &JsonFormatter) -> (Element<'_, Msg>, Vec<Layered<'_, Msg>>) {
    // 输入编辑器
    //
    // `text_editor(&formatter.input)` 创建一个多行文本编辑器，
    // 绑定到 `formatter.input` 的内容。
    // `.on_action(Msg::InputChanged)` 当用户编辑时发送 InputChanged 消息。
    //
    // 由于 iced 0.14 的 text_editor 只接受 Pixels 宽度，
    // 这里用外层 container 设置 Length::Fill 实现自适应宽度。
    let input_editor = container(
        text_editor(&formatter.input)
            .on_action(Msg::InputChanged)
            .padding(theme::padding2(0.5, 1.0))
            .height(Length::Fixed(150.0))
            .placeholder("输入JSON..."),
    )
    .width(Length::Fill);

    // 错误提示 —— 只有发生 JSON 解析错误时才显示
    //
    // Rust: `if let` 条件匹配
    // `if let Some(ref e) = formatter.error` 匹配 Option 的 Some 变体，
    // 并借用到内部值。`ref` 关键字表示通过引用绑定（而非移动）。
    // 也可以写 `if let Some(e) = &formatter.error`（match ergonomics 自动解引用）。
    let error_text = if let Some(ref e) = formatter.error {
        text(e.to_string())
            .color([1.0, 0.3, 0.3])  // 红褐色文字
            .size(theme::font(0.9))
    } else {
        // 错误为空时显示空白占位，保持布局稳定
        text("").size(theme::font(0.9))
    };

    // 输出编辑器 —— 格式化结果或空白
    //
    // `text_editor(&formatter.output)` 绑定到输出内容。
    // `on_action(Msg::OutputChanged)` 虽然输出框理论上不需要编辑，
    // 但 iced 的 text_editor 需要绑定 action 才能正常工作。
    let output_editor = container(
        text_editor(&formatter.output)
            .on_action(Msg::OutputChanged)
            .padding(theme::padding2(0.5, 1.0))
            .height(Length::Fixed(200.0)),
    )
    .width(Length::Fill);

    // 按钮行：格式化 | 空白间隔 | 复制结果 | 空白间隔 | 清空
    //
    // Rust: `Row::with_children(vec![...])` —— 从 Vec 构建 Row
    // `iced` 也提供 `row![...]` 宏，语法更简洁。
    let btn_row = Row::with_children(vec![
        button("格式化")
            .on_press(Msg::Format)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::primary)
            .into(),
        // 用空白 text 做按钮间间隔（spacing 也可以，但手动控制更灵活）
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

    // 辅助闭包：创建空白间隔 text
    //
    // Rust: 闭包语法 `|param| expression`
    // 这是一个简单的闭包，接收字体大小，返回一个空白 text 元素。
    let sp = |size| text("").size(size);

    // 主容器：垂直排列所有元素
    //
    // `column![...]` 是 iced 的宏，创建垂直布局。
    // `.spacing()` 设置子元素间距。
    // `.padding()` 设置内边距。
    // `.style(container::transparent)` 使容器背景透明（不遮挡暗色主题）。
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
    .style(container::transparent)
    .into();

    // 生成 Toast 叠加层（如果没有 toast 则返回空 Vec）
    let overlays = toast::view_toasts(&formatter.toasts, Msg::AutoDismiss);
    (content, overlays)
}
