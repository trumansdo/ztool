//! 配置定义与加载
//!
//! 配置文件为 TOML 格式，存储在 `~/.config/ai_cli/config.toml`。
//! 支持多个 AI 提供商的配置。

use crate::error::{AiCliError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 默认使用的 AI 提供商
    pub default_provider: String,
    /// 各 AI 提供商配置
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub ollama: Option<OllamaConfig>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_provider: "openai".into(),
            openai: Some(ProviderConfig::default()),
            anthropic: None,
            ollama: Some(OllamaConfig::default()),
        }
    }
}

/// AI 提供商通用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API Key
    pub api_key: String,
    /// API 基础 URL（可选，用于自定义网关）
    #[serde(default = "default_api_base")]
    pub api_base: String,
    /// 默认模型
    #[serde(default = "default_model_for_provider")]
    pub model: String,
    /// 最大 tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// 温度
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: default_api_base(),
            model: default_model_for_provider(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

/// Ollama 特有配置（本地模型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama 服务地址
    #[serde(default = "default_ollama_host")]
    pub host: String,
    /// 默认模型
    #[serde(default = "default_ollama_model")]
    pub model: String,
    /// 最大 tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// 温度
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: default_ollama_host(),
            model: default_ollama_model(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
        }
    }
}

// --- 默认值函数 ---

fn default_api_base() -> String {
    "https://api.openai.com/v1".into()
}

fn default_model_for_provider() -> String {
    "gpt-4o".into()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.7
}

fn default_ollama_host() -> String {
    "http://localhost:11434".into()
}

fn default_ollama_model() -> String {
    "qwen2.5:7b".into()
}

impl Settings {
    /// 获取配置文件路径: `~/.config/ai_cli/config.toml`
    pub fn config_path() -> Result<PathBuf> {
        let base = dirs::config_dir()
            .ok_or_else(|| AiCliError::Config("Cannot determine config directory".into()))?;
        let dir = base.join("ai_cli");
        std::fs::create_dir_all(&dir)
            .map_err(|e| AiCliError::Config(format!("Cannot create config dir: {}", e)))?;
        Ok(dir.join("config.toml"))
    }

    /// 从磁盘加载配置，如果文件不存在则返回默认值
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            let config = Settings::default();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| AiCliError::Config(format!("Cannot read config file: {}", e)))?;
        let config: Settings = toml::from_str(&content)
            .map_err(|e| AiCliError::Config(format!("Cannot parse config file: {}", e)))?;
        Ok(config)
    }

    /// 保存配置到磁盘
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| AiCliError::Config(format!("Cannot serialize config: {}", e)))?;
        std::fs::write(&path, content)
            .map_err(|e| AiCliError::Config(format!("Cannot write config file: {}", e)))?;
        Ok(())
    }
}
