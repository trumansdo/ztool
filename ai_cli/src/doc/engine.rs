//! Doc 引擎枚举与选择逻辑

/// Doc 操作引擎
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DocEngine {
    /// office_oxide：纯 Rust，读 DOCX+DOC，写 DOCX，极快
    OfficeOxide,
    /// docx-rs：纯 Rust，写 DOCX，生态最成熟
    DocxRs,
}

/// 根据操作类型自动选择引擎
pub fn resolve_engine(explicit: Option<DocEngine>, is_write: bool) -> DocEngine {
    if let Some(engine) = explicit {
        return engine;
    }
    if is_write {
        DocEngine::DocxRs
    } else {
        DocEngine::OfficeOxide
    }
}
