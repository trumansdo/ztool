//! 数据库操作模块
//!
//! 提供统一的数据库访问 API：
//! - 列出数据库配置
//! - 执行 SQL 查询
//! - 查看表结构
//! - 列出表
//!
//! 当前支持 Oracle，通过 trait 可扩展其他数据库。

pub mod config;
pub mod connector;
pub mod oracle;
pub mod types;

use crate::db::config::BinConfig;
use crate::db::connector::DbConnector;
use crate::db::oracle::OracleConnector;
use crate::db::types::{DbSummary, QueryResult, TableInfo};
use crate::error::{AiCliError, Result};
use std::collections::HashMap;
use tracing::info;

/// 数据库管理器
///
/// 加载配置，管理连接器注册，提供统一的操作入口。
pub struct DatabaseManager {
    config: BinConfig,
    connectors: HashMap<String, Box<dyn DbConnector>>,
}

impl DatabaseManager {
    /// 从 binconfig.toml 创建管理器
    pub fn new() -> Result<Self> {
        let config = BinConfig::load()?;
        let mut connectors: HashMap<String, Box<dyn DbConnector>> = HashMap::new();

        // 注册 Oracle 连接器
        connectors.insert("oracle".into(), Box::new(OracleConnector));

        // 后续可注册其他数据库连接器
        // connectors.insert("mysql".into(), Box::new(MySQLConnector));
        // connectors.insert("postgres".into(), Box::new(PgConnector));

        info!(db_count = config.db_tool.databases.len(), "Database manager initialized");
        Ok(Self { config, connectors })
    }

    /// 列出所有数据库摘要
    pub fn list_databases(&self) -> Vec<DbSummary> {
        self.config.summaries()
    }

    /// 执行 SQL 查询
    pub fn execute_query(&self, db_name: &str, sql: &str, limit: usize) -> Result<QueryResult> {
        let db_cfg = self.find_database(db_name)?;
        let connector = self.get_connector(&db_cfg.r#type)?;

        let mut conn = connector.connect(db_cfg)?;
        conn.execute_query(sql, limit)
    }

    /// 执行 INSERT / UPDATE / DELETE（统一事务 commit）
    pub fn execute_dml(&self, db_name: &str, sqls: &[String]) -> Result<u64> {
        let db_cfg = self.find_database(db_name)?;
        let connector = self.get_connector(&db_cfg.r#type)?;

        let mut conn = connector.connect(db_cfg)?;
        conn.execute_dml(sqls)
    }

    /// 获取表结构
    pub fn get_table_struct(&self, db_name: &str, owner: &str, table: &str) -> Result<TableInfo> {
        let db_cfg = self.find_database(db_name)?;
        let connector = self.get_connector(&db_cfg.r#type)?;

        let mut conn = connector.connect(db_cfg)?;
        conn.get_table_struct(owner, table)
    }

    /// 列出表
    pub fn list_tables(&self, db_name: &str, owner: Option<&str>) -> Result<Vec<String>> {
        let db_cfg = self.find_database(db_name)?;
        let connector = self.get_connector(&db_cfg.r#type)?;

        let mut conn = connector.connect(db_cfg)?;
        conn.list_tables(owner)
    }

    /// 查找数据库配置
    fn find_database(&self, name: &str) -> Result<&crate::db::config::DbConnectionConfig> {
        self.config.find_database(name).ok_or_else(|| {
            AiCliError::Config(format!(
                "Database '{}' not found in config. Available: {}",
                name,
                self.config.db_tool.databases.iter()
                    .map(|d| d.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
    }

    /// 获取对应类型的连接器
    fn get_connector(&self, db_type: &str) -> Result<&dyn DbConnector> {
        let t = if db_type.is_empty() { "oracle" } else { db_type };
        self.connectors.get(t).map(|c| c.as_ref()).ok_or_else(|| {
            AiCliError::Config(format!(
                "Unsupported database type '{}'. Supported: {}",
                db_type,
                self.connectors.keys()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
    }
}
