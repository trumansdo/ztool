//! 跳转定义
//!
//! # 已知限制
//!
//! javac-lsp 使用 FindNameAt (含 visitVariable) 查找元素，对字段/方法/类型均支持。
//! 但需要 classpath 完整才能解析跨文件定义。
//! jdtls 无此限制。

use crate::lsp::client::{lsp_position, LspClient};
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn goto_definition(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
) -> Result<Option<GotoDefinitionResponse>> {
    let uri = client.open_file(file)?;
    let params = GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse()?,
            },
            position: lsp_position(line, character),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("textDocument/definition", &params)
}
