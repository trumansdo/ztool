//! dayu-domain: 大禹水利记忆网络 领域类型与抽象 trait
//!
//! 本 crate 定义所有核心 newtype、枚举、结构体和抽象 trait。
//! 零外部依赖（除标准库和必要的序列化/错误处理）。
//! 所有模块都依赖本 crate，本 crate 不依赖任何 workspace 内 crate。

pub mod types;
pub mod traits;
pub mod error;
pub mod math;

// 重导出常用类型
pub use types::*;
pub use traits::*;
pub use error::*;
