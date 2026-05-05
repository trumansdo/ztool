//! # serving 模块 — 跨模块函数调用演示
//!
//! ## Rust 概念 — `use crate::` 绝对路径导入
//! `use crate::mod01::hosting::add_to_waitlist;` 演示了如何从 crate 根开始
//! 使用绝对路径导入函数。
//!
//! ## Rust 概念 — 路径语法对比
//! | 路径类型 | 语法 | 含义 |
//! |---------|------|------|
//! | crate 根路径 | `crate::mod01::hosting` | 从当前 crate 根开始的绝对路径 |
//! | 相对路径 | `super::` | 父模块 |
//! | 同级路径 | `self::` | 当前模块 |
//! | 外部 crate | `serde::Serialize` | 依赖项在 Cargo.toml 中声明的外部库 |
//!
//! ## Rust 概念 — 私有模块中的函数
//! `serving` 模块是私有的（在 mod01.rs 中用 `mod` 而非 `pub mod` 声明），
//! 所以这里的所有函数外部都不可访问，仅 mod01 树内部可用。

use crate::mod01::hosting::add_to_waitlist;
#[allow(unused)]
fn take_order() {
    // 通过 use 导入后可直接用短名称调用
    add_to_waitlist();
}
#[allow(unused)]
fn serve_order() {}

#[allow(unused)]
fn take_payment() {}
