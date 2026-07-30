//! Excel 引擎枚举与选择逻辑

use std::path::Path;

/// Excel 操作引擎
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ExcelEngine {
    /// calamine：纯 Rust 只读，最快
    Calamine,
    /// rust_xlsxwriter：纯 Rust 只写，功能最全
    Xlsxwriter,
    /// umya-spreadsheet：读写，保留样式
    Umya,
}

/// 根据操作类型自动选择引擎
pub fn resolve_engine(
    explicit: Option<ExcelEngine>,
    is_write: bool,
    file_exists: bool,
    keep_style: bool,
    will_override: bool,
) -> ExcelEngine {
    // 显式指定优先
    if let Some(engine) = explicit {
        return engine;
    }
    // --keep-style → umya
    if keep_style {
        return ExcelEngine::Umya;
    }
    // 写入且要覆盖原文件 → umya（保留样式）
    if is_write && file_exists && will_override {
        return ExcelEngine::Umya;
    }
    // 写入新文件 → xlsxwriter
    if is_write {
        return ExcelEngine::Xlsxwriter;
    }
    // 读取 → calamine
    ExcelEngine::Calamine
}

/// 自动序号生成文件名
/// demo.xlsx → demo (1).xlsx → demo (2).xlsx
pub fn auto_numbered_path(path: &Path) -> std::path::PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut n = 1u32;
    loop {
        let name = format!("{stem} ({n}).{ext}");
        let candidate = parent.join(&name);
        if !candidate.exists() {
            return candidate;
        }
        n += 1;
    }
}
