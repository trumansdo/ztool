//! # 集成测试 — 最简单的测试演示
//!
//! ## Rust 概念 — `#[test]` 集成测试
//! `tests/` 目录下的每个 `.rs` 文件都是一个独立的测试 crate。
//! 与单元测试（在源代码中 `#[cfg(test)] mod tests {}`）不同，
//! 集成测试只能访问 crate 的公开 API，模拟外部用户的使用方式。
//!
//! ## Rust 概念 — `assert_eq!` 宏
//! 断言两个值相等。不相等时 panic，测试失败。
//! Rust 标准库提供三种断言宏：
//! - `assert!` — 条件为 true
//! - `assert_eq!` — 左值 == 右值
//! - `assert_ne!` — 左值 != 右值
//!
//! 运行方式：`cargo test -p study_example --test demo`

#[test]
fn demo() {
    assert_eq!(2 + 2, 4);
}
