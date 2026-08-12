//! Code Action (快速修复: 导包、实现方法、异常处理等)

use crate::lsp::client::{lsp_position, LspClient};
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn code_action(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
) -> Result<Option<Vec<CodeActionOrCommand>>> {
    let uri = client.open_file(file)?;
    let params = CodeActionParams {
        text_document: TextDocumentIdentifier {
            uri: uri.parse()?,
        },
        range: Range {
            start: lsp_position(line, character),
            end: lsp_position(line, character),
        },
        context: CodeActionContext {
            diagnostics: vec![],
            only: None,
            trigger_kind: None,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("textDocument/codeAction", &params)
}
