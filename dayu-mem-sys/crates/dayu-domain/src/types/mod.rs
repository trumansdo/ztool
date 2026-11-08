//! 核心 newtype 与枚举定义
//!
//! 设计原则：
//! - 每种 ID 独立类型，消灭混用
//! - 受约束标量构造时验证，持有即合法
//! - 枚举封闭世界，新增变体编译器强制处理
//!
//! 子模块按概念域拆分：
//! - `ids` — 标识符 newtype
//! - `constrained` — 受约束标量
//! - `enums` — 状态与类型枚举
//! - `entities` — 核心数据模型
//! - `config` — 系统配置

mod ids;
mod constrained;
mod enums;
mod entities;
mod config;

// 扁平 re-export，保持外部 API 不变
pub use ids::*;
pub use constrained::*;
pub use enums::*;
pub use entities::*;
pub use config::*;
