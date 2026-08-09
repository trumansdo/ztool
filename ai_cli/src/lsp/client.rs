//! LspClient —— 单个语言服务器的 stdio 客户端
//!
//! 生命周期：`start`(spawn+initialize) → 查询方法 → `shutdown_exit`
//! 特性：mtime 缓存（文件未变更跳过 didOpen）、1-based 行号转换、publishDiagnostics 捕获

use crate::lsp::jsonrpc::{read_msg, write_msg};
use crate::lsp::servers::ServerDef;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, SystemTime};

/// 文件路径 → file:// URI（Windows 盘符转 file:///D:/...）
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

/// LSP 客户端
pub struct LspClient {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    stderr: Option<BufReader<ChildStderr>>,
    next_id: u32,
    root_uri: String,
    language_id: String,
    opened: HashMap<String, SystemTime>, // uri -> mtime
    /// didOpen 后缓冲等待时长 (等服务器处理完文档再查询, 防空结果)
    open_delay: Duration,
    /// 捕获的 publishDiagnostics 通知（request 循环中收集）
    pub pushed_diagnostics: Vec<Vec<Value>>,
}

impl LspClient {
    /// 启动服务器进程并完成 initialize 握手
    /// open_delay_ms: didOpen 后缓冲等待, 默认 200ms
    pub fn start(root: &Path, server: &ServerDef, open_delay_ms: u64) -> anyhow::Result<Self> {
        let mut cmd = Command::new(&server.command[0]);
        cmd.args(&server.command[1..])
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            // stderr 保留管道以便诊断(EOF/崩溃时读出错误); 有后台消费者时不会积压
            .stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| {
            anyhow::anyhow!(
                "启动 LSP 服务器 '{}' 失败: {} (命令: {}，安装提示: {})",
                server.id,
                e,
                server.command.join(" "),
                server.install_hint
            )
        })?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("无法获取服务器 stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("无法获取服务器 stdout"))?;
        let stderr = child.stderr.take().map(BufReader::new);
        let mut client = LspClient {
            child,
            stdin,
            reader: BufReader::new(stdout),
            stderr,
            next_id: 0,
            root_uri: path_to_uri(root),
            language_id: server.language_id.clone(),
            opened: HashMap::new(),
            open_delay: Duration::from_millis(open_delay_ms),
            pushed_diagnostics: Vec::new(),
        };
        client.initialize()?;
        Ok(client)
    }

    /// initialize 握手（声明客户端能力）
    fn initialize(&mut self) -> anyhow::Result<Value> {
        let params = json!({
            "processId": std::process::id(),
            "rootUri": self.root_uri,
            "capabilities": {
                "textDocument": {
                    "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "definition": { "linkSupport": true }
                }
            },
            "workspaceFolders": [{ "uri": self.root_uri, "name": "root" }]
        });
        let r = self.request("initialize", params)?;
        self.notify("initialized", json!({}))?;
        Ok(r)
    }

