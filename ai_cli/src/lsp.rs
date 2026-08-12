//! LSP 客户端模块 —— 类型安全的 Language Server Protocol CLI 工具
//!
//! # 架构
//! ```text
//! lsp/
//!   ├── mod.rs        → 模块声明 + 公共 re-export
//!   ├── client.rs     → LspClient<T: LspTransport> 泛型客户端
//!   ├── transport.rs  → LspTransport trait + StdioTransport 实现
//!   ├── types.rs      → lsp_types 扩展 (jdtls BuildStatus 等)
//!   ├── config.rs     → LspConfig 配置类型
//!   ├── servers.rs    → ServerDef + resolve_server + 内置服务器列表
//!   ├── output.rs     → 输出格式化 (format_symbols, format_hover 等)
//!   └── commands/     → 各 LSP 命令实现
//!       ├── mod.rs
//!       ├── init.rs       → 初始化 + classpath 自动生成
//!       ├── symbols.rs    → 文档符号
//!       ├── hover.rs      → 悬停文档
//!       ├── definition.rs → 跳转定义
//!       ├── references.rs → 查找引用
//!       ├── completion.rs → 代码补全
//!       ├── build.rs      → 构建 (jdtls)
//!       └── clean.rs      → 清理缓存 (jdtls)
//! ```
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

pub mod client;
pub mod transport;
pub mod types;
pub mod config;
pub mod servers;
pub mod output;
pub mod commands;

// 重新导出常用类型
pub use client::LspClient;
pub use transport::{LspTransport, StdioTransport};
