//! Doc 读取逻辑：office_oxide

use anyhow::{anyhow, Result};
use office_oxide::Document;
use std::path::Path;

/// 读取结果
pub struct ReadResult {
    /// 纯文本内容
    pub text: String,
    /// Markdown 格式内容
    pub markdown: String,
    /// 文件格式（docx / doc）
    pub format: String,
}

/// 读取 doc/docx 文件，返回纯文本和 Markdown
pub fn read_doc(path: &Path) -> Result<ReadResult> {
    let doc = Document::open(path)
        .map_err(|e| anyhow!("打开文件失败 '{}': {e}", path.display()))?;

    let text = doc.plain_text();
    let markdown = doc.to_markdown();
    let format = detect_format(path);

    Ok(ReadResult { text, markdown, format })
}

/// 仅提取纯文本（轻量路径）
pub fn read_text(path: &Path) -> Result<String> {
    let doc = Document::open(path)
        .map_err(|e| anyhow!("打开文件失败 '{}': {e}", path.display()))?;
    Ok(doc.plain_text())
}

/// 仅提取 Markdown
pub fn read_markdown(path: &Path) -> Result<String> {
    let doc = Document::open(path)
        .map_err(|e| anyhow!("打开文件失败 '{}': {e}", path.display()))?;
    Ok(doc.to_markdown())
}

fn detect_format(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("doc") => "doc".into(),
        Some("docx") => "docx".into(),
        _ => "unknown".into(),
    }
}
