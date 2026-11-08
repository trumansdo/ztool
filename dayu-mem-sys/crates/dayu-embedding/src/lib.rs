//! dayu-embedding: 嵌入抽象与后端实现
//!
//! 提供 EmbeddingBackend trait 的三个实现：
//! - `mock` — MockEmbeddingBackend：确定性向量（同文本→同向量）
//! - `fixed` — FixedEmbeddingBackend：固定向量（所有文本→同一向量）
//! - `ort` — OrtEmbeddingBackend：ONNX Runtime 后端（需 `onnx` feature）

mod mock;
mod fixed;
#[cfg(feature = "onnx")]
pub mod ort;

pub use mock::MockEmbeddingBackend;
pub use fixed::FixedEmbeddingBackend;
#[cfg(feature = "onnx")]
pub use ort::OrtEmbeddingBackend;
