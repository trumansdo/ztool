//! AI提示词生成器 —— 视图层
//!
//! 渲染提示词列表 + 编辑区 + 预览区的完整 UI。

use iced::widget::{
    button, column, container, row, scrollable, text, text_editor, text_input,
};
use iced::{Color, Element, Length};

use crate::features::theme;
use crate::theme::ThemeStyles;

use super::update::{Msg, PromptGenerator};

pub fn view<'a>(pg: &'a PromptGenerator, styles: &'a dyn ThemeStyles) -> Element<'a, Msg> {
    let t = styles.tokens();

    // ========== 左侧：提示词列表 ==========
    let list_header = row![
        text("提示词列表").size(theme::font(1.0)),
        button("+ 新建")
            .on_press(Msg::NewPrompt)
            .padding(theme::padding2(0.2, 0.5))
            .style(button::primary),
    ]
    .spacing(theme::size(0.5).0);

    let prompt_items = iced::widget::Column::with_children(
        pg.store
            .prompts
            .iter()
            .map(|prompt| {
                let is_selected = pg.selected_id.as_deref() == Some(&prompt.id);
                button(text(&prompt.title).size(theme::font(0.9)))
                    .on_press(Msg::SelectPrompt(prompt.id.clone()))
                    .padding(theme::padding2(0.2, 0.4))
                    .style(move |_theme, _status| {
                        if is_selected {
                            iced::widget::button::Style {
                                background: Some(iced::Background::Color(t.accent)),
                                text_color: t.text_primary,
                                ..Default::default()
                            }
                        } else {
                            iced::widget::button::Style {
                                text_color: t.text_secondary,
                                ..Default::default()
                            }
                        }
                    })
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(2);

    let left_panel = container(
        column![list_header, scrollable(prompt_items).height(Length::Fill)]
            .spacing(theme::size(0.5).0),
    )
    .padding(theme::size(0.5).0)
    .style(panel_style(t));

    // ========== 右侧：编辑区 ==========
    let title_row = row![
        text("标题:").size(theme::font(0.9)),
        text_input("输入提示词标题...", &pg.title_input)
            .on_input(Msg::TitleChanged)
            .padding(theme::padding2(0.2, 0.4))
            .width(Length::Fill),
    ]
    .spacing(theme::size(0.3).0)
    .align_y(iced::Alignment::Center);

    let action_row = row![
        button("保存")
            .on_press(Msg::Save)
            .style(button::secondary)
            .padding(theme::padding2(0.2, 0.5)),
        button("删除")
            .on_press(Msg::DeletePrompt)
            .style(button::secondary)
            .padding(theme::padding2(0.2, 0.5)),
        button("复制预览")
            .on_press(Msg::CopyPreview)
            .style(button::primary)
            .padding(theme::padding2(0.2, 0.5)),
        button("刷新预览")
            .on_press(Msg::RefreshPreview)
            .style(button::primary)
            .padding(theme::padding2(0.2, 0.5)),
    ]
    .spacing(theme::size(0.3).0);

    fn section_style(t: &crate::theme::Tokens) -> impl Fn(&iced::Theme) -> iced::widget::text::Style + '_ {
        let color = t.text_secondary;
        move |_theme: &iced::Theme| iced::widget::text::Style {
            color: Some(color),
        }
    }

    fn section_label<'a>(label: &'a str, t: &'a crate::theme::Tokens) -> Element<'a, Msg> {
        iced::Element::from(
            text(label)
                .size(theme::font(0.85))
                .style(section_style(t)),
        )
    }

    let user_editor = text_editor(&pg.user_input)
        .on_action(Msg::UserInputChanged)
        .height(Length::Fixed(120.0))
        .style(move |_theme, _status| text_editor::Style {
            background: iced::Background::Color(t.surface),
            border: iced::Border {
                color: t.border,
                width: 1.0,
                radius: t.radius_sm.into(),
            },
            placeholder: t.text_secondary,
            value: t.text_primary,
            selection: t.accent,
        });

    let template_editor = text_editor(&pg.template_input)
        .on_action(Msg::TemplateInputChanged)
        .height(Length::Fixed(120.0))
        .style(move |_theme, _status| text_editor::Style {
            background: iced::Background::Color(t.surface),
            border: iced::Border {
                color: t.border,
                width: 1.0,
                radius: t.radius_sm.into(),
            },
            placeholder: t.text_secondary,
            value: t.text_primary,
            selection: t.accent,
        });

    // 预览区
    let preview_header = row![
        text("预览结果").size(theme::font(0.9)),
        button("复制")
            .on_press(Msg::CopyPreview)
            .style(button::secondary)
            .padding(theme::padding2(0.1, 0.3)),
    ]
    .spacing(theme::size(0.3).0);

    fn placeholder_style(t: &crate::theme::Tokens) -> impl Fn(&iced::Theme) -> iced::widget::text::Style + '_ {
        let color = t.text_secondary;
        move |_theme: &iced::Theme| iced::widget::text::Style {
            color: Some(color),
        }
    }

    let preview_content: Element<'_, Msg> = if pg.preview.is_empty() {
        iced::Element::from(
            text("点击「刷新预览」查看结果")
                .size(theme::font(0.85))
                .style(placeholder_style(t)),
        )
    } else {
        iced::Element::from(text(&pg.preview).size(theme::font(0.85)))
    };

    let preview_box = container(scrollable(preview_content).height(Length::Fixed(150.0)))
        .padding(theme::size(0.5).0)
        .style(preview_style(t));

    // 错误提示
    let error_bar: Element<'_, Msg> = if let Some(ref err) = pg.error {
        container(
            iced::Element::from(
                text(err)
                    .size(theme::font(0.8))
                    .style({
                        let color = t.danger;
                        move |_theme: &iced::Theme| iced::widget::text::Style {
                            color: Some(color),
                        }
                    }),
            ),
        )
        .padding(theme::size(0.3).0)
        .style({
            let danger = t.danger;
            let radius = t.radius_sm;
            move |_theme: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color {
                    r: danger.r,
                    g: danger.g,
                    b: danger.b,
                    a: 0.1,
                })),
                border: iced::Border {
                    color: danger,
                    width: 1.0,
                    radius: radius.into(),
                },
                ..Default::default()
            }
        })
        .into()
    } else {
        iced::widget::Column::new().into()
    };

    let editor_content = column![
        title_row,
        section_label("用户输入区（用 #变量名 定义变量）", t),
        user_editor,
        section_label("固定模板（用 ${变量名} 引用变量）", t),
        template_editor,
        action_row,
        error_bar,
        preview_header,
        preview_box,
    ]
    .spacing(theme::size(0.3).0);

    let right_panel = container(editor_content)
        .padding(theme::size(0.5).0)
        .style(panel_style(t));

    // ========== 整体布局 ==========
    row![
        left_panel.width(Length::Fixed(220.0)),
        right_panel.width(Length::Fill),
    ]
    .spacing(theme::size(0.5).0)
    .height(Length::Fill)
    .into()
}

/// 面板背景样式
fn panel_style(t: &crate::theme::Tokens) -> impl Fn(&iced::Theme) -> iced::widget::container::Style + '_ {
    let bg = t.surface;
    move |_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(bg)),
        ..Default::default()
    }
}

/// 预览区背景样式
fn preview_style(t: &crate::theme::Tokens) -> impl Fn(&iced::Theme) -> iced::widget::container::Style + '_ {
    let bg = t.background;
    let border_color = t.border;
    let text_color = t.text_primary;
    move |_theme: &iced::Theme| iced::widget::container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        text_color: Some(text_color),
        ..Default::default()
    }
}
