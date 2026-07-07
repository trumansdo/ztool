//! 应用状态管理和左右两栏布局渲染

use crate::features::json_fmt::JsonFormatter;
use crate::features::net_capture::PacketCapture;
use crate::features::net_port_scan::NetScanner;
use crate::features::pyramid_3d::Pyramid;
use crate::features::ui_libs::UiLibs;
use crate::ui::widgets::tree_menu::{TreeItem, render_tree_item};
use iced::widget::{column, container, row};
use iced::{Color, Element, Length, Task};

use super::{Message, Tab};

#[derive(Default)]
pub struct App {
    pub selected_tab: Tab,
    pub json_formatter: JsonFormatter,
    pub net_port_scan: NetScanner,
    pub packet_capture: PacketCapture,
    pub ui_libs: UiLibs,
    pub pyramid: Pyramid,
    pub expanded: std::collections::HashSet<String>,
}

impl App {
    pub fn new() -> Self {
        let mut expanded = std::collections::HashSet::new();
        expanded.insert("net".to_string());
        expanded.insert("data".to_string());
        expanded.insert("ui".to_string());
        Self {
            expanded,
            ..Self::default()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleCategory(id) => {
                if self.expanded.contains(&id) {
                    self.expanded.remove(&id);
                } else {
                    self.expanded.insert(id);
                }
                Task::none()
            }
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
            Message::JsonFmt(m) => {
                crate::features::json_fmt::update(&mut self.json_formatter, m).map(Message::JsonFmt)
            }
            Message::NetPortScan(m) => {
                crate::features::net_port_scan::update(&mut self.net_port_scan, m).map(Message::NetPortScan)
            }
            Message::NetCapture(m) => {
                crate::features::net_capture::update(&mut self.packet_capture, m).map(Message::NetCapture)
            }
            Message::UiLibs(m) => crate::features::ui_libs::update(&mut self.ui_libs, m).map(Message::UiLibs),
            Message::ShaderPyramid(m) => {
                crate::features::pyramid_3d::update(&mut self.pyramid, m).map(Message::ShaderPyramid)
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let menu_tree = vec![
            TreeItem::new("net", "网络工具")
                .child(TreeItem::new("net_port_scan", "端口扫描"))
                .child(TreeItem::new("net_capture", "网络抓包")),
            TreeItem::new("data", "数据工具").child(TreeItem::new("json_fmt", "JSON格式化")),
            TreeItem::new("ui", "组件库").child(TreeItem::new("ui_libs", "组件示例")),
            TreeItem::new("3d", "3D展示").child(TreeItem::new("pyramid_3d", "金字塔")),
        ];

        let selected_id = match self.selected_tab {
            Tab::JsonFmt => "json_fmt",
            Tab::NetPortScan => "net_port_scan",
            Tab::NetCapture => "net_capture",
            Tab::UiLibs => "ui_libs",
            Tab::Pyramid3d => "pyramid_3d",
        };

        let mut menu_col = column![].spacing(0);
        for item in &menu_tree {
            menu_col = menu_col.push(render_tree_item(item, 0, &self.expanded, selected_id));
        }

        let menu_panel = container(menu_col)
            .width(Length::Fixed(160.0))
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(25, 25, 40))),
                ..Default::default()
            });

        let content: Element<'_, Message> = match self.selected_tab {
            Tab::JsonFmt => {
                crate::features::json_fmt::view(&self.json_formatter).map(Message::JsonFmt)
            }
            Tab::NetPortScan => {
                crate::features::net_port_scan::view(&self.net_port_scan).map(Message::NetPortScan)
            }
            Tab::NetCapture => {
                crate::features::net_capture::view(&self.packet_capture).map(Message::NetCapture)
            }
            Tab::UiLibs => {
                crate::features::ui_libs::view(&self.ui_libs).map(Message::UiLibs)
            }
            Tab::Pyramid3d => {
                crate::features::pyramid_3d::view(&self.pyramid).map(Message::ShaderPyramid)
            }
        };

        let content_panel = container(content)
            .width(Length::Fill)
            .height(Length::Fill);

        row![menu_panel, content_panel]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn title() -> String {
        "综合工具".to_string()
    }

    pub fn theme() -> iced::Theme {
        iced::Theme::Dark
    }
}
