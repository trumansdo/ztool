//! # 网络抓包功能模块入口
//!
//! ## Rust 概念 — 模块架构
//! 这里用到了 Rust 模块系统的三种声明方式：
//! 1. `pub mod parser;` — 子模块声明，编译器将自动查找 `parser.rs` 或 `parser/mod.rs`
//! 2. `pub mod update;` — 同上，声明 update 子模块
//! 3. `pub mod view;` — 同上，声明 view 子模块
//! 4. `pub use update::{...}` — re-export（重导出），把子模块中的类型提升到此模块的公开 API
//!
//! 这是 Rust 中常见的三层架构模式：view(视图) + update(逻辑) + parser(解析)
//!
//! # 调用链 (从深到浅)
//! ```
//! parser/update/view (第1层)
//!   ↑
//! net_capture.rs (第2层)
//!   ↑
//! App::update/view (第4层)
//! ```

pub mod parser;
pub mod update;
pub mod view;

pub use update::{update, PacketCapture, Msg};
pub use view::view;
