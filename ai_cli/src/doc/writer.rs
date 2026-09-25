//! Doc 写入逻辑：docx-rs

use anyhow::{anyhow, Result};
use docx_rs::*;
use std::fs::File;
use std::path::Path;

/// 段落数据
pub struct ParaData {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub size: Option<usize>, // 字号
}

/// 表格数据
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// 写入新 docx 文件（段落列表）
pub fn write_docx(path: &Path, paragraphs: &[ParaData]) -> Result<()> {
    let file = File::create(path)
        .map_err(|e| anyhow!("创建文件失败 '{}': {e}", path.display()))?;

    let mut doc = Docx::new();
    for p in paragraphs {
        let mut run = Run::new().add_text(&p.text);
        if p.bold { run = run.bold(); }
        if p.italic { run = run.italic(); }
        if let Some(sz) = p.size { run = run.size(sz); }
        doc = doc.add_paragraph(Paragraph::new().add_run(run));
    }

    doc.build().pack(file)
        .map_err(|e| anyhow!("写入 docx 失败: {e}"))?;
    Ok(())
}

/// 写入带表格的 docx 文件
pub fn write_docx_with_table(path: &Path, title: Option<&str>, table: &TableData) -> Result<()> {
    let file = File::create(path)
        .map_err(|e| anyhow!("创建文件失败 '{}': {e}", path.display()))?;

    let mut doc = Docx::new();

    // 标题
    if let Some(t) = title {
        doc = doc.add_paragraph(
            Paragraph::new()
                .add_run(Run::new().add_text(t).bold().size(28))
        );
    }

    // 表格
    let mut tbl_rows = vec![];
    // 表头
    let header_cells: Vec<TableCell> = table.headers.iter()
        .map(|h| TableCell::new().add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(h).bold())
        ))
        .collect();
    tbl_rows.push(TableRow::new(header_cells));

    // 数据行
    for row in &table.rows {
        let cells: Vec<TableCell> = row.iter()
            .map(|v| TableCell::new().add_paragraph(
                Paragraph::new().add_run(Run::new().add_text(v))
            ))
            .collect();
        tbl_rows.push(TableRow::new(cells));
    }

    doc = doc.add_table(Table::new(tbl_rows));
    doc.build().pack(file)
        .map_err(|e| anyhow!("写入 docx 失败: {e}"))?;
    Ok(())
}
