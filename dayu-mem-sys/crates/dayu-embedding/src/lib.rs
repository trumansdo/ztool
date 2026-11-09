//! dayu-embedding: 嵌入抽象与后端实现
//!
//! 提供 EmbeddingBackend trait 的四种实现：
//! - `ort`  — OrtEmbeddingBackend：ONNX Runtime 后端（需 `onnx` feature）
//! - `gguf` — GgufBackend：mistral.rs GGUF 后端（需 `gguf` feature）
//! - `http` — HttpBackend：OpenAI 兼容 HTTP API 后端（需 `http` feature）
//! - `test` — TestEmbeddingBackend：确定性哈希后端（需 `test-support` feature，仅测试用）

#[cfg(feature = "onnx")]
pub mod ort;
#[cfg(feature = "onnx")]
pub use ort::OrtEmbeddingBackend;

#[cfg(feature = "gguf")]
pub mod gguf;
#[cfg(feature = "gguf")]
pub use gguf::GgufBackend;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::HttpBackend;

#[cfg(feature = "test-support")]
pub mod test;
#[cfg(feature = "test-support")]
pub use test::TestEmbeddingBackend;
