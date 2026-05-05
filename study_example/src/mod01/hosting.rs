//! # hosting 模块 — 函数可见性演示
//!
//! ## Rust 概念 — `pub fn` vs `fn`
//! 在 Rust 中，模块内部的项目（函数、结构体、枚举等）默认也是**私有的**。
//! 即使模块是 `pub` 的，模块内的函数也需要单独声明 `pub` 才能被外部访问。
//!
//! - `pub fn add_to_waitlist()` — 公开函数，外部可以通过 `hosting::add_to_waitlist()` 调用
//! - `fn seat_at_table()` — 私有函数，仅当前模块内可见
//!
//! ## Rust 概念 — `#[allow(unused)]`
//! 这是一个**属性 (attribute)**，用于抑制编译器的 "未使用" 警告。
//! 这里 `seat_at_table()` 是为教学目的而定义但从未被调用，
//! 如果不加 `#[allow(unused)]`，编译时会收到死代码警告。
//!
//! 相关属性：`#[allow(dead_code)]` — 专门抑制死代码警告而不抑制其他未使用警告。

// 要访问mod中的函数，mod本身得是pub，函数本身也必须声明为pub
pub fn add_to_waitlist() {
    println!("The waitlist ");
}
#[allow(unused)]
fn seat_at_table() {}
