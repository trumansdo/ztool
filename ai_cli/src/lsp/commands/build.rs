//! 构建工作空间 (jdtls 私有协议 java/buildWorkspace)

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use crate::lsp::types::BuildStatus;
use anyhow::Result;

/// 增量构建 (force_rebuild = false) 或全量重建 (force_rebuild = true)
pub fn build_workspace(
    client: &mut LspClient<impl LspTransport>,
    force_rebuild: bool,
) -> Result<BuildStatus> {
    // jdtls 要求 [bool] 数组格式
    let params = vec![force_rebuild];
    let status: u64 = client.request("java/buildWorkspace", &params)?;
    Ok(BuildStatus::from_ordinal(status))
}
