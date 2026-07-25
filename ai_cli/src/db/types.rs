//! 数据库模块共享类型定义

use serde::{Deserialize, Serialize};

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// 列名列表
    pub columns: Vec<String>,
    /// 数据行（每行为 Vec<String>）
    pub rows: Vec<Vec<String>>,
    /// 实际返回行数
    pub row_count: usize,
    /// SQL 是否可能还有更多行
    pub truncated: bool,
}

/// 表信息摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSummary {
    pub owner: String,
    pub table_name: String,
    pub num_rows: Option<u64>,
}

/// 表结构信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub table_name: String,
    pub owner: String,
    pub comment: Option<String>,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub column_id: u32,
    pub column_name: String,
    pub comment: Option<String>,
    pub nullable: String,
    pub data_type: String,
    pub data_length: Option<i64>,
    pub data_precision: Option<i64>,
    pub data_scale: Option<i64>,
    pub data_default: Option<String>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub index_name: String,
    pub column_name: String,
    pub column_position: u32,
    pub index_type: Option<String>,
    pub uniqueness: Option<String>,
}

/// 数据库摘要信息（用于 list 命令）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSummary {
    pub name: String,
    pub desc: String,
    pub db_type: String,
    pub env: String,
    pub user: String,
}

/// 输出格式
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// JSON（默认）
    Json,
    /// CSV
    Csv,
    /// 对齐表格文本
    Table,
}
