use crate::features::json_fmt::JsonFormatter;
use crate::features::net_capture::PacketCapture;
use crate::features::net_scan::NetScanner;
use crate::features::ui_libs::UiLibs;
use iced::widget::container;
use iced::{Element, Task};

use super::{Message, Tab};

#[derive(Default)]
pub struct App {
    pub selected_tab: Tab,
    pub expanded_categories: std::collections::HashSet<String>,
    pub json_formatter: JsonFormatter,
    pub net_scanner: NetScanner,
    pub packet_capture: PacketCapture,
    pub ui_libs: UiLibs,
}

impl App {
    pub fn new() -> Self {
        let mut expanded = std::collections::HashSet::new();
        expanded.insert("net_tools".to_string());
        expanded.insert("data_tools".to_string());
        expanded.insert("ui_libs".to_string());

        Self {
            selected_tab: Tab::JsonFmt,
            expanded_categories: expanded,
            json_formatter: JsonFormatter::new(),
            net_scanner: NetScanner::new(),
            packet_capture: PacketCapture::new(),
            ui_libs: UiLibs::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleCategory(name) => {
                if self.expanded_categories.contains(&name) {
                    self.expanded_categories.remove(&name);
                } else {
                    self.expanded_categories.insert(name);
                }
                Task::none()
            }
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
            Message::JsonFmt(m) => {
                let _ = crate::features::json_fmt::update(&mut self.json_formatter, m);
                Task::none()
            }
            Message::NetScan(m) => {
                let _ = crate::features::net_scan::update(&mut self.net_scanner, m);
                Task::none()
            }
            Message::NetCapture(m) => {
                let _ = crate::features::net_capture::update(&mut self.packet_capture, m);
                Task::none()
            }
            Message::UiLibs(m) => {
                let _ = crate::features::ui_libs::update(&mut self.ui_libs, m);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::Row;

        let menu_panel = super::menu::view_menu_panel(
            &self.expanded_categories,
            self.selected_tab,
        );

        let content: Element<'_, Message> = match self.selected_tab {
            Tab::JsonFmt => crate::features::json_fmt::view(&self.json_formatter).map(Message::JsonFmt),
            Tab::NetScan => crate::features::net_scan::view(&self.net_scanner).map(Message::NetScan),
            Tab::NetCapture => crate::features::net_capture::view(&self.packet_capture).map(Message::NetCapture),
            Tab::UiLibs => crate::features::ui_libs::view(&self.ui_libs).map(Message::UiLibs),
        };

        container(Row::with_children(vec![menu_panel, content]).spacing(0))
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