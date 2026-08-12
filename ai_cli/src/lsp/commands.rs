//! LSP 命令实现 —— 每个 LSP 功能一个文件
//!
//! 所有命令函数签名统一: `fn cmd(client: &mut LspClient<impl LspTransport>, ...) -> Result<T>`

pub mod build;
pub mod clean;
pub mod code_action;
pub mod completion;
pub mod definition;
pub mod folding;
pub mod formatting;
pub mod hover;
pub mod init;
pub mod references;
pub mod rename;
pub mod signature_help;
pub mod symbols;
pub mod workspace_symbols;
