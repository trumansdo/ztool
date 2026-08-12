//! 类型安全的 LSP 客户端
//!
//! 泛型 `LspClient<T: LspTransport>`，编译期多态，零开销。
//! 与旧 `lsp::client::LspClient` 完全独立。
//!
//! # javac-lsp vs jdtls 关键差异
//!
//! | 特性 | jdtls | javac-lsp |
//! |:--|:--|:--|
//! | classpath 推断 | 自动从 pom.xml/gradle 推断 | 需手动发送 didChangeConfiguration |
//! | 引用查找 | 全项目索引 | 仅搜索 FileStore 中已打开的文件 |
//! | hover 字段声明 | 支持 | 不支持 (FindHoverElement 缺 visitVariable) |
//! | data-dir | 有 (缓存索引) | 无 (纯内存) |
//! | 构建 | java/buildWorkspace | 不支持 |
//!
//! 因此 javac-lsp 适合轻量符号/定义查询，jdtls 适合完整的 IDE 级功能。

use crate::lsp::servers::ServerDef;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// 文件路径 → file:// URI
pub fn path_to_uri(path: &Path) -> String {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let p = abs.to_string_lossy().replace('\\', "/");
    if p.starts_with('/') {
        format!("file://{}", p)
    } else {
        format!("file:///{}", p)
    }
}

/// 将 1-based 行列 (AI/CLI 习惯) 转换为 LSP 协议要求的 0-based Position。
///
/// 所有 commands 层函数接收 1-based 行列并内部调用本函数转换，
/// 避免 1-based 参数被原样发送导致请求位置偏移 (+1, +1) 的 bug。
/// `saturating_sub(1)` 保证输入 0 时输出 0 (LSP 最小合法值)，不 panic。
pub fn lsp_position(line: u32, character: u32) -> Position {
    Position {
        line: line.saturating_sub(1),
        character: character.saturating_sub(1),
    }
}

/// 类型安全 LSP 客户端
pub struct LspClient<T: LspTransport> {
    transport: T,
    next_id: u64,
    /// 后台线程消费 stderr 后的尾部内容 (供 EOF/崩溃诊断)
    stderr_tail: Arc<Mutex<String>>,
    /// 子进程句柄 (用于 shutdown/exit)
    child: Option<Child>,
    /// 已打开文件 mtime 缓存
    opened: HashMap<String, SystemTime>,
    /// didOpen 后缓冲等待时长
    open_delay: Duration,
    /// 捕获的 publishDiagnostics
    pub pushed_diagnostics: Vec<Vec<Value>>,
}

impl LspClient<crate::lsp::transport::StdioTransport> {
    /// 启动服务器进程并完成 initialize 握手
    pub fn start(project_dir: &Path, server: &ServerDef, open_delay_ms: u64) -> Result<Self> {
        let mut cmd = Command::new(&server.command[0]);
        cmd.args(&server.command[1..])
            .current_dir(project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            anyhow::anyhow!(
                "启动 LSP 服务器 '{}' 失败: {} (命令: {})",
                server.id,
                e,
                server.command.join(" ")
            )
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("无法获取 stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("无法获取 stdout"))?;

        // 后台消费 stderr
        let stderr_tail: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        if let Some(mut se) = child.stderr.take() {
            let tail = stderr_tail.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut kept: Vec<u8> = Vec::new();
                loop {
                    match se.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            kept.extend_from_slice(&buf[..n]);
                            if kept.len() > 4096 {
                                kept.drain(..kept.len() - 4096);
                            }
                        }
                    }
                }
                if let Ok(mut t) = tail.lock() {
                    *t = String::from_utf8_lossy(&kept).into_owned();
                }
            });
        }

        let transport = crate::lsp::transport::StdioTransport::new(stdout, stdin);
        let root_uri = path_to_uri(project_dir);

        let mut client = Self {
            transport,
            next_id: 1,
            stderr_tail,
            child: Some(child),
            opened: HashMap::new(),
            open_delay: Duration::from_millis(open_delay_ms),
            pushed_diagnostics: Vec::new(),
        };

        // initialize 握手
        client.initialize(&root_uri)?;

        Ok(client)
    }
}

