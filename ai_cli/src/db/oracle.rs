//! Oracle 数据库连接器实现
//!
//! 使用 `oracle` crate (v0.6) 连接 Oracle 数据库。
//! 支持多版本 Oracle Instant Client 共存：每个连接可指定独立的 `lib_dir`。
//!
//! # 连接方式
//! - 优先使用 `url` 字段直接连接
//! - 否则通过 host/port/service_name 构建连接字符串
//!
//! # 多版本策略
//! 每次查询通过设置 PATH 环境变量切换到对应的 Oracle Client 目录，
//! 然后创建新连接，用完即关闭。不启用连接池。

use crate::db::config::DbConnectionConfig;
use crate::db::connector::{DbConnection, DbConnector};
use crate::db::types::{ColumnInfo, IndexInfo, QueryResult, TableInfo};
use crate::error::{AiCliError, Result};
use oracle::Connection;
use tracing::{debug, info};

pub struct OracleConnector;

impl DbConnector for OracleConnector {
    fn connect(&self, config: &DbConnectionConfig) -> Result<Box<dyn DbConnection>> {
        let conn = OracleConnection::open(config)?;
        Ok(Box::new(conn))
    }

    fn db_type(&self) -> &'static str {
        "oracle"
    }
}

pub struct OracleConnection {
    inner: Connection,
    db_name: String,
}

impl OracleConnection {
    fn open(config: &DbConnectionConfig) -> Result<Self> {
        // 多版本：设置 Oracle Client 路径到 PATH 前
        if let Some(ref lib_dir) = config.lib_dir {
            prepend_path(lib_dir);
        }

        let connect_string = build_connect_string(config);
        debug!(db = %config.name, %connect_string, "Connecting to Oracle");

        let conn = Connection::connect(
            &config.user,
            &config.password,
            &connect_string,
        )
        .map_err(|e| {
            AiCliError::Config(format!(
                "Oracle connection failed [{}]: {}",
                config.name, e
            ))
        })?;

        info!(db = %config.name, "Oracle connected");
        Ok(Self {
            inner: conn,
            db_name: config.name.clone(),
        })
    }

    /// 将值转为字符串，处理 NULL
    fn row_str(row: &oracle::Row, idx: usize) -> String {
        row.get::<usize, String>(idx)
            .unwrap_or_else(|_| "NULL".to_string())
    }

    fn opt_row_str(row: &oracle::Row, idx: usize) -> Option<String> {
        row.get::<usize, String>(idx).ok()
    }
}

impl DbConnection for OracleConnection {
    fn execute_query(&mut self, sql: &str, max_rows: usize) -> Result<QueryResult> {
        debug!(db = %self.db_name, %sql, max_rows, "Executing query");

        let result_set = self
            .inner
            .query(sql, &[])
            .map_err(|e| AiCliError::General(format!("Query failed: {}", e)))?;

        // 获取列信息
        let column_names: Vec<String> = result_set
            .column_info()
            .iter()
            .map(|ci| ci.name().to_string())
            .collect();

        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut truncated = false;

        for row_result in result_set {
            if rows.len() >= max_rows {
                truncated = true;
                break;
            }
            let row = row_result
                .map_err(|e| AiCliError::General(format!("Row fetch error: {}", e)))?;
            let values: Vec<String> = (0..column_names.len())
                .map(|i| Self::row_str(&row, i))
                .collect();
            rows.push(values);
        }

        let row_count = rows.len();
        Ok(QueryResult {
            columns: column_names,
            rows,
            row_count,
            truncated,
        })
    }

    fn execute_dml(&mut self, sql: &str) -> Result<u64> {
        debug!(db = %self.db_name, %sql, "Executing DML");

        let stmt = self
            .inner
            .execute(sql, &[])
            .map_err(|e| AiCliError::General(format!("DML execution failed: {}", e)))?;

        let rows_affected = stmt.row_count().unwrap_or(0);
        info!(db = %self.db_name, rows_affected, "DML executed");
        Ok(rows_affected)
    }

