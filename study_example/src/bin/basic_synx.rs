//! # 基础 Rust 语法示例
//!
//! ## Rust 概念 — 结构体定义
//! ```rust
//! struct Individual {
//!     real_life: u32,     // 字段名: 类型
//!     virtual_life: u32,
//!     is_fix: bool,
//! }
//! ```
//! - `struct` 关键字定义结构体（类似 C 的 struct、Java 的 class）
//! - `u32` — 32 位无符号整数（0 到 42 亿）
//! - `bool` — 布尔类型（true / false）
//!
//! ## Rust 概念 — 结构体实例化
//! ```rust
//! let person = Individual {
//!     real_life: 25,
//!     virtual_life: 30,
//!     is_fix: true,       // 最后一个字段的逗号是可选的，但推荐保留
//! };
//! ```
//! - `let` 用于变量绑定（Rust 中变量默认不可变，需要 `let mut` 才能修改）
//! - 字段名必须与定义一致，顺序可以任意
//!
//! ## Rust 概念 — `let _` 忽略未使用变量
//! 如果定义了变量但未使用，Rust 编译器会警告。
//! 这里 `person` 在创建后从未使用，实际运行时会看到 `unused variable` 警告。
//! 可以用 `let _person = ...` 来消除警告。

struct Individual {
    real_life: u32,
    virtual_life: u32,
    is_fix: bool,
}

fn main() {
    println!("Hello, world!");
    let person = Individual {
        real_life: 25,
        virtual_life: 30,
        is_fix: true,
    };
}
