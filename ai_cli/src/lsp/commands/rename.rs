//! 重命名符号

use crate::lsp::client::{lsp_position, LspClient};
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

/// 检查位置是否可重命名 (返回可重命名范围)
pub fn prepare_rename(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
) -> Result<Option<PrepareRenameResponse>> {
    let uri = client.open_file(file)?;
    let params = TextDocumentPositionParams {
        text_document: TextDocumentIdentifier {
            uri: uri.parse()?,
        },
        position: lsp_position(line, character),
    };
    client.request("textDocument/prepareRename", &params)
}

/// 执行重命名
pub fn rename(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
    new_name: &str,
) -> Result<Option<WorkspaceEdit>> {
    let uri = client.open_file(file)?;
    let params = RenameParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse()?,
            },
            position: lsp_position(line, character),
        },
        new_name: new_name.to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    client.request("textDocument/rename", &params)
}
