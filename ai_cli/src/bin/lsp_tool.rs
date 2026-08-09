//! lsp_tool —— LSP 命令行工具（薄 CLI 层，逻辑在 ai_cli::lsp 库）
//!
//! # 用法
//! ```text
//! lsp_tool symbols <file>
//! lsp_tool hover <file> --line N --char N
//! lsp_tool definition <file> --line N --char N
//! lsp_tool references <file> --line N --char N [--no-declaration]
//! lsp_tool completion <file> --line N --char N
//! lsp_tool test <file>
//!
//! 全局: --json --server <id> --command <cmd> --maven-settings <path> --config <path> --root <dir>
//! ```
//!
//! # 配置（统一走 ai_cli::config::Settings，~/.config/ai_cli/config.toml）
//! ```toml
//! [lsp.jdtls]
//! command = ["java", "-Declipse.application=org.eclipse.jdt.ls.core.id1", "...", "-jar", "...launcher.jar"]
//! maven_settings = "D:\\dev_program\\apache-maven-3.9.5\\conf\\settings.xml"
//! ```

use ai_cli::config::Settings;
use ai_cli::lsp::client::LspClient;
use ai_cli::lsp::output::{format_completions, format_hover, format_locations, format_symbols};
use ai_cli::lsp::servers::{apply_data_dir, apply_maven_settings, resolve_server};
use ai_cli::lsp::test_suite::run_full_test;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};

/// 统一配置加载: 一律走 settings.rs 的 Settings 解析
/// 坑: 不要在本 bin 里自行实现配置查找(曾误加 exe 同目录扫描+系统目录回退),
///     与 settings.rs 的 config_path() 约定重复冲突, 必须统一走 Settings::load()
fn load_settings(path: Option<&Path>) -> Settings {
    if let Some(p) = path {
        return match fs::read_to_string(p) {
            Ok(s) => match toml::from_str::<Settings>(&s) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[warn] 配置解析失败 ({}): {}", p.display(), e);
                    Settings::default()
                }
            },
            Err(e) => {
                eprintln!("[warn] 配置读取失败 ({}): {}", p.display(), e);
                Settings::default()
            }
        };
    }
    match Settings::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[warn] 配置加载失败: {}", e);
            Settings::default()
        }
    }
}

#[derive(Parser)]
#[command(name = "lsp_tool", version, about = "LSP 命令行工具: 符号大纲/悬停/导航/补全/完整测试")]
struct Cli {
    /// JSON 结构化输出
    #[arg(long)]
    json: bool,
    /// 服务器 id (内置: jdtls/typescript-language-server/pyright/kotlin-language-server/clangd)
    #[arg(long, default_value = "auto")]
    server: String,
    /// 覆盖服务器启动命令
    #[arg(long)]
    command: Option<String>,
    /// 方案 B: 显式指定 Maven user settings.xml (注入 -Djava.configuration.maven.userSettings)
    #[arg(long)]
    maven_settings: Option<String>,
    /// didOpen 后缓冲等待 ms (覆盖配置 open_delay_ms, 默认取配置/200)
    #[arg(long)]
    open_delay_ms: Option<u64>,
    /// workspace 根目录 (覆盖配置 data_root, 子目录仍按 sha1(root) 隔离)
    #[arg(long)]
    data_root: Option<String>,
    /// TOML 配置文件路径 (默认 ~/.config/ai_cli/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,
    /// 工作区根目录 (必填: AI 场景多项目切换, 不写入配置)
    #[arg(long, required = true)]
    root: PathBuf,
    #[command(subcommand)]
    cmd: Sub,
}

