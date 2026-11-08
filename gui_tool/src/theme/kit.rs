//! iced-kit 主题适配器
//!
//! `iced-kit` 的唯一耦合点。
//! 将 iced-kit 设计系统包装为 `ThemeStyles` + `ThemeSource` 实现。
//!
//! # 策略模式角色
//! - **ConcreteStrategy**: `KitStyles` — 默认策略
//! - **StrategyFactory**: `KitThemeSource` — 从 iced-kit 构造策略

use std::fmt;

use crate::theme::source::ThemeSource;
use crate::theme::styles::ThemeStyles;
use crate::theme::theme_id::ThemeId;
use crate::theme::tokens::{derive_iced_theme, FontLevel, Tokens};

// ============================================================================
// KitStyles — 默认策略：提取 + 补全 + 派生
// ============================================================================

/// iced-kit 主题样式实现（默认策略）
///
/// 只持有抽象层自有 `Tokens`，不再持有 `iced_kit::Theme`。
/// 通过 `from_kit()` 从 iced-kit 提取+补全令牌。
pub struct KitStyles {
    tokens: Tokens,
}

impl KitStyles {
    /// 从 ThemeId 创建
    pub fn new(id: &ThemeId) -> Self {
        let kit = id.to_iced_kit_theme();
        Self::from_kit(&kit)
    }

    /// 从 iced-kit 主题提取 + 补全令牌
    ///
    /// - **提取**：iced-kit 有的字段直接映射
    /// - **补全**：iced-kit 没有的字段按其设计节奏推导
    pub fn from_kit(kit: &iced_kit::Theme) -> Self {
        let kt = kit.tokens();
        let colors = &kt.colors;

        // ---- 提取：iced-kit 颜色直接映射 ----
        // ---- 补全：间距/圆角/字体从 iced-kit 设计节奏推导 ----
        let tokens = Tokens {
            // 提取：颜色
            background: colors.background,
            surface: colors.surface,
            surface_raised: colors.secondary, // iced-kit 无 surface_raised，用 secondary 近似
            text_primary: colors.foreground,
            text_secondary: colors.muted_foreground,
            accent: colors.primary,
            success: colors.success,
            warning: colors.warning,
            danger: colors.destructive,
            border: colors.border,

            // 补全：间距（iced-kit Spacing 是 u16，转为 f32）
            spacing_xs: f32::from(kt.spacing.xs),
            spacing_sm: f32::from(kt.spacing.sm),
            spacing_md: f32::from(kt.spacing.md),
            spacing_lg: f32::from(kt.spacing.lg),
            spacing_xl: f32::from(kt.spacing.xl),

            // 补全：圆角（iced-kit Radius 是 u16，转为 f32）
            radius_sm: f32::from(kt.radius.sm),
            radius_md: f32::from(kt.radius.md),
            radius_lg: f32::from(kt.radius.lg),
            radius_full: f32::from(kt.radius.full),

            // 补全：字体层级（iced-kit Typography 提供 sans/mono + xs-xl 字号）
            font_body: FontLevel::new(
                kt.typography.sans,
                kt.typography.sm.size,
                iced::font::Weight::Normal,
            ),
            font_title: FontLevel::new(
                kt.typography.sans,
                kt.typography.xl.size,
                iced::font::Weight::Bold,
            ),
            font_caption: FontLevel::new(
                kt.typography.sans,
                kt.typography.xs.size,
                iced::font::Weight::Normal,
            ),
            font_mono: FontLevel::new(
                kt.typography.mono,
                kt.typography.sm.size,
                iced::font::Weight::Normal,
            ),

            border_width: 1.0,
        };

        Self { tokens }
    }
}

impl fmt::Debug for KitStyles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KitStyles").finish()
    }
}

impl ThemeStyles for KitStyles {
    fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    fn iced_theme(&self) -> iced::Theme {
        derive_iced_theme(&self.tokens)
    }
}

// ============================================================================
// KitThemeSource — 加载 iced-kit 主题
// ============================================================================

/// iced-kit 主题加载器（策略工厂）
///
/// 根据 ThemeId 构造 KitStyles。
pub struct KitThemeSource;

impl ThemeSource for KitThemeSource {
    fn load(&self, id: &ThemeId) -> Result<Box<dyn ThemeStyles>, String> {
        Ok(Box::new(KitStyles::new(id)))
    }

    fn system(&self) -> Result<Box<dyn ThemeStyles>, String> {
        // 默认使用暗色主题（后续可接入 dark-light crate 检测系统主题）
        Ok(Box::new(KitStyles::new(&ThemeId::dark())))
    }

    fn available(&self) -> Vec<ThemeId> {
        ThemeId::all().to_vec()
    }
}
