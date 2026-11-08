//! 主题样式 trait（策略接口）
//!
//! 定义主题抽象层的核心契约 —— 策略模式中的 Strategy 角色。
//!
//! # 设计原则（v3.0）
//! - 接口中**没有** `iced_kit_theme()`、`&iced_kit::Theme` 或任何 Catalog 承诺
//! - `tokens()` 返回抽象层自有 `Tokens` 类型，不暴露具体主题库
//! - `iced_theme()` 由策略从自己的 `Tokens` 派生，不依赖外部库
//! - 内置控件样式提供默认实现，策略可按需覆盖
//!
//! # 策略模式角色
//! - **Strategy**: `ThemeStyles`
//! - **ConcreteStrategy**: `KitStyles`、`HandWrittenStyles` 等
//! - **Context**: `UiState`，持有 `Box<dyn ThemeStyles>`

use std::fmt::Debug;

use crate::theme::Tokens;

/// 主题样式策略接口
///
/// 三个核心算法：
/// 1. `tokens()` — 返回设计令牌（各策略自己决定怎么得到）
/// 2. `iced_theme()` — 从自己的 Tokens 派生 iced::Theme
/// 3. 内置控件样式 — 带默认实现，策略可覆盖
///
/// 要求 `Debug` 以便 `UiState` 可自动派生 Debug。
/// 要求 `Send + Sync` 以便跨线程传递。
pub trait ThemeStyles: Debug + Send + Sync {
    // ========================================================================
    // 必须实现
    // ========================================================================

    /// 设计令牌访问
    fn tokens(&self) -> &Tokens;

    /// 从自己的 Tokens 派生 iced::Theme
    fn iced_theme(&self) -> iced::Theme;

    // ========================================================================
    // 内置控件样式（默认实现，策略可按需覆盖）
    // ========================================================================

    // ---- 按钮 ----

    fn button_primary(&self) -> iced::widget::button::Style {
        let theme = self.iced_theme();
        iced::widget::button::primary(&theme, iced::widget::button::Status::Active)
    }

    fn button_secondary(&self) -> iced::widget::button::Style {
        let theme = self.iced_theme();
        iced::widget::button::secondary(&theme, iced::widget::button::Status::Active)
    }

    fn button_destructive(&self) -> iced::widget::button::Style {
        let theme = self.iced_theme();
        iced::widget::button::danger(&theme, iced::widget::button::Status::Active)
    }

    fn button_text(&self) -> iced::widget::button::Style {
        let theme = self.iced_theme();
        iced::widget::button::text(&theme, iced::widget::button::Status::Active)
    }

    // ---- 文本输入 ----

    fn text_input(&self) -> iced::widget::text_input::Style {
        let theme = self.iced_theme();
        iced::widget::text_input::default(&theme, iced::widget::text_input::Status::Active)
    }

    // ---- 容器 ----

    fn container(&self) -> iced::widget::container::Style {
        iced::widget::container::Style::default()
    }

    // ---- 开关 ----

    fn switch_style(&self, is_active: bool) -> iced::widget::toggler::Style {
        let t = self.tokens();
        iced::widget::toggler::Style {
            background: if is_active {
                iced::Background::Color(t.accent)
            } else {
                iced::Background::Color(t.surface_raised)
            },
            background_border_width: 1.0,
            background_border_color: t.border,
            foreground: if is_active {
                iced::Background::Color(t.text_primary)
            } else {
                iced::Background::Color(t.text_secondary)
            },
            foreground_border_width: 1.0,
            foreground_border_color: t.border,
            text_color: Some(t.text_primary),
            border_radius: Some(t.radius_full.into()),
            padding_ratio: 0.15,
        }
    }

    // ---- 分割线 ----

    fn divider_style(&self) -> iced::widget::rule::Style {
        let t = self.tokens();
        iced::widget::rule::Style {
            color: t.border,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        }
    }

    // ---- 进度条 ----

    fn progress_bar_style(&self) -> iced::widget::progress_bar::Style {
        let t = self.tokens();
        iced::widget::progress_bar::Style {
            background: t.surface_raised.into(),
            bar: t.accent.into(),
            border: iced::Border {
                radius: t.radius_full.into(),
                ..Default::default()
            },
        }
    }

    // ---- 滑块 ----

    fn slider_style(&self) -> iced::widget::slider::Style {
        let t = self.tokens();
        iced::widget::slider::Style {
            rail: iced::widget::slider::Rail {
                backgrounds: (t.surface_raised.into(), t.accent.into()),
                width: 4.0,
                border: iced::Border {
                    radius: t.radius_full.into(),
                    ..Default::default()
                },
            },
            handle: iced::widget::slider::Handle {
                shape: iced::widget::slider::HandleShape::Circle { radius: 8.0 },
                background: t.accent.into(),
                border_color: t.background,
                border_width: 2.0,
            },
        }
    }

    // ---- 复选框 ----

    fn checkbox_style(&self) -> iced::widget::checkbox::Style {
        let t = self.tokens();
        iced::widget::checkbox::Style {
            background: t.surface_raised.into(),
            icon_color: t.accent,
            border: iced::Border {
                radius: t.radius_sm.into(),
                width: t.border_width,
                color: t.border,
            },
            text_color: Some(t.text_primary),
        }
    }

    // ---- 标签页 ----

    fn tab_active_style(&self) -> iced::widget::container::Style {
        let t = self.tokens();
        iced::widget::container::Style {
            background: Some(iced::Background::Color(t.accent)),
            text_color: Some(t.text_primary),
            border: iced::Border {
                radius: t.radius_md.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn tab_inactive_style(&self) -> iced::widget::container::Style {
        let t = self.tokens();
        iced::widget::container::Style {
            background: Some(iced::Background::Color(t.surface_raised)),
            text_color: Some(t.text_secondary),
            border: iced::Border {
                radius: t.radius_md.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    // ---- 加载动画 ----

    fn spinner_color(&self) -> iced::Color {
        self.tokens().accent
    }
}
