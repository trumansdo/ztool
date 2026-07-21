//! Ollama 本地模型对话实现
//!
//! 使用 Ollama API 进行本地模型对话。
//! 默认地址: http://localhost:11434

use crate::chat::provider::{ChatParams, ChatProvider, ChatResponse, Message};
use crate::error::{AiCliError, Result};
use serde::Deserialize;
use std::time::Instant;
use tracing::info;

/// Ollama Provider
pub struct OllamaProvider;

/// Chat 请求体
#[derive(serde::Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    stream: bool,
}

#[derive(serde::Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Chat 响应体
#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    model: String,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

impl ChatProvider for OllamaProvider {
    fn chat(&self, messages: &[Message], params: &ChatParams) -> Result<ChatResponse> {
        let start = Instant::now();

        let chat_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request_body = OllamaChatRequest {
            model: params.model.clone(),
            messages: chat_messages,
            options: Some(OllamaOptions {
                num_predict: Some(params.max_tokens),
                temperature: Some(params.temperature),
            }),
            stream: false,
        };

        let client = reqwest::blocking::Client::new();
        let url = format!("{}/api/chat", params.api_base.trim_end_matches('/'));

        info!(%url, model = %params.model, "Sending Ollama chat request");

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| AiCliError::AiApi(format!("Ollama request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            return Err(AiCliError::AiApi(format!(
                "Ollama API error ({}): {}",
                status, body
            )));
        }

        let body: OllamaChatResponse = response
            .json()
            .map_err(|e| AiCliError::AiApi(format!("Ollama response parse failed: {}", e)))?;

        let duration = start.elapsed().as_secs_f64();

        info!(duration_secs = %duration, "Ollama response received");

        Ok(ChatResponse {
            content: body.message.content,
            model: body.model,
            duration_secs: duration,
            input_tokens: body.prompt_eval_count,
            output_tokens: body.eval_count,
        })
    }
}
