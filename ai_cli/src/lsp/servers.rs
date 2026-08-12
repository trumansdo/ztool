//! 语言服务器注册表与配置解析
//!
//! 内置服务器表（对齐 pi-coding-tools 的 servers.ts），支持：
//! - TOML 配置覆盖（exe 同级 binconfig.toml 的 `[lsp.<id>]` 段）
//! - CLI `--command` 覆盖（最高优先级）
//! - Maven settings 注入（方案 B：`-Djava.configuration.maven.userSettings`）

use crate::config::BinConfig;
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::path::Path;

/// 语言服务器定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDef {
    pub id: String,
    pub command: Vec<String>,
    pub extensions: Vec<String>,
    pub language_id: String,
    pub install_hint: String,
}

/// 内置服务器注册表
/// 所有 command 置空，强制通过 binconfig.toml 配置启动命令
pub fn builtin_servers() -> Vec<ServerDef> {
    vec![
        ServerDef {
            id: "typescript-language-server".into(),
            command: vec![],
            extensions: vec![".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"]
                .into_iter()
                .map(String::from)
                .collect(),
            language_id: "typescript".into(),
            install_hint: "npm install -g typescript-language-server typescript".into(),
        },
        ServerDef {
            id: "pyright".into(),
            command: vec![],
            extensions: vec![".py".into()],
            language_id: "python".into(),
            install_hint: "npm install -g pyright".into(),
        },
        ServerDef {
            id: "javac-lsp".into(),
            command: vec![],
            extensions: vec![".java".into()],
            language_id: "java".into(),
            install_hint: "在 binconfig.toml 的 [lsp.javac-lsp] 段配置 command 和 vars".into(),
        },
        ServerDef {
            id: "jdtls".into(),
            command: vec![],
            extensions: vec![".java".into()],
            language_id: "java".into(),
            install_hint: "Install Eclipse JDT Language Server (jdtls). Requires JDK 17+.".into(),
        },
        ServerDef {
            id: "rust-analyzer".into(),
            command: vec![],
            extensions: vec![".rs".into()],
            language_id: "rust".into(),
            install_hint: "rustup component add rust-analyzer".into(),
        },
        ServerDef {
            id: "kotlin-language-server".into(),
            command: vec![],
            extensions: vec![".kt".into(), ".kts".into()],
            language_id: "kotlin".into(),
            install_hint: "Install kotlin-language-server. Requires JDK.".into(),
        },
        ServerDef {
            id: "clangd".into(),
            command: vec![],
            extensions: vec![".c", ".h", ".cpp", ".cc", ".cxx", ".hpp", ".hxx"]
                .into_iter()
                .map(String::from)
                .collect(),
            language_id: "cpp".into(),
            install_hint: "Install clangd. Needs compile_commands.json for best results.".into(),
        },
    ]
}

/// 按文件扩展名检测语言 id
pub fn detect_language(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    let full = format!(".{}", ext);
    builtin_servers().into_iter().find_map(|s| {
        if s.extensions.iter().any(|e| e == &full) {
            Some(s.language_id.clone())
        } else {
            None
        }
    })
}

/// 解析文件对应的服务器（内置 + 配置覆盖 + 命令行覆盖）
///
/// - `cli_server_id`: `--server` 显式指定服务器 id
/// - `cli_command`: `--command` 覆盖启动命令（最高优先级）
/// - `config`: 统一配置（`[lsp.<id>]` 段）
pub fn resolve_server(
    file: &Path,
    cli_server_id: &str,
    cli_command: Option<&str>,
    config: &BinConfig,
) -> anyhow::Result<ServerDef> {
    // 显式指定 server 时跳过文件类型检测
    let builtin = if cli_server_id != "auto" {
        builtin_servers()
            .into_iter()
            .find(|s| s.id == cli_server_id)
            .ok_or_else(|| anyhow::anyhow!("未知服务器: {}", cli_server_id))?
    } else {
        let lang = detect_language(file).ok_or_else(|| {
            anyhow::anyhow!(
                "无法识别的文件类型: {} (支持: java/ts/py/kt/cpp)",
                file.display()
            )
        })?;
        builtin_servers()
            .into_iter()
            .find(|s| s.language_id == lang)
            .unwrap()
    };

    let mut server = builtin.clone();
    // 配置覆盖 ([lsp.<id>])
    // 坑: TOML 的 [lsp.jdtls] 反序列化后是 lsp: {jdtls: {...}},
    //     因此 BinConfig.lsp 是 Option<LspSection>{server: HashMap} 且 server 用 #[serde(flatten)] 收集,
    //     否则 [lsp.jdtls] 子表会被当作未知字段静默丢弃
    if let Some(ov) = config.lsp.as_ref().and_then(|l| l.server.get(&builtin.id)) {
        if ov.disabled {
            anyhow::bail!("服务器 '{}' 已在配置中禁用 (disabled = true)", builtin.id);
        }
        if let Some(cmd) = &ov.command {
            server.command = cmd.clone();
        }
    }
    // 命令行覆盖（最高优先级）
    if let Some(cmd) = cli_command {
        server.command = parse_command(cmd);
    }
    Ok(server)
}

