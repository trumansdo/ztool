//! # UI 组件库展示模块入口
//!
//! 类似 net_capture/json_fmt，这是一个功能模块的入口文件，
//! 声明子模块并 re-export 公共 API。
//!
//! ## Rust 概念 — re-export 模式
//! `pub use update::...` 把子模块的类型提升到父模块，
//! 这样外部代码只需 `use crate::features::ui_libs::UiLibs` 而不是
//! `use crate::features::ui_libs::update::UiLibs`。
//! 这是一种封装技巧，对外隐藏内部模块结构。

pub mod update;
pub mod view;

pub use update::{update, Msg, UiLibs};
pub use view::view;
