//! Excel 写入逻辑：rust_xlsxwriter / umya

use anyhow::{anyhow, Result};
use std::path::Path;

use super::engine::ExcelEngine;
use super::reader::ReadResult;

pub struct CellSet { pub col: u32, pub row: u32, pub value: String }

pub fn parse_cell_set(s: &str) -> Result<CellSet> {
    let eq = s.find('=').ok_or_else(|| anyhow!("缺少 '='：{s}，格式应为 A1=value"))?;
    let (col, row) = super::reader::parse_cell_ref(&s[..eq])?;
    Ok(CellSet { col, row, value: s[eq + 1..].to_string() })
}

pub fn write_excel(
    path: &Path, sheet_name: Option<&str>, engine: ExcelEngine,
    cells: &[CellSet], _source: Option<ReadResult>,
) -> Result<()> {
    match engine {
        ExcelEngine::Xlsxwriter => write_xlsxwriter(path, sheet_name, cells),
        ExcelEngine::Umya => write_umya(path, sheet_name, cells),
        ExcelEngine::Calamine => Err(anyhow!("calamine 引擎不支持写入")),
    }
}

fn write_xlsxwriter(path: &Path, sheet_name: Option<&str>, cells: &[CellSet]) -> Result<()> {
    let mut wb = rust_xlsxwriter::Workbook::new();
    let name = sheet_name.unwrap_or("Sheet1");
    let ws = wb.add_worksheet();
    ws.set_name(name)?;
    for c in cells { ws.write(c.row - 1, (c.col - 1) as u16, &c.value)?; }
    wb.save(path)?;
    Ok(())
}

fn write_umya(path: &Path, sheet_name: Option<&str>, cells: &[CellSet]) -> Result<()> {
    let mut book = if path.exists() {
        umya_spreadsheet::reader::xlsx::read(path)
            .map_err(|e| anyhow!("读取原文件失败: {e}"))?
    } else {
        umya_spreadsheet::new_file()
    };

    let name = sheet_name.unwrap_or("Sheet1");
    if book.get_sheet_by_name(name).is_none() {
        book.new_sheet(name).map_err(|e| anyhow!("创建工作表 '{name}' 失败: {e}"))?;
        book.set_active_sheet(0);
    }

    let ws = book.get_sheet_by_name_mut(name)
        .ok_or_else(|| anyhow!("工作表 '{name}' 不存在"))?;

    for c in cells {
        let cell = ws.get_cell_mut((c.col, c.row));
        cell.get_cell_value_mut().set_value_string(&c.value);
    }

    umya_spreadsheet::writer::xlsx::write(&book, path)
        .map_err(|e| anyhow!("写入文件失败: {e}"))?;
    Ok(())
}