/// 注入 Maven settings 参数 (方案 B): `-Djava.configuration.maven.userSettings=<path>`
///
/// 直接 java 命令时插入到 `-jar` 之前（否则被当作应用参数）；其余命令追加尾部。
pub fn apply_maven_settings(
    mut server: ServerDef,
    cli_ms: Option<&str>,
    config: &BinConfig,
) -> ServerDef {
    let lsp = config.lsp.as_ref();
    let ms = cli_ms
        .map(|s| s.to_string())
        .or_else(|| lsp.and_then(|l| l.server.get(&server.id).and_then(|o| o.maven_settings.clone())));
    if let Some(path) = ms {
        let arg = format!("-Djava.configuration.maven.userSettings={}", path);
        // 坑: java 命令的 -D 系统属性必须放在 -jar 之前,
        //     -jar 之后的参数会被 JVM 当作传给主类的应用参数而非 JVM 属性,
        //     导致 maven settings 静默不生效
        if let Some(idx) = server.command.iter().position(|a| a == "-jar") {
            server.command.insert(idx, arg);
        } else {
            server.command.push(arg);
        }
    }
    server
}

/// 按 data_dir 配置注入 -data (workspace 目录): data_dir/<sha1(root绝对路径)>
/// 优先级: CLI --data-dir > 配置 [lsp].data_dir
/// 设计: AI 场景多项目切换, root 由命令行必填传入;
///      workspace 按项目路径哈希自动隔离/复用, 无需为每个项目手写 -data
/// 若 command 已显式包含 -data, 则尊重显式配置不覆盖
pub fn apply_data_dir(
    mut server: ServerDef,
    project_dir: &Path,
    cli_data_dir: Option<&str>,
    config: &BinConfig,
) -> ServerDef {
    if server.command.iter().any(|a| a == "-data") {
        return server;
    }
    // 坑: -data 是 jdtls(equinox launcher) 专用 workspace 参数,
    //     注入到 rust-analyzer/clangd 等其它服务器会报 "unexpected flag: -data" 直接罢工
    //     (实测: rust-analyzer 1.94.1 收到未知参数后 stdout 关闭, 表现为 EOF)
    //     故仅对含 -jar 的命令(equinox launcher)注入
    if !server.command.iter().any(|a| a == "-jar") {
        return server;
    }
    let lsp = config.lsp.as_ref();
    // 优先级: CLI --data-dir > [lsp.<id>].data_dir > [lsp].data_dir (全局兜底)
    let data_dir = cli_data_dir
        .map(|s| s.to_string())
        .or_else(|| lsp.and_then(|l| l.server.get(&server.id).and_then(|o| o.data_dir.clone())))
        .or_else(|| lsp.and_then(|l| l.data_dir.clone()));
    if let Some(data_dir) = data_dir {
        let abs = if project_dir.is_absolute() {
            project_dir.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(project_dir)
        };
        let norm = abs.to_string_lossy().replace('\\', "/");
        // sha1 0.10 需通过 digest trait 使用 (Sha1::digest 静态方法)
        let digest = format!("{:x}", sha1::Sha1::digest(norm.as_bytes()));
        let dir = Path::new(&data_dir).join(digest);
        let _ = std::fs::create_dir_all(&dir);
        // 坑: -data 是 launcher 参数, 必须放在 -jar <launcher.jar> 之后 (等号形式不被识别);
        //     -jar 与其 jar 路径是成对的, 插入点应为 idx+2 (跳过 launcher.jar)
        //     且 workspace 不能放在项目目录内部(Eclipse 报 overlaps 错误)
        if let Some(idx) = server.command.iter().position(|a| a == "-jar") {
            server.command.insert(idx + 2, "-data".into());
            server.command.insert(idx + 3, dir.to_string_lossy().into_owned());
        } else {
            server.command.push("-data".into());
            server.command.push(dir.to_string_lossy().into_owned());
        }
    }
    server
}

/// "python D:\\x\\jdtls.py --opt" -> ["python", "D:\\x\\jdtls.py", "--opt"]
/// 支持双引号/单引号包裹的含空格参数
/// 坑: Windows 下 spawn 直接执行 .bat 会失败(Node/Rust 均如此),
///     必须指向真实可执行文件(python.exe / java.exe); 含空格路径需引号包裹
pub fn parse_command(cmd: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for ch in cmd.chars() {
        match quote {
            Some(q) if ch == q => quote = None,
            Some(_) => cur.push(ch),
            None => match ch {
                '"' | '\'' => quote = Some(ch),
                ' ' | '\t' => {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                }
                _ => cur.push(ch),
            },
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
