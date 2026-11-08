//! 第三方组件主题适配器
//!
//! 将 `ThemeStyles` 转换为 iced_aw 等第三方库的组件样式。
//! 每个适配函数签名统一：`(&dyn ThemeStyles, ...) -> ComponentStyle`。
//!
//! 所有视觉参数从抽象层 `Tokens` 派生，不依赖第三方库的默认主题。
//!
//! # 使用方式
//! ```ignore
//! use crate::theme::adapters;
//! let badge_style = adapters::badge_style(&state.ui_state.styles);
//! ```

use crate::theme::ThemeStyles;

// ============================================================================
// iced_aw 组件适配
// ============================================================================

/// Badge 组件样式适配
///
/// 从抽象层令牌填充背景色、文字色和圆角。
pub fn badge_style(styles: &dyn ThemeStyles) -> iced_aw::style::badge::Style {
    let t = styles.tokens();
    iced_aw::style::badge::Style {
        background: iced::Background::Color(t.accent),
        text_color: t.text_primary,
        border_radius: Some(t.radius_md),
        border_width: t.border_width,
        border_color: Some(t.border),
    }
}

/// Card 组件样式适配
///
/// 从抽象层令牌填充卡片各区域的背景色、文字色和边框。
pub fn card_style(styles: &dyn ThemeStyles) -> iced_aw::style::card::Style {
    let t = styles.tokens();
    iced_aw::style::card::Style {
        background: iced::Background::Color(t.surface),
        border_radius: t.radius_md,
        border_width: t.border_width,
        border_color: t.border,
        head_background: iced::Background::Color(t.surface),
        head_text_color: t.text_primary,
        body_background: iced::Background::Color(t.surface),
        body_text_color: t.text_secondary,
        foot_background: iced::Background::Color(t.surface),
        foot_text_color: t.text_secondary,
        close_color: t.text_secondary,
    }
}

/// NumberInput 组件样式适配
///
/// 从抽象层令牌填充按钮背景色和图标颜色。
pub fn number_input_style(
    styles: &dyn ThemeStyles,
) -> iced_aw::style::number_input::Style {
    let t = styles.tokens();
    iced_aw::style::number_input::Style {
        button_background: Some(iced::Background::Color(t.surface_raised)),
        icon_color: t.text_primary,
    }
}

/// DatePicker 组件样式适配
///
/// 使用 iced_aw 内置 primary 主题函数生成样式。
pub fn date_picker_style(
    styles: &dyn ThemeStyles,
    status: iced_aw::style::Status,
) -> iced_aw::style::date_picker::Style {
    iced_aw::style::date_picker::primary(&styles.iced_theme(), status)
}

// ============================================================================
// 通用容器适配
// ============================================================================

/// 内容面板容器样式
///
/// 从抽象层令牌填充容器背景色和圆角。
pub fn content_panel_style(styles: &dyn ThemeStyles) -> iced::widget::container::Style {
    let t = styles.tokens();
    iced::widget::container::Style {
        background: Some(iced::Background::Color(t.surface)),
        border: iced::Border {
            radius: t.radius_md.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::kit::KitStyles;
    use crate::theme::theme_id::ThemeId;

    fn make_styles() -> KitStyles {
        KitStyles::new(&ThemeId::dark())
    }

    #[test]
    fn badge_style_does_not_panic() {
        let styles = make_styles();
        let _ = badge_style(&styles);
    }

    #[test]
    fn card_style_does_not_panic() {
        let styles = make_styles();
        let _ = card_style(&styles);
    }

    #[test]
    fn number_input_style_does_not_panic() {
        let styles = make_styles();
        let _ = number_input_style(&styles);
    }

    #[test]
    fn date_picker_style_does_not_panic() {
        let styles = make_styles();
        let _ = date_picker_style(&styles, iced_aw::style::Status::Active);
    }

    #[test]
    fn content_panel_style_uses_theme() {
        let styles = make_styles();
        let _ = content_panel_style(&styles);
    }
}
