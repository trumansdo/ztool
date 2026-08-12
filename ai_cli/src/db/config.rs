//! db_tool 配置段定义
//!
//! TOML 结构:
//! ```toml
//! [db_tool]
//! output_format = "json"
//!
//! [[db_tool.databases]]
//! name = "MY_DB"
//! type = "oracle"
//! ...
//! ```

use crate::error::{AiCliError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DbToolSection {
    pub output_format: String,
    pub databases: Vec<DbConnectionConfig>,
}

impl Default for DbToolSection {
    fn default() -> Self {
        Self {
            output_format: "json".into(),
            databases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConnectionConfig {
    pub name: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub env: String,
    pub user: String,
    pub password: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub service_name: String,
    #[serde(default)]
    pub lib_dir: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub extra_params: HashMap<String, String>,
}

impl DbToolSection {
    pub fn find_database(&self, name: &str) -> Option<&DbConnectionConfig> {
        self.databases.iter().find(|db| db.name.eq_ignore_ascii_case(name))
    }

    pub fn summaries(&self) -> Vec<crate::db::types::DbSummary> {
        self.databases.iter().map(|db| crate::db::types::DbSummary {
            name: db.name.clone(),
            desc: db.desc.clone(),
            db_type: db.r#type.clone(),
            env: db.env.clone(),
            user: db.user.clone(),
        }).collect()
    }
}

/// 写入示例配置文件
pub fn write_example_config(path: &Path) -> Result<()> {
    let example = r#"# binconfig.toml —— 工具配置文件
# 将此文件放在 exe 同级目录

[web_fetch]
# http_proxy = "http://proxy.example.com:8080"

[db_tool]
output_format = "json"

[[db_tool.databases]]
name = "EXAMPLE_DEV"
desc = "示例开发数据库"
type = "oracle"
env = "dev"
user = "scott"
password = "tiger"
host = "localhost"
port = 1521
service_name = "XEPDB1"
lib_dir = "D:\\oracle\\instantclient_19_22"
"#;
    std::fs::write(path, example)
        .map_err(|e| AiCliError::Config(format!("Cannot write example config: {}", e)))?;
    Ok(())
}
