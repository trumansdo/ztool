//! 代码补全
//!
//! # 已知限制
//!
//! javac-lsp 的补全依赖 compiler.compile() 能成功解析当前文件。
//! 如果 classpath 不完整导致编译失败，补全返回空。
//! 字段声明位置的补全通常无意义（声明语法已确定），返回空属正常行为。
//!
//! jdtls 无此限制。

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn completion(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
) -> Result<Option<CompletionResponse>> {
    let uri = client.open_file(file)?;
    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse()?,
            },
            position: Position { line, character },
        },
        context: None,
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("textDocument/completion", &params)
}
