//! ai_fetch —— 独立网页获取 CLI 工具
//!
//! 使用方式: ai_fetch [OPTIONS] <URL>
//!
//! 这是 `ai_cli` 库的薄包装，专门用于网页内容获取。
//! 所有业务逻辑在 `ai_cli::run_fetch` 中。

use clap::Parser;
use ai_cli::{OutputFormat, run_fetch};
use std::path::PathBuf;
use tracing::error;

#[derive(Parser, Debug)]
#[command(
    name = "ai_fetch",
    version,
    about = "网页内容获取工具",
    long_about = "基于 ai_cli 库的独立网页获取工具。\n\
                  支持静态 HTTP 和 Headless 浏览器两种模式，\n\
                  自动检测 SPA 页面并降级。"
)]
struct FetchCli {
    /// 目标网页 URL
    #[arg(value_name = "URL")]
    url: String,

    /// 强制使用 headless 浏览器渲染
    #[arg(long, short = 's', default_value_t = false)]
    spa: bool,

    /// 输出文件路径（默认 stdout）
    #[arg(long, short = 'o', value_name = "PATH")]
    output: Option<PathBuf>,

    /// 输出格式
    #[arg(long, short = 'f', value_enum, default_value_t = OutputFormat::Md)]
    format: OutputFormat,

    /// 自定义浏览器路径
    #[arg(long, value_name = "PATH")]
    browser: Option<String>,

    /// 超时秒数
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()))
        .with_target(false)
        .without_time()
        .init();

    let cli = FetchCli::parse();

    let result = run_fetch(
        &cli.url,
        cli.spa,
        cli.output.as_deref(),
        &cli.format,
        cli.browser.as_deref(),
        cli.timeout,
    ).await;

    if let Err(e) = result {
        error!(%e, "Fetch failed");
        eprintln!("\x1b[31merror:\x1b[0m {}", e);
        std::process::exit(1);
    }
}
