//! # netdump 模块入口
//!
//! ## Rust 概念 — mod.rs 约定
//! 当一个模块有多个子文件时，可以用 `mod.rs` 文件作为模块的入口。
//! 编译器查找规则（2024 edition）：
//! 1. 先查找 `netdump.rs`（同名文件）
//! 2. 再查找 `netdump/mod.rs`（目录中的入口文件）
//!
//! 这里使用 `netdump/mod.rs` 方式，因为 httpdump.rs 作为子模块放在同一目录下。

pub mod httpdump;
