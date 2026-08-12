# 基于 lsp-types 的 CLI 工具设计方案

> 目标: 用 `lsp-types` 统一 LSP 类型，重构 CLI 工具实现类型安全  
> 日期: 2025-08-12  
> 状态: 设计草案

---

## 目录

1. [lsp-types 定位分析](#1-lsp-types-定位分析)
2. [架构设计](#2-架构设计)
3. [模块设计](#3-模块设计)
4. [核心代码](#4-核心代码)
5. [与现有实现的对比](#5-与现有实现的对比)
6. [迁移路线图](#6-迁移路线图)

---

## 1. lsp-types 定位分析

### 1.1 是什么

`lsp-types` 是 LSP 协议规范的 **纯 Rust 类型定义** 库。

| 有 | 没有 |
|:--|:--|
| ✅ 所有 LSP 结构体/枚举定义 | ❌ JSON-RPC 消息帧 |
| ✅ serde Serialize/Deserialize | ❌ 网络/stdio 传输 |
| ✅ 完整的文档注释 | ❌ 请求/响应路由 |
| ✅ 类型安全的请求参数/响应 | ❌ 进程管理 |
| ✅ 与 LSP 规范同步更新 | ❌ 取消机制 |

**一句话**: 它给你类型，不给你通信。通信层自己写。

### 1.2 核心类型一览

```rust
// 基础类型
lsp_types::Uri
lsp_types::Position { line: u32, character: u32 }
lsp_types::Range { start: Position, end: Position }
lsp_types::Location { uri: Uri, range: Range }

// 请求参数
lsp_types::InitializeParams { root_uri, capabilities, ... }
lsp_types::CompletionParams { text_document_position, ... }
lsp_types::HoverParams { text_document_position_params, ... }
lsp_types::GotoDefinitionParams { text_document_position_params, ... }
lsp_types::ReferenceParams { text_document_position, context, ... }
lsp_types::DocumentSymbolParams { text_document, ... }

// 响应类型
lsp_types::InitializeResult { capabilities, server_info }
lsp_types::CompletionResponse          // enum: Array | List
lsp_types::Hover { contents, range }
lsp_types::GotoDefinitionResponse      // type alias: Option<Vec<Location>>
lsp_types::DocumentSymbolResponse      // enum: Flat | Nested

// 通知
lsp_types::DidOpenTextDocumentParams
lsp_types::DidChangeTextDocumentParams
lsp_types::PublishDiagnosticsParams
lsp_types::ShowMessageParams

// 能力声明
lsp_types::ServerCapabilities
lsp_types::ClientCapabilities
```

### 1.3 与现有手写 JSON 的对比

```rust
// 现有方式: 手写 JSON Value
let params = serde_json::json!({
    "textDocument": { "uri": "file:///Foo.java" },
    "position": { "line": 10, "character": 5 }
});
let resp: serde_json::Value = client.request("textDocument/definition", &params);

// lsp-types 方式: 类型安全
let params = GotoDefinitionParams {
    text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: Uri::from("file:///Foo.java") },
        position: Position { line: 10, character: 5 },
    },
    work_done_progress_params: Default::default(),
    partial_result_params: Default::default(),
};
let resp: Option<GotoDefinitionResponse> = client.request("textDocument/definition", params);
```

---

## 2. 架构设计

### 2.1 分层架构

```
┌──────────────────────────────────────────────┐
│                 CLI Layer                      │
│  clap::Parser → LspCommand → OutputFormatter  │
├──────────────────────────────────────────────┤
│              Service Layer                     │
│  LspTool::run(command)                        │
│    ├─ 启动 LSP 进程                           │
│    ├─ initialize → initialized                │
│    ├─ didOpen 文件                            │
│    ├─ 发送类型化请求                          │
│    └─ 返回类型化响应                          │
├──────────────────────────────────────────────┤
│            Transport Layer                     │
│  JsonRpcTransport                             │
│    ├─ read_message() → String                 │
│    ├─ write_message(json)                     │
│    └─ MessageStream (Iterator)                │
├──────────────────────────────────────────────┤
│              Type Layer                        │
│  lsp_types (外部 crate)                       │
│    ├─ 所有 LSP 请求/响应/通知类型             │
│    └─ serde 自动序列化/反序列化               │
└──────────────────────────────────────────────┘
```

### 2.2 核心原则

1. **lsp-types 只管类型**: 不引入 tower-lsp 等服务端框架
2. **传输层自研**: 保留现有 `jsonrpc.rs` 的消息帧逻辑，升级为 trait 抽象
3. **渐进替换**: 一个命令一个命令地从 `serde_json::Value` 迁移到 `lsp_types`
4. **零破坏**: 配置格式、命令行接口、输出格式不变

---

## 3. 模块设计

### 3.1 依赖

```toml
[dependencies]
lsp-types = { version = "0.97", features = ["proposed"] }
serde = "1"
serde_json = "1"
clap = "4"
toml = "0.8"
sha1 = "0.10"
```

不引入 tokio/tower-lsp，保持同步模型，与现有架构一致。

### 3.2 模块树

```
src/
├── lsp/
│   ├── mod.rs
│   ├── types.rs              # [NEW] lsp-types 的 re-export + 扩展
│   │   pub use lsp_types::*;
│   │   // 自定义扩展类型 (不在标准 LSP 中)
│   │   pub struct JavaBuildWorkspaceParams { ... }
│   │
│   ├── transport.rs          # [NEW] JSON-RPC 传输抽象
│   │   pub trait LspTransport {
│   │       fn read_message(&mut self) -> Result<String>;
│   │       fn write_message(&mut self, json: &str) -> Result<()>;
│   │   }
│   │   pub struct StdioTransport { stdin, stdout }
│   │
│   ├── client.rs             # [REWRITE] 泛型 LSP 客户端
│   │   pub struct LspClient<T: LspTransport> {
│   │       transport: T,
│   │       next_id: u64,
│   │   }
│   │   impl<T: LspTransport> LspClient<T> {
│   │       pub fn request<P: Serialize, R: DeserializeOwned>(
│   │           &mut self, method: &str, params: &P
│   │       ) -> Result<R>;
│   │       pub fn notify<P: Serialize>(&mut self, method: &str, params: &P);
│   │   }
│   │
│   ├── servers.rs            # [保留] 服务器定义
│   ├── config.rs             # [保留] 配置
│   │
│   ├── commands/             # [NEW] 命令实现 (每个命令一个文件)
│   │   ├── mod.rs
│   │   ├── initialize.rs     # 初始化序列
│   │   ├── symbols.rs        # documentSymbol + workspaceSymbol
│   │   ├── definition.rs     # gotoDefinition
│   │   ├── references.rs     # findReferences
│   │   ├── hover.rs          # hover
│   │   ├── completion.rs     # completion
│   │   ├── build.rs          # java/buildWorkspace (jdtls)
│   │   └── clean.rs          # 清理缓存 (jdtls)
│   │
│   └── output.rs             # [保留] 输出格式化
│
├── bin/
│   └── lsp_tool.rs           # [REWRITE] CLI 入口
│
└── lib.rs
```

### 3.3 关键设计决策

| 决策 | 选择 | 理由 |
|:--|:--|:--|
| 异步 vs 同步 | **同步** | 与现有架构一致，CLI 不需要高并发 |
| 传输抽象 | `trait LspTransport` | 可 mock 测试，可切换 stdio/TCP |
| 客户端泛型 | `LspClient<T: LspTransport>` | 编译期多态，零开销 |
| 命令组织 | 每个命令独立文件 | 关注点分离，易维护 |
| 错误类型 | `anyhow::Result` + 自定义 `LspError` | 统一错误处理 |

---

## 4. 核心代码

### 4.1 传输层

```rust
// transport.rs
use std::io::{BufRead, BufReader, Read, Write};

pub trait LspTransport {
    fn read_message(&mut self) -> Result<String>;
    fn write_message(&mut self, json: &str) -> Result<()>;
}

/// stdio 实现 (连接外部 LSP 进程)
pub struct StdioTransport {
    reader: BufReader<Box<dyn Read + Send>>,
    writer: Box<dyn Write + Send>,
}

impl StdioTransport {
    pub fn new(reader: impl Read + Send + 'static, writer: impl Write + Send + 'static) -> Self {
        Self {
            reader: BufReader::new(Box::new(reader)),
            writer: Box::new(writer),
        }
    }
}

impl LspTransport for StdioTransport {
    fn read_message(&mut self) -> Result<String> {
        // 读 Content-Length header
        let mut content_length = None;
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            if line == "\r\n" { break; }
            if let Some(len) = line.strip_prefix("Content-Length: ") {
                content_length = Some(len.trim().parse::<usize>()?);
            }
        }
        let len = content_length.ok_or(anyhow::anyhow!("missing Content-Length"))?;
        // 读 body
        let mut body = vec![0u8; len];
        self.reader.read_exact(&mut body)?;
        Ok(String::from_utf8(body)?)
    }

    fn write_message(&mut self, json: &str) -> Result<()> {
        let bytes = json.as_bytes();
        let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
        self.writer.write_all(header.as_bytes())?;
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }
}
```

### 4.2 客户端

```rust
// client.rs
use serde::{de::DeserializeOwned, Serialize};

pub struct LspClient<T: LspTransport> {
    transport: T,
    next_id: u64,
}

impl<T: LspTransport> LspClient<T> {
    pub fn new(transport: T) -> Self {
        Self { transport, next_id: 1 }
    }

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

        // 等待匹配 id 的响应
        loop {
            let raw = self.transport.read_message()?;
            let msg: serde_json::Value = serde_json::from_str(&raw)?;
            if msg.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(err) = msg.get("error") {
                    anyhow::bail!("LSP error: {}", err);
                }
                let result = msg.get("result")
                    .ok_or_else(|| anyhow::anyhow!("missing result"))?;
                return Ok(serde_json::from_value(result.clone())?);
            }
            // 非匹配消息: 可能是通知，忽略或记录
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
}
```

### 4.3 命令实现示例

```rust
// commands/definition.rs
use lsp_types::*;

pub fn goto_definition(
    client: &mut LspClient<impl LspTransport>,
    file: &str,
    line: u32,
    character: u32,
) -> Result<Option<GotoDefinitionResponse>> {
    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: Uri::from_file_path(file)
                    .map_err(|_| anyhow::anyhow!("invalid file path"))?,
            },
            position: Position { line, character },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    client.request::<_, Option<GotoDefinitionResponse>>(
        "textDocument/definition",
        &params,
    )
}
```

```rust
// commands/symbols.rs
use lsp_types::*;

pub fn document_symbols(
    client: &mut LspClient<impl LspTransport>,
    file: &str,
) -> Result<DocumentSymbolResponse> {
    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier {
            uri: Uri::from_file_path(file)
                .map_err(|_| anyhow::anyhow!("invalid file path"))?,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    client.request("textDocument/documentSymbol", &params)
}
```

### 4.4 CLI 入口

```rust
// bin/lsp_tool.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lsp_tool")]
struct Cli {
    #[arg(short, long)]
    project: Option<String>,
    #[arg(short, long)]
    server: Option<String>,
    #[arg(long, default_value = "table")]
    format: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 文件符号
    Symbols { file: String },
    /// 跳转定义
    Def { file: String, line: u32, character: u32 },
    /// 查找引用
    Refs { file: String, line: u32, character: u32 },
    /// 悬停文档
    Hover { file: String, line: u32, character: u32 },
    /// 代码补全
    Comp { file: String, line: u32, character: u32 },
    /// 构建 (jdtls)
    Build { #[arg(long)] rebuild: bool, #[arg(long)] json: bool },
    /// 清理缓存 (jdtls)
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. 加载配置
    let config = BinConfig::load()?;

    // 2. 解析服务器
    let server_def = resolve_server(cli.server.as_deref(), cli.project.as_deref())?;

    // 3. 启动 LSP 进程
    let mut child = std::process::Command::new(&server_def.command[0])
        .args(&server_def.command[1..])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let transport = StdioTransport::new(
        child.stdout.take().unwrap(),
        child.stdin.take().unwrap(),
    );
    let mut client = LspClient::new(transport);

    // 4. 初始化
    let init_result = commands::initialize(&mut client, &server_def)?;

    // 5. 打开文件 (如果需要)
    if let Some(file) = cli.command.file() {
        let content = std::fs::read_to_string(file)?;
        commands::did_open(&mut client, file, &content)?;
    }

    // 6. 执行命令
    let output = match cli.command {
        Command::Symbols { file } => {
            commands::document_symbols(&mut client, &file)?.into()
        }
        Command::Def { file, line, character } => {
            commands::goto_definition(&mut client, &file, line, character)?.into()
        }
        Command::Refs { file, line, character } => {
            commands::find_references(&mut client, &file, line, character)?.into()
        }
        Command::Hover { file, line, character } => {
            commands::hover(&mut client, &file, line, character)?.into()
        }
        Command::Comp { file, line, character } => {
            commands::completion(&mut client, &file, line, character)?.into()
        }
        Command::Build { rebuild, json } => {
            commands::build(&mut client, rebuild, json)?.into()
        }
        Command::Clean => {
            commands::clean(&config, cli.project.as_deref())?.into()
        }
    };

    // 7. 输出
    print_output(&output, &cli.format);

    // 8. 关闭
    client.request::<_, ()>("shutdown", &serde_json::json!(null))?;
    client.notify("exit", &serde_json::json!(null))?;

    Ok(())
}
```

---

## 5. 与现有实现的对比

| 维度 | 现有 lsp_tool | lsp-types 方案 |
|:--|:--|:--|
| **类型安全** | `serde_json::Value` 到处传 | `lsp_types::*` 编译期检查 |
| **IDE 支持** | 无自动补全/类型提示 | 完整 rust-analyzer 支持 |
| **参数构造** | 手写 JSON，易拼错字段名 | 结构体，字段名编译器校验 |
| **响应解析** | `value["result"]["uri"]` 易 panic | `resp.uri` 类型安全 |
| **协议更新** | 手动跟踪 LSP 规范变更 | lsp-types 跟随规范更新 |
| **代码量** | ~500 行 client.rs | ~200 行 client.rs + ~50 行/命令 |
| **依赖** | 0 额外 crate | + lsp-types |
| **学习成本** | 低 (JSON 人人会) | 中 (需了解 lsp-types 类型体系) |
| **编译时间** | 快 | 略慢 (lsp-types 类型多) |
| **jdtls 私有协议** | 手写 JSON | 需自定义类型 (不在 lsp-types 中) |

### jdtls 私有协议处理

`java/buildWorkspace` 不在标准 LSP 中，需自定义:

```rust
// types.rs — 扩展类型
#[derive(Serialize)]
pub struct JavaBuildWorkspaceParams {
    pub force_rebuild: bool,
}

// 使用
let params = JavaBuildWorkspaceParams { force_rebuild: true };
let status: u32 = client.request("java/buildWorkspace", &params)?;
```

---

## 6. 迁移路线图

### Phase 1: 基础设施 (1天)

- [ ] 添加 `lsp-types` 依赖
- [ ] 实现 `transport.rs` (StdioTransport)
- [ ] 重写 `client.rs` (泛型 LspClient)
- [ ] 单元测试: mock transport 验证请求/响应

### Phase 2: 命令迁移 (2天)

- [ ] `symbols` — 第一个迁移，验证全流程
- [ ] `definition`
- [ ] `references`
- [ ] `hover`
- [ ] `completion`
- [ ] `build/rebuild/clean` (jdtls 私有协议)

### Phase 3: 清理 (0.5天)

- [ ] 删除旧 `jsonrpc.rs` 中的手写类型
- [ ] 统一错误类型
- [ ] 更新 SKILL.md 文档

### 不迁移的部分

- `servers.rs` — 服务器定义不变
- `config.rs` — 配置格式不变
- `output.rs` — 输出格式化不变
- `db/`, `excel/`, `fetch.rs` — 无关模块不变

---

## 附录: 关键参考

| 资源 | 链接 |
|:--|:--|
| lsp-types 文档 | https://docs.rs/lsp-types/latest/lsp_types/ |
| LSP 规范 | https://microsoft.github.io/language-server-protocol/specification |
| 本项目 java-language-server 分析 | `docs/java-language-server-guide.md` |
