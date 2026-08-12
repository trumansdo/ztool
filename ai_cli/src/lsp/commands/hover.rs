//! 悬停文档
//!
//! # 已知限制
//!
//! javac-lsp 的 FindHoverElement 只处理 IdentifierTree / MemberSelectTree /
//! MemberReferenceTree，不处理 VariableTree（字段/变量声明）。
//! 因此对字段声明位置（如 `String foo = "bar"` 中的 `foo`）hover 返回空。
//! 方法调用和类型引用的 hover 正常。
//!
//! jdtls 无此限制。

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn hover(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
) -> Result<Option<Hover>> {
    let uri = client.open_file(file)?;
    let params = HoverParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse()?,
            },
            position: Position { line, character },
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    client.request("textDocument/hover", &params)
}
