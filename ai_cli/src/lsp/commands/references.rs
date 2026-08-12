//! 查找引用
//!
//! # 已知限制
//!
//! javac-lsp 的引用搜索基于 FileStore.all()（所有已通过 didOpen 打开的文件），
//! 而非全项目索引。CLI 模式下通常只打开单个文件，因此跨文件引用可能找不到。
//! VS Code 插件靠用户日常打开文件逐步累积 FileStore 来缓解此问题。
//!
//! jdtls 构建全项目索引，无此限制。

use crate::lsp::client::{lsp_position, LspClient};
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn find_references(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
    include_declaration: bool,
) -> Result<Option<Vec<Location>>> {
    let uri = client.open_file(file)?;
    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse()?,
            },
            position: lsp_position(line, character),
        },
        context: ReferenceContext {
            include_declaration,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("textDocument/references", &params)
}
