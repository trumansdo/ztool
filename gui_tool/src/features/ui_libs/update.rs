//! # UI 组件库状态管理与消息处理
//!
//! 管理组件示例页面中所有子 tab 的交互状态。

use crate::ui::widgets::toast::{Toast, ToastLevel, ToastPosition};

/// UI 组件库的子 tab 枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComponentTab {
    #[default]
    Badge,
    Card,
    Button,
    Toggle,
    Separator,
    Tab,
    NumberInput,
    Spinner,
    Wrap,
    Split,
    Toast,
    ColorPicker,
    DatePicker,
    Menu,
}

#[derive(Debug, Clone)]
pub enum Msg {
    TabSelected(ComponentTab),
    ButtonPressed,
    ToggleChanged(bool),
    NumberChanged(i32),
    ToastShow(ToastLevel, String, ToastPosition),
    CloseToast(usize),
    /// 菜单项被点击（携带菜单项文本）
    MenuClicked(String),
}

#[derive(Default)]
pub struct UiLibs {
    pub selected_tab: ComponentTab,
    pub click_count: u32,
    pub toggle_value: bool,
    pub number_value: i32,
    pub selected_color: iced::Color,
    pub toasts: Vec<Toast>,
}

impl UiLibs {
    fn push_toast(&mut self, level: ToastLevel, text: String, position: ToastPosition) {
        self.toasts.push(Toast {
            level,
            text,
            position,
        });
    }
}

pub fn update(libs: &mut UiLibs, msg: Msg) -> iced::Task<Msg> {
    match msg {
        Msg::TabSelected(tab) => {
            libs.selected_tab = tab;
        }
        Msg::ButtonPressed => {
            libs.click_count += 1;
        }
        Msg::ToggleChanged(value) => {
            libs.toggle_value = value;
        }
        Msg::NumberChanged(value) => {
            libs.number_value = value;
        }
        Msg::ToastShow(level, text, position) => {
            libs.push_toast(level, text, position);
        }
        Msg::CloseToast(index) => {
            libs.toasts.remove(index);
        }
        Msg::MenuClicked(label) => {
            // 菜单点击反馈：记录次数 + 弹出 Toast 提示
            libs.click_count += 1;
            libs.push_toast(ToastLevel::Info, format!("菜单项: {} 被点击", label), ToastPosition::BottomRight);
        }
    }
    iced::Task::none()
}
