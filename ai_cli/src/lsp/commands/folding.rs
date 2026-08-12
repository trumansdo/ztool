//! 代码折叠范围

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn folding_range(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
) -> Result<Option<Vec<FoldingRange>>> {
    let uri = client.open_file(file)?;
    let params = FoldingRangeParams {
        text_document: TextDocumentIdentifier {
            uri: uri.parse()?,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("textDocument/foldingRange", &params)
}
