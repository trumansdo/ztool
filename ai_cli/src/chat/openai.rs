//! OpenAI API 对话实现
//!
//! 使用 OpenAI Chat Completions API 进行对话。
//! 支持非流式响应，兼容所有 OpenAI 兼容接口（如 Azure、自定义网关）。

use crate::chat::provider::{ChatParams, ChatProvider, ChatResponse, Message};
use crate::error::{AiCliError, Result};
use serde::Deserialize;
use std::time::Instant;
use tracing::info;

/// OpenAI Provider
pub struct OpenAIProvider;

/// Chat Completions 请求体
#[derive(serde::Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(serde::Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// Chat Completions 响应体
#[derive(Deserialize)]
struct ChatResponseBody {
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    prompt_tokens: Option<u32>,
    #[serde(default)]
    completion_tokens: Option<u32>,
}

impl ChatProvider for OpenAIProvider {
    fn chat(&self, messages: &[Message], params: &ChatParams) -> Result<ChatResponse> {
        let start = Instant::now();

        // 构造请求
        let chat_messages: Vec<ChatMessage> = messages
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request_body = ChatRequest {
            model: params.model.clone(),
            messages: chat_messages,
            max_tokens: params.max_tokens,
            temperature: params.temperature,
            stream: false,
        };

        let client = reqwest::blocking::Client::new();
        let url = format!("{}/chat/completions", params.api_base.trim_end_matches('/'));

        info!(%url, model = %params.model, "Sending OpenAI chat request");

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", params.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| AiCliError::AiApi(format!("OpenAI request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(AiCliError::AiApi(format!(
                "OpenAI API error ({}): {}",
                status, body
            )));
        }

        let body: ChatResponseBody = response
            .json()
            .map_err(|e| AiCliError::AiApi(format!("OpenAI response parse failed: {}", e)))?;

        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        let duration = start.elapsed().as_secs_f64();

        info!(duration_secs = %duration, content_len = %content.len(), "OpenAI response received");

        Ok(ChatResponse {
            content,
            model: params.model.clone(),
            duration_secs: duration,
            input_tokens: body.usage.as_ref().and_then(|u| u.prompt_tokens),
            output_tokens: body.usage.as_ref().and_then(|u| u.completion_tokens),
        })
    }
}
