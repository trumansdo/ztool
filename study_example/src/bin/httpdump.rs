//! # httpdump — HTTP 抓包工具的薄入口
//!
//! ## Rust 概念 — bin 调用 lib
//! `use study_example::netdump::httpdump;` — 二进制目标引用同项目 library 的模块。
//! 这是 Rust 项目中常见的分层架构：核心逻辑放 library，bin 只做入口和传参。
//!
//! ## Rust 概念 — `let _ =` 忽略返回值
//! `httpdump::run()` 返回 `Result<(), Box<dyn Error>>`。
//! `let _ =` 显式丢弃返回值（避免 unused-must-use 警告），
//! 比 `httpdump::run();` 少了返回值检查的提醒。
//! 这里假设 run() 的失败不影响后续流程。

use std::error::Error;

use study_example::netdump::httpdump;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = httpdump::run();
    Ok(())
}
