//! LSP 配置段定义
//!
//! TOML 结构:
//! ```toml
//! [lsp]
//! data_dir = "D:/code_workspace/jdtls-ws"
//! open_delay_ms = 200
//!
//! [lsp.jdtls]
//! command = [...]
//! data_dir = "..."
//! vars = { java = "...", launcher = "..." }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspSection {
    #[serde(default)]
    pub data_dir: Option<String>,
    #[serde(default = "default_open_delay_ms")]
    pub open_delay_ms: u64,
    #[serde(flatten)]
    pub server: HashMap<String, LspServerConfig>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LspServerConfig {
    #[serde(default)]
    pub command: Option<Vec<String>>,
    #[serde(default = "default_disabled")]
    pub disabled: bool,
    #[serde(default)]
    pub maven_settings: Option<String>,
    #[serde(default)]
    pub data_dir: Option<String>,
    #[serde(default)]
    pub vars: HashMap<String, String>,
}

fn default_open_delay_ms() -> u64 { 200 }
fn default_disabled() -> bool { true }
