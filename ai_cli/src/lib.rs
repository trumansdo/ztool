//! ai_cli 库 —— AI 模型专用 CLI 交互工具的公共 API
//!
//! 提供可复用的核心逻辑：
//! - 网页内容获取（HTTP + Headless 浏览器）
//! - 多后端 AI 对话（OpenAI / Anthropic / Ollama）
//! - 交互式 REPL 模式
//! - 配置管理
//!
//! # 架构
//! ```text
//! lib.rs  (public API)
//!   ├── config    → 配置加载/保存
//!   ├── chat      → AI 对话（Provider 模式）
//!   │   ├── provider  → ProviderType, ChatProvider trait, Message
//!   │   ├── openai    → OpenAI 实现
//!   │   ├── anthropic → Anthropic 实现
//!   │   └── ollama    → Ollama 实现
//!   ├── fetch     → 网页获取
//!   │   ├── http      → reqwest 静态获取
//!   │   ├── browser   → headless_chrome 渲染
//!   │   └── extract   → 正文提取
//!   ├── output    → 输出格式化
//!   │   └── format    → OutputFormat 枚举 + 格式化函数
//!   └── error     → 统一错误类型
//! ```

pub mod config;
pub mod chat;
pub mod fetch;
pub mod output;
pub mod error;

// 重新导出高频类型，方便 CLI 层使用
pub use output::format::OutputFormat;
pub use chat::provider::ProviderType;

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
) -> Result<()> {
    info!(%url, force_spa, ?format, "Starting fetch command");

    let (html, fetch_mode, final_url, status_code, content_type) = if force_spa {
        info!("Forcing headless browser mode");
        let result = fetch::browser::fetch_with_browser(url, browser_path, timeout)?;
        (result.html, "headless_browser", result.final_url, None::<u16>, None)
    } else {
        match fetch::http::fetch_static(url, timeout).await {
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
// Chat API - 交互式对话
// ============================

/// 启动交互式 AI 对话（REPL 模式）
pub fn run_chat(provider_type: ProviderType, model: Option<&str>) -> Result<()> {
    let settings = Settings::load()?;

    let (params, provider_name) = build_chat_params(&settings, provider_type, model)?;

    println!("\x1b[36m╭─ AI CLI Interactive Chat ─────────────────────────────╮\x1b[0m");
    println!("\x1b[36m│\x1b[0m  Provider: \x1b[33m{}\x1b[0m  |  Model: \x1b[33m{}\x1b[0m               \x1b[36m│\x1b[0m", provider_name, params.model);
    println!("\x1b[36m│\x1b[0m  Type '/help' for commands, '/exit' to quit          \x1b[36m│\x1b[0m");
    println!("\x1b[36m╰──────────────────────────────────────────────────────╯\x1b[0m");

    let mut messages: Vec<chat::Message> = Vec::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("\n\x1b[32m>>>\x1b[0m ");
        stdout.flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }
        let input = input.trim();

        match input {
            "/exit" | "/quit" | "/q" => {
                println!("\x1b[33mGoodbye!\x1b[0m");
                break;
            }
            "/help" | "/h" => {
                print_help();
                continue;
            }
            "/clear" | "/c" => {
                messages.clear();
                println!("\x1b[32m✓ Conversation cleared\x1b[0m");
                continue;
            }
            "" => continue,
            _ => {}
        }

        let prompt = if input.ends_with('\\') {
            let mut full = input.trim_end_matches('\\').to_string();
            loop {
                print!("... ");
                stdout.flush()?;
                let mut line = String::new();
                if stdin.read_line(&mut line)? == 0 { break; }
                let line = line.trim();
                if line.is_empty() { break; }
                if line.ends_with('\\') {
                    full.push_str(line.trim_end_matches('\\'));
                } else {
                    full.push_str(line);
                    break;
                }
            }
            full
        } else {
            input.to_string()
        };

        messages.push(chat::Message::user(&prompt));

        let provider: Box<dyn chat::ChatProvider> = match provider_type {
            ProviderType::Openai => Box::new(chat::openai::OpenAIProvider),
            ProviderType::Anthropic => Box::new(chat::anthropic::AnthropicProvider),
            ProviderType::Ollama => Box::new(chat::ollama::OllamaProvider),
        };

        let mut full_response = String::new();
        match provider.chat_stream(&messages, &params, &mut |chunk| {
            full_response.push_str(chunk);
            print!("{}", chunk);
            stdout.flush()?;
            Ok(())
        }) {
            Ok(resp) => {
                if resp.output_tokens.is_some() || resp.input_tokens.is_some() {
                    let it = resp.input_tokens.map(|t| t.to_string()).unwrap_or_else(|| "?".into());
                    let ot = resp.output_tokens.map(|t| t.to_string()).unwrap_or_else(|| "?".into());
                    println!("\n\x1b[90m[{} | in: {} tok | out: {} tok | {:.1}s]\x1b[0m",
                        resp.model, it, ot, resp.duration_secs);
                } else {
                    println!("\n\x1b[90m[{:.1}s]\x1b[0m", resp.duration_secs);
                }
                messages.push(chat::Message::assistant(full_response));
            }
            Err(e) => {
                error!(%e, "Chat request failed");
                eprintln!("\n\x1b[31merror:\x1b[0m {}", e);
            }
        }
    }

    Ok(())
}

/// 打印交互模式帮助
fn print_help() {
    println!("\x1b[36m── Commands ────────────────\x1b[0m");
    println!("  /exit, /quit, /q  - 退出对话");
    println!("  /help, /h         - 显示此帮助");
    println!("  /clear, /c        - 清空对话历史");
    println!();
    println!("  以 \\ 结尾输入多行内容");
    println!();
    println!("\x1b[36m── Tips ────────────────────\x1b[0m");
    println!("  - 使用 /fetch <url> 获取网页内容（未来支持）");
    println!("  - 使用 /sys <cmd> 执行系统命令（未来支持）");
}

// ============================
// Ask API - 单轮问答
// ============================

/// 执行单轮 AI 问答并输出结果
pub fn run_ask(prompt: &str, provider_type: ProviderType, model: Option<&str>) -> Result<()> {
    let settings = Settings::load()?;
    let (params, provider_name) = build_chat_params(&settings, provider_type, model)?;

    info!(%prompt, provider = %provider_name, model = %params.model, "Running single-turn ask");

    let provider: Box<dyn chat::ChatProvider> = match provider_type {
        ProviderType::Openai => Box::new(chat::openai::OpenAIProvider),
        ProviderType::Anthropic => Box::new(chat::anthropic::AnthropicProvider),
        ProviderType::Ollama => Box::new(chat::ollama::OllamaProvider),
    };

    let messages = vec![
        chat::Message::system("You are a helpful AI assistant. Respond concisely and accurately."),
        chat::Message::user(prompt),
    ];

    let response = provider.chat(&messages, &params)?;

    println!("{}", response.content);

    if response.output_tokens.is_some() || response.input_tokens.is_some() {
        let it = response.input_tokens.map(|t| t.to_string()).unwrap_or_else(|| "?".into());
        let ot = response.output_tokens.map(|t| t.to_string()).unwrap_or_else(|| "?".into());
        eprintln!("\x1b[90m[{} | {} → {} tok | {:.1}s]\x1b[0m",
            response.model, it, ot, response.duration_secs);
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

/// 根据 provider 类型和配置构建 ChatParams
fn build_chat_params(
    settings: &Settings,
    provider_type: ProviderType,
    model: Option<&str>,
) -> Result<(chat::ChatParams, String)> {
    let provider_name = provider_type.as_str();

    let params = match provider_type {
        ProviderType::Openai => {
            let cfg = settings.openai.clone().unwrap_or_default();
            chat::ChatParams {
                model: model.unwrap_or(&cfg.model).to_string(),
                api_key: cfg.api_key,
                api_base: cfg.api_base,
                max_tokens: cfg.max_tokens,
                temperature: cfg.temperature,
            }
        }
        ProviderType::Anthropic => {
            let cfg = settings.anthropic.clone().unwrap_or_default();
            chat::ChatParams {
                model: model.unwrap_or(&cfg.model).to_string(),
                api_key: cfg.api_key,
                api_base: if cfg.api_base.is_empty() {
                    "https://api.anthropic.com/v1".into()
                } else {
                    cfg.api_base
                },
                max_tokens: cfg.max_tokens,
                temperature: cfg.temperature,
            }
        }
        ProviderType::Ollama => {
            let cfg = settings.ollama.clone().unwrap_or_default();
            chat::ChatParams {
                model: model.unwrap_or(&cfg.model).to_string(),
                api_base: cfg.host,
                api_key: String::new(),
                max_tokens: cfg.max_tokens,
                temperature: cfg.temperature,
            }
        }
    };

    Ok((params, provider_name.to_string()))
}
