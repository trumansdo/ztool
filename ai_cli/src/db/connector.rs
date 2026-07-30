//! 数据库连接器 Trait 定义
//!
//! 定义统一的数据库访问接口，不同数据库通过实现此 trait 接入。
//! 后续添加 MySQL/PostgreSQL 等只需新增实现模块。

use crate::db::config::DbConnectionConfig;
use crate::db::types::{QueryResult, TableInfo};
use crate::error::Result;

/// 数据库连接器工厂
///
/// 每种数据库类型提供一个实现，负责创建连接。
pub trait DbConnector: Send + Sync {
    /// 创建数据库连接
    fn connect(&self, config: &DbConnectionConfig) -> Result<Box<dyn DbConnection>>;

    /// 返回数据库类型标识
    fn db_type(&self) -> &'static str;
}

/// 数据库连接（单次会话）
///
/// 每次操作创建新连接，用完即关闭。
/// 不实现连接池，以简化 Oracle 多版本共存场景。
pub trait DbConnection {
    /// 执行 SELECT 查询，返回结构化结果
    fn execute_query(&mut self, sql: &str, max_rows: usize) -> Result<QueryResult>;

    /// 执行 INSERT / UPDATE / DELETE（统一事务 commit）
    fn execute_dml(&mut self, sqls: &[String]) -> Result<u64>;

    /// 获取表结构（列 + 索引）
    fn get_table_struct(&mut self, owner: &str, table: &str) -> Result<TableInfo>;

    /// 列出指定 owner 下的所有表
    fn list_tables(&mut self, owner: Option<&str>) -> Result<Vec<String>>;
}
