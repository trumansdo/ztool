//! ai_cli 库 —— AI 模型专用 CLI 交互工具的公共 API
//!
//! 提供可复用的核心逻辑：
//! - 网页内容获取（HTTP + Headless 浏览器）
//! - 数据库操作（Oracle 优先可扩展）
//! - 配置管理
//!
//! # 架构
//! ```text
//! lib.rs  (public API)
//!   ├── config    → 配置加载/保存
//!   ├── fetch     → 网页获取
//!   │   ├── http      → reqwest 静态获取
//!   │   ├── browser   → headless_chrome 渲染
//!   │   └── extract   → 正文提取
//!   ├── output    → 输出格式化
//!   │   └── format    → OutputFormat 枚举 + 格式化函数
//!   ├── db        → 数据库操作
//!   │   ├── config    → binconfig.toml 加载
//!   │   ├── connector → DbConnector / DbConnection trait
//!   │   ├── oracle    → Oracle 实现
//!   │   └── types     → 共享类型定义
//!   └── error     → 统一错误类型
//! ```

pub mod config;
pub mod fetch;
pub mod output;
pub mod error;
pub mod db;
pub mod excel;
pub mod lsp;

// 重新导出高频类型，方便 CLI 层使用
pub use output::format::OutputFormat;

use config::Settings;
use error::Result;
use fetch::extract::{extract_content, is_likely_js_rendered, ExtractOptions};
use output::format::format_output;
use std::io::{self, Write};
use tracing::{error, info};

// ============================
// Fetch API
// ============================

/// 执行网页获取并输出结果
///
/// 支持静态 HTTP 和 Headless 浏览器两种模式，自动检测 SPA 降级。
pub async fn run_fetch(
    url: &str,
    force_spa: bool,
    output_path: Option<&std::path::Path>,
    format: &OutputFormat,
    browser_path: Option<&str>,
    timeout: u64,
    proxy_url: Option<&str>,
) -> Result<()> {
    info!(%url, force_spa, ?format, "Starting fetch command");

    let (html, fetch_mode, final_url, status_code, content_type) = if force_spa {
        info!("Forcing headless browser mode");
        let result = fetch::browser::fetch_with_browser(url, browser_path, timeout)?;
        (result.html, "headless_browser", result.final_url, None::<u16>, None)
    } else {
        match fetch::http::fetch_static(url, timeout, proxy_url).await {
            Ok(result) => {
                if is_likely_js_rendered(&result.html) {
                    info!("Detected JS-rendered page, falling back to browser");
                    match fetch::browser::fetch_with_browser(url, browser_path, timeout) {
                        Ok(br) => (br.html, "headless_browser", br.final_url, None, None),
                        Err(e) => {
                            error!(%e, "Browser fallback failed, using static HTML");
                            (result.html, "static_http", result.final_url, Some(result.status), result.content_type)
                        }
                    }
                } else {
                    (result.html, "static_http", result.final_url, Some(result.status), result.content_type)
                }
            }
            Err(e) => {
                error!(%e, "Static HTTP fetch failed, trying browser fallback");
                let result = fetch::browser::fetch_with_browser(url, browser_path, timeout)?;
                (result.html, "headless_browser", result.final_url, None, None)
            }
        }
    };

    let opts = ExtractOptions {
        final_url: Some(final_url),
        status_code,
        content_type,
        fetch_mode: Some(fetch_mode),
    };
    let content = extract_content(&html, url, &opts)?;
    let formatted = format_output(&content, format);

    if let Some(path) = output_path {
        std::fs::write(path, &formatted)?;
        info!(?path, "Output written to file");
        println!("✓ Saved to {}", path.display());
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(formatted.as_bytes())?;
        if !formatted.ends_with('\n') {
            handle.write_all(b"\n")?;
        }
    }

    Ok(())
}

// ============================
// Config API - 配置管理
// ============================

