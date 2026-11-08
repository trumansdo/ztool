//! # UI 组件库状态管理与消息处理
//!
//! 管理组件示例页面中所有子 tab 的交互状态。

use crate::ui::widgets::toast::{Toast, ToastLevel, ToastPosition};

/// UI 组件库的子 tab 枚举
///
/// 按功能领域分组，每个 tab 同时展示 iced-kit 和 iced-shadcn 版本（如可用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComponentTab {
    // ---- 按钮 ----
    #[default]
    Button,
    ButtonGroup,
    // ---- 表单 ----
    Toggle,
    Checkbox,
    Radio,
    Slider,
    Select,
    Input,
    Textarea,
    NumberInput,
    // ---- 展示 ----
    Badge,
    Card,
    Separator,
    Spinner,
    Progress,
    Skeleton,
    Avatar,
    Kbd,
    // ---- 导航 ----
    Tab,
    Breadcrumb,
    Pagination,
    Stepper,
    // ---- 浮层 ----
    Dialog,
    Drawer,
    Popover,
    Tooltip,
    // ---- 反馈 ----
    Toast,
    Alert,
    // ---- 数据 ----
    Table,
    DatePicker,
    ColorPicker,
    // ---- 布局 ----
    Accordion,
    Collapsible,
    Resizable,
    Wrap,
    Split,
    // ---- 其他 ----
    Menu,
    StarRating,
    CopyButton,
    Meter,
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

pub struct UiLibs {
    pub selected_tab: ComponentTab,
    pub click_count: u32,
    pub toggle_value: bool,
    pub number_value: i32,
    pub selected_color: iced::Color,
    pub toasts: Vec<Toast>,
    pub kit_theme: iced_kit::Theme,
}

impl Default for UiLibs {
    fn default() -> Self {
        Self {
            selected_tab: ComponentTab::Badge,
            click_count: 0,
            toggle_value: false,
            number_value: 0,
            selected_color: iced::Color::from_rgb(0.5, 0.5, 0.5),
            toasts: Vec::new(),
            kit_theme: iced_kit::Theme::dark(),
        }
    }
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
