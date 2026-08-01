//! # 菜单组件模块
//!
//! 基于 libcosmic 菜单系统架构实现的自定义菜单组件，包含三个层次：
//!
//! | 文件 | 层次 | 职责 |
//! |:--|:--|:--|
//! | `menu/menu_tree.rs` | 数据模型层 | `MenuTree` 递归枚举（Item/Folder/Separator） |
//! | `menu/menu_bar.rs` | 核心组件层 | `MenuBar` Widget + `overlay()` 构造 |
//! | `menu/menu_inner.rs` | 状态控制层 + 渲染层 | `MenuBarState` FSM + `Menu` Overlay 递归渲染 |
//!
//! # 模块化说明
//!
//! 本文件采用 **Rust 2018 新式模块化**（目录同名入口文件）：
//! - 入口文件为 `widgets/menu.rs`（与目录 `menu/` 同名，替代旧式 `menu/mod.rs`）
//! - 子模块文件置于 `menu/` 目录内，由本入口通过 `mod` 声明挂载
//! - 这与项目既有风格一致（如 `features/ui_libs.rs` + `features/ui_libs/` 目录）
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::ui::widgets::menu::{MenuBar, MenuTree};
//!
//! let menu_bar = MenuBar::new(vec![
//!     MenuTree::folder("文件", vec![
//!         MenuTree::item("新建", MyMessage::New),
//!         MenuTree::separator(),
//!         MenuTree::item("退出", MyMessage::Quit),
//!     ]),
//!     MenuTree::item("帮助", MyMessage::Help),
//! ]);
//! ```

mod menu_bar;
mod menu_inner;
mod menu_tree;

pub use menu_bar::MenuBar;
pub use menu_tree::MenuTree;
