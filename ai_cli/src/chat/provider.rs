//! AI 对话 Provider 抽象
//!
//! 定义统一的 `ChatProvider` trait，所有 AI 后端必须实现此 trait。
//! 同时定义 `ProviderType` 枚举，供 CLI 层选择提供商。

use clap::ValueEnum;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// AI 提供商枚举
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum ProviderType {
    /// OpenAI API（包括兼容接口）
    Openai,
    /// Anthropic API
    Anthropic,
    /// Ollama 本地模型
    Ollama,
}

impl ProviderType {
    /// 返回提供商字符串标识
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::Openai => "openai",
            ProviderType::Anthropic => "anthropic",
            ProviderType::Ollama => "ollama",
        }
    }
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 角色: system / user / assistant
    pub role: String,
    /// 消息内容
    #[serde(default)]
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
}

/// AI 响应
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// 回复内容
    pub content: String,
    /// 使用的模型
    pub model: String,
    /// 耗时（秒）
    pub duration_secs: f64,
    /// 输入 tokens（部分后端提供）
    pub input_tokens: Option<u32>,
    /// 输出 tokens（部分后端提供）
    pub output_tokens: Option<u32>,
}

/// 对话参数
#[derive(Debug, Clone)]
pub struct ChatParams {
    /// 模型名称
    pub model: String,
    /// 最大 tokens
    pub max_tokens: u32,
    /// 温度
    pub temperature: f32,
    /// API 基础 URL
    pub api_base: String,
    /// API Key
    pub api_key: String,
}

impl Default for ChatParams {
    fn default() -> Self {
        Self {
            model: "gpt-4o".into(),
            max_tokens: 4096,
            temperature: 0.7,
            api_base: "https://api.openai.com/v1".into(),
            api_key: String::new(),
        }
    }
}

/// AI 对话 Provider trait
///
/// 所有 AI 后端必须实现此 trait。
pub trait ChatProvider: Send + Sync {
    /// 发送单轮对话，返回完整响应
    fn chat(&self, messages: &[Message], params: &ChatParams) -> Result<ChatResponse>;

    /// 发送对话并流式输出（可选）
    fn chat_stream(
        &self,
        messages: &[Message],
        params: &ChatParams,
        on_chunk: &mut dyn FnMut(&str) -> Result<()>,
    ) -> Result<ChatResponse> {
        // 默认实现：非流式
        let result = self.chat(messages, params)?;
        on_chunk(&result.content)?;
        Ok(result)
    }
}