/// 执行配置管理操作
pub fn run_config(action: Option<&str>, key: Option<&str>, value: Option<&str>) -> Result<()> {
    let mut settings = Settings::load()?;

    match action {
        None | Some("show") | Some("list") => {
            let path = Settings::config_path()?;
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| "Default config (not yet saved)".into());
            println!("\x1b[36mConfig file:\x1b[0m {}", path.display());
            println!("\x1b[36m──────────────────────────────────\x1b[0m");
            println!("{}", content);
        }
        Some("set") => {
            match (key, value) {
                (Some(k), Some(v)) => {
                    set_config_value(&mut settings, k, v)?;
                    settings.save()?;
                    println!("\x1b[32m✓\x1b[0m Set \x1b[33m{}\x1b[0m = \x1b[33m{}\x1b[0m", k, v);
                }
                _ => {
                    eprintln!("\x1b[31mUsage:\x1b[0m ai_cli config set <key> <value>");
                    eprintln!("  Keys: default_provider, openai.api_key, openai.model, anthropic.api_key, ollama.model, etc.");
                }
            }
        }
        Some("init") => {
            settings.save()?;
            let path = Settings::config_path()?;
            println!("\x1b[32m✓\x1b[0m Default config created at \x1b[33m{}\x1b[0m", path.display());
            println!("  Edit this file to configure API keys and model preferences.");
        }
        Some(cmd) => {
            eprintln!("\x1b[31mUnknown config action:\x1b[0m {}", cmd);
            eprintln!("  Available: show, set, init");
        }
    }

    Ok(())
}

// ============================
// 内部辅助函数
// ============================

/// 通过点分隔的 key 路径设置配置值
fn set_config_value(settings: &mut Settings, key: &str, value: &str) -> Result<()> {
    match key {
        "default_provider" => settings.default_provider = value.to_string(),
        "openai.api_key" => {
            let mut cfg = settings.openai.clone().unwrap_or_default();
            cfg.api_key = value.to_string();
            settings.openai = Some(cfg);
        }
        "openai.model" => {
            let mut cfg = settings.openai.clone().unwrap_or_default();
            cfg.model = value.to_string();
            settings.openai = Some(cfg);
        }
        "openai.api_base" => {
            let mut cfg = settings.openai.clone().unwrap_or_default();
            cfg.api_base = value.to_string();
            settings.openai = Some(cfg);
        }
        "openai.max_tokens" => {
            let mut cfg = settings.openai.clone().unwrap_or_default();
            cfg.max_tokens = value.parse().map_err(|_| error::AiCliError::Config("Invalid number".into()))?;
            settings.openai = Some(cfg);
        }
        "openai.temperature" => {
            let mut cfg = settings.openai.clone().unwrap_or_default();
            cfg.temperature = value.parse().map_err(|_| error::AiCliError::Config("Invalid float".into()))?;
            settings.openai = Some(cfg);
        }
        "anthropic.api_key" => {
            let mut cfg = settings.anthropic.clone().unwrap_or_default();
            cfg.api_key = value.to_string();
            settings.anthropic = Some(cfg);
        }
        "anthropic.model" => {
            let mut cfg = settings.anthropic.clone().unwrap_or_default();
            cfg.model = value.to_string();
            settings.anthropic = Some(cfg);
        }
        "anthropic.api_base" => {
            let mut cfg = settings.anthropic.clone().unwrap_or_default();
            cfg.api_base = value.to_string();
            settings.anthropic = Some(cfg);
        }
        "ollama.model" => {
            let mut cfg = settings.ollama.clone().unwrap_or_default();
            cfg.model = value.to_string();
            settings.ollama = Some(cfg);
        }
        "ollama.host" => {
            let mut cfg = settings.ollama.clone().unwrap_or_default();
            cfg.host = value.to_string();
            settings.ollama = Some(cfg);
        }
        _ => return Err(error::AiCliError::Config(format!("Unknown config key: {}", key))),
    }
    Ok(())
}

// ============================
// Database API
// ============================

use db::DatabaseManager;
use db::types::OutputFormat as DbOutputFormat;

