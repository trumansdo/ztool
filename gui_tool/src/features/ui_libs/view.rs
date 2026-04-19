use crate::features::theme;
use crate::features::ui_libs::update::ComponentTab;
use iced::widget::{button, column, container, text, toggler, Row};
use iced_aw::widget::Badge;
use iced::Element;

use super::Msg;
use super::UiLibs;

pub fn view(libs: &UiLibs) -> Element<'static, Msg> {
    let tabs = Row::with_children(vec![
        tab_button("Badge", ComponentTab::Badge, libs.selected_tab),
        tab_button("Card", ComponentTab::Card, libs.selected_tab),
        tab_button("Button", ComponentTab::Button, libs.selected_tab),
        tab_button("Toggle", ComponentTab::Toggle, libs.selected_tab),
    ])
    .spacing(theme::size(0.5).0 as u32);

    let content = match libs.selected_tab {
        ComponentTab::Badge => view_badge(),
        ComponentTab::Card => view_card(libs),
        ComponentTab::Button => view_button(libs),
        ComponentTab::Toggle => view_toggle(libs),
    };

    container(
        column![
            text("iced_aw 组件示例").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),
            tabs,
            text("").size(theme::font(0.5)),
            content,
        ]
        .spacing(theme::size(0.5).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0))
    .into()
}

fn tab_button(label: &'static str, tab: ComponentTab, selected: ComponentTab) -> Element<'static, Msg> {
    let is_selected = selected == tab;
    button(label)
        .on_press(Msg::TabSelected(tab))
        .padding(theme::padding2(0.3, 0.5))
        .style(if is_selected {
            button::primary
        } else {
            button::secondary
        })
        .into()
}

fn view_badge() -> Element<'static, Msg> {
    let badges = Row::with_children(vec![
        Badge::new("Primary").into(),
        Badge::new("Success").into(),
        Badge::new("Danger").into(),
        Badge::new("Warning").into(),
    ])
    .spacing(10);

    column![
        text("Badge 徽章组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        badges,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_card(libs: &UiLibs) -> Element<'static, Msg> {
    use iced_aw::widget::Card;

    let card = Card::new(
        text("卡片标题").size(18),
        text(format!("这是卡片内容，点击次数: {}", libs.click_count)).size(14),
    )
    .foot(
        button("点击我")
            .on_press(Msg::ButtonPressed)
            .padding(theme::padding2(0.3, 0.8)),
    );

    column![
        text("Card 卡片组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        card,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_button(libs: &UiLibs) -> Element<'static, Msg> {
    let buttons = column![
        text("普通按钮").size(theme::font(0.9)),
        Row::with_children(vec![
            button("Primary").style(button::primary).into(),
            button("Secondary").style(button::secondary).into(),
            button("Danger").style(button::danger).into(),
        ])
        .spacing(10),
        text("").size(theme::font(0.3)),
        text(format!("点击次数: {}", libs.click_count)).size(theme::font(0.9)),
    ]
    .spacing(theme::size(0.3).0 as u32);

    column![
        text("Button 按钮组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        buttons,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_toggle(libs: &UiLibs) -> Element<'static, Msg> {
    let toggle = toggler(libs.toggle_value)
        .on_toggle(Msg::ToggleChanged);

    let status = if libs.toggle_value { "开启" } else { "关闭" };

    column![
        text("Toggle 开关组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        toggle,
        text("").size(theme::font(0.3)),
        text(format!("状态: {}", status)).size(theme::font(0.9)),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}