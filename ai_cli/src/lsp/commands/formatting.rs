//! 文档格式化 (自动修复 imports / 添加 @Override 等)

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;
use std::path::Path;

pub fn formatting(
    client: &mut LspClient<impl LspTransport>,
    file: &Path,
) -> Result<Option<Vec<TextEdit>>> {
    let uri = client.open_file(file)?;
    let params = DocumentFormattingParams {
        text_document: TextDocumentIdentifier {
            uri: uri.parse()?,
        },
        options: FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    client.request("textDocument/formatting", &params)
}
