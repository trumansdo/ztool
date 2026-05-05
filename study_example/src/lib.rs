//! # study_example 库根文件
//!
//! ## Rust 概念 — lib.rs vs main.rs
//! - `lib.rs`: 库 crate 的入口，定义公开 API 供其他 crate（或本项目的 bin）使用。
//!   `cargo test` 也会编译 lib.rs 中的代码。
//! - `main.rs`: 默认二进制入口（`cargo run` 执行），可独立存在或依赖 lib。
//!
//! ## Rust 概念 — `pub mod`
//! 每个 `pub mod` 声明告诉编译器去查找同名文件或目录。
//! 只有被声明为 `pub` 的模块才能被外部 crate 使用。
//! 这里的三个模块是通用的工程化模块组织示范。

pub mod init;    // 初始化模块（日志等）
pub mod mod01;   // 模块系统教学示例
pub mod netdump; // 网络抓包核心库
