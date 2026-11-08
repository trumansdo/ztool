//! dayu-storage: 存储抽象与实现
//!
//! - `trivium` — TriviumStorage：TriviumDB 三模引擎（唯一实现）
//! - `testing` — 测试辅助（feature = "test-support"）

pub mod trivium;

#[cfg(feature = "test-support")]
pub mod testing;

pub use trivium::TriviumStorage;
