use crate::features::json_fmt::JsonFormatter;
use crate::features::net_capture::PacketCapture;
use crate::features::net_port_scan::NetScanner;
use crate::features::ui_libs::UiLibs;
use iced::widget::container;
use iced::widget::Row;
use iced::{Element, Task};

use super::{Message, Tab};

/// 应用状态：存储所有功能模块的状态
#[derive(Default)]
pub struct App {
    pub selected_tab: Tab,                            // 当前选中的Tab
    pub expanded_categories: std::collections::HashSet<String>, // 展开的分类集合
    pub json_formatter: JsonFormatter,                  // JSON格式化器状态
    pub net_port_scan: NetScanner,                      // 端口扫描器状态
    pub packet_capture: PacketCapture,               // 网络抓包状态
    pub ui_libs: UiLibs,                            // 组件示例状态
}

impl App {
    /// 创建新的应用实例
    pub fn new() -> Self {
        let mut expanded = std::collections::HashSet::new();
        expanded.insert("net_tools".to_string());
        expanded.insert("data_tools".to_string());
        expanded.insert("ui_libs".to_string());

        Self {
            selected_tab: Tab::JsonFmt,
            expanded_categories: expanded,
            json_formatter: JsonFormatter::new(),
            net_port_scan: NetScanner::new(),
            packet_capture: PacketCapture::new(),
            ui_libs: UiLibs::new(),
        }
    }

    /// 处理消息，更新各功能模块状态
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // 切换分类展开状态
            Message::ToggleCategory(name) => {
                if self.expanded_categories.contains(&name) {
                    self.expanded_categories.remove(&name);
                } else {
                    self.expanded_categories.insert(name);
                }
                Task::none()
            }
            // 切换选中Tab
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
            // 转发JSON格式化消息
            Message::JsonFmt(m) => {
                let _ = crate::features::json_fmt::update(&mut self.json_formatter, m);
                Task::none()
            }
            // 转发端口扫描消息
            Message::NetPortScan(m) => {
                let _ = crate::features::net_port_scan::update(&mut self.net_port_scan, m);
                Task::none()
            }
            // 转发网络抓包消息
            Message::NetCapture(m) => {
                let _ = crate::features::net_capture::update(&mut self.packet_capture, m);
                Task::none()
            }
            // 转发组件示例消息
            Message::UiLibs(m) => {
                let _ = crate::features::ui_libs::update(&mut self.ui_libs, m);
                Task::none()
            }
        }
    }

    /// 渲染应用视图：左侧菜单+右侧内容
    pub fn view(&self) -> Element<'_, Message> {

        // 渲染左侧菜单面板
        let menu_panel = super::menu::view_menu_panel(
            &self.expanded_categories,
            self.selected_tab,
        );

        // 根据选中的Tab渲染对应的功能内容
        let content: Element<'_, Message> = match self.selected_tab {
            Tab::JsonFmt => crate::features::json_fmt::view(&self.json_formatter).map(Message::JsonFmt),
            Tab::NetPortScan => crate::features::net_port_scan::view(&self.net_port_scan).map(Message::NetPortScan),
            Tab::NetCapture => crate::features::net_capture::view(&self.packet_capture).map(Message::NetCapture),
            Tab::UiLibs => crate::features::ui_libs::view(&self.ui_libs).map(Message::UiLibs),
        };

        // 水平排列：菜单+内容
        container(Row::with_children(vec![menu_panel, content]).spacing(0))
            .padding(0)
            .into()
    }

    /// 获取窗口标题
    pub fn title() -> String {
        "综合工具".to_string()
    }

    /// 获取应用主题
    pub fn theme() -> iced::Theme {
        iced::Theme::Dark
    }
}
