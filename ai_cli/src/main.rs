//! ai_cli —— AI 模型专用 CLI 交互工具（组合版）
//!
//! 子命令: fetch, chat, ask, config

mod cli;

use clap::Parser;
use cli::args::{Cli, Commands};
use ai_cli::{run_fetch, run_chat, run_ask, run_config};
use tracing::error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()),
        )
        .with_target(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Fetch { url, spa, output, format, browser, timeout } => {
            run_fetch(url, *spa, output.as_deref(), format, browser.as_deref(), *timeout).await
        }
        Commands::Chat { provider, model } => {
            run_chat(*provider, model.as_deref())
        }
        Commands::Ask { prompt, provider, model } => {
            run_ask(prompt, *provider, model.as_deref())
        }
        Commands::Config { action, key, value } => {
            run_config(action.as_deref(), key.as_deref(), value.as_deref())
        }
    };

    if let Err(e) = result {
        error!(%e, "Command failed");
        eprintln!("\x1b[31merror:\x1b[0m {}", e);
        std::process::exit(1);
    }
}
