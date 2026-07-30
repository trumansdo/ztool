//! ai_cli —— AI 模型专用 CLI 交互工具（组合版）
//!
//! 子命令: fetch, config

mod cli;

use clap::Parser;
use cli::args::{Cli, Commands};
use ai_cli::{run_fetch, run_config};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()),
        )
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Fetch { url, spa, output, format, browser, timeout } => {
            run_fetch(url, *spa, output.as_deref(), format, browser.as_deref(), *timeout, None).await?;
        }
        Commands::Config { action, key, value } => {
            run_config(action.as_deref(), key.as_deref(), value.as_deref())?;
        }
    }

    Ok(())
}
