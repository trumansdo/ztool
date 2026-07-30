//! db_tool —— 数据库操作 CLI 工具
//!
//! 使用方式: db_tool <COMMAND>
//!
//! 子命令:
//!   list              列出所有数据库
//!   query  <DB> <SQL> 执行 SELECT 查询
//!   execute <DB> ...   执行 INSERT/UPDATE/DELETE（--sql 可多次，统一事务）
//!   tables <DB> [OWN] 列出表
//!   struct <DB> <TBL> [OWN] 查看表结构
//!   init               生成示例配置文件

use clap::{Parser, Subcommand};
use ai_cli::{
    run_db_list, run_db_query, run_db_execute, run_db_struct, run_db_tables, run_db_init_config,
    db::types::OutputFormat as DbOutputFormat,
};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(
    name = "db_tool",
    version,
    about = "数据库操作工具",
    long_about = "基于 ai_cli 库的独立数据库操作工具。\n\
                  支持 SELECT 查询和 INSERT/UPDATE/DELETE 操作，\n\
                  Oracle 优先（可扩展 MySQL/PostgreSQL），\n\
                  多版本 Oracle Client 共存，通过 binconfig.toml 配置连接。"
)]
struct DbToolCli {
    /// 输出格式（仅 query 子命令使用）
    #[arg(long, short = 'f', value_enum, default_value_t = DbOutputFormat::Json, global = true)]
    format: DbOutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 列出所有配置的数据库连接
    List,

    /// 执行 SELECT 查询
    Query {
        /// 数据库名称（binconfig.toml 中的 name）
        #[arg(value_name = "DB")]
        db_name: String,

        /// SQL 语句
        #[arg(value_name = "SQL")]
        sql: String,

        /// 最大返回行数（默认 100）
        #[arg(long, short = 'n', default_value_t = 100)]
        limit: usize,
    },

    /// 执行 INSERT / UPDATE / DELETE（--sql 可多次指定，统一事务 commit）
    Execute {
        /// 数据库名称
        #[arg(value_name = "DB")]
        db_name: String,

        /// SQL 语句（可多次指定）
        #[arg(long = "sql", value_name = "SQL", num_args = 1.., required = true)]
        sql: Vec<String>,
    },

    /// 查看表结构（列信息 + 索引信息）
    #[command(name = "struct")]
    TableStruct {
        /// 数据库名称
        #[arg(value_name = "DB")]
        db_name: String,

        /// 表名
        #[arg(value_name = "TABLE")]
        table: String,

        /// 表所有者（Schema），不指定则使用当前用户
        #[arg(value_name = "OWNER", default_value = "")]
        owner: String,
    },

    /// 列出数据库中的表
    Tables {
        /// 数据库名称
        #[arg(value_name = "DB")]
        db_name: String,

        /// 表所有者（Schema），不指定则列出当前用户的表
        #[arg(value_name = "OWNER")]
        owner: Option<String>,
    },

    /// 在 exe 同级目录生成示例 binconfig.toml
    Init,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()))
        .with_target(false)
        .without_time()
        .init();

    let cli = DbToolCli::parse();
    let format = &cli.format;

    match &cli.command {
        Commands::List => run_db_list()?,
        Commands::Query { db_name, sql, limit } => {
            run_db_query(db_name, sql, *limit, format)?;
        }
        Commands::Execute { db_name, sql } => {
            run_db_execute(db_name, sql)?;
        }
        Commands::TableStruct { db_name, table, owner } => {
            let owner = if owner.is_empty() { None } else { Some(owner.as_str()) };
            run_db_struct(db_name, owner.unwrap_or(""), table, format)?;
        }
        Commands::Tables { db_name, owner } => {
            run_db_tables(db_name, owner.as_deref(), format)?;
        }
        Commands::Init => run_db_init_config()?,
    }

    Ok(())
}
