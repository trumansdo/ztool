//! # study_example 默认二进制入口
//!
//! ## Rust 概念 — `Box<dyn Error>`
//! `Box<dyn Error>` 是 Rust 中最通用的错误类型：
//! - `dyn Error` — trait 对象，代表"任何实现了 Error trait 的类型"
//! - `Box<...>` — 堆分配的智能指针，因为 trait 对象的大小在编译期不确定
//!
//! 这种写法允许 main 函数通过 `?` 运算符处理任意类型的错误，
//! 而无需显式列出所有可能的错误类型。
//!
//! ## Rust 概念 — main 函数
//! `fn main()` 是 Rust 程序的入口点（类似 C 的 `int main()`）。
//! 这里 main 返回 `Result` 是 Rust 的惯用法：程序可以在 main 中优雅处理错误。

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
