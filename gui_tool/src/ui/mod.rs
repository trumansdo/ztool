use crate::features::json_fmt::JsonFormatter;
use crate::features::net_capture::PacketCapture;
use crate::features::net_scan::NetScanner;
use crate::features::theme;
use iced::widget::{container, mouse_area, row, text, Column};
use iced::{Color, Element, Task};

#[derive(Default)]
pub struct App {
    selected_tab: Tab,
    expanded_categories: std::collections::HashSet<String>,
    json_formatter: JsonFormatter,
    net_scanner: NetScanner,
    packet_capture: PacketCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tab {
    #[default]
    JsonFmt,
    NetScan,
    NetCapture,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleCategory(String),
    TabSelected(Tab),
    JsonFmt(crate::features::json_fmt::Msg),
    NetScan(crate::features::net_scan::Msg),
    NetCapture(crate::features::net_capture::Msg),
}

impl App {
    pub fn new() -> Self {
        let mut expanded = std::collections::HashSet::new();
        expanded.insert("网络工具".to_string());
        expanded.insert("数据工具".to_string());
        Self {
            selected_tab: Tab::JsonFmt,
            expanded_categories: expanded,
            json_formatter: JsonFormatter::new(),
            net_scanner: NetScanner::new(),
            packet_capture: PacketCapture::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleCategory(name) => {
                if self
                    .expanded_categories
                    .contains(&name)
                {
                    self.expanded_categories
                        .remove(&name);
                } else {
                    self.expanded_categories
                        .insert(name);
                }
                Task::none()
            }
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
            Message::JsonFmt(m) => {
                let _ = self.json_formatter.update(m);
                Task::none()
            }
            Message::NetScan(m) => {
                let _ = self.net_scanner.update(m);
                Task::none()
            }
            Message::NetCapture(m) => {
                let _ = self.packet_capture.update(m);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let menu_panel = view_menu_panel(&self.expanded_categories, self.selected_tab);

        let content: Element<'_, Message> = match self.selected_tab {
            Tab::JsonFmt => self
                .json_formatter
                .view()
                .map(Message::JsonFmt),
            Tab::NetScan => self
                .net_scanner
                .view()
                .map(Message::NetScan),
            Tab::NetCapture => self
                .packet_capture
                .view()
                .map(Message::NetCapture),
        };

        container(
            row![menu_panel, content]
                .spacing(0)
                .padding(0),
        )
        .padding(0)
        .into()
    }

    pub fn title() -> String {
        "综合工具".to_string()
    }

    pub fn theme() -> iced::Theme {
        iced::Theme::Dark
    }
}

fn view_menu_panel(
    expanded_categories: &std::collections::HashSet<String>,
    selected: Tab,
) -> Element<'static, Message> {
    let expand_icon = |name: &str| -> &'static str {
        if expanded_categories.contains(name) {
            "▼"
        } else {
            "▶"
        }
    };

    let category_row = |name: &'static str, _icon: &'static str| -> Element<'static, Message> {
        let is_expanded = expanded_categories.contains(name);
        let icon_text = expand_icon(name);
        let bg_color = if is_expanded { Color::from_rgb8(60, 60, 80) } else { Color::from_rgb8(50, 50, 65) };

        mouse_area(
            container(
                row![
                    text(icon_text)
                        .size(theme::font(0.85))
                        .font(iced::Font::DEFAULT),
                    text(name)
                        .size(theme::font(1.0))
                        .font(iced::Font::DEFAULT),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            )
            .width(iced::Length::Fill)
            .padding(theme::padding2(0.2, 0.4))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                ..Default::default()
            }),
        )
        .on_press(Message::ToggleCategory(name.to_string()))
        .into()
    };

    let item_row = |tab: Tab, _icon: &'static str, label: &'static str| -> Element<'static, Message> {
        let is_selected = selected == tab;
        let bg_color = if is_selected { Color::from_rgb8(40, 80, 120) } else { Color::from_rgb8(35, 35, 50) };

        mouse_area(
            container(
                row![
                    text("  ").size(theme::font(1.0)),
                    text(label)
                        .size(theme::font(0.95))
                        .font(iced::Font::DEFAULT),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            )
            .width(iced::Length::Fill)
            .padding(theme::padding2(0.15, 0.4))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                ..Default::default()
            }),
        )
        .on_press(Message::TabSelected(tab))
        .into()
    };

    let item_row = |tab: Tab, _icon: &'static str, label: &'static str| -> Element<'static, Message> {
        let is_selected = selected == tab;
        let bg_color = if is_selected { Color::from_rgb8(40, 80, 120) } else { Color::from_rgb8(35, 35, 50) };

        mouse_area(
            container(
                row![
                    text("  ").size(theme::font(1.0)),
                    text(label)
                        .size(theme::font(0.95))
                        .font(iced::Font::DEFAULT),
                ]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            )
            .width(iced::Length::Fill)
            .padding(iced::Padding::from([2, 6]))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg_color)),
                ..Default::default()
            }),
        )
        .on_press(Message::TabSelected(tab))
        .into()
    };

    let net_tools: Element<'static, Message> = if expanded_categories.contains("网络工具") {
        let col: Element<'static, Message> = Column::with_children(vec![
            category_row("网络工具", "📡"),
            item_row(Tab::NetScan, "", "网络扫描"),
            item_row(Tab::NetCapture, "", "网络抓包"),
        ])
        .spacing(theme::size(0.1).0 as u32)
        .into();
        col
    } else {
        Column::with_children(vec![category_row("网络工具", "📡")]).into()
    };

    let data_tools: Element<'static, Message> = if expanded_categories.contains("数据工具") {
        let col: Element<'static, Message> =
            Column::with_children(vec![category_row("数据工具", "📊"), item_row(Tab::JsonFmt, "", "JSON格式化")])
                .spacing(theme::size(0.1).0 as u32)
                .into();
        col
    } else {
        Column::with_children(vec![category_row("数据工具", "📊")]).into()
    };

    let menu_col: Element<'static, Message> = Column::with_children(vec![
        text("菜单")
            .size(theme::font(1.1))
            .into(),
        text("")
            .size(theme::font(0.5))
            .into(),
        net_tools,
        text("")
            .size(theme::font(0.3))
            .into(),
        data_tools,
    ])
    .spacing(theme::size(0.2).0 as u32)
    .into();

    container(menu_col)
        .width(theme::size(9.0).0)
        .padding(theme::padding(0.5))
        .style(container::transparent)
        .into()
}

fn new() -> (App, Task<Message>) {
    (App::new(), Task::none())
}

fn update(state: &mut App, message: Message) -> Task<Message> {
    state.update(message)
}

fn view(state: &App) -> Element<'_, Message> {
    state.view()
}

fn title(_state: &App) -> String {
    App::title()
}

fn theme(_state: &App) -> iced::Theme {
    App::theme()
}

pub fn run() -> iced::Result {
    iced::application(new, update, view)
        .title(title)
        .theme(theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1000.0, 700.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            resizable: true,
            ..iced::window::Settings::default()
        })
        .run()
}
