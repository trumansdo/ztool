//! # UI 组件库展示视图
//!
//! 展示 iced 和 iced_aw 库中各组件的用法。

use crate::features::theme;
use crate::features::ui_libs::update::ComponentTab;
use iced::widget::{
    button, column, combo_box, container, float, pane_grid, pick_list,
    row, scrollable, text, toggler, tooltip, Pin,
};
use iced::Element;

use super::Msg;
use super::UiLibs;
use crate::ui::widgets::toast::{ToastLevel, ToastPosition};
use crate::ui::widgets::Toaster;

/// 主视图：渲染 sub-tab 切换栏 + 对应的组件展示内容 + Toast
pub fn view(libs: &UiLibs) -> Element<'_, Msg> {
    let tabs = row![
        tab_button("Badge", ComponentTab::Badge, libs.selected_tab),
        tab_button("Card", ComponentTab::Card, libs.selected_tab),
        tab_button("Button", ComponentTab::Button, libs.selected_tab),
        tab_button("Toggle", ComponentTab::Toggle, libs.selected_tab),
        tab_button("Separator", ComponentTab::Separator, libs.selected_tab),
        tab_button("Tab", ComponentTab::Tab, libs.selected_tab),
        tab_button("NumberInput", ComponentTab::NumberInput, libs.selected_tab),
        tab_button("Spinner", ComponentTab::Spinner, libs.selected_tab),
        tab_button("Wrap", ComponentTab::Wrap, libs.selected_tab),
        tab_button("Split", ComponentTab::Split, libs.selected_tab),
        tab_button("Toast", ComponentTab::Toast, libs.selected_tab),
        tab_button("Color", ComponentTab::ColorPicker, libs.selected_tab),
        tab_button("Date", ComponentTab::DatePicker, libs.selected_tab),
        tab_button("Tooltip", ComponentTab::Tooltip, libs.selected_tab),
        tab_button("PickList", ComponentTab::PickList, libs.selected_tab),
        tab_button("ComboBox", ComponentTab::ComboBox, libs.selected_tab),
        tab_button("Float", ComponentTab::Float, libs.selected_tab),
        tab_button("Pin", ComponentTab::Pin, libs.selected_tab),
        tab_button("Table", ComponentTab::Table, libs.selected_tab),
        tab_button("PaneGrid", ComponentTab::PaneGrid, libs.selected_tab),
    ]
    .spacing(theme::size(0.3).0 as u32);

    let content = match libs.selected_tab {
        ComponentTab::Badge => view_badge(),
        ComponentTab::Card => view_card(libs),
        ComponentTab::Button => view_button(libs),
        ComponentTab::Toggle => view_toggle(libs),
        ComponentTab::Separator => view_separator(),
        ComponentTab::Tab => view_tab(libs),
        ComponentTab::NumberInput => view_number_input(libs),
        ComponentTab::Spinner => view_spinner(),
        ComponentTab::Wrap => view_wrap(),
        ComponentTab::Split => view_split(),
        ComponentTab::Toast => view_toast(),
        ComponentTab::ColorPicker => view_color_picker(libs),
        ComponentTab::DatePicker => view_date_picker(),
        ComponentTab::Tooltip => view_tooltip(),
        ComponentTab::PickList => view_pick_list(libs),
        ComponentTab::ComboBox => view_combo_box(libs),
        ComponentTab::Float => view_float(),
        ComponentTab::Pin => view_pin(),
        ComponentTab::Table => view_table(),
        ComponentTab::PaneGrid => view_pane_grid(libs),
    };

    let content = container(
        column![
            text("iced UI 组件示例").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),
            scrollable(row![tabs]).horizontal(),
            text("").size(theme::font(0.5)),
            content,
        ]
        .spacing(theme::size(0.5).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0));

    Toaster::new(content, &libs.toasts, Msg::CloseToast).into()
}

