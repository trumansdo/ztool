//! 配置定义与加载
//!
//! 配置文件为 TOML 格式，存储在 `~/.config/ai_cli/config.toml`。
//! 支持多个 AI 提供商的配置。

use crate::error::{AiCliError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// LSP 语言服务器配置段 ([lsp] 父级: 全局项 + 每种语言子级 [lsp.<server_id>])
    pub lsp: LspSection,
}

/// LSP 配置段
/// TOML 结构:
/// ```toml
/// [lsp]
/// data_dir = "D:/code_space/jdtls-ws"   # workspace 根(子目录按 root 路径 sha1 隔离)
/// open_delay_ms = 200                    # didOpen 后缓冲(默认 200ms)
/// [lsp.jdtls]                            # 语言子级(flatten 到 server map)
/// command = [...]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspSection {
    /// jdtls workspace 根目录; 子目录 = sha1(root 绝对路径), 自动按项目隔离
    #[serde(default)]
    pub data_dir: Option<String>,
    /// didOpen 通知后等待缓冲, 让服务器完成文档处理再查询 (默认 200ms)
    /// 坑: rust-analyzer 首次语义分析需 ~15s(比 jdtls 慢得多),
    ///     200ms 缓冲下 hover/definition 会返回 null(非错误, 模型会误判"无符号")
    ///     Rust 场景请调大: CLI --open-delay-ms 15000 或配置 open_delay_ms
    #[serde(default = "default_open_delay_ms")]
    pub open_delay_ms: u64,
    /// 各语言服务器配置 (TOML 的 [lsp.<id>] 子表, 通过 flatten 收集)
    #[serde(flatten)]
    pub server: HashMap<String, LspServerConfig>,
}

/// open_delay_ms 配置默认值
fn default_open_delay_ms() -> u64 {
    200
}

impl Default for LspSection {
    fn default() -> Self {
        Self {
            data_dir: None,
            open_delay_ms: default_open_delay_ms(),
            server: HashMap::new(),
        }
    }
}

/// 单个语言服务器的 LSP 配置 (对应 TOML: [lsp.jdtls] 等)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LspServerConfig {
    /// 覆盖服务器启动命令 (如 ["python", "D:\\...\\jdtls.py"] 或完整 java 命令)
    #[serde(default)]
    pub command: Option<Vec<String>>,
    /// 禁用该服务器
    #[serde(default)]
    pub disabled: Option<bool>,
    /// 方案 B: 显式指定 Maven user settings.xml (注入 -Djava.configuration.maven.userSettings)
    #[serde(default)]
    pub maven_settings: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_provider: "openai".into(),
            openai: Some(ProviderConfig::default()),
            anthropic: None,
            ollama: Some(OllamaConfig::default()),
            lsp: LspSection::default(),
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
    /// 获取配置文件路径: 默认与可执行程序同目录 (config.toml)
    /// 约定(用户指定): 配置只有两种来源 —— ① exe 同级 config.toml ② --config 显式指定,
    ///                 不查系统配置目录, 保证便携可分发
    pub fn config_path() -> Result<PathBuf> {
        let exe = std::env::current_exe()
            .map_err(|e| AiCliError::Config(format!("Cannot determine exe path: {}", e)))?;
        let dir = exe
            .parent()
            .ok_or_else(|| AiCliError::Config("Cannot determine exe directory".into()))?;
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
