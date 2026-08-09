//! LSP 语言服务器客户端模块（可复用库）
//!
//! 提供：
//! - [`servers`] 语言服务器注册表 / 配置解析（复用 `config::Settings` 的 `[lsp]` 段）
//! - [`jsonrpc`] LSP 帧编解码（Content-Length 头）
//! - [`client`] `LspClient`：stdio 通信 + initialize/didOpen/查询方法
//! - [`output`] 结果格式化（符号树 / hover / 位置 / 补全）
//! - [`test_suite`] 完整 LSP 能力测试套件
//!
//! # 配置结构（~/.config/ai_cli/config.toml）
//! ```toml
//! [lsp.jdtls]
//! command = ["java", "-Declipse.application=org.eclipse.jdt.ls.core.id1", "..."]
//! maven_settings = "D:\\dev_program\\apache-maven-3.9.5\\conf\\settings.xml"
//! ```

pub mod client;
pub mod jsonrpc;
pub mod output;
pub mod servers;
pub mod test_suite;

pub use client::LspClient;
pub use servers::{resolve_server, ServerDef};
