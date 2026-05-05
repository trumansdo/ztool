//! # 模块系统综合演示
//!
//! 这个文件集中演示了 Rust 模块系统的所有关键概念。
//!
//! ## Rust 概念 — 三类路径语法
//!
//! ### 1. `use crate::` — 从库根开始的绝对路径
//! ```rust
//! use study_example::mod01::hosting;  // 引用 library crate 的模块
//! ```
//! `study_example` 是库的名称（在 Cargo.toml 中定义的 name）。
//! bin 目标可以像引用外部 crate 一样引用自己的 library。
//!
//! ### 2. `self::` — 当前模块路径
//! ```rust
//! self::back_of_house::cook_order();
//! ```
//! 等价于 `back_of_house::cook_order()`，显式表示从当前模块找起。
//!
//! ### 3. `super::` — 父模块路径
//! ```rust
//! super::serve_order();  // 从 back_of_house 跳到父模块
//! ```
//! 在嵌套模块中访问父模块/兄弟模块的函数。
//!
//! ## Rust 概念 — 内联模块 `mod foo { }`
//! ```rust
//! mod back_of_house {
//!     fn fix_incorrect_order() { ... }
//!     pub fn cook_order() {}
//! }
//! ```
//! 模块不一定需要单独文件，可以直接在代码中用花括号定义。
//! 小模块用内联定义更方便，大模块用文件组织更清晰。
//!
//! ## Rust 概念 — `#[cfg(test)]` 条件编译
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     #[test]
//!     fn it_works() { ... }
//! }
//! ```
//! - `#[cfg(test)]` — 只在 `cargo test` 编译时包含此模块
//! - `use super::*` — 通配符导入父模块的所有公开项
//! - `#[test]` — 标记为测试函数
//! 这种"测试模块紧贴被测试代码"的模式是 Rust 的惯用法。

//访问父级目录的库得通过crate来访问
use study_example::mod01::hosting;

fn serve_order() {
    self::back_of_house::cook_order();
    back_of_house::cook_order();
}

// 厨房模块
mod back_of_house {
    fn fix_incorrect_order() {
        cook_order();
        super::serve_order();  // super 跳到父模块
    }
    pub fn cook_order() {}
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
#[cfg(test)]
mod tests {
    use super::*;  // 导入父模块的所有项

    #[test]
    fn it_works() {
        println!("Hello, world!");
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

fn main() {
    println!("Hello, world!");
    study_example::mod01::hosting::add_to_waitlist();
}
