//! 工作区符号搜索

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use lsp_types::*;

pub fn workspace_symbols(
    client: &mut LspClient<impl LspTransport>,
    query: &str,
) -> Result<Option<Vec<SymbolInformation>>> {
    let params = WorkspaceSymbolParams {
        query: query.to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    client.request("workspace/symbol", &params)
}
