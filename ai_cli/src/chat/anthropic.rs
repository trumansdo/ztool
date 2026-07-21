//! Anthropic API 对话实现
//!
//! 使用 Anthropic Messages API 进行对话。

use crate::chat::provider::{ChatParams, ChatProvider, ChatResponse, Message};
use crate::error::{AiCliError, Result};
use serde::Deserialize;
use std::time::Instant;
use tracing::info;

/// Anthropic Provider
pub struct AnthropicProvider;

/// Messages API 请求体
#[derive(serde::Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(serde::Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Messages API 响应体
#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    model: String,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: Option<u32>,
    #[serde(default)]
    output_tokens: Option<u32>,
}

impl ChatProvider for AnthropicProvider {
    fn chat(&self, messages: &[Message], params: &ChatParams) -> Result<ChatResponse> {
        let start = Instant::now();

        // 分离 system 消息
        let (system, user_messages): (Vec<&Message>, Vec<&Message>) = messages
            .iter()
            .partition(|m| m.role == "system");

        let chat_messages: Vec<AnthropicMessage> = user_messages
            .iter()
            .map(|m| AnthropicMessage {
                role: if m.role == "assistant" { "assistant".into() } else { "user".into() },
                content: m.content.clone(),
            })
            .collect();

        let request_body = MessagesRequest {
            model: params.model.clone(),
            messages: chat_messages,
            max_tokens: params.max_tokens,
            temperature: Some(params.temperature),
            system: system.first().map(|m| m.content.clone()),
        };

        let client = reqwest::blocking::Client::new();
        let url = format!(
            "{}/messages",
            params.api_base.trim_end_matches('/')
        );

        info!(%url, model = %params.model, "Sending Anthropic chat request");

        let response = client
            .post(&url)
            .header("x-api-key", &params.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| AiCliError::AiApi(format!("Anthropic request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(AiCliError::AiApi(format!(
                "Anthropic API error ({}): {}",
                status, body
            )));
        }

        let body: MessagesResponse = response
            .json()
            .map_err(|e| AiCliError::AiApi(format!("Anthropic response parse failed: {}", e)))?;

        let content: String = body
            .content
            .into_iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("\n");

        let duration = start.elapsed().as_secs_f64();

        info!(duration_secs = %duration, content_len = %content.len(), "Anthropic response received");

        Ok(ChatResponse {
            content,
            model: body.model,
            duration_secs: duration,
            input_tokens: body.usage.as_ref().and_then(|u| u.input_tokens),
            output_tokens: body.usage.as_ref().and_then(|u| u.output_tokens),
        })
    }
}
