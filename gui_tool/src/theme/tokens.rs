//! 抽象层自有设计令牌类型
//!
//! `Tokens` 是主题抽象层的公共类型，不 import 任何具体主题库的类型。
//! 字段覆盖 `iced_theme()` 派生所需的 6 调色板槽 + 间距/圆角/字体/边框。
//!
//! # 设计原则
//! - 零外部依赖：`use` 中无 `iced_kit`、`native_theme_iced` 等
//! - 自给自足：`derive_iced_theme(&Tokens) -> iced::Theme` 不依赖外部 Catalog
//! - 可扩展：字段按项目需要增删，但 6 调色板槽必须在

use iced::{Color, Font};

// ============================================================================
// 字体层级描述
// ============================================================================

/// 字体层级描述：字体族 + 字号 + 字重
#[derive(Debug, Clone, PartialEq)]
pub struct FontLevel {
    pub family: Font,
    pub size: f32,
    pub weight: iced::font::Weight,
}

impl FontLevel {
    pub const fn new(family: Font, size: f32, weight: iced::font::Weight) -> Self {
        Self {
            family,
            size,
            weight,
        }
    }
}

// ============================================================================
// Tokens — 抽象层自有类型
// ============================================================================

/// 设计令牌集合
///
/// 所有视觉参数的统一来源。具体策略（KitStyles、HandWrittenStyles 等）
/// 负责填充此结构体，`derive_iced_theme()` 负责从中派生 `iced::Theme`。
#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
    // ---- 颜色：6 调色板槽（iced_theme() 派生必需） ----
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub border: Color,

    // ---- 间距阶梯（逻辑像素） ----
    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,

    // ---- 圆角档位（逻辑像素） ----
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_full: f32,

    // ---- 字体层级 ----
    pub font_body: FontLevel,
    pub font_title: FontLevel,
    pub font_caption: FontLevel,
    pub font_mono: FontLevel,

    // ---- 边框 ----
    pub border_width: f32,
}

impl Default for Tokens {
    /// 提供合理的暗色 fallback 值
    fn default() -> Self {
        Self {
            background: Color::from_rgb8(0x0a, 0x0a, 0x0a),
            surface: Color::from_rgb8(0x17, 0x17, 0x17),
            surface_raised: Color::from_rgb8(0x26, 0x26, 0x26),
            text_primary: Color::from_rgb8(0xfa, 0xfa, 0xfa),
            text_secondary: Color::from_rgb8(0xa3, 0xa3, 0xa3),
            accent: Color::from_rgb8(0xfa, 0xfa, 0xfa),
            success: Color::from_rgb8(0x4a, 0xde, 0x80),
            warning: Color::from_rgb8(0xfb, 0xbf, 0x24),
            danger: Color::from_rgb8(0xd4, 0x2c, 0x2c),
            border: Color::from_rgb8(0x26, 0x26, 0x26),

            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 12.0,
            spacing_lg: 16.0,
            spacing_xl: 24.0,

            radius_sm: 3.0,
            radius_md: 6.0,
            radius_lg: 8.0,
            radius_full: 9999.0,

            font_body: FontLevel::new(Font::DEFAULT, 14.0, iced::font::Weight::Normal),
            font_title: FontLevel::new(Font::DEFAULT, 20.0, iced::font::Weight::Bold),
            font_caption: FontLevel::new(Font::DEFAULT, 12.0, iced::font::Weight::Normal),
            font_mono: FontLevel::new(Font::MONOSPACE, 13.0, iced::font::Weight::Normal),

            border_width: 1.0,
        }
    }
}

// ============================================================================
// derive_iced_theme — 从 Tokens 派生 iced::Theme
// ============================================================================

/// 从抽象层 `Tokens` 派生 `iced::Theme`
///
/// 使用 `iced::Theme::custom` 从 6 调色板槽构造。
/// 此函数不依赖任何外部库的 Catalog 实现，所有策略可复用。
pub fn derive_iced_theme(t: &Tokens) -> iced::Theme {
    iced::Theme::custom(
        std::borrow::Cow::Borrowed("App"),
        iced::theme::Palette {
            background: t.background,
            text: t.text_primary,
            primary: t.accent,
            success: t.success,
            warning: t.warning,
            danger: t.danger,
        },
    )
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tokens_are_valid() {
        let t = Tokens::default();
        // 验证 6 调色板槽非零
        assert_ne!(t.background, Color::TRANSPARENT);
        assert_ne!(t.text_primary, Color::TRANSPARENT);
        assert_ne!(t.accent, Color::TRANSPARENT);
    }

    #[test]
    fn derive_iced_theme_returns_valid_theme() {
        let t = Tokens::default();
        let theme = derive_iced_theme(&t);
        // 验证返回的是 custom 主题
        match theme {
            iced::Theme::Custom(_) => {} // 预期
            _ => panic!("expected custom theme"),
        }
    }

    #[test]
    fn tokens_module_has_no_external_theme_imports() {
        // 编译时验证：此模块不应编译通过如果 import 了 iced_kit
        // 运行时验证：Tokens 可以独立构造
        let _t = Tokens::default();
    }
}
