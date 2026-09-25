//! AI提示词生成器 —— 视图层
//!
//! 渲染提示词列表 + 编辑区 + 预览区的完整 UI。

use iced::widget::{
    button, column, container, row, scrollable, text, text_editor, text_input, Column,
};
use iced::{Color, Element, Length};

use crate::features::theme;

use super::update::{Msg, PromptGenerator};

pub fn view(pg: &PromptGenerator) -> Element<'_, Msg> {
    // ========== 左侧：提示词列表 ==========
    let list_header = row![
        text("提示词列表").size(theme::font(1.0)),
        button("+ 新建")
            .on_press(Msg::NewPrompt)
            .padding(theme::padding2(0.2, 0.5))
            .style(button::primary),
    ]
    .spacing(theme::size(0.5).0 as u32);

    let mut list_items = Column::new().spacing(2);
    for prompt in &pg.store.prompts {
        let is_selected = pg.selected_id.as_deref() == Some(&prompt.id);
        let label = if prompt.title.is_empty() {
            "(无标题)"
        } else {
            &prompt.title
        };

        let item_btn = button(
            text(label)
                .size(theme::font(0.85))
                .width(Length::Fill),
        )
        .on_press(Msg::SelectPrompt(prompt.id.clone()))
        .padding(theme::padding2(0.3, 0.5))
        .width(Length::Fill)
        .style(move |_theme, status| {
            if is_selected {
                button::primary(_theme, status)
            } else {
                button::text(_theme, status)
            }
        });

        list_items = list_items.push(item_btn);
    }

    let list_scroll = scrollable(list_items)
        .width(Length::Fill)
        .height(Length::Fill);

    let left_panel = container(
        column![
            list_header,
            list_scroll,
        ]
        .spacing(theme::size(0.3).0 as u32),
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fill)
    .padding(theme::padding(0.5))
    .style(panel_style);

    // ========== 右侧：编辑区 ==========
    let empty_editor = pg.selected_id.is_none();

    // 删除按钮：无选中时不可点击
    let delete_btn = if empty_editor {
        button("删除")
            .padding(theme::padding2(0.2, 0.5))
            .style(button::secondary)
    } else {
        button("删除")
            .on_press(Msg::DeletePrompt)
            .padding(theme::padding2(0.2, 0.5))
            .style(button::secondary)
    };

    // 保存按钮：无选中时不可点击
    let save_btn = if empty_editor {
        button("保存")
            .padding(theme::padding2(0.2, 0.5))
            .style(button::primary)
    } else {
        button("保存")
            .on_press(Msg::Save)
            .padding(theme::padding2(0.2, 0.5))
            .style(button::primary)
    };

    let title_row = row![
        text("标题").size(theme::font(0.9)),
        text("")
            .size(theme::font(1.0))
            .width(Length::Fill),
        delete_btn,
        save_btn,
    ]
    .spacing(theme::size(0.5).0 as u32);

    // 标题输入框（使用 text_input）
    let title_input = text_input("输入提示词标题...", &pg.title_input)
        .on_input(Msg::TitleChanged)
        .padding(theme::padding2(0.5, 0.8));

    // 用户输入区
    let user_label = text("用户输入区（#变量名 定义变量，${input} 引用全文）")
        .size(theme::font(0.85))
        .style(|_| text::Style {
            color: Some(Color::from_rgb8(150, 150, 200)),
        });

    let user_editor = text_editor(&pg.user_input)
        .on_action(Msg::UserInputChanged)
        .padding(theme::padding2(0.5, 1.0))
        .height(Length::Fill)
        .placeholder(
            "#role\n你是一个专业的AI助手\n#task\n请帮我完成以下任务\n\n使用 #变量名 定义变量，内容紧跟其后",
        );

    // 固定模板区
    let template_label = text("固定模板区（${变量名} / ${input} / ${user_prompt} 引用）")
        .size(theme::font(0.85))
        .style(|_| text::Style {
            color: Some(Color::from_rgb8(150, 150, 200)),
        });

    let template_editor = text_editor(&pg.template_input)
        .on_action(Msg::TemplateInputChanged)
        .padding(theme::padding2(0.5, 1.0))
        .height(Length::Fill)
        .placeholder(
            "你是${role}。\n\n${task}\n\n用户输入：${user_prompt}\n\n原始输入：${input}\n\n请给出你的回答：",
        );

    // 预览区
    let preview_header = row![
        text("预览结果").size(theme::font(0.9)),
        text("")
            .size(theme::font(1.0))
            .width(Length::Fill),
        button("复制")
            .on_press(Msg::CopyPreview)
            .padding(theme::padding2(0.2, 0.5))
            .style(button::secondary),
    ]
    .spacing(theme::size(0.5).0 as u32);

    let preview_content = if pg.preview.is_empty() {
        text("预览内容将在此显示...")
            .size(theme::font(0.85))
            .style(|_| text::Style {
                color: Some(Color::from_rgb8(120, 120, 140)),
            })
    } else {
        text(&pg.preview).size(theme::font(0.85))
    };

    let preview_box = container(scrollable(preview_content).height(Length::Fixed(150.0)))
        .padding(theme::padding2(0.5, 1.0))
        .style(preview_style);

    // 错误提示
    let error_display = if let Some(ref err) = pg.error {
        text(err)
            .size(theme::font(0.8))
            .style(|_| text::Style {
                color: Some(Color::from_rgb8(255, 80, 80)),
            })
    } else {
        text("").size(theme::font(0.8))
    };

    let editor_content = if empty_editor {
        container(
            text("请新建或选择一个提示词模板")
                .size(theme::font(1.0))
                .style(|_| text::Style {
                    color: Some(Color::from_rgb8(120, 120, 140)),
                }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
    } else {
        container(
            column![
                title_row,
                title_input,
                text("").size(theme::font(0.3)),
                user_label,
                user_editor,
                text("").size(theme::font(0.3)),
                template_label,
                template_editor,
                text("").size(theme::font(0.3)),
                preview_header,
                preview_box,
                error_display,
            ]
            .spacing(theme::size(0.2).0 as u32),
        )
        .padding(theme::padding(0.8))
    };

    let right_panel = container(editor_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(panel_style);

    // ========== 整体布局 ==========
    row![left_panel, right_panel]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// 面板背景样式
fn panel_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(iced::Background::Color(palette.background.base.color)),
        ..Default::default()
    }
}

/// 预览区背景样式
fn preview_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgb8(20, 20, 35))),
        border: iced::Border {
            color: Color::from_rgb8(60, 60, 80),
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(palette.background.base.text),
        ..Default::default()
    }
}
