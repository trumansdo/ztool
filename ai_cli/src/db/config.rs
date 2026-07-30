//! 数据库配置文件加载
//!
//! 配置文件为 `binconfig.toml`，位于 exe 同级目录。
//! 支持配置多个数据库连接，每个连接可指定独立的 Oracle Client 路径。

use crate::error::{AiCliError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 根配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BinConfig {
    /// db_tool 工具配置
    pub db_tool: DbToolConfig,
    /// web_fetch 工具配置
    pub web_fetch: WebFetchConfig,
}

/// db_tool 工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DbToolConfig {
    /// 默认输出格式
    pub output_format: String,
    /// 数据库连接列表
    pub databases: Vec<DbConnectionConfig>,
}

impl Default for DbToolConfig {
    fn default() -> Self {
        Self {
            output_format: "json".into(),
            databases: Vec::new(),
        }
    }
}

impl Default for BinConfig {
    fn default() -> Self {
        Self {
            db_tool: DbToolConfig::default(),
            web_fetch: WebFetchConfig::default(),
        }
    }
}

/// web_fetch 工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebFetchConfig {
    /// HTTP 代理地址，如 http://proxy.example.com:8080
    pub http_proxy: Option<String>,
}

impl Default for WebFetchConfig {
    fn default() -> Self {
        Self { http_proxy: None }
    }
}

/// 单个数据库连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConnectionConfig {
    /// 数据库名称（唯一标识）
    pub name: String,
    /// 数据库描述
    #[serde(default)]
    pub desc: String,
    /// 数据库类型: oracle / mysql / postgres
    #[serde(default)]
    pub r#type: String,
    /// 环境: dev / test / prod
    #[serde(default)]
    pub env: String,
    /// 用户名
    pub user: String,
    /// 密码
    pub password: String,
    /// 主机地址
    #[serde(default)]
    pub host: String,
    /// 端口
    #[serde(default)]
    pub port: u16,
    /// 服务名 / SID（Oracle）
    #[serde(default)]
    pub service_name: String,
    /// Oracle Instant Client 路径（多版本共存关键）
    #[serde(default)]
    pub lib_dir: Option<String>,
    /// 连接 URL 模板（可选，直接指定时忽略 host/port/service_name）
    #[serde(default)]
    pub url: Option<String>,
    /// 额外连接参数
    #[serde(default)]
    pub extra_params: HashMap<String, String>,
}

impl BinConfig {
    /// 查找 exe 所在目录并加载 binconfig.toml
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Err(AiCliError::Config(format!(
                "binconfig.toml not found at: {}\n\
                 Please create one with database connection configurations.",
                config_path.display()
            )));
        }
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| AiCliError::Config(format!("Cannot read {}: {}", config_path.display(), e)))?;
        let config: BinConfig = toml::from_str(&content)
            .map_err(|e| AiCliError::Config(format!("Cannot parse {}: {}", config_path.display(), e)))?;
        Ok(config)
    }

    /// 获取 binconfig.toml 路径：exe 同级目录
    pub fn config_path() -> Result<PathBuf> {
        let exe = std::env::current_exe()
            .map_err(|e| AiCliError::Config(format!("Cannot get exe path: {}", e)))?;
        let exe_dir = exe.parent()
            .ok_or_else(|| AiCliError::Config("Cannot get exe directory".into()))?;
        Ok(exe_dir.join("binconfig.toml"))
    }

    /// 根据名称查找数据库配置
    pub fn find_database(&self, name: &str) -> Option<&DbConnectionConfig> {
        self.db_tool.databases.iter().find(|db| db.name.eq_ignore_ascii_case(name))
    }

    /// 获取所有数据库摘要（隐藏密码）
    pub fn summaries(&self) -> Vec<crate::db::types::DbSummary> {
        self.db_tool.databases.iter().map(|db| crate::db::types::DbSummary {
            name: db.name.clone(),
            desc: db.desc.clone(),
            db_type: db.r#type.clone(),
            env: db.env.clone(),
            user: db.user.clone(),
        }).collect()
    }
}

/// 写入示例配置文件到指定路径
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

# [[db_tool.databases]]
# name = "EXAMPLE_MYSQL"
# desc = "MySQL示例"
# type = "mysql"
# env = "dev"
# user = "root"
# password = "root"
# host = "localhost"
# port = 3306
# service_name = "testdb"
"#;
    std::fs::write(path, example)
        .map_err(|e| AiCliError::Config(format!("Cannot write example config: {}", e)))?;
    Ok(())
}
