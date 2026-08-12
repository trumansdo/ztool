//! 文档符号

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn document_symbols(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
) -> Result<DocumentSymbolResponse> {
    let uri = client.open_file(file)?;
    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier {
            uri: uri.parse()?,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("textDocument/documentSymbol", &params)
}
