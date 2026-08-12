//! 配置管理模块
//!
//! 统一配置入口 `BinConfig`，所有工具共用 exe 同级 `binconfig.toml`。

pub mod binconfig;

pub use binconfig::*;
