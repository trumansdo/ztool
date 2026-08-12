# Java Language Server 深度分析与 CLI 接入指南

> 项目: [georgewfraser/java-language-server](https://github.com/georgewfraser/java-language-server)  
> 版本: 0.2.49 | LSP v3.0 | Java 25  
> 分析日期: 2025-08-12

---

## 目录

1. [LSP 协议概述](#1-lsp-协议概述)
2. [项目架构总览](#2-项目架构总览)
3. [已实现的 LSP 功能清单](#3-已实现的-lsp-功能清单)
4. [通信协议详解](#4-通信协议详解)
5. [启动与配置](#5-启动与配置)
6. [VS Code 扩展集成分析](#6-vs-code-扩展集成分析)
7. [CLI 工具接入方案](#7-cli-工具接入方案)
8. [实战: Rust 实现 CLI 客户端](#8-实战-rust-实现-cli-客户端)

---

## 1. LSP 协议概述

Language Server Protocol 定义了编辑器(客户端)与语言服务器之间的标准化通信协议，基于 JSON-RPC 2.0。

### 核心交互模型

```
编辑器 (IDE)  ←→  JSON-RPC over stdio  ←→  Language Server
```

- 服务器作为独立进程运行，通过 stdin/stdout 通信
- 所有消息以 HTTP 风格 Header `Content-Length: <N>\r\n\r\n` 开头
- 三种消息类型: **Request**（请求-响应）、**Notification**（单向通知）、**Response**（响应）

### 生命周期

```
initialize → initialized → [正常工作] → shutdown → exit
```

### 关键数据类型

| 类型 | 说明 |
|:--|:--|
| `textDocument.uri` | `file://` 协议的文档 URI |
| `Position` | `{line: int, character: int}` 0-based |
| `Range` | `{start: Position, end: Position}` |
| `Location` | `{uri: string, range: Range}` |

---

## 2. 项目架构总览

### 技术栈

| 层 | 技术 |
|:--|:--|
| 语言分析 | Java Compiler API (`com.sun.tools.javac`) |
| JSON 序列化 | Gson 2.8.9 |
| 协议序列化 | Protobuf 3 (Bazel 构建分析) |
| 构建工具 | Maven |
| 编辑器前端 | TypeScript / VS Code Extension |

### 目录结构

```
java-language-server/
├── src/main/java/org/javacs/
│   ├── Main.java                  # 入口: LSP.connect(JavaLanguageServer::new, System.in, System.out)
│   ├── JavaLanguageServer.java    # 核心: LSP 请求路由 + 功能调度
│   ├── JavaCompilerService.java   # 编译器封装: 增量编译、类路径管理
│   ├── CompilerProvider.java      # 编译器接口定义
│   ├── ReusableCompiler.java      # 可复用 javac 实例
│   ├── InferConfig.java           # 类路径/源码路径自动推断
│   ├── FileStore.java             # 虚拟文件系统 (内存文档管理)
│   ├── Parser.java                # Java 文件解析
│   │
│   ├── lsp/                       # LSP 协议类型定义 (纯手写, 无三方库)
│   │   ├── LSP.java               # JSON-RPC 消息读写引擎
│   │   ├── LanguageServer.java    # 服务端抽象接口 (26个方法)
│   │   ├── LanguageClient.java    # 客户端接口 (通知/注册)
│   │   └── *.java                 # 80+ 协议数据结构
│   │
│   ├── completion/                # 代码补全实现
│   ├── navigation/                # 跳转定义、查找引用
│   ├── hover/                     # 悬停文档
│   ├── index/                     # 符号索引与搜索
│   ├── markup/                    # 语法错误/语义高亮
│   ├── rewrite/                   # 重构操作 (重命名/提取/内联)
│   ├── action/                    # Code Action 快速修复
│   ├── lens/                      # Code Lens 实现
│   ├── fold/                      # 代码折叠
│   └── debug/                     # DAP 调试适配协议
│
├── dist/                          # 启动脚本
│   ├── lang_server_windows.cmd    # Windows 启动
│   ├── launch_windows.cmd         # JVM 参数 + classpath
│   └── classpath/                 # 依赖 jar
│
├── pom.xml                        # Maven: artifact=javac-services
├── package.json                   # VS Code Extension
└── README.md
```

### 核心类关系

```
Main.main()
  └─ LSP.connect(serverFactory, stdin, stdout)       ← JSON-RPC 引擎
       ├─ MessageReader (daemon thread)               ← 从 stdin 读消息
       │    ├─ nextToken() → parseHeader → readLength
       │    └─ parseMessage() → Gson.fromJson
       └─ Main Thread (消息处理循环)
            └─ switch(r.method):
                 ├─ "initialize"        → server.initialize()
                 ├─ "textDocument/definition" → server.gotoDefinition()
                 ├─ "textDocument/completion" → server.completion()
                 └─ ... (共 30+ 方法)
```

---

## 3. 已实现的 LSP 功能清单

### 协议方法映射表

| LSP 方法 | 实现类 | 功能 |
|:--|:--|:--|
| `initialize` | `JavaLanguageServer` | 握手，返回 capabilities |
| `initialized` | `JavaLanguageServer` | 注册文件监听 |
| `shutdown` / `exit` | `JavaLanguageServer` | 关闭服务 |
| **文档同步** | | |
| `textDocument/didOpen` | `JavaLanguageServer` | 文件打开 |
| `textDocument/didChange` | `JavaLanguageServer` | 增量同步 |
| `textDocument/didClose` | `JavaLanguageServer` | 文件关闭 |
| `textDocument/didSave` | `JavaLanguageServer` | 保存时 lint |
| **导航** | | |
| `textDocument/definition` | `DefinitionProvider` | Go to Definition |
| `textDocument/references` | `ReferenceProvider` | Find References |
| `textDocument/documentSymbol` | `SymbolProvider` | Document Symbols |
| `workspace/symbol` | `SymbolProvider` | Workspace Symbol Search |
| **补全与文档** | | |
| `textDocument/completion` | `CompletionProvider` | Code Completion |
| `completionItem/resolve` | `HoverProvider` | 补全详情 |
| `textDocument/signatureHelp` | `SignatureProvider` | 参数提示 |
| `textDocument/hover` | `HoverProvider` | Hover 文档 |
| **诊断** | | |
| `textDocument/publishDiagnostics` | `ErrorProvider` | 编译错误/警告 |
| `java/colors` | `ColorProvider` | 语义高亮 |
| **代码操作** | | |
| `textDocument/codeAction` | `CodeActionProvider` | Quick Fix |
| `textDocument/codeLens` | `CodeLensProvider` | Code Lens |
| `textDocument/formatting` | `AutoFixImports` | Format / 自动导包 |
| **重命名** | | |
| `textDocument/prepareRename` | `JavaLanguageServer` | 重命名预检 |
| `textDocument/rename` | `RenameMethod` 等 | 重命名重构 |
| **其他** | | |
| `textDocument/foldingRange` | `FoldProvider` | 代码折叠 |
| `textDocument/documentLink` | — | 文档链接 |
| `workspace/didChangeWatchedFiles` | `JavaLanguageServer` | 文件系统变更 |

### 已声明的 Capabilities (initialize 响应)

```json
{
  "capabilities": {
    "textDocumentSync": 2,           // 增量同步
    "hoverProvider": true,
    "completionProvider": {
      "resolveProvider": true,
      "triggerCharacters": ["."]
    },
    "signatureHelpProvider": {
      "triggerCharacters": ["(", ","]
    },
    "referencesProvider": true,
    "definitionProvider": true,
    "workspaceSymbolProvider": true,
    "documentSymbolProvider": true,
    "documentFormattingProvider": true,
    "codeLensProvider": {},
    "foldingRangeProvider": true,
    "codeActionProvider": true,
    "renameProvider": {
      "prepareProvider": true
    }
  }
}
```

### 支持的重构操作

| 操作 | 实现类 | 触发方式 |
|:--|:--|:--|
| 重命名方法 | `RenameMethod` | `textDocument/rename` |
| 重命名字段 | `RenameField` | `textDocument/rename` |
| 重命名变量/参数 | `RenameVariable` | `textDocument/rename` |
| 添加 import | `AddImport` | Code Action |
| 实现抽象方法 | `ImplementAbstractMethods` | Code Action |
| 创建缺失方法 | `CreateMissingMethod` | Quick Fix |
| 创建缺失字段 | `CreateMissingField` | Quick Fix |
| 提取变量 | `ExtractVariable` | 选中 → 重构 |
| 提取方法 | `ExtractMethod` | 选中 → 重构 |
| 提取常量 | `ExtractConstant` | 选中 → 重构 |
| 内联变量/方法/字段 | `InlineVariable` 等 | 选中 → 重构 |
| 捕获异常 | `CatchException` | Quick Fix |
| 添加异常声明 | `AddException` | Quick Fix |
| 自动添加 @Override | `AutoAddOverrides` | Format |
| 自动修复 imports | `AutoFixImports` | Format |

---

## 4. 通信协议详解

### 4.1 消息帧格式

```
Content-Length: <消息体字节数>\r\n
\r\n
<JSON 消息体>
```

**关键点**: 使用**字节数**而非字符数 (`messageBytes.length`)，UTF-8 编码。

### 4.2 消息类型

#### Request (有 id, 有 method)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "textDocument/definition",
  "params": {
    "textDocument": {"uri": "file:///path/to/File.java"},
    "position": {"line": 10, "character": 5}
  }
}
```

#### Response (有 id, 有 result)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "uri": "file:///path/to/File.java",
    "range": {
      "start": {"line": 5, "character": 4},
      "end": {"line": 5, "character": 11}
    }
  }
}
```

#### Error Response
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

#### Notification (无 id, 有 method)
```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didOpen",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/File.java",
      "languageId": "java",
      "version": 1,
      "text": "package com.example;\n\npublic class Main {\n}\n"
    }
  }
}
```

### 4.3 初始化握手序列

```
Client                               Server
  │                                    │
  ├─ initialize ──────────────────────→│
  │  {processId, rootUri, capabilities}│
  │                                    │
  │←───────────── InitializeResult ───┤
  │              {capabilities}        │
  │                                    │
  ├─ initialized ─────────────────────→│  (notification, 无 id)
  │                                    │
  │←── registerCapability ────────────┤  (文件监听注册)
  │                                    │
  │         [正常工作阶段]              │
```

### 4.4 LSP.java 消息解析源码要点

```java
// 消息读取流程
static String nextToken(InputStream client) {
    // 1. 逐字符读 Header 行 (直到空行)
    //    "Content-Length: 123\r\n"
    //    "\r\n"  ← 空行标志 Header 结束
    // 2. 读取 body (Content-Length 指定字节数)
    // 3. 返回 JSON 字符串
}

// 消息处理主循环 (LSP.java:connect)
// - daemon 线程: 读 stdin → parseMessage → 放入 pending 队列
// - main 线程: poll 队列 → switch(method) → 调用 server 方法 → writeClient
```

### 4.5 取消请求

```json
{"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id": 1}}
```

在消息入队列前处理——直接在队列中移除对应 id 的待处理请求。

---

## 5. 启动与配置

### 5.1 启动命令

```bash
# Windows
dist/lang_server_windows.cmd [--quiet]

# 等价于
java \
  --add-exports jdk.compiler/com.sun.tools.javac.api=ALL-UNNAMED \
  --add-exports jdk.compiler/com.sun.tools.javac.code=ALL-UNNAMED \
  --add-exports jdk.compiler/com.sun.tools.javac.comp=ALL-UNNAMED \
  --add-exports jdk.compiler/com.sun.tools.javac.main=ALL-UNNAMED \
  --add-exports jdk.compiler/com.sun.tools.javac.tree=ALL-UNNAMED \
  --add-exports jdk.compiler/com.sun.tools.javac.model=ALL-UNNAMED \
  --add-exports jdk.compiler/com.sun.tools.javac.util=ALL-UNNAMED \
  --add-opens jdk.compiler/com.sun.tools.javac.api=ALL-UNNAMED \
  -classpath gson-2.8.9.jar;protobuf-java-3.19.6.jar;java-language-server.jar \
  org.javacs.Main
```

### 5.2 环境要求

| 依赖 | 版本 |
|:--|:--|
| JDK | 25+ (需要 JavacTask API 新特性) |
| Maven | 用于构建 |
| Node.js + npm | 用于 VS Code 扩展 |

### 5.3 关键配置项 (通过 `workspace/didChangeConfiguration` 传递)

```json
{
  "java": {
    "classPath": ["lib/dep.jar"],              // 显式类路径
    "docPath": ["lib/dep-sources.jar"],         // 源码/javadoc 路径
    "externalDependencies": [                   // Maven/Gradle 坐标
      "junit:junit:4.12"
    ],
    "extraCompilerArgs": ["--enable-preview"],  // 额外 javac 参数
    "addExports": [
      "jdk.compiler/com.sun.tools.javac.api"    // 模块访问权限
    ],
    "testMethod": ["mvn", "test", "-Dtest=${class}#${method}"],
    "home": "/path/to/jdk"                      // JDK 路径
  }
}
```

---

## 6. VS Code 扩展集成分析

> 扩展: [georgewfraser/vscode-javac](https://marketplace.visualstudio.com/items?itemName=georgewfraser.vscode-javac)  
> 版本: 0.2.49 | 与 java-language-server 同一作者

### 6.1 扩展架构

```
VS Code Extension (TypeScript)
├── extension.js              ← 入口: activate()
├── vscode-languageclient     ← LSP 客户端库 (npm)
├── textMate.js               ← 语义高亮主题映射
└── 子进程: lang_server_*.sh  ← Java Language Server
```

**核心依赖**: `vscode-languageclient` (npm) — 微软官方 LSP 客户端 SDK，封装了 JSON-RPC 通信、文档同步、配置传递等。

### 6.2 服务器生命周期管理

```typescript
// 1. 定位启动脚本
let launcher = Path.resolve(context.extensionPath, 'dist', 'lang_server_windows.sh');

// 2. 配置服务器选项
let serverOptions = {
    command: launcher,
    args: [],
    options: { cwd: context.extensionPath }
};

// 3. 创建 LanguageClient
let client = new LanguageClient('java', 'Java Language Server', serverOptions, clientOptions);

// 4. 启动
await client.start();
```

**关键点**: 扩展不直接管理 Java 进程——`vscode-languageclient` 负责 spawn、stdin/stdout 管道、消息帧、错误处理。

### 6.3 自定义通知协议

该 LSP 实现了 4 个超出标准 LSP 的自定义通知:

| 通知方法 | 方向 | 用途 | 数据结构 |
|:--|:--|:--|:--|
| `java/colors` | Server→Client | 语义高亮信息 | `{uri, fields[], statics[]}` |
| `java/startProgress` | Server→Client | 开始进度条 | `{message: string}` |
| `java/reportProgress` | Server→Client | 更新进度条 | `{message, increment}` |
| `java/endProgress` | Server→Client | 结束进度条 | `null` |

**语义高亮流程**:
```
Server 编译完成
  → 发送 java/colors 通知
  → Extension 接收 → 按 TextMate scope 映射颜色
  → 应用到编辑器 (setDecorations)
```

**进度条流程**:
```
Server 开始配置推断
  → java/startProgress("Configure javac")
  → java/reportProgress("Finding source roots")
  → java/reportProgress("Inferring class path")
  → java/endProgress()
```

### 6.4 测试集成机制

扩展通过 VS Code Task 系统运行测试，而非 LSP 协议:

```typescript
// 配置模板 (settings.json)
"java.testMethod": ["mvn", "test", "-Dtest=${class}#${method}"]
"java.debugTestMethod": ["mvn", "test", "-Dmaven.surefire.debug", "-Dtest=${class}#${method}"]

// 运行时替换变量
command.replace('${file}', file)
       .replace('${class}', className)
       .replace('${method}', methodName)
```

**调试流程**:
1. 以 debug 模式启动 `mvn test` (监听 5005 端口)
2. VS Code 通过 DAP (Debug Adapter Protocol) attach 到 5005
3. 使用扩展内置的 `debug_adapter_*.sh` 作为 DAP 服务端

### 6.5 JAR 文件系统提供者

扩展注册了自定义 `jar:` URI scheme，允许直接在编辑器中打开 JAR 内的源码:

```typescript
class JarFileSystemProvider implements TextDocumentContentProvider {
    // jar:file:///path/to/dep.jar!/com/foo/Thing.java
    //                              ↓
    // 解压 JAR → 读取 Thing.java → 返回内容
}
```

用于 Go to Definition 跳转到依赖库源码时。

### 6.6 配置传递路径

```
VS Code settings.json
  → workspace.getConfiguration('java')
  → LanguageClient (synchronize.configurationSection = 'java')
  → workspace/didChangeConfiguration 通知
  → JavaLanguageServer.didChangeConfiguration()
  → settings 字段更新 → 重建 CompilerProvider
```

### 6.7 对 CLI 工具设计的启示

| VS Code 扩展做法 | CLI 工具对应策略 |
|:--|:--|
| `vscode-languageclient` 管理进程 | 手动 `spawn` + stdin/stdout 管道 |
| 自动文档同步 | 手动发送 `didOpen`/`didChange` |
| `java/colors` 语义高亮 | 可选: 解析后用于终端着色 |
| `java/*progress` 进度通知 | 可选: 显示 spinner/进度条 |
| Task 系统运行测试 | 直接 `Command::new("mvn")` 更简单 |
| JAR 文件系统 | 非必须 (CLI 不需要在编辑器中打开 JAR) |
| `configurationSection` 自动同步 | 手动构造 `didChangeConfiguration` 参数 |

---

## 7. CLI 工具接入方案

### 6.1 基本原则

**language server 就是黑盒进程**——你不需要理解 Java 编译器内部，只需:

1. 启动 `java org.javacs.Main` 子进程
2. 通过 stdin/stdout 发送/接收 JSON-RPC 消息
3. 按 LSP 协议发送请求，解析响应

### 6.2 最小 CLI 客户端工作流

```
1. spawn("java org.javacs.Main")      → 获取 stdin/stdout 句柄
2. 发送 initialize 请求               → 等待 InitializeResult
3. 发送 initialized 通知              → 服务准备就绪
4. 发送 textDocument/didOpen           → 打开目标文件
5. 发送功能请求 (definition/hover...)  → 获取结果
6. 发送 shutdown + exit               → 关闭
```

### 6.3 消息帧封装伪代码

```
发送:
  json = serialize(request)
  header = "Content-Length: ${len(json_bytes)}\r\n\r\n"
  stdin.write(header + json_bytes)

接收:
  header = read_until(stdout, "\r\n\r\n")
  content_length = parse_header(header)
  json_bytes = stdout.read_exact(content_length)
  response = deserialize(json_bytes)
```

### 6.4 需要处理的协议细节

| 问题 | 方案 |
|:--|:--|
| 异步通知 | 启动后可能收到 `publishDiagnostics`, `showMessage` 等，需忽略或捕获 |
| 请求 ID | 自增整数，用于匹配响应 |
| 取消请求 | 实现 `$/cancelRequest` 或忽略(服务器会自动处理死请求) |
| 编码 | 全部 UTF-8 |
| 阻塞 I/O | stdin/stdout 是阻塞的，建议单线程同步模式 |

---

## 8. 实战: Rust 实现 CLI 客户端

### 8.1 核心思路

本项目的 `ai_cli` 已实现 LSP 客户端框架 (`lsp/client.rs` + `lsp/jsonrpc.rs`)，可直接复用其 JSON-RPC 引擎，新增一个 **非 Eclipse jdtls 的轻量级 Java LSP** 适配器。

与 jdtls 的关键区别:

| 特性 | Eclipse jdtls | java-language-server |
|:--|:--|:--|
| 实现语言 | Java (Eclipse JDT) | Java (Compiler API) |
| 依赖 | Eclipse 全套 (~100个jar) | javac + gson + protobuf (~4个jar) |
| 启动速度 | 慢 (~5-10s) | 快 (~1-2s) |
| Maven 支持 | 完整 | 基础 (外部依赖解析) |
| 项目模型 | Eclipse workspace | 纯 javac |
| 启动命令 | `jdtls` wrapper | `java org.javacs.Main` |

### 8.2 推荐的 CLI 命令设计

参考现有 `lsp_tool` 架构，新增 `--server javac` 选项:

```bash
# 符号查询
lsp_tool --server javac -p /path/to/project symbols File.java

# 跳转定义
lsp_tool --server javac -p /path/to/project def File.java 10 5

# 查找引用
lsp_tool --server javac -p /path/to/project refs File.java 10 5

# 代码补全
lsp_tool --server javac -p /path/to/project comp File.java 10 5

# 文档悬停
lsp_tool --server javac -p /path/to/project hover File.java 10 5

# 诊断
lsp_tool --server javac -p /path/to/project lint File.java
```

### 8.3 实现步骤

1. **配置定义** (`lsp/config.rs`):
   ```rust
   pub struct JavacSection {
       pub java_home: Option<String>,
       pub classpath: Option<Vec<String>>,
   }
   ```

2. **服务器定义** (`lsp/servers.rs`):
   ```rust
   ServerDef {
       id: "javac".into(),
       command: vec![
           "java".into(),
           "--add-exports".into(), "jdk.compiler/...=ALL-UNNAMED".into(),
           // ... 7 个 --add-exports
           "-classpath".into(), "gson.jar;protobuf.jar;jls.jar".into(),
           "org.javacs.Main".into(),
       ],
       extensions: vec![".java".into()],
       language_id: "java".into(),
       install_hint: "mvn package -DskipTests in java-language-server directory".into(),
   }
   ```

3. **发送 initialize + initialized**:
   ```rust
   let init_params = serde_json::json!({
       "processId": std::process::id(),
       "rootUri": format!("file:///{}", project_path),
       "capabilities": {}
   });
   let result = client.request("initialize", &init_params);
   client.notify("initialized", &serde_json::json!({}));
   ```

4. **打开文件**:
   ```rust
   client.notify("textDocument/didOpen", &serde_json::json!({
       "textDocument": {
           "uri": format!("file:///{}", file_path),
           "languageId": "java",
           "version": 1,
           "text": file_content
       }
   }));
   ```

5. **发送查询请求**:
   ```rust
   let params = serde_json::json!({
       "textDocument": {"uri": format!("file:///{}", file_path)},
       "position": {"line": line, "character": character}
   });
   let response = client.request("textDocument/definition", &params);
   ```

### 8.4 与现有 jdtls 的对比

| 维度 | jdtls (已实现) | javac-ls (建议新增) |
|:--|:--|:--|
| jar 数量 | ~90+ | ~4 |
| 构建命令 | `buildWorkspace` (私有) | 标准 LSP 方法 |
| 状态检查 | ordinal 0-3 | — (无此功能) |
| 缓存清理 | SHA1 项目目录 | — (无 workspace 缓存) |
| Maven 集成 | 深度集成 | 仅依赖解析 |
| 适用场景 | 大型企业项目 | 小项目/脚本/教学 |

### 8.5 注意事项

- **无 `buildWorkspace`**: 该轻量服务器不支持 `java/buildWorkspace`，构建结果通过 `publishDiagnostics` 返回
- **类路径配置**: 必须通过 `workspace/didChangeConfiguration` 发送 `java.classPath`
- **无 Gradle/Bazel 深度集成**: `InferConfig` 会尝试自动推断，但不如 jdtls 成熟
- **JDK 源码**: `src.zip` 需手动配置在 `docPath` 中才有 Javadoc

---

## 附录: 有用的源文件索引

| 文件 | 用途 |
|:--|:--|
| `LSP.java` | JSON-RPC 引擎(消息帧+路由)，核心引用 |
| `LanguageServer.java` | 服务端抽象类，所有方法签名 |
| `JavaLanguageServer.java` | 实现类，30+ 方法分发 |
| `JavaCompilerService.java` | 编译器封装，增量编译核心 |
| `CompilerProvider.java` | 编译服务接口 |
| `FileStore.java` | 虚拟文件系统，在内存维护文档状态 |
| `InferConfig.java` | 智能推断类路径 |
| `PublishDiagnosticsParams.java` | 诊断数据结构 |
| `launch_windows.cmd` | JVM 启动参数模板 |
