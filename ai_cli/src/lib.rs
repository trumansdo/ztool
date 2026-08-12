//! ai_cli 库 —— AI 模型专用 CLI 交互工具的公共 API
//!
//! 提供可复用的核心逻辑：
//! - 网页内容获取（HTTP + Headless 浏览器）
//! - 数据库操作（Oracle 优先可扩展）
//! - 配置管理（统一 BinConfig）
//!
//! # 架构
//! ```text
//! lib.rs  (public API)
//!   ├── config    → 统一配置 BinConfig (binconfig.toml)
//!   ├── fetch     → 网页获取
//!   │   ├── http      → reqwest 静态获取
//!   │   ├── browser   → headless_chrome 渲染
//!   │   └── extract   → 正文提取
//!   ├── output    → 输出格式化
//!   │   └── format    → OutputFormat 枚举 + 格式化函数 + 公共工具
//!   ├── db        → 数据库操作
//!   │   ├── config    → re-export 到统一 BinConfig
//!   │   ├── connector → DbConnector / DbConnection trait
//!   │   ├── oracle    → Oracle 实现
//!   │   └── types     → 共享类型定义
//!   ├── excel     → Excel 读写
//!   ├── lsp       → LSP 客户端
//!   └── error     → 统一错误类型
//! ```

pub mod config;
pub mod fetch;
pub mod output;
pub mod error;
pub mod db;
pub mod excel;
pub mod lsp;

pub use output::format::OutputFormat;

use config::BinConfig;
use error::Result;
use fetch::extract::{extract_content, is_likely_js_rendered, ExtractOptions};
use output::format::{escape_csv, format_output, print_table};
use std::io::{self, Write};
use tracing::{error, info};

// ============================
// Fetch API
// ============================

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
// Config API
// ============================

pub fn run_config(action: Option<&str>, key: Option<&str>, value: Option<&str>) -> Result<()> {
    let mut settings = BinConfig::load()?;

    match action {
        None | Some("show") | Some("list") => {
            let path = BinConfig::config_path()?;
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| "Default config (not yet saved)".into());
            println!("\x1b[36mConfig file:\x1b[0m {}", path.display());
            println!("\x1b[36m──────────────────────────────────\x1b[0m");
            println!("{}", content);
        }
        Some("set") => match (key, value) {
            (Some(k), Some(v)) => {
                set_config_value(&mut settings, k, v)?;
                settings.save()?;
                println!("\x1b[32m✓\x1b[0m Set \x1b[33m{}\x1b[0m = \x1b[33m{}\x1b[0m", k, v);
            }
            _ => {
                eprintln!("\x1b[31mUsage:\x1b[0m ai_cli config set <key> <value>");
                eprintln!("  Keys: default_provider, openai.api_key, openai.model, etc.");
            }
        },
        Some("init") => {
            settings.save()?;
            let path = BinConfig::config_path()?;
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
// 配置 set 辅助宏
// ============================

macro_rules! set_field {
    ($settings:expr, $field:ident, $sub:ident, $val:expr) => {{
        let mut cfg = $settings.$field.clone().unwrap_or_default();
        cfg.$sub = $val;
        $settings.$field = Some(cfg);
    }};
    ($settings:expr, $field:ident, $val:expr) => {
        $settings.$field = $val
    };
}

macro_rules! set_parse {
    ($settings:expr, $field:ident, $sub:ident, $val:expr, $ty:ty) => {{
        let mut cfg = $settings.$field.clone().unwrap_or_default();
        cfg.$sub = $val.parse::<$ty>().map_err(|_| error::AiCliError::Config("Invalid number".into()))?;
        $settings.$field = Some(cfg);
    }};
}

fn set_config_value(settings: &mut BinConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "default_provider" => set_field!(settings, default_provider, value.to_string()),
        "openai.api_key" => set_field!(settings, openai, api_key, value.to_string()),
        "openai.model" => set_field!(settings, openai, model, value.to_string()),
        "openai.api_base" => set_field!(settings, openai, api_base, value.to_string()),
        "openai.max_tokens" => set_parse!(settings, openai, max_tokens, value, u32),
        "openai.temperature" => set_parse!(settings, openai, temperature, value, f32),
        "anthropic.api_key" => set_field!(settings, anthropic, api_key, value.to_string()),
        "anthropic.model" => set_field!(settings, anthropic, model, value.to_string()),
        "anthropic.api_base" => set_field!(settings, anthropic, api_base, value.to_string()),
        "ollama.model" => set_field!(settings, ollama, model, value.to_string()),
        "ollama.host" => set_field!(settings, ollama, host, value.to_string()),
        _ => return Err(error::AiCliError::Config(format!("Unknown config key: {}", key))),
    }
    Ok(())
}

// ============================
// Database API
// ============================

use db::DatabaseManager;
use db::types::OutputFormat as DbOutputFormat;

pub fn run_db_list() -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let dbs = mgr.list_databases();
    let json = serde_json::to_string_pretty(&dbs)?;
    println!("{}", json);
    Ok(())
}

pub fn run_db_query(db_name: &str, sql: &str, limit: usize, output_format: &DbOutputFormat) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let result = mgr.execute_query(db_name, sql, limit)?;
    format_and_print(&result, output_format)
}

pub fn run_db_execute(db_name: &str, sqls: &[String]) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let rows = mgr.execute_dml(db_name, sqls)?;
    println!("{{\"rows_affected\": {}}}", rows);
    Ok(())
}

pub fn run_db_struct(db_name: &str, owner: &str, table: &str) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let info = mgr.get_table_struct(db_name, owner, table)?;
    let json = serde_json::to_string_pretty(&info)?;
    println!("{}", json);
    Ok(())
}

pub fn run_db_tables(db_name: &str, owner: Option<&str>) -> Result<()> {
    let mgr = DatabaseManager::new()?;
    let tables = mgr.list_tables(db_name, owner)?;
    let json = serde_json::to_string_pretty(&tables)?;
    println!("{}", json);
    Ok(())
}

pub fn run_db_init_config() -> Result<()> {
    let path = BinConfig::config_path()?;
    if path.exists() {
        eprintln!("Config already exists at: {}", path.display());
        eprintln!("Use --force to overwrite.");
        return Ok(());
    }
    db::config::write_example_config(&path)?;
    println!("Example config written to: {}", path.display());
    Ok(())
}

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
                let escaped: Vec<String> = row.iter().map(|v| escape_csv(v)).collect();
                writeln!(handle, "{}", escaped.join(","))?;
            }
        }
        DbOutputFormat::Table => {
            let rows: Vec<Vec<String>> = result.rows.clone();
            print_table(&result.columns, &rows, result.row_count)?;
        }
    }
    if result.truncated {
        eprintln!("\x1b[33m⚠ Result truncated at {} rows\x1b[0m", result.row_count);
    }
    Ok(())
}
