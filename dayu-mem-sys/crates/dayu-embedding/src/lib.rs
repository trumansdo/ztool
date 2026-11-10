//! dayu-embedding: 嵌入抽象与后端实现
//!
//! 提供 EmbeddingBackend trait 的三种实现：
//! - `ort`  — OrtEmbeddingBackend：ONNX Runtime 后端（需 `onnx` feature）
//! - `http` — HttpBackend：OpenAI 兼容 HTTP API 后端（需 `http` feature）
//! - `test` — TestEmbeddingBackend：确定性哈希后端（需 `test-support` feature，仅测试用）
//!
//! 不提供进程内 GGUF 后端：`llama-cpp-2` 无法链接预编译 `llama.dll`
//! （符号全在，但结构体布局不兼容 → 段错误），必须从源码编译；
//! 若要用本地 GGUF 模型，起一个 `llama-server` 走 `http` 后端即可。

#[cfg(feature = "onnx")]
pub mod ort;
#[cfg(feature = "onnx")]
pub use ort::OrtEmbeddingBackend;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::HttpBackend;

#[cfg(feature = "test-support")]
pub mod test;
#[cfg(feature = "test-support")]
pub use test::TestEmbeddingBackend;
