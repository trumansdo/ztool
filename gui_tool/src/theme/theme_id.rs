//! 主题标识类型
//!
//! `ThemeId` 是主题的唯一标识符，可序列化，用于菜单选择与持久化。

use serde::{Deserialize, Serialize};

/// 主题标识符
///
/// 菜单和持久化只认此类型，不直接依赖具体主题实现。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThemeId {
    /// 主题名称（如 "Dark"、"Light"、"Dracula"）
    pub name: String,
    /// 是否为暗色主题
    pub dark: bool,
}

impl ThemeId {
    /// 创建新的主题标识
    pub fn new(name: impl Into<String>, dark: bool) -> Self {
        Self {
            name: name.into(),
            dark,
        }
    }

    /// 预定义：暗色主题
    pub fn dark() -> Self {
        Self::new("Dark", true)
    }

    /// 预定义：亮色主题
    pub fn light() -> Self {
        Self::new("Light", false)
    }

    /// 所有可用主题（iced 0.14 内置 22 个）
    pub fn all() -> &'static [ThemeId] {
        Box::leak(Box::new(vec![
            Self::light(),
            Self::dark(),
            Self::new("Dracula", true),
            Self::new("Nord", true),
            Self::new("Solarized Light", false),
            Self::new("Solarized Dark", true),
            Self::new("Gruvbox Light", false),
            Self::new("Gruvbox Dark", true),
            Self::new("Catppuccin Latte", false),
            Self::new("Catppuccin Frappé", true),
            Self::new("Catppuccin Macchiato", true),
            Self::new("Catppuccin Mocha", true),
            Self::new("Tokyo Night", true),
            Self::new("Tokyo Night Storm", true),
            Self::new("Tokyo Night Light", false),
            Self::new("Kanagawa Wave", true),
            Self::new("Kanagawa Dragon", true),
            Self::new("Kanagawa Lotus", false),
            Self::new("Moonfly", true),
            Self::new("Nightfly", true),
            Self::new("Oxocarbon", true),
            Self::new("Ferra", true),
        ]))
    }

    /// 映射到 iced_kit::Theme
    ///
    /// iced-kit 目前仅支持 light/dark 两种主题，
    /// 其他主题名称回退到 dark。
    pub fn to_iced_kit_theme(&self) -> iced_kit::Theme {
        if self.dark {
            iced_kit::Theme::dark()
        } else {
            iced_kit::Theme::light()
        }
    }

    /// 映射到 iced::Theme（保留兼容）
    pub fn to_iced_theme(&self) -> iced::Theme {
        match self.name.as_str() {
            "Light" => iced::Theme::Light,
            "Dark" => iced::Theme::Dark,
            "Dracula" => iced::Theme::Dracula,
            "Nord" => iced::Theme::Nord,
            "Solarized Light" => iced::Theme::SolarizedLight,
            "Solarized Dark" => iced::Theme::SolarizedDark,
            "Gruvbox Light" => iced::Theme::GruvboxLight,
            "Gruvbox Dark" => iced::Theme::GruvboxDark,
            "Catppuccin Latte" => iced::Theme::CatppuccinLatte,
            "Catppuccin Frappé" => iced::Theme::CatppuccinFrappe,
            "Catppuccin Macchiato" => iced::Theme::CatppuccinMacchiato,
            "Catppuccin Mocha" => iced::Theme::CatppuccinMocha,
            "Tokyo Night" => iced::Theme::TokyoNight,
            "Tokyo Night Storm" => iced::Theme::TokyoNightStorm,
            "Tokyo Night Light" => iced::Theme::TokyoNightLight,
            "Kanagawa Wave" => iced::Theme::KanagawaWave,
            "Kanagawa Dragon" => iced::Theme::KanagawaDragon,
            "Kanagawa Lotus" => iced::Theme::KanagawaLotus,
            "Moonfly" => iced::Theme::Moonfly,
            "Nightfly" => iced::Theme::Nightfly,
            "Oxocarbon" => iced::Theme::Oxocarbon,
            "Ferra" => iced::Theme::Ferra,
            _ => {
                if self.dark {
                    iced::Theme::Dark
                } else {
                    iced::Theme::Light
                }
            }
        }
    }
}

impl std::fmt::Display for ThemeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
