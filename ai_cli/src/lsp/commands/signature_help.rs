//! 签名帮助 (方法参数提示)

use crate::lsp::client::{lsp_position, LspClient};
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn signature_help(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
    line: u32,
    character: u32,
) -> Result<Option<SignatureHelp>> {
    let uri = client.open_file(file)?;
    let params = SignatureHelpParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse()?,
            },
            position: lsp_position(line, character),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        context: None,
    };
    client.request("textDocument/signatureHelp", &params)
}
