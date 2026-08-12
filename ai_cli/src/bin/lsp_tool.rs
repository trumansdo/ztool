//! lsp_tool —— 基于 lsp-types 的类型安全 LSP CLI 工具
//!
//! # 双服务器支持
//!
//! - jdtls: 功能完整 (hover/def/refs/comp/build/clean)，全项目索引，推荐首选
//! - javac-lsp: 轻量级，启动快，但有以下限制:
//!   * hover 不支持字段/变量声明 (上游 FindHoverElement 缺 visitVariable)
//!   * refs 仅搜索已打开文件 (基于 FileStore，非全项目索引)
//!   * comp 需要完整 classpath 才能工作
//!   * 无 data-dir 概念，无 build/clean 命令
//!   * classpath 通过 mvn dependency:build-classpath 自动生成并缓存

use ai_cli::config::BinConfig;
use ai_cli::lsp::client::LspClient;
use ai_cli::lsp::commands;
use ai_cli::lsp::output::{
    format_completions, format_diagnostics, format_hover, format_locations, format_symbols,
};
use ai_cli::lsp::servers::{apply_data_dir, apply_maven_settings, resolve_server};
use ai_cli::lsp::transport::StdioTransport;
use clap::{Parser, Subcommand};
use serde_json::Value;
use std::path::{Path, PathBuf};

// ─── CLI 定义 ────────────────────────────────────────

#[derive(Parser)]
#[command(name = "lsp_tool", version, about = "LSP CLI 工具 (lsp-types 类型安全)")]
struct Cli {
    #[arg(long)]
    json: bool,
    #[arg(long, default_value = "auto")]
    server: String,
    #[arg(long)]
    command: Option<String>,
    #[arg(long)]
    maven_settings: Option<String>,
    #[arg(long)]
    open_delay_ms: Option<u64>,
    #[arg(long)]
    data_dir: Option<String>,
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    classpath: Option<String>,
    #[arg(long)]
    classpath_file: Option<PathBuf>,
    #[arg(long, required = true)]
    project_dir: PathBuf,
    #[command(subcommand)]
    cmd: Sub,
}

#[derive(Subcommand)]
enum Sub {
    /// 文件符号大纲
    Symbols { path: PathBuf },
    /// 悬停文档
    Hover {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
    },
    /// 跳转定义
    Def {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
    },
    /// 查找引用
    Refs {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
        #[arg(long)]
        no_declaration: bool,
    },
    /// 代码补全
    Comp {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
    },
    /// 增量构建 (jdtls)
    Build,
    /// 全量重建 (jdtls)
    Rebuild,
    /// 清理工作空间缓存 (jdtls)
    Clean,
    /// 工作区符号搜索
    WsSymbols { query: String },
    /// 签名帮助 (方法参数)
    Signature {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
    },
    /// 代码格式化 (自动修复 imports)
    Format { path: PathBuf },
    /// Code Action (快速修复)
    Action {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
    },
    /// 代码折叠范围
    Folding { path: PathBuf },
    /// 重命名符号
    Rename {
        path: PathBuf,
        #[arg(long)]
        line: u32,
        #[arg(long)]
        char: u32,
        #[arg(long)]
        new_name: String,
    },
}

