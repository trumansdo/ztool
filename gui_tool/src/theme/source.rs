//! 主题加载 trait
//!
//! 隔离具体主题库（如 `native-theme-iced`），
//! 使 `App` 不依赖任何特定主题实现。

use super::styles::ThemeStyles;
use super::theme_id::ThemeId;

/// 主题源契约
///
/// 负责加载主题实例，是主题专属的 Service。
/// 与 `DocumentService`、`SettingsStore` 遵循同一模式：trait + 适配器 + Task 回流。
pub trait ThemeSource {
    /// 按标识加载主题
    fn load(&self, id: &ThemeId) -> Result<Box<dyn ThemeStyles>, String>;

    /// 加载系统主题（跟随 OS 明暗设置）
    fn system(&self) -> Result<Box<dyn ThemeStyles>, String>;

    /// 列出所有可用主题
    fn available(&self) -> Vec<ThemeId>;
}
