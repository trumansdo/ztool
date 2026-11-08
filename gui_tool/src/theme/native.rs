//! 原生主题适配器
//!
//! `native-theme-iced` 的唯一耦合点。
//! 将原生主题包装为 `ThemeStyles` + `ThemeSource` 实现。
//!
//! 当前作为兼容层保留，默认策略已切换为 `KitStyles`。

use std::fmt;

use crate::theme::source::ThemeSource;
use crate::theme::styles::ThemeStyles;
use crate::theme::theme_id::ThemeId;
use crate::theme::tokens::{derive_iced_theme, Tokens};

// ============================================================================
// NativeStyles — 包装 iced::Theme，实现 ThemeStyles
// ============================================================================

/// 原生主题样式实现
///
/// 从 `iced::Theme` 构造 `Tokens`，再通过 `derive_iced_theme` 派生。
pub struct NativeStyles {
    tokens: Tokens,
}

impl NativeStyles {
    /// 从 iced::Theme 创建
    pub fn new(theme: iced::Theme) -> Self {
        let palette = theme.palette();
        let tokens = Tokens {
            background: palette.background,
            surface: palette.background, // 无 surface 概念，用 background
            surface_raised: palette.background,
            text_primary: palette.text,
            text_secondary: palette.text, // 无 secondary text
            accent: palette.primary,
            success: palette.success,
            warning: palette.warning,
            danger: palette.danger,
            border: palette.text, // 无 border 概念，用 text 近似
            ..Tokens::default()
        };
        Self { tokens }
    }
}

impl fmt::Debug for NativeStyles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeStyles").finish()
    }
}

impl ThemeStyles for NativeStyles {
    fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    fn iced_theme(&self) -> iced::Theme {
        derive_iced_theme(&self.tokens)
    }
}

// ============================================================================
// NativeThemeSource — 加载 iced 内置主题
// ============================================================================

/// 原生主题加载器
///
/// 提供 iced 内置的 Dark / Light 主题，以及系统主题检测。
pub struct NativeThemeSource;

impl ThemeSource for NativeThemeSource {
    fn load(&self, id: &ThemeId) -> Result<Box<dyn ThemeStyles>, String> {
        Ok(Box::new(NativeStyles::new(id.to_iced_theme())))
    }

    fn system(&self) -> Result<Box<dyn ThemeStyles>, String> {
        Ok(Box::new(NativeStyles::new(iced::Theme::Dark)))
    }

    fn available(&self) -> Vec<ThemeId> {
        ThemeId::all().to_vec()
    }
}