// ─── 入口 ────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        if cli.json {
            println!("{}", serde_json::json!({ "error": e.to_string() }));
        } else {
            eprintln!("[error] {}", e);
        }
        std::process::exit(1);
    }
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    let config = load_config(cli.config.as_deref());
    let open_delay = cli
        .open_delay_ms
        .unwrap_or(config.lsp.as_ref().map(|l| l.open_delay_ms).unwrap_or(200));

    // classpath 优先级: CLI --classpath > CLI --classpath-file > auto (mvn) > config
    let javac_classpath: Option<Vec<String>> = cli
        .classpath
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| s.split(';').map(|p| p.trim().to_string()).collect())
        .or_else(|| {
            cli.classpath_file.as_ref().and_then(|f| {
                std::fs::read_to_string(f).ok().map(|s| {
                    s.trim()
                        .split(';')
                        .filter(|p| !p.is_empty())
                        .map(|p| p.trim().to_string())
                        .collect()
                })
            })
        })
        .or_else(|| {
            if cli.server == "javac-lsp" || cli.server == "auto" {
                commands::init::auto_classpath(&cli.project_dir)
            } else {
                None
            }
        })
        .or_else(|| {
            config
                .lsp
                .as_ref()
                .and_then(|l| l.server.get("javac-lsp"))
                .and_then(|o| o.vars.get("project_classpath"))
                .filter(|s| !s.is_empty())
                .map(|s| s.split(';').map(|p| p.trim().to_string()).collect())
        });

    match &cli.cmd {
        Sub::Symbols { path } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result = commands::symbols::document_symbols(&mut client, path)?;
            output_json_or(cli.json, &result, || format_symbols_value(&result));
            client.shutdown_exit()?;
        }
        Sub::Hover { path, line, char } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result = commands::hover::hover(&mut client, path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(h) => println!("{}", format_hover_value(&h)),
                    None => eprintln!("[warn] 该位置无 hover 信息"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Def { path, line, char } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result = commands::definition::goto_definition(&mut client, path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(GotoDefinitionResponse::Array(locs)) => {
                        println!("{}", format_locations_value(&locs))
                    }
                    Some(GotoDefinitionResponse::Scalar(loc)) => {
                        println!("{}", format_locations_value(&[loc]))
                    }
                    Some(GotoDefinitionResponse::Link(links)) => {
                        let locs: Vec<Location> = links
                            .into_iter()
                            .map(|l| Location {
                                uri: l.target_uri,
                                range: l.target_selection_range,
                            })
                            .collect();
                        println!("{}", format_locations_value(&locs));
                    }
                    None => eprintln!("[warn] 未找到定义"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Refs {
            path,
            line,
            char,
            no_declaration,
        } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let mut result = commands::references::find_references(
                &mut client,
                path,
                *line,
                *char,
                !*no_declaration,
            )?;
            // 方案C: javac-lsp 空结果时自动打开关联文件重试
            let is_empty = result.as_ref().map(|v| v.is_empty()).unwrap_or(true);
            if is_empty && cli.server == "javac-lsp" {
                if fallback_open_related(&mut client, &cli.project_dir, path, *line, *char) {
                    result = commands::references::find_references(
                        &mut client,
                        path,
                        *line,
                        *char,
                        !*no_declaration,
                    )?;
                }
                if result.as_ref().map(|v| v.is_empty()).unwrap_or(true) {
                    eprintln!("[warn] javac-lsp 跨文件引用受限，建议换 --server jdtls");
                }
            }
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(locs) => println!("{}", format_locations_value(&locs)),
                    None => eprintln!("[warn] 未找到引用"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Comp { path, line, char } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let mut result = commands::completion::completion(&mut client, path, *line, *char)?;
            // 方案C: javac-lsp 空结果时自动打开关联文件重试
            let is_empty = result.as_ref().map(|v| is_empty_completion(v)).unwrap_or(true);
            if is_empty && cli.server == "javac-lsp" {
                if fallback_open_related(&mut client, &cli.project_dir, path, *line, *char) {
                    result = commands::completion::completion(&mut client, path, *line, *char)?;
                }
                if result.as_ref().map(|v| is_empty_completion(v)).unwrap_or(true) {
                    eprintln!("[warn] javac-lsp 补全受限，建议换 --server jdtls");
                }
            }
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(c) => println!("{}", format_completions_value(&c)),
                    None => eprintln!("[warn] 该位置无补全项"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Build | Sub::Rebuild => {
            if cli.server == "javac-lsp" {
                anyhow::bail!("build/rebuild 仅支持 jdtls，javac-lsp 无此功能");
            }
            let force = matches!(&cli.cmd, Sub::Rebuild);
            let label = if force { "全量重建" } else { "增量构建" };
            let server = {
                let s = resolve_server(
                    &cli.project_dir.join("pom.xml"),
                    "jdtls",
                    cli.command.as_deref(),
                    &config,
                )?;
                apply_data_dir(
                    apply_maven_settings(s, cli.maven_settings.as_deref(), &config),
                    &cli.project_dir,
                    cli.data_dir.as_deref(),
                    &config,
                )
            };
            let mut client = LspClient::start(&cli.project_dir, &server, open_delay)?;
            println!("[build] {}...", label);
            match commands::build::build_workspace(&mut client, force) {
                Ok(status) => {
                    let diags = client.pushed_diagnostics.clone();
                    let all: Vec<_> = diags.iter().flatten().cloned().collect();
                    let errs = all
                        .iter()
                        .filter(|d| d.get("severity").and_then(Value::as_u64) == Some(1))
                        .count();
                    let warns = all.len() - errs;
                    if cli.json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "status": status.as_str(),
                                "label": label,
                                "errors": errs,
                                "warnings": warns
                            })
                        );
                    } else if all.is_empty()
                        && status == ai_cli::lsp::types::BuildStatus::Succeeded
                    {
                        println!("✓ {} 完成", label);
                    } else {
                        println!(
                            "{} 完成 [{}]: {} errors, {} warnings",
                            label,
                            status.as_str(),
                            errs,
                            warns
                        );
                        if !all.is_empty() {
                            println!("{}", format_diagnostics(&all));
                        }
                    }
                }
                Err(e) => eprintln!("[error] {}: {}", label, e),
            }
            client.shutdown_exit()?;
        }
        Sub::Clean => {
            if cli.server == "javac-lsp" {
                anyhow::bail!("clean 仅支持 jdtls，javac-lsp 无 workspace 缓存");
            }
            let lsp_cfg = config.lsp.as_ref();
            let jdtls_cfg = lsp_cfg.and_then(|l| l.server.get("jdtls"));
            let msg = commands::clean::clean_workspace(
                &cli.project_dir,
                cli.data_dir.as_deref(),
                lsp_cfg.and_then(|l| l.data_dir.as_deref()),
                jdtls_cfg.and_then(|o| o.data_dir.as_deref()),
            )?;
            println!("{}", msg);
        }
        Sub::WsSymbols { query } => {
            let server = build_server(&cli.project_dir, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result = commands::workspace_symbols::workspace_symbols(&mut client, query)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(syms) => {
                        for s in syms {
                            println!(
                                "{:?} {} {:?}",
                                s.kind,
                                s.name,
                                s.location.uri
                            );
                        }
                    }
                    None => eprintln!("[warn] 未找到符号"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Signature { path, line, char } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result =
                commands::signature_help::signature_help(&mut client, path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(s) => {
                        for sig in s.signatures {
                            println!("{}", sig.label);
                        }
                    }
                    None => eprintln!("[warn] 该位置无签名信息"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Format { path } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result = commands::formatting::formatting(&mut client, path)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(edits) => {
                        for e in edits {
                            println!(
                                "{}:{} - {}:{}",
                                e.range.start.line,
                                e.range.start.character,
                                e.range.end.line,
                                e.range.end.character
                            );
                            println!("{}", e.new_text);
                        }
                    }
                    None => eprintln!("[warn] 无格式化内容"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Action { path, line, char } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result =
                commands::code_action::code_action(&mut client, path, *line, *char)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(actions) => {
                        for a in actions {
                            match a {
                                CodeActionOrCommand::CodeAction(ca) => {
                                    println!("{}: {:?}", ca.title, ca.kind);
                                }
                                CodeActionOrCommand::Command(cmd) => {
                                    println!("{}: {}", cmd.title, cmd.command);
                                }
                            }
                        }
                    }
                    None => eprintln!("[warn] 无可用的 code action"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Folding { path } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result = commands::folding::folding_range(&mut client, path)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(ranges) => {
                        for r in ranges {
                            println!(
                                "{}:{} - {}:{}",
                                r.start_line, r.start_character.unwrap_or(0),
                                r.end_line, r.end_character.unwrap_or(0)
                            );
                        }
                    }
                    None => eprintln!("[warn] 无折叠范围"),
                }
            }
            client.shutdown_exit()?;
        }
        Sub::Rename {
            path,
            line,
            char,
            new_name,
        } => {
            let server = build_server(path, cli, &config);
            let mut client =
                start_client(&cli.project_dir, &server, open_delay, &javac_classpath)?;
            let result =
                commands::rename::rename(&mut client, path, *line, *char, new_name)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                match result {
                    Some(edit) => {
                        for (uri, edits) in edit.changes.unwrap_or_default() {
                            println!("{:?}:", uri);
                            for e in edits {
                                println!("  {:?} → {}", e.range.start, e.new_text);
                            }
                        }
                    }
                    None => eprintln!("[warn] 无法重命名"),
                }
            }
            client.shutdown_exit()?;
        }
    }
    Ok(())
}

// ─── 辅助函数 ────────────────────────────────────────

fn load_config(path: Option<&Path>) -> BinConfig {
    if let Some(p) = path {
        return match std::fs::read_to_string(p) {
            Ok(s) => match toml::from_str::<BinConfig>(&s) {
                Ok(mut c) => {
                    c.interpolate();
                    c
                }
                Err(e) => {
                    eprintln!("[warn] 配置解析失败 ({}): {}", p.display(), e);
                    BinConfig::default()
                }
            },
            Err(e) => {
                eprintln!("[warn] 配置读取失败 ({}): {}", p.display(), e);
                BinConfig::default()
            }
        };
    }
    match BinConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[warn] 配置加载失败: {}", e);
            BinConfig::default()
        }
    }
}

fn build_server(path: &Path, cli: &Cli, config: &BinConfig) -> ai_cli::lsp::servers::ServerDef {
    let s = resolve_server(path, &cli.server, cli.command.as_deref(), config).unwrap();
    apply_data_dir(
        apply_maven_settings(s, cli.maven_settings.as_deref(), config),
        &cli.project_dir,
        cli.data_dir.as_deref(),
        config,
    )
}

fn start_client(
    project_dir: &Path,
    server: &ai_cli::lsp::servers::ServerDef,
    open_delay: u64,
    classpath: &Option<Vec<String>>,
) -> anyhow::Result<LspClient<StdioTransport>> {
    let mut client = LspClient::start(project_dir, server, open_delay)?;
    if let Some(cp) = classpath {
        client.configure_classpath(cp)?;
    }
    Ok(client)
}

/// 方案C: 从文件提取光标处单词 → grep 候选文件 → didOpen → 重试
/// 返回 true 表示打开了额外文件
fn fallback_open_related(
    client: &mut LspClient<StdioTransport>,
    project_dir: &Path,
    file: &Path,
    line: u32,
    char: u32,
) -> bool {
    // 1. 提取光标处单词
    let word = match extract_word_at(file, line, char) {
        Some(w) => w,
        None => return false,
    };
    // 跳过太短的词（容易命中太多）
    if word.len() < 3 {
        return false;
    }

    // 2. grep 候选文件
    let candidates = match grep_files(project_dir, &word) {
        Ok(files) => files,
        Err(_) => return false,
    };
    if candidates.is_empty() {
        return false;
    }

    // 3. didOpen 候选文件（跳过当前文件）
    let current = std::fs::canonicalize(file).unwrap_or(file.to_path_buf());
    let mut opened = 0;
    for f in &candidates {
        if opened >= 30 {
            break; // 最多打开30个
        }
        let abs = std::fs::canonicalize(f).unwrap_or(f.clone());
        if abs == current {
            continue;
        }
        if client.open_file(f).is_ok() {
            opened += 1;
        }
    }
    eprintln!("[javac-lsp] 自动打开了 {opened} 个关联文件，重试...");
    opened > 0
}

/// 提取文件中指定行列的单词
fn extract_word_at(file: &Path, line: u32, char: u32) -> Option<String> {
    let content = std::fs::read_to_string(file).ok()?;
    let target_line = content.lines().nth(line as usize)?;
    let chars: Vec<char> = target_line.chars().collect();
    let idx = char as usize;
    if idx >= chars.len() || !chars[idx].is_alphanumeric() && chars[idx] != '_' {
        return None;
    }
    // 向左扩展
    let mut start = idx;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    // 向右扩展
    let mut end = idx;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    Some(chars[start..end].iter().collect())
}

/// 在项目目录中搜索包含指定单词的 .java 文件（使用 ripgrep 库 grep）
fn grep_files(project_dir: &Path, word: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    use grep::matcher::Matcher;
    use grep::regex::RegexMatcherBuilder;

    let matcher = RegexMatcherBuilder::new()
        .build(&regex::escape(word))
        .expect("regex compile");
    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(project_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "java").unwrap_or(false))
    {
        if let Ok(content) = std::fs::read(entry.path()) {
            if matcher.is_match(&content).unwrap_or(false) {
                results.push(entry.path().to_path_buf());
                if results.len() >= 50 {
                    break;
                }
            }
        }
    }
    Ok(results)
}

fn output_json_or<T: serde::Serialize>(json_mode: bool, v: &T, text: impl FnOnce() -> String) {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(v).unwrap());
    } else {
        print!("{}", text());
    }
}

/// 判断 CompletionResponse 是否为空
fn is_empty_completion(resp: &CompletionResponse) -> bool {
    match resp {
        CompletionResponse::Array(arr) => arr.is_empty(),
        CompletionResponse::List(list) => list.items.is_empty(),
    }
}

// ─── lsp-types → output 格式桥接 ─────────────────────

use lsp_types::*;

fn format_symbols_value(resp: &DocumentSymbolResponse) -> String {
    let v = serde_json::to_value(resp).unwrap_or_default();
    format_symbols(&v)
}

fn format_hover_value(h: &Hover) -> String {
    let v = serde_json::to_value(h).unwrap_or_default();
    format_hover(&v)
}

fn format_locations_value(locs: &[Location]) -> String {
    let v = serde_json::to_value(locs).unwrap_or_default();
    format_locations(&v)
}

fn format_completions_value(resp: &CompletionResponse) -> String {
    let v = serde_json::to_value(resp).unwrap_or_default();
    format_completions(&v)
}
