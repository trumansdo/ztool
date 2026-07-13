//! # UI 组件库状态管理与消息处理
//!
//! 管理组件示例页面中所有子 tab 的交互状态。

use iced::widget::{combo_box, pane_grid};
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
    Tooltip,
    PickList,
    ComboBox,
    Float,
    Pin,
    Table,
    PaneGrid,
}

#[derive(Debug, Clone)]
pub enum Msg {
    TabSelected(ComponentTab),
    ButtonPressed,
    ToggleChanged(bool),
    NumberChanged(i32),
    ToastShow(ToastLevel, String, ToastPosition),
    CloseToast(usize),
    PickListSelected(Option<String>),
    ComboBoxSelected(String),
}

pub struct UiLibs {
    pub selected_tab: ComponentTab,
    pub click_count: u32,
    pub toggle_value: bool,
    pub number_value: i32,
    pub selected_color: iced::Color,
    pub toasts: Vec<Toast>,
    pub pick_list_selected: Option<String>,
    pub combo_box_state: combo_box::State<String>,
    pub combo_box_selected: Option<String>,
    pub pane_grid_state: pane_grid::State<()>,
}

impl Default for UiLibs {
    fn default() -> Self {
        Self {
            selected_tab: ComponentTab::default(),
            click_count: 0,
            toggle_value: false,
            number_value: 0,
            selected_color: iced::Color::TRANSPARENT,
            toasts: Vec::new(),
            pick_list_selected: None,
            combo_box_state: combo_box::State::new(vec![
                "Rust".to_string(),
                "Go".to_string(),
                "Python".to_string(),
                "TypeScript".to_string(),
                "Zig".to_string(),
            ]),
            combo_box_selected: None,
            pane_grid_state: pane_grid::State::new(()).0,
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
        Msg::PickListSelected(selected) => {
            libs.pick_list_selected = selected;
        }
        Msg::ComboBoxSelected(selected) => {
            libs.combo_box_selected = Some(selected);
        }
    }
    iced::Task::none()
}
