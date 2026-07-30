//! Excel 读取逻辑：calamine / umya

use calamine::{open_workbook_auto, Reader, Data};
use anyhow::{anyhow, Result};
use std::path::Path;

use super::engine::ExcelEngine;

pub type Row = Vec<String>;

pub struct ReadResult {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub row_count: usize,
}

pub fn parse_cell_ref(s: &str) -> Result<(u32, u32)> {
    let mut col_str = String::new();
    let mut row_str = String::new();
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            if !row_str.is_empty() { return Err(anyhow!("非法单元格引用: {s}")); }
            col_str.push(c.to_ascii_uppercase());
        } else if c.is_ascii_digit() {
            row_str.push(c);
        } else { return Err(anyhow!("非法单元格引用: {s}")); }
    }
    if col_str.is_empty() || row_str.is_empty() { return Err(anyhow!("非法单元格引用: {s}")); }
    let mut col = 0u32;
    for c in col_str.chars() { col = col * 26 + (c as u32 - 'A' as u32 + 1); }
    let row: u32 = row_str.parse().map_err(|_| anyhow!("非法行号: {row_str}"))?;
    Ok((col, row))
}

pub fn parse_row_range(s: &str) -> Result<(Option<usize>, Option<usize>)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 { return Err(anyhow!("非法行范围: {s}，格式: 1-50")); }
    let start = if parts[0].is_empty() { None } else { Some(parts[0].parse().map_err(|_| anyhow!("非法行范围: {s}"))?) };
    let end = if parts[1].is_empty() { None } else { Some(parts[1].parse().map_err(|_| anyhow!("非法行范围: {s}"))?) };
    Ok((start, end))
}

pub fn parse_col_range(s: &str) -> Result<(Option<u32>, Option<u32>)> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 { return Err(anyhow!("非法列范围: {s}，格式: A-D")); }
    let start = if parts[0].is_empty() { None } else { Some(parse_col_letter(parts[0])?) };
    let end = if parts[1].is_empty() { None } else { Some(parse_col_letter(parts[1])?) };
    Ok((start, end))
}

fn parse_col_letter(s: &str) -> Result<u32> {
    let mut col = 0u32;
    for c in s.chars() {
        if !c.is_ascii_uppercase() { return Err(anyhow!("非法列名: {s}")); }
        col = col * 26 + (c as u32 - 'A' as u32 + 1);
    }
    if col == 0 { return Err(anyhow!("非法列名: {s}")); }
    Ok(col)
}

pub fn read_excel(
    path: &Path, sheet_name: Option<&str>, engine: ExcelEngine,
    row_range: Option<(Option<usize>, Option<usize>)>, col_range: Option<(Option<u32>, Option<u32>)>,
    cells: Option<&[(u32, u32)]>, default_rows: usize,
) -> Result<ReadResult> {
    match engine {
        ExcelEngine::Calamine => read_calamine(path, sheet_name, row_range, col_range, cells, default_rows),
        ExcelEngine::Umya => read_umya(path, sheet_name, row_range, col_range, cells, default_rows),
        ExcelEngine::Xlsxwriter => Err(anyhow!("xlsxwriter 引擎不支持读取")),
    }
}

fn read_calamine(
    path: &Path, sheet_name: Option<&str>,
    row_range: Option<(Option<usize>, Option<usize>)>, col_range: Option<(Option<u32>, Option<u32>)>,
    cells: Option<&[(u32, u32)]>, default_rows: usize,
) -> Result<ReadResult> {
    let mut workbook = open_workbook_auto(path)?;
    let sheet = if let Some(name) = sheet_name {
        name.to_string()
    } else {
        workbook.sheet_names().first().cloned().ok_or_else(|| anyhow!("文件中没有工作表"))?
    };
    let range = workbook.worksheet_range(&sheet).map_err(|e| anyhow!("读取工作表 '{sheet}' 失败: {e}"))?;

    if let Some(cells) = cells {
        read_cells_cal(&range, cells)
    } else {
        read_range_cal(&range, row_range, col_range, default_rows)
    }
}

fn read_cells_cal(range: &calamine::Range<Data>, cells: &[(u32, u32)]) -> Result<ReadResult> {
    let mut vs: Vec<Row> = Vec::new();
    let count = cells.len();
    for &(c, r) in cells {
        let val = range.get(((r - 1) as usize, (c - 1) as usize))
            .map(|d| data_str(d))
            .unwrap_or_default();
        vs.push(vec![val]);
    }
    Ok(ReadResult { columns: vec!["Value".into()], rows: vs, row_count: count })
}

