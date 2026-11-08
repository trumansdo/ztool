//! UI 状态模块
//!
//! 管理仅影响渲染、不影响业务语义的 UI 层面状态：
//! - 标签页选择
//! - 主题标识与样式实例
//! - 面板比例、焦点等

use crate::theme::{ThemeId, ThemeStyles};
use crate::ui::Tab;

/// UI 层面状态
#[derive(Debug)]
pub struct UiState {
    /// 当前选中的标签页
    pub selected_tab: Tab,
    /// 当前主题标识
    pub theme_id: ThemeId,
    /// 当前主题样式实例
    pub styles: Box<dyn ThemeStyles>,
}

impl UiState {
    /// 使用指定主题创建 UI 状态
    pub fn new(theme_id: ThemeId, styles: Box<dyn ThemeStyles>) -> Self {
        Self {
            selected_tab: Tab::default(),
            theme_id,
            styles,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        let theme_id = ThemeId::dark();
        let styles = Box::new(crate::theme::kit::KitStyles::new(&theme_id));
        Self::new(theme_id, styles)
    }
}