fn tab_button(label: &'static str, tab: ComponentTab, selected: ComponentTab) -> Element<'static, Msg> {
    let is_selected = selected == tab;
    button(label)
        .on_press(Msg::TabSelected(tab))
        .padding(theme::padding2(0.25, 0.4))
        .style(if is_selected {
            button::primary
        } else {
            button::secondary
        })
        .into()
}

fn view_badge() -> Element<'static, Msg> {
    use iced_aw::widget::Badge;

    let badges = row![
        Badge::new("Primary"),
        Badge::new("Success"),
        Badge::new("Danger"),
        Badge::new("Warning"),
    ]
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
        row![
            button("Primary").style(button::primary),
            button("Secondary").style(button::secondary),
            button("Danger").style(button::danger),
        ]
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

fn view_separator() -> Element<'static, Msg> {
    use iced::widget::rule;

    column![
        text("Separator 分隔线组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("上面的内容"),
        rule::horizontal(30),
        text("下面的内容"),
        rule::horizontal(30),
        text("再下面的内容"),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_tab(libs: &UiLibs) -> Element<'static, Msg> {
    let tabs = row![
        button("标签1")
            .on_press(Msg::TabSelected(ComponentTab::Tab))
            .style(button::primary),
        button("标签2")
            .on_press(Msg::TabSelected(ComponentTab::Tab)),
        button("标签3")
            .on_press(Msg::TabSelected(ComponentTab::Tab)),
    ]
    .spacing(5);

    let tab_content = column![
        text("Tab 标签页组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text(format!("当前点击次数: {}", libs.click_count)),
        text("这是标签页的内容区域"),
    ]
    .spacing(theme::size(0.3).0 as u32);

    column![
        text("Tab 标签页组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        tabs,
        text("").size(theme::font(0.3)),
        container(tab_content)
            .padding(10),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_number_input(libs: &UiLibs) -> Element<'static, Msg> {
    use iced_aw::widget::NumberInput;

    let float_value = libs.number_value as f64;

    column![
        text("NumberInput 数字输入组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        row![
            NumberInput::new(&float_value, -100.0..=100.0, |value| Msg::NumberChanged(value as i32)),
            text(format!("当前值: {}", libs.number_value)).size(theme::font(0.9)),
        ]
        .spacing(10),
        text("").size(theme::font(0.3)),
        row![
            button("+1").on_press(Msg::NumberChanged(libs.number_value + 1)),
            button("-1").on_press(Msg::NumberChanged(libs.number_value - 1)),
            button("重置").on_press(Msg::NumberChanged(0)),
        ]
        .spacing(10),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_spinner() -> Element<'static, Msg> {
    use iced_aw::widget::Spinner;

    column![
        text("Spinner 加载动画组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        Spinner::new(),
        text("").size(theme::font(0.3)),
        text("加载中...").size(theme::font(0.9)),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_wrap() -> Element<'static, Msg> {
    use iced_aw::widget::Wrap;

    let items = Wrap::with_elements(vec![
        button("按钮 1").into(),
        button("按钮 2").into(),
        button("按钮 3").into(),
        button("按钮 4").into(),
        button("按钮 5").into(),
        button("按钮 6").into(),
        button("按钮 7").into(),
        button("按钮 8").into(),
    ]);

    column![
        text("Wrap 自动换行组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("下面的按钮会自动换行:"),
        text("").size(theme::font(0.3)),
        items,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_split() -> Element<'static, Msg> {
    use iced_split::Split;

    let left_content = column![
        text("左侧面板").size(theme::font(0.9)),
        button("左侧按钮 1"),
        button("左侧按钮 2"),
    ]
    .spacing(5);

    let right_content = column![
        text("右侧面板").size(theme::font(0.9)),
        button("右侧按钮 1"),
        button("右侧按钮 2"),
    ]
    .spacing(5);

    let split = Split::new(
        left_content,
        right_content,
        0.3,
    );

    column![
        text("Split 分屏组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("拖动分割线可调整左右比例:"),
        text("").size(theme::font(0.3)),
        container(split)
            .height(iced::Length::Fixed(200.0))
            .width(iced::Length::Fill),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_toast() -> Element<'static, Msg> {
    let positions = [
        (ToastPosition::BottomRight, "右下"),
        (ToastPosition::TopRight, "右上"),
    ];

    let levels = [
        (ToastLevel::Info, "Info"),
        (ToastLevel::Success, "Success"),
        (ToastLevel::Warning, "Warning"),
        (ToastLevel::Error, "Error"),
    ];

    let mut rows = Vec::new();
    for (pos, pos_name) in &positions {
        let mut btns = Vec::new();
        for (level, level_name) in &levels {
            btns.push(
                button(text(format!("{} {}", level_name, pos_name)).size(12))
                    .on_press(Msg::ToastShow(*level, format!("{}提示 - {}", level_name, pos_name), *pos))
                    .padding([4, 8])
                    .style(button::secondary)
                    .into()
            );
        }
        rows.push(row(btns).spacing(4).into());
    }

    column![
        text("Toast 提示消息组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("点击按钮显示 Toast，支持2个位置（右下、右上）").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        column(rows).spacing(4),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_color_picker(libs: &UiLibs) -> Element<'static, Msg> {
    let color_display = container(
        text("颜色预览")
    )
    .width(iced::Length::Fixed(80.0))
    .height(iced::Length::Fixed(50.0));

    column![
        text("ColorPicker 颜色选择器组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        row![
            button("选择颜色")
                .on_press(Msg::TabSelected(ComponentTab::ColorPicker)),
            color_display,
            column![
                text("当前颜色:").size(theme::font(0.9)),
                text(format!("RGB: {:?}", libs.selected_color)),
            ]
            .spacing(5),
        ]
        .spacing(20),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_date_picker() -> Element<'static, Msg> {
    use iced_aw::date_picker::Date;

    let today = Date::today();
    let date_str = format!("{}/{}/{}", today.year, today.month, today.day);

    column![
        text("DatePicker 日期选择器组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("点击下方按钮可打开日期选择器").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        row![
            button("选择日期")
                .on_press(Msg::TabSelected(ComponentTab::DatePicker)),
            text(format!("当前日期: {}", date_str)),
        ]
        .spacing(10),
        text("").size(theme::font(0.3)),
        text("(需要状态管理，实际使用需要配合 show_picker 和 date 状态)")
            .size(theme::font(0.7)),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_tooltip() -> Element<'static, Msg> {
    let content = text("Hover over me").size(18);
    let tip = text("This is a tooltip!").size(14);

    column![
        text("Tooltip 提示框组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("鼠标悬停在下方文本上查看提示效果:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        tooltip(content, tip, tooltip::Position::Top)
            .gap(8)
            .snap_within_viewport(true),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_pick_list(libs: &UiLibs) -> Element<'static, Msg> {
    let options = vec![
        "Apple".to_string(),
        "Banana".to_string(),
        "Cherry".to_string(),
        "Durian".to_string(),
    ];

    column![
        text("PickList 下拉选择组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("选择一个水果:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        pick_list::PickList::new(
            options,
            libs.pick_list_selected.clone(),
            |s| Msg::PickListSelected(Some(s)),
        )
        .padding(8),
        text("").size(theme::font(0.3)),
        text(format!(
            "已选择: {}",
            libs.pick_list_selected.as_deref().unwrap_or("(无)")
        )),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_combo_box(libs: &UiLibs) -> Element<'_, Msg> {
    let selection = libs.combo_box_selected.as_ref();

    column![
        text("ComboBox 组合框组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("输入或选择编程语言:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        combo_box::ComboBox::new(
            &libs.combo_box_state,
            "Type to search...",
            selection,
            |s| Msg::ComboBoxSelected(s),
        ),
        text("").size(theme::font(0.3)),
        text(format!(
            "已选择: {}",
            libs.combo_box_selected.as_deref().unwrap_or("(无)")
        )),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_float() -> Element<'static, Msg> {
    use iced::{Background, Border, Color, Length, Shadow, Theme, Vector};
    use iced::widget::container;

    let inner = container(
        column![
            text("Floating Content").size(18),
            text("").size(theme::font(0.3)),
            text("This box floats above").size(14),
            text("other content when").size(14),
            text("scale > 1.0").size(14),
        ]
        .spacing(4)
        .padding(10),
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(150.0))
    .style(|_: &Theme| container::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.3, 0.5))),
        border: Border {
            color: Color::from_rgb(0.5, 0.6, 0.8),
            width: 2.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    });

    column![
        text("Float 浮动组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("Float 将内容以 overlay 形式浮动（scale=1.3）:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        float::Float::new(inner)
            .scale(1.3)
            .style(|_: &Theme| float::Style {
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: Vector::new(2.0, 4.0),
                    blur_radius: 12.0,
                },
                shadow_border_radius: 4.0.into(),
            }),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_pin() -> Element<'static, Msg> {
    column![
        text("Pin 固定组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("Pin 将内容通过 overlay 固定在视口位置:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        Pin::new(text("This content uses Pin overlay")),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_table() -> Element<'static, Msg> {
    use iced::{Border, Color, Length, Theme};

    let header_cell = |label: &'static str| -> Element<'static, Msg> {
        container(text(label).size(14))
            .width(Length::Fill)
            .padding([6, 10])
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.2, 0.25, 0.35))),
                ..Default::default()
            })
            .into()
    };

    let data_cell = |label: &'static str| -> Element<'static, Msg> {
        container(text(label).size(13))
            .width(Length::Fill)
            .padding([5, 10])
            .into()
    };

    let header = row![
        header_cell("Name"),
        header_cell("Type"),
        header_cell("Value"),
    ]
    .spacing(0);

    let rows = column![
        row![data_cell("Alpha"), data_cell("String"), data_cell("foo")].spacing(0),
        row![data_cell("Beta"), data_cell("i32"), data_cell("42")].spacing(0),
        row![data_cell("Gamma"), data_cell("f64"), data_cell("3.14")].spacing(0),
        row![data_cell("Delta"), data_cell("bool"), data_cell("true")].spacing(0),
    ]
    .spacing(1);

    column![
        text("Table 表格组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("简单数据表格（手动 Column/Row 布局）:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        container(column![header, rows])
            .width(Length::Fixed(400.0))
            .style(|_: &Theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.14, 0.18))),
                border: Border {
                    color: Color::from_rgb(0.3, 0.35, 0.45),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

fn view_pane_grid(libs: &UiLibs) -> Element<'_, Msg> {
    use iced::Length;

    let grid = pane_grid::PaneGrid::new(
        &libs.pane_grid_state,
        |_pane, _data, _is_maximized| {
            text("Pane content").into()
        },
    )
    .width(Length::Fill)
    .height(Length::Fixed(300.0));

    column![
        text("PaneGrid 分格面板组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("可拖拽分割线的多面板布局:").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        grid,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}