#[derive(Subcommand)]
enum Sub {
    /// 文件符号大纲 (类/方法/字段树)
    Symbols { path: PathBuf },
    /// 悬停信息 (类型/文档)
    Hover {
        path: PathBuf,
        #[arg(long)]
        line: usize,
        #[arg(long)]
        char: usize,
    },
    /// 跳转定义
    Definition {
        path: PathBuf,
        #[arg(long)]
        line: usize,
        #[arg(long)]
        char: usize,
    },
    /// 查找引用
    References {
        path: PathBuf,
        #[arg(long)]
        line: usize,
        #[arg(long)]
        char: usize,
        #[arg(long)]
        no_declaration: bool,
    },
    /// 代码补全
    Completion {
        path: PathBuf,
        #[arg(long)]
        line: usize,
        #[arg(long)]
        char: usize,
    },
    /// 完整测试套件 (对齐 jdtls_lsp_full_test.py)
    Test { path: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        // 输出优化: --json 模式错误结构化到 stdout(AI 可直接解析), 否则 stderr
        if cli.json {
            println!("{}", serde_json::json!({ "error": e.to_string() }));
        } else {
            eprintln!("[error] {}", e);
        }
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    let config = load_settings(cli.config.as_deref());

    match &cli.cmd {
        Sub::Symbols { path } => {
            let server = apply_data_dir(apply_maven_settings(
            resolve_server(path, &cli.server, cli.command.as_deref(), &config)?,
            cli.maven_settings.as_deref(),
            &config,
        ), &cli.root, cli.data_root.as_deref(), &config);
            let mut client = LspClient::start(&cli.root, &server, cli.open_delay_ms.unwrap_or(config.lsp.open_delay_ms))?;
            let v = client.document_symbols(path)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                print!("{}", format_symbols(&v));
            }
            client.shutdown_exit()?;
        }
        Sub::Hover { path, line, char } => {
            let server = apply_data_dir(apply_maven_settings(
            resolve_server(path, &cli.server, cli.command.as_deref(), &config)?,
            cli.maven_settings.as_deref(),
            &config,
        ), &cli.root, cli.data_root.as_deref(), &config);
            let mut client = LspClient::start(&cli.root, &server, cli.open_delay_ms.unwrap_or(config.lsp.open_delay_ms))?;
            let v = client.hover(path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                let txt = format_hover(&v);
                if txt.trim().is_empty() {
                    eprintln!("[warn] 该位置无 hover 信息 (line={} char={})", line, char);
                } else {
                    println!("{}", txt);
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Definition { path, line, char } => {
            let server = apply_data_dir(apply_maven_settings(
            resolve_server(path, &cli.server, cli.command.as_deref(), &config)?,
            cli.maven_settings.as_deref(),
            &config,
        ), &cli.root, cli.data_root.as_deref(), &config);
            let mut client = LspClient::start(&cli.root, &server, cli.open_delay_ms.unwrap_or(config.lsp.open_delay_ms))?;
            let v = client.definition(path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                let txt = format_locations(&v);
                if txt.trim().is_empty() {
                    eprintln!("[warn] 未找到定义 (line={} char={})", line, char);
                } else {
                    println!("{}", txt);
                }
            }
            client.shutdown_exit()?;
        }
        Sub::References { path, line, char, no_declaration } => {
            let server = apply_data_dir(apply_maven_settings(
            resolve_server(path, &cli.server, cli.command.as_deref(), &config)?,
            cli.maven_settings.as_deref(),
            &config,
        ), &cli.root, cli.data_root.as_deref(), &config);
            let mut client = LspClient::start(&cli.root, &server, cli.open_delay_ms.unwrap_or(config.lsp.open_delay_ms))?;
            let v = client.references(path, *line, *char, !*no_declaration)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                let txt = format_locations(&v);
                if txt.trim().is_empty() {
                    eprintln!("[warn] 未找到引用 (line={} char={})", line, char);
                } else {
                    println!("{}", txt);
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Completion { path, line, char } => {
            let server = apply_data_dir(apply_maven_settings(
            resolve_server(path, &cli.server, cli.command.as_deref(), &config)?,
            cli.maven_settings.as_deref(),
            &config,
        ), &cli.root, cli.data_root.as_deref(), &config);
            let mut client = LspClient::start(&cli.root, &server, cli.open_delay_ms.unwrap_or(config.lsp.open_delay_ms))?;
            let v = client.completion(path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                let txt = format_completions(&v);
                if txt.trim().is_empty() {
                    eprintln!("[warn] 该位置无补全项 (line={} char={})", line, char);
                } else {
                    println!("{}", txt);
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Test { path } => {
            let server = apply_data_dir(apply_maven_settings(
            resolve_server(path, &cli.server, cli.command.as_deref(), &config)?,
            cli.maven_settings.as_deref(),
            &config,
        ), &cli.root, cli.data_root.as_deref(), &config);
            println!("[test] 服务器: {} | 命令: {}", server.id, server.command.join(" "));
            let mut client = LspClient::start(&cli.root, &server, cli.open_delay_ms.unwrap_or(config.lsp.open_delay_ms))?;
            println!("[test] initialize 完成, 运行测试套件...");
            let reports = run_full_test(&mut client, path);
            let passed = reports.iter().filter(|r| r.ok).count();
            println!("{}", "=".repeat(60));
            println!("SUMMARY: {}/{} PASSED", passed, reports.len());
            for r in &reports {
                println!(
                    "  [{}] {:<40} {}",
                    if r.ok { "PASS" } else { "FAIL" },
                    r.name,
                    r.detail
                );
            }
            let _ = client.shutdown_exit();
            if passed != reports.len() {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