fn read_range_cal(
    range: &calamine::Range<Data>,
    row_range: Option<(Option<usize>, Option<usize>)>, col_range: Option<(Option<u32>, Option<u32>)>,
    default_rows: usize,
) -> Result<ReadResult> {
    let (total_cols, total_rows) = range.get_size();
    let total_cols = total_cols as u32;
    let total_rows = total_rows as usize;

    let start_c = col_range.and_then(|(s, _)| s).unwrap_or(1);
    let end_c = col_range.and_then(|(_, e)| e).unwrap_or(total_cols);
    let start_r = row_range.as_ref().and_then(|(s, _)| *s).unwrap_or(1).max(1);
    let end_r = if row_range.is_some() {
        row_range.and_then(|(_, e)| e).unwrap_or(total_rows)
    } else {
        (start_r + default_rows - 1).min(total_rows)
    };

    let mut all_rows: Vec<Row> = Vec::new();
    for r in start_r..=end_r {
        let mut row = Vec::new();
        for c in start_c..=end_c {
            let val = range.get(((r - 1) as usize, (c - 1) as usize))
                .map(|d| data_str(d))
                .unwrap_or_default();
            row.push(val);
        }
        all_rows.push(row);
    }
    let cols: Vec<String> = (start_c..=end_c).map(col_idx).collect();
    Ok(ReadResult { columns: cols, row_count: all_rows.len(), rows: all_rows })
}

fn data_str(d: &Data) -> String {
    match d {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format!("{f}"),
        Data::Int(i) => format!("{i}"),
        Data::Bool(b) => format!("{b}"),
        Data::Error(e) => format!("#ERR:{e:?}"),
        Data::DateTime(f) => format!("{f}"),
        _ => format!("{d:?}"),
    }
}

fn read_umya(
    path: &Path, sheet_name: Option<&str>,
    row_range: Option<(Option<usize>, Option<usize>)>, col_range: Option<(Option<u32>, Option<u32>)>,
    cells: Option<&[(u32, u32)]>, default_rows: usize,
) -> Result<ReadResult> {
    let book = umya_spreadsheet::reader::xlsx::read(path)
        .map_err(|e| anyhow!("读取文件失败: {e}"))?;

    let ws = if let Some(name) = sheet_name {
        book.get_sheet_by_name(name).ok_or_else(|| anyhow!("工作表 '{name}' 不存在"))?.clone()
    } else {
        book.get_sheet(&0).ok_or_else(|| anyhow!("文件中没有工作表"))?.clone()
    };
    let (max_col, max_row) = ws.get_highest_column_and_row();

    if let Some(cells) = cells {
        let mut rows = Vec::new();
        for &(c, r) in cells {
            let v = ws.get_cell_value((c, r)).get_value().to_string();
            rows.push(vec![v]);
        }
        let cnt = rows.len();
        return Ok(ReadResult { columns: vec!["Value".into()], rows, row_count: cnt });
    }

    let start_c = col_range.and_then(|(s, _)| s).unwrap_or(1);
    let end_c = col_range.and_then(|(_, e)| e).unwrap_or(max_col);
    let start_r = row_range.as_ref().and_then(|(s, _)| s.map(|v| v.max(1) as u32)).unwrap_or(1);
    let end_r = if row_range.is_some() {
        row_range.and_then(|(_, e)| e.map(|v| v as u32)).unwrap_or(max_row)
    } else {
        (start_r + default_rows as u32 - 1).min(max_row)
    };

    let mut rows = Vec::new();
    for r in start_r..=end_r {
        let mut row = Vec::new();
        for c in start_c..=end_c {
            row.push(ws.get_cell_value((c, r)).get_value().to_string());
        }
        rows.push(row);
    }
    let cols: Vec<String> = (start_c..=end_c).map(col_idx).collect();
    Ok(ReadResult { columns: cols, rows, row_count: (end_r - start_r + 1) as usize })
}

fn col_idx(mut n: u32) -> String {
    let mut s = String::new();
    while n > 0 { n -= 1; s.push((b'A' + (n % 26) as u8) as char); n /= 26; }
    s.chars().rev().collect()
}
