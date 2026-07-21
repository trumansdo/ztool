//! 配置管理模块
//!
//! 管理 AI CLI 的全局配置，包括 API Key、默认模型、偏好设置等。
//! 配置文件路径: `~/.config/ai_cli/config.toml`

pub mod settings;

pub use settings::*;
