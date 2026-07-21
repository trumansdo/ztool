//! AI 对话模块
//!
//! 提供与多种 AI 模型后端（OpenAI / Anthropic / Ollama）的交互能力。
//! 每个后端实现统一的 `ChatProvider` trait，支持流式和非流式对话。

pub mod provider;
pub mod openai;
pub mod anthropic;
pub mod ollama;

pub use provider::*;
