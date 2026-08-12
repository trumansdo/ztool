//! 清理 jdtls 工作空间缓存

use anyhow::Result;
use sha1::Digest;
use std::path::{Path, PathBuf};

/// 清理指定项目的工作空间缓存
/// data_dir 优先级: cli_data_dir > config [lsp.jdtls].data_dir > config [lsp].data_dir > $TMP/lsp_tool
pub fn clean_workspace(
    project_dir: &Path,
    cli_data_dir: Option<&str>,
    config_data_dir: Option<&str>,
    jdtls_data_dir: Option<&str>,
) -> Result<String> {
    let base = cli_data_dir
        .map(PathBuf::from)
        .or_else(|| jdtls_data_dir.map(PathBuf::from))
        .or_else(|| config_data_dir.map(PathBuf::from))
        .unwrap_or_else(|| std::env::temp_dir().join("lsp_tool"));

    let norm = project_dir.to_string_lossy().replace('\\', "/");
    let hash = format!("{:x}", sha1::Sha1::digest(norm.as_bytes()));
    let ws_dir = base.join(&hash);

    if ws_dir.exists() {
        std::fs::remove_dir_all(&ws_dir)?;
        Ok(format!("✓ 工作空间已清理: {}", ws_dir.display()))
    } else {
        Ok(format!("工作空间不存在: {}", ws_dir.display()))
    }
}