    fn get_table_struct(&mut self, owner: &str, table: &str) -> Result<TableInfo> {
        info!(db = %self.db_name, %owner, %table, "Getting table structure");

        // 表注释
        let comment = self
            .inner
            .query(
                "SELECT comments FROM all_tab_comments \
                 WHERE lower(table_name) = lower(:1) AND lower(owner) = lower(:2)",
                &[&table, &owner],
            )
            .ok()
            .and_then(|rows| {
                rows.into_iter()
                    .next()
                    .and_then(|r| r.ok())
                    .and_then(|row| row.get::<usize, String>(0).ok())
            });

        // 列信息
        let columns = self
            .inner
            .query(
                "SELECT a.column_id, a.column_name, cc.comments, \
                        a.nullable, a.data_type, a.data_length, \
                        a.data_precision, a.data_scale, a.data_default \
                   FROM all_tab_columns a \
                   JOIN all_tab_comments c \
                     ON c.table_name = a.table_name AND c.owner = a.owner \
                   LEFT JOIN all_col_comments cc \
                     ON cc.table_name = c.table_name \
                    AND cc.column_name = a.column_name \
                    AND cc.owner = c.owner \
                  WHERE lower(a.table_name) = lower(:1) \
                    AND lower(a.owner) = lower(:2) \
                  ORDER BY a.column_id",
                &[&table, &owner],
            )
            .map_err(|e| AiCliError::General(format!("Column query failed: {}", e)))?
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|row| ColumnInfo {
                column_id: row.get(0).unwrap_or(0u32),
                column_name: Self::row_str(&row, 1),
                comment: Self::opt_row_str(&row, 2),
                nullable: Self::row_str(&row, 3),
                data_type: Self::row_str(&row, 4),
                data_length: row.get(5).unwrap_or(None),
                data_precision: row.get(6).unwrap_or(None),
                data_scale: row.get(7).unwrap_or(None),
                data_default: Self::opt_row_str(&row, 8),
            })
            .collect();

        // 索引信息
        let indexes = self
            .inner
            .query(
                "SELECT t.index_name, t.column_name, t.column_position, \
                        i.index_type, i.uniqueness \
                   FROM all_indexes i \
                   JOIN all_ind_columns t \
                     ON t.index_name = i.index_name \
                    AND t.table_owner = i.table_owner \
                    AND t.table_name = i.table_name \
                  WHERE lower(i.table_owner) = lower(:1) \
                    AND lower(t.table_name) = lower(:2) \
                  ORDER BY t.index_name, t.column_position",
                &[&owner, &table],
            )
            .map_err(|e| AiCliError::General(format!("Index query failed: {}", e)))?
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|row| IndexInfo {
                index_name: Self::row_str(&row, 0),
                column_name: Self::row_str(&row, 1),
                column_position: row.get(2).unwrap_or(0u32),
                index_type: Self::opt_row_str(&row, 3),
                uniqueness: Self::opt_row_str(&row, 4),
            })
            .collect();

        Ok(TableInfo {
            table_name: table.to_string(),
            owner: owner.to_string(),
            comment,
            columns,
            indexes,
        })
    }

    fn list_tables(&mut self, owner: Option<&str>) -> Result<Vec<String>> {
        info!(db = %self.db_name, ?owner, "Listing tables");

        let result_set = if let Some(owner) = owner {
            self.inner.query(
                "SELECT table_name FROM all_tables \
                 WHERE lower(owner) = lower(:1) ORDER BY table_name",
                &[&owner as &dyn oracle::sql_type::ToSql],
            )
        } else {
            self.inner.query(
                "SELECT table_name FROM user_tables ORDER BY table_name",
                &[],
            )
        };

        let tables = result_set
            .map_err(|e| AiCliError::General(format!("List tables failed: {}", e)))?
            .into_iter()
            .filter_map(|r| r.ok())
            .filter_map(|row| row.get::<usize, String>(0).ok())
            .collect();

        Ok(tables)
    }
}

// ---- helpers ----

fn build_connect_string(config: &DbConnectionConfig) -> String {
    if let Some(ref url) = config.url {
        if !url.is_empty() {
            return url.clone();
        }
    }
    if config.host.is_empty() {
        return String::new();
    }
    format!("{}:{}/{}", config.host, config.port, config.service_name)
}

/// 将目录添加到 PATH 最前面（Oracle 多版本共存关键）
fn prepend_path(lib_dir: &str) {
    let separator = if cfg!(windows) { ";" } else { ":" };
    // Oracle crate 需要 bin 目录在 PATH 中
    let bin_dir = std::path::Path::new(lib_dir);
    let current_path = std::env::var("PATH").unwrap_or_default();

    // 避免重复添加
    let path_str = bin_dir.to_string_lossy();
    if !current_path
        .split(separator)
        .any(|p| p == path_str.as_ref())
    {
        let new_path = format!("{}{}{}", path_str, separator, current_path);
        debug!(%path_str, "Prepending Oracle client to PATH");
        // Safety: setting PATH is safe, just modifies environment for child processes/libs
        unsafe {
            std::env::set_var("PATH", &new_path);
        }
    }
}
