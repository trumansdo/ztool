//! web_fetch —— 并发网页获取 CLI 工具
//!
//! 使用方式: web_fetch [OPTIONS] <URL> [URL...]
//!
//! 支持多个 URL 并发获取，自动检测 SPA 页面并降级到 Headless 浏览器。

use clap::Parser;
use ai_cli::{OutputFormat, run_fetch};
use ai_cli::db::config::BinConfig;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(
    name = "web_fetch",
    version,
    about = "并发网页内容获取工具",
    long_about = "基于 ai_cli 库的独立网页获取工具。\n\
                  支持多个 URL 并发获取，静态 HTTP 和 Headless 浏览器两种模式，\n\
                  自动检测 SPA 页面并降级。"
)]
struct WebFetchCli {
    /// 目标网页 URL（可多个，并发获取）
    #[arg(value_name = "URL", required = true, num_args = 1..)]
    urls: Vec<String>,

    /// 强制使用 headless 浏览器渲染
    #[arg(long, short = 's', default_value_t = false)]
    spa: bool,

    /// 输出文件路径（多个 URL 时作为目录，单 URL 时作为文件）
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

    let cli = WebFetchCli::parse();
    let url_count = cli.urls.len();

    // 加载 binconfig.toml 中的 web_fetch 配置
    let proxy_url = BinConfig::load()
        .ok()
        .and_then(|cfg| cfg.web_fetch.http_proxy)
        .filter(|p| !p.is_empty());
    if proxy_url.is_some() {
        info!("Using proxy from binconfig.toml [web_fetch]");
    }

    if url_count == 1 {
        // 单 URL：保持原有行为
        let result = run_fetch(
            &cli.urls[0],
            cli.spa,
            cli.output.as_deref(),
            &cli.format,
            cli.browser.as_deref(),
            cli.timeout,
            proxy_url.as_deref(),
        ).await;

        if let Err(e) = result {
            error!(%e, "Fetch failed");
            eprintln!("\x1b[31merror:\x1b[0m {}", e);
            std::process::exit(1);
        }
    } else {
        // 多 URL：并发获取
        info!(count = url_count, "Starting concurrent fetch");

        // 如果指定了 output，确保是目录
        if let Some(ref out) = cli.output {
            if out.exists() && !out.is_dir() {
                eprintln!("\x1b[31merror:\x1b[0m multi-URL mode requires --output to be a directory");
                std::process::exit(1);
            }
            if !out.exists() {
                std::fs::create_dir_all(out).unwrap_or_else(|e| {
                    eprintln!("\x1b[31merror:\x1b[0m failed to create output directory: {}", e);
                    std::process::exit(1);
                });
            }
        }

        let handles: Vec<_> = cli.urls.iter().enumerate().map(|(i, url)| {
            let url = url.clone();
            let spa = cli.spa;
            let format = cli.format.clone();
            let browser = cli.browser.clone();
            let timeout = cli.timeout;
            let output_dir = cli.output.clone();
            let proxy_url = proxy_url.clone();

            tokio::spawn(async move {
                let output_path: Option<PathBuf> = output_dir.as_ref().map(|dir| {
                    let filename = url_to_filename(&url, &format);
                    dir.join(filename)
                });

                let result = run_fetch(
                    &url,
                    spa,
                    output_path.as_deref(),
                    &format,
                    browser.as_deref(),
                    timeout,
                    proxy_url.as_deref(),
                ).await;

                (i, url, result)
            })
        }).collect();

        let mut results: Vec<(usize, String, Result<(), ai_cli::error::AiCliError>)> = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(tuple) => results.push(tuple),
                Err(e) => {
                    error!(%e, "Join error");
                    eprintln!("\x1b[31merror:\x1b[0m task panicked: {}", e);
                }
            }
        }

        // 按原始顺序排序
        results.sort_by_key(|(i, _, _)| *i);

        // 汇总报告
        let success_count = results.iter().filter(|(_, _, r)| r.is_ok()).count();
        let fail_count = results.len() - success_count;

        if cli.output.is_none() {
            // stdout 模式：输出到 stdout 的结果已在 run_fetch 中打印
            // 这里只输出汇总
        }

        eprintln!(
            "\x1b[90m── {}/{} succeeded{}──\x1b[0m",
            success_count,
            results.len(),
            if fail_count > 0 { format!(", {} failed", fail_count) } else { String::new() }
        );

        for (_, url, result) in &results {
            if let Err(e) = result {
                eprintln!("\x1b[31m  ✗\x1b[0m {} — {}", url, e);
            }
        }

        if fail_count > 0 {
            std::process::exit(1);
        }
    }
}

/// 将 URL 转换为安全的文件名
fn url_to_filename(url: &str, format: &OutputFormat) -> String {
    let ext = match format {
        OutputFormat::Md => "md",
        OutputFormat::Json => "json",
        OutputFormat::Text => "txt",
    };

    // 去掉协议前缀
    let s = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    // 替换不安全字符
    let sanitized: String = s
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' => c,
            _ => '_',
        })
        .collect();

    // 截断过长的文件名
    let max_len = 120;
    let truncated = if sanitized.len() > max_len {
        &sanitized[..max_len]
    } else {
        &sanitized
    };

    format!("{}.{}", truncated, ext)
}
