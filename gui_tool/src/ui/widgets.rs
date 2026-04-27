//! 自定义 Widget 模块注册
//!
//! # Rust: `pub mod` 声明
//! 这些声明告诉 Rust 编译器在当前 `widgets` 模块名下查找对应的源文件。
//! 编译器按 2024 edition 的规则查找：
//! 1. `widgets/menu_bar.rs`
//! 2. `widgets/menu_bar/mod.rs` (备选)
//!
//! # 子模块列表
//! - `overlay`: 分层叠加系统 (Layered + Anchor)，实现 Toast 等浮动元素定位
//! - `toast`: Toast 通知弹窗组件（纯自定义实现）
//! - `tree_menu`: 树形导航菜单（递归渲染、展开/折叠、选中高亮）
//! - `menu_bar`: 顶部菜单栏 + 下拉菜单 (当前未在 UI 中使用)

pub mod menu_bar;
pub mod overlay;
pub mod toast;
pub mod tree_menu;

pub use overlay::{layer, Layered};