impl<T: LspTransport> LspClient<T> {
    /// 发送请求，返回类型化响应
    pub fn request<P: Serialize, R: DeserializeOwned>(
        &mut self,
        method: &str,
        params: &P,
    ) -> Result<R> {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        self.transport.write_message(&request.to_string())?;

        loop {
            let raw = self.transport.read_message().map_err(|e| {
                let exit = self
                    .child
                    .as_mut()
                    .and_then(|c| c.try_wait().ok().flatten());
                let err_tail = self
                    .stderr_tail
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .clone();
                anyhow::anyhow!(
                    "{} 阶段读响应失败: {} (进程退出: {:?}, stderr: {})",
                    method,
                    e,
                    exit,
                    err_tail.chars().take(300).collect::<String>()
                )
            })?;

            let msg: Value = serde_json::from_str(&raw)?;

            // 匹配响应
            if msg.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(err) = msg.get("error") {
                    anyhow::bail!("LSP 错误 {}: {}", method, err);
                }
                let result = msg.get("result").cloned().unwrap_or(Value::Null);
                return Ok(serde_json::from_value(result)?);
            }

            // 收集 publishDiagnostics 通知
            if msg.get("method").and_then(Value::as_str)
                == Some("textDocument/publishDiagnostics")
            {
                if let Some(diags) =
                    msg.pointer("/params/diagnostics").and_then(Value::as_array)
                {
                    self.pushed_diagnostics.push(diags.clone());
                }
            }
        }
    }

    /// 发送通知 (无响应)
    pub fn notify<P: Serialize>(&mut self, method: &str, params: &P) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.transport.write_message(&notification.to_string())
    }

    /// initialize 握手
    #[allow(deprecated)]
    fn initialize(&mut self, root_uri: &str) -> Result<()> {
        let root_uri: Uri = root_uri
            .parse()
            .map_err(|e| anyhow::anyhow!("无效 URI: {}", e))?;

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: Some(root_uri.clone()),
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    document_symbol: Some(DocumentSymbolClientCapabilities {
                        hierarchical_document_symbol_support: Some(true),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                        ..Default::default()
                    }),
                    definition: Some(GotoCapability {
                        link_support: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_folders: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri,
                name: "root".to_string(),
            }]),
            ..Default::default()
        };

        let _: InitializeResult = self.request("initialize", &params)?;
        self.notify("initialized", &serde_json::json!({}))?;
        Ok(())
    }

    /// 打开文件 (mtime 缓存优化)
    pub fn open_file(&mut self, path: &Path) -> Result<String> {
        let uri = path_to_uri(path);
        let mtime = fs::metadata(path).and_then(|m| m.modified())?;

        if let Some(last) = self.opened.get(&uri) {
            if *last == mtime {
                return Ok(uri);
            }
            self.notify(
                "textDocument/didClose",
                &serde_json::json!({"textDocument": {"uri": uri}}),
            )?;
            self.opened.remove(&uri);
        }

        let text = fs::read_to_string(path)?;
        self.notify(
            "textDocument/didOpen",
            &serde_json::json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "java",
                    "version": 1,
                    "text": text
                }
            }),
        )?;
        self.opened.insert(uri.clone(), mtime);
        std::thread::sleep(self.open_delay);
        Ok(uri)
    }

    /// 发送 workspace/didChangeConfiguration 配置 classpath (javac-lsp 需要)
    pub fn configure_classpath(&mut self, classpath: &[String]) -> Result<()> {
        let settings = serde_json::json!({
            "java": {
                "classPath": classpath
            }
        });
        self.notify("workspace/didChangeConfiguration", &serde_json::json!({
            "settings": settings
        }))
    }

    /// 优雅关闭
    pub fn shutdown_exit(&mut self) -> Result<()> {
        let _ = self.request::<_, Value>("shutdown", &serde_json::json!(null));
        let _ = self.notify("exit", &serde_json::json!({}));
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}