    fn next_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// 发送请求并等待对应 id 的响应（自动跳过通知；捕获 publishDiagnostics）
    pub fn request(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = self.next_id();
        write_msg(
            &mut self.stdin,
            &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
        )?;
        loop {
            let msg = read_msg(&mut self.reader).map_err(|e| {
                let exit = self.child.try_wait().ok().flatten();
                // 读 stderr 尾部用于诊断 (非阻塞尽力而为)
                let mut err_tail = String::new();
                if let Some(s) = self.stderr.as_mut() {
                    let mut tmp = [0u8; 2048];
                    loop {
                        match s.read(&mut tmp) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => err_tail.push_str(&String::from_utf8_lossy(&tmp[..n])),
                        }
                    }
                }
                anyhow::anyhow!(
                    "{} 阶段读响应失败: {} (进程退出: {:?}, stderr: {})",
                    method, e, exit, err_tail.chars().take(300).collect::<String>()
                )
            })?;
            // 坑: 不能像 subprocess.communicate() 那样等待进程退出——LSP 服务器响应后
            //     会持续存活等待后续请求, communicate() 会一直阻塞被误判为"卡死/超时"
            //     (曾因此把 3 秒能响应的 jdtls 误判为 220s 无响应)
            if msg.get("id").and_then(Value::as_u64) == Some(id as u64) {
                if let Some(err) = msg.get("error") {
                    anyhow::bail!("LSP 错误 {}: {}", method, err);
                }
                return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
            }
            // 通知: 捕获 publishDiagnostics, 其余跳过
            // 坑: publishDiagnostics 是异步推送, 只有在 request 驱动的读循环中才会被
            //     消费到(服务器不会主动推给无人读的 socket), 故在此统一收集
            if msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics") {
                if let Some(diags) = msg
                    .pointer("/params/diagnostics")
                    .and_then(Value::as_array)
                {
                    self.pushed_diagnostics.push(diags.clone());
                }
            }
        }
    }

    /// 发送通知（无响应）
    pub fn notify(&mut self, method: &str, params: Value) -> anyhow::Result<()> {
        write_msg(
            &mut self.stdin,
            &json!({"jsonrpc": "2.0", "method": method, "params": params}),
        )?;
        Ok(())
    }

    /// 打开文件（mtime 缓存: 未变更跳过; 变更则 didClose 后重开）
    /// 对齐 pi-coding-tools client.ts 的 mtime 优化: 避免重复 didOpen 浪费 IO,
    /// 外部编辑(mtime 变化)时通过 didClose→didOpen 重新加载
    pub fn open_file(&mut self, path: &Path) -> anyhow::Result<String> {
        let uri = path_to_uri(path);
        let mtime = fs::metadata(path).and_then(|m| m.modified())?;
        if let Some(last) = self.opened.get(&uri) {
            if *last == mtime {
                return Ok(uri);
            }
            self.notify("textDocument/didClose", json!({"textDocument": {"uri": uri}}))?;
            self.opened.remove(&uri);
        }
        let text = fs::read_to_string(path)?;
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": { "uri": uri, "languageId": self.language_id, "version": 1, "text": text }
            }),
        )?;
        self.opened.insert(uri.clone(), mtime);
        // 坑: didOpen 是异步通知(无响应), 立即查询可能返回空结果;
        //     需缓冲 open_delay 让服务器完成文档解析/reconcile
        std::thread::sleep(self.open_delay);
        Ok(uri)
    }

    /// 悬停信息（line 为 1-based，内部转 0-based）
    /// 坑: AI/命令行习惯传 1-based 行号, LSP 协议是 0-based, 漏转换会错位一行
    pub fn hover(&mut self, path: &Path, line: usize, character: usize) -> anyhow::Result<Value> {
        let uri = self.open_file(path)?;
        self.request(
            "textDocument/hover",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": line - 1, "character": character}
            }),
        )
    }

    /// 文档符号大纲
    pub fn document_symbols(&mut self, path: &Path) -> anyhow::Result<Value> {
        let uri = self.open_file(path)?;
        self.request("textDocument/documentSymbol", json!({"textDocument": {"uri": uri}}))
    }

    /// 跳转定义
    pub fn definition(
        &mut self,
        path: &Path,
        line: usize,
        character: usize,
    ) -> anyhow::Result<Value> {
        let uri = self.open_file(path)?;
        self.request(
            "textDocument/definition",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": line - 1, "character": character}
            }),
        )
    }

    /// 查找引用
    pub fn references(
        &mut self,
        path: &Path,
        line: usize,
        character: usize,
        include_decl: bool,
    ) -> anyhow::Result<Value> {
        let uri = self.open_file(path)?;
        self.request(
            "textDocument/references",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": line - 1, "character": character},
                "context": {"includeDeclaration": include_decl}
            }),
        )
    }

    /// 代码补全
    pub fn completion(
        &mut self,
        path: &Path,
        line: usize,
        character: usize,
    ) -> anyhow::Result<Value> {
        let uri = self.open_file(path)?;
        self.request(
            "textDocument/completion",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": line - 1, "character": character}
            }),
        )
    }

    /// 优雅关闭: shutdown → exit → 强制 kill 兜底
    pub fn shutdown_exit(&mut self) -> anyhow::Result<()> {
        let _ = self.request("shutdown", Value::Null);
        let _ = self.notify("exit", json!({}));
        let _ = self.child.kill();
        let _ = self.child.wait();
        Ok(())
    }
}