/// 列出所有配置的数据库
pub fn run_db_list() -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let dbs = mgr.list_databases();
    let json = serde_json::to_string_pretty(&dbs)?;
    println!("{}", json);
    Ok(())
}

/// 执行数据库查询
pub fn run_db_query(db_name: &str, sql: &str, limit: usize, output_format: &DbOutputFormat) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let result = mgr.execute_query(db_name, sql, limit)?;
    format_and_print(&result, output_format)
}

/// 执行 INSERT / UPDATE / DELETE（多条统一事务 commit）
pub fn run_db_execute(db_name: &str, sqls: &[String]) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let rows = mgr.execute_dml(db_name, sqls)?;
    println!("{{\"rows_affected\": {}}}", rows);
    Ok(())
}

/// 查看表结构
pub fn run_db_struct(db_name: &str, owner: &str, table: &str, _output_format: &DbOutputFormat) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let info = mgr.get_table_struct(db_name, owner, table)?;
    let json = serde_json::to_string_pretty(&info)?;
    println!("{}", json);
    Ok(())
}

/// 列出数据库中的表
pub fn run_db_tables(db_name: &str, owner: Option<&str>, _output_format: &DbOutputFormat) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let tables = mgr.list_tables(db_name, owner)?;
    let json = serde_json::to_string_pretty(&tables)?;
    println!("{}", json);
    Ok(())
}

/// 生成示例配置文件
pub fn run_db_init_config() -> Result<()> {
    let path = db::config::BinConfig::config_path()?;
    if path.exists() {
        eprintln!("Config already exists at: {}", path.display());
        eprintln!("Use --force to overwrite.");
        return Ok(());
    }
    db::config::write_example_config(&path)?;
    println!("Example config written to: {}", path.display());
    Ok(())
}

/// 格式化查询结果并输出
fn format_and_print(result: &db::types::QueryResult, format: &DbOutputFormat) -> Result<()> {
    match format {
        DbOutputFormat::Json => {
            let json = serde_json::to_string_pretty(result)?;
            println!("{}", json);
        }
        DbOutputFormat::Csv => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{}", result.columns.join(","))?;
            for row in &result.rows {
                let escaped: Vec<String> = row.iter().map(|v| {
                    if v.contains(',') || v.contains('"') || v.contains('\n') {
                        format!("\"{}\"", v.replace('"', "\"\""))
                    } else {
                        v.clone()
                    }
                }).collect();
                writeln!(handle, "{}", escaped.join(","))?;
            }
        }
        DbOutputFormat::Table => {
            print_table(result)?;
        }
    }
    if result.truncated {
        eprintln!("\x1b[33m⚠ Result truncated at {} rows\x1b[0m", result.row_count);
    }
    Ok(())
}

/// 打印对齐的文本表格
fn print_table(result: &db::types::QueryResult) -> Result<()> {
    if result.columns.is_empty() {
        return Ok(());
    }
    let _col_count = result.columns.len();
    let mut widths: Vec<usize> = result.columns.iter().map(|c| c.len()).collect();
    for row in &result.rows {
        for (i, val) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(val.len());
            }
        }
    }
    // 限制最大列宽
    for w in &mut widths {
        *w = (*w).min(60);
    }

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    // 分隔线
    let sep: String = widths.iter().map(|w| "-".repeat(*w + 2)).collect::<Vec<_>>().join("+");

    // 表头
    write!(handle, "| ")?;
    for (i, col) in result.columns.iter().enumerate() {
        write!(handle, "{:width$} | ", truncate(col, widths[i]), width = widths[i])?;
    }
    writeln!(handle)?;
    writeln!(handle, "|{}|", sep)?;

    // 数据行
    for row in &result.rows {
        write!(handle, "| ")?;
        for (i, val) in row.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or(10);
            write!(handle, "{:width$} | ", truncate(val, w), width = w)?;
        }
        writeln!(handle)?;
    }
    writeln!(handle, "{} rows", result.row_count)?;
    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
