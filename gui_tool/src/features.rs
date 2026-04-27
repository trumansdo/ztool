//! 功能模块注册文件
//!
//! 声明 `features/` 目录下的所有子模块。Rust 的模块树在此文件中建立。
//!
//! # Rust 模块系统 (Module System)
//!
//! ## 模块声明语法
//! `pub mod <模块名>;` 告诉编译器存在一个名为 `<模块名>` 的子模块，
//! 编译器会在以下位置查找其源代码：
//! 1. `features/<模块名>.rs`
//! 2. `features/<模块名>/mod.rs` (旧式风格，2024 edition 仍支持)
//!
//! ## 模块 vs 文件
//! Rust 的模块系统是语义级的，不完全等同于文件系统。`mod` 声明不是 `#include`：
//! - 模块树在编译时构建，`mod` 声明确定了模块的父子关系
//! - 模块路径用 `::` 分隔，如 `crate::features::json_fmt::Msg`
//!
//! ## `pub mod` vs `mod`
//! - `pub mod json_fmt;`: 公开模块，父模块可以通过 `self::json_fmt` 访问
//! - `mod json_fmt;`: 私有模块，仅当前模块及其子模块可以访问
//!
//! ## 模块作为命名空间
//! `features` 模块相当于一个命名空间，其下每个子模块是独立的功能域。
//! 这种组织方式遵循 Rust 的"一个模块一个关注点"原则。

pub mod json_fmt;
pub mod net_capture;
pub mod net_port_scan;
pub mod theme;
pub mod ui_libs;
