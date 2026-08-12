//! 统一配置定义与加载
//!
//! 配置文件为 TOML 格式，存储在 exe 同级目录 `binconfig.toml`。
//! 所有工具（db_tool/lsp_tool/web_fetch/excel_tool/ai_cli）共用此配置。
//!
//! 各子模块的配置类型定义在对应模块中：
//! - `db::config` → DbToolSection / DbConnectionConfig
//! - `lsp::config` → LspSection / LspServerConfig

use crate::db::config::DbToolSection;
use crate::error::{AiCliError, Result};
use crate::lsp::config::LspSection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================
// 根配置
// ============================================================

/// 统一根配置 —— 所有工具的配置入口
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BinConfig {
    pub default_provider: String,
    pub openai: Option<ProviderConfig>,
    pub anthropic: Option<ProviderConfig>,
    pub ollama: Option<OllamaConfig>,
    /// web_fetch 工具配置（可选）
    pub web_fetch: Option<WebFetchSection>,
    /// db_tool 工具配置（可选）
    pub db_tool: Option<DbToolSection>,
    /// LSP 语言服务器配置（可选）
    pub lsp: Option<LspSection>,
}

impl Default for BinConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".into(),
            openai: Some(ProviderConfig::default()),
            anthropic: None,
            ollama: Some(OllamaConfig::default()),
            web_fetch: None,
            db_tool: None,
            lsp: None,
        }
    }
}

// ============================================================
// AI 提供商配置
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    #[serde(default = "default_api_base")]
    pub api_base: String,
    #[serde(default = "default_model_for_provider")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_host")]
    pub host: String,
    #[serde(default = "default_ollama_model")]
    pub model: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
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

// ============================================================
// web_fetch 配置（太小，不值得单独文件）
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebFetchSection {
    pub http_proxy: Option<String>,
}

impl Default for WebFetchSection {
    fn default() -> Self {
        Self { http_proxy: None }
    }
}

// ============================================================
// 默认值函数
// ============================================================

fn default_api_base() -> String { "https://api.openai.com/v1".into() }
fn default_model_for_provider() -> String { "gpt-4o".into() }
fn default_max_tokens() -> u32 { 4096 }
fn default_temperature() -> f32 { 0.7 }
fn default_ollama_host() -> String { "http://localhost:11434".into() }
fn default_ollama_model() -> String { "qwen2.5:7b".into() }

// ============================================================
// BinConfig 方法
// ============================================================

impl BinConfig {
    pub fn config_path() -> Result<PathBuf> {
        let exe = std::env::current_exe()
            .map_err(|e| AiCliError::Config(format!("Cannot determine exe path: {}", e)))?;
        let dir = exe
            .parent()
            .ok_or_else(|| AiCliError::Config("Cannot determine exe directory".into()))?;
        Ok(dir.join("binconfig.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            let config = BinConfig::default();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| AiCliError::Config(format!("Cannot read {}: {}", path.display(), e)))?;
        let mut config: BinConfig = toml::from_str(&content)
            .map_err(|e| AiCliError::Config(format!("Cannot parse {}: {}", path.display(), e)))?;
        config.interpolate();
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| AiCliError::Config(format!("Cannot serialize config: {}", e)))?;
        std::fs::write(&path, content)
            .map_err(|e| AiCliError::Config(format!("Cannot write {}: {}", path.display(), e)))?;
        Ok(())
    }

    pub fn interpolate(&mut self) {
        let Some(lsp) = self.lsp.as_mut() else { return };
        for sc in lsp.server.values_mut() {
            if sc.vars.is_empty() {
                continue;
            }
            if let Some(cmd) = &sc.command {
                sc.command = Some(Self::interpolate_vec(cmd, &sc.vars));
            }
            if let Some(ms) = &sc.maven_settings {
                sc.maven_settings = Some(Self::interpolate_str(ms, &sc.vars));
            }
        }
    }

    fn interpolate_str(s: &str, vars: &HashMap<String, String>) -> String {
        let mut out = s.to_string();
        for (k, v) in vars {
            out = out.replace(&format!("${{{}}}", k), v);
        }
        out
    }

    fn interpolate_vec(cmd: &[String], vars: &HashMap<String, String>) -> Vec<String> {
        cmd.iter().map(|s| Self::interpolate_str(s, vars)).collect()
    }

    pub fn find_database(&self, name: &str) -> Option<&crate::db::config::DbConnectionConfig> {
        self.db_tool.as_ref()?.find_database(name)
    }

    pub fn db_summaries(&self) -> Vec<crate::db::types::DbSummary> {
        self.db_tool.as_ref().map(|d| d.summaries()).unwrap_or_default()
    }
}
