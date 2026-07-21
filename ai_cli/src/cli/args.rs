//! 组合 CLI 参数定义（仅用于 main.rs 组合工具）
//!
//! 使用 `clap` 的 derive API 声明式定义命令行接口。
//! 共享类型（OutputFormat, ProviderType）从 `ai_cli` 库导入。

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use ai_cli::{OutputFormat, ProviderType};

/// AI CLI —— 纯本地 AI 模型交互与网页获取工具
#[derive(Parser, Debug)]
#[command(
    name = "ai_cli",
    version,
    about = "AI 模型专用 CLI 交互与网页获取工具",
    long_about = "纯本地、多后端的 AI 模型 CLI 工具。\n\
                  支持 OpenAI / Anthropic / Ollama 等多模型后端，\n\
                  提供交互式对话、单轮问答、网页内容获取等功能。"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// 顶层子命令
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 获取网页内容
    Fetch {
        #[arg(value_name = "URL")]
        url: String,

        #[arg(long, short = 's', default_value_t = false)]
        spa: bool,

        #[arg(long, short = 'o', value_name = "PATH")]
        output: Option<PathBuf>,

        #[arg(long, short = 'f', value_enum, default_value_t = OutputFormat::Md)]
        format: OutputFormat,

        #[arg(long, value_name = "PATH")]
        browser: Option<String>,

        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },

    /// 交互式 AI 对话（REPL 模式）
    Chat {
        #[arg(long, short = 'p', value_enum, default_value_t = ProviderType::Openai)]
        provider: ProviderType,

        #[arg(long, short = 'm', value_name = "MODEL")]
        model: Option<String>,
    },

    /// 单轮 AI 问答
    Ask {
        #[arg(value_name = "PROMPT")]
        prompt: String,

        #[arg(long, short = 'p', value_enum, default_value_t = ProviderType::Openai)]
        provider: ProviderType,

        #[arg(long, short = 'm', value_name = "MODEL")]
        model: Option<String>,
    },

    /// 配置管理
    Config {
        #[arg(value_name = "ACTION")]
        action: Option<String>,

        #[arg(value_name = "KEY")]
        key: Option<String>,

        #[arg(value_name = "VALUE")]
        value: Option<String>,
    },
}
