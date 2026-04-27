//! JSON 格式化工具模块
//!
//! 提供 JSON 的格式化（美化）、验证和复制功能。
//!
//! # 架构: view/update 分离
//! 每个功能模块遵循 iced 的最佳实践：
//! - `view.rs`: 纯渲染函数，接收只读状态引用，返回 Element 树
//! - `update.rs`: 纯状态更新函数，接收消息和可变状态引用，返回 Task
//! 这种分离实现了"关注点分离" (Separation of Concerns)，
//! 视图逻辑和业务逻辑互不侵扰。
//!
//! # Rust: `pub mod` 声明
//! `pub mod json_fmt;` 已在上层 `features.rs` 中声明，
//! 编译器会自动查找 `features/json_fmt.rs` 或 `features/json_fmt/mod.rs`。
//! 这里 `json_fmt.rs` 作为模块的入口，声明 `update` 和 `view` 子模块，
//! 然后 re-export 关键类型。

// 声明子模块 —— 编译器查找 json_fmt/update.rs 和 json_fmt/view.rs
pub mod update;
pub mod view;

// Re-export: 将子模块中的类型提升到当前命名空间
// 外部可用 `crate::features::json_fmt::JsonFormatter` 而非 `...::update::JsonFormatter`
pub use update::{update, JsonFormatter, Msg};
pub use view::view;
