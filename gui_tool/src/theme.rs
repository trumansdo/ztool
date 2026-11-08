//! 主题抽象层模块
//!
//! 提供可替换的主题系统：
//! - `ThemeId`: 主题标识，可序列化，菜单/持久化只认它
//! - `ThemeStyles`: 主题样式 trait，向内提供 iced 主题 + 内置控件样式
//! - `ThemeSource`: 主题加载 trait，隔离具体主题库
//!
//! # 架构约束
//! - `native-theme-iced` 的符号只出现在 `native.rs` 适配器子模块中
//! - 第三方库样式适配函数放在应用层（`theme_adapters` 模块），不在此处

pub mod adapters;
pub mod kit;
pub mod kit_adapter;

pub mod native;
pub mod source;
pub mod styles;
pub mod theme_id;
pub mod tokens;

pub use styles::ThemeStyles;
pub use theme_id::ThemeId;
pub use tokens::Tokens;
