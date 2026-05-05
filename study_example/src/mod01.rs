//! # 模块系统教学 (第一章)
//!
//! ## Rust 概念 — `pub mod` vs `mod`
//! - `pub mod hosting;` — 公开模块，外部代码可以通过 `crate::mod01::hosting` 访问
//! - `mod serving;` — 私有模块，仅 mod01 及其子模块内部可见
//!
//! Rust 的模块可见性默认是**私有的**（与 Java 的 package-private 不同，
//! 与 Python 的约定式 `_` 前缀也不同）。需要显式使用 `pub` 才能公开。
//!
//! ## 教学背景
//! 这个模块来自《Rust 程序设计语言》书中第 7 章的餐厅示例，
//! 用 `hosting`(前台) 和 `serving`(服务) 模块演示 Rust 的模块系统。
//! 这是一个经典的"学习模块可见性"的教学模型。

pub mod hosting;  // 前台 —— 公开子模块
mod serving;      // 服务 —— 私有子模块（外部不可访问）
