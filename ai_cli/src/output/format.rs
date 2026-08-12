//! 输出格式化
//!
//! 将提取后的内容按指定格式输出：Markdown / JSON / 纯文本。

use clap::ValueEnum;
use crate::fetch::extract::ExtractedContent;

/// 输出格式枚举
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    /// Markdown 格式（默认）
    Md,
    /// JSON 格式（含元数据）
    Json,
    /// 纯文本格式
    Text,
}

/// 格式化提取内容为字符串
///
/// # 参数
/// - `content`: 提取后的内容
/// - `format`: 输出格式
///
/// # 返回
/// 格式化后的字符串
pub fn format_output(content: &ExtractedContent, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Md => format_markdown(content),
        OutputFormat::Json => format_json(content),
        OutputFormat::Text => format_text(content),
    }
}

/// Markdown 格式输出
fn format_markdown(content: &ExtractedContent) -> String {
    let mut out = String::new();

    // 标题
    if !content.title.is_empty() && content.title != "Untitled" {
        out.push_str(&format!("# {}\n\n", content.title));
    }

    // 元信息
    out.push_str(&format!("> 来源: {}\n", content.url));
    if let Some(ref final_url) = content.final_url {
        if *final_url != content.url {
            out.push_str(&format!("> 最终 URL: {}\n", final_url));
        }
    }
    if let Some(status) = content.status_code {
        out.push_str(&format!("> HTTP 状态: {}\n", status));
    }
    out.push_str(&format!("> 模式: {}\n", content.fetch_mode));
    out.push_str(&format!("> 字符数: {}\n\n", content.content_length));

    // 分隔线
    out.push_str("---\n\n");

    // 正文
    out.push_str(&content.content);

    out
}

/// JSON 格式输出
fn format_json(content: &ExtractedContent) -> String {
    serde_json::to_string_pretty(content).unwrap_or_else(|e| {
        format!("{{\"error\": \"JSON serialization failed: {}\"}}", e)
    })
}

// ============================================================
// 公共格式化工具（db_tool / excel_tool 共用）
// ============================================================

/// 截断字符串到指定宽度
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

/// 打印对齐的文本表格（通用）
pub fn print_table(columns: &[String], rows: &[Vec<String>], row_count: usize) -> std::io::Result<()> {
    use std::io::Write;
    if columns.is_empty() {
        return Ok(());
    }
    let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
    for row in rows {
        for (i, val) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(val.len());
            }
        }
    }
    for w in &mut widths {
        *w = (*w).min(60);
    }

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    let sep: String = widths.iter().map(|w| "-".repeat(*w + 2)).collect::<Vec<_>>().join("+");

    write!(handle, "| ")?;
    for (i, col) in columns.iter().enumerate() {
        write!(handle, "{:width$} | ", truncate(col, widths[i]), width = widths[i])?;
    }
    writeln!(handle)?;
    writeln!(handle, "|{}|", sep)?;

    for row in rows {
        write!(handle, "| ")?;
        for (i, val) in row.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or(10);
            write!(handle, "{:width$} | ", truncate(val, w), width = w)?;
        }
        writeln!(handle)?;
    }
    writeln!(handle, "{} rows", row_count)?;
    Ok(())
}

/// CSV 值转义
pub fn escape_csv(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') {
        format!("\"{}\"", val.replace('"', "\"\""))
    } else {
        val.to_string()
    }
}

/// 纯文本格式输出
fn format_text(content: &ExtractedContent) -> String {
    let mut out = String::new();

    if !content.title.is_empty() && content.title != "Untitled" {
        out.push_str(&format!("{}\n", content.title));
        out.push_str(&"=".repeat(content.title.chars().count()));
        out.push_str("\n\n");
    }

    out.push_str(&format!("来源: {}\n", content.url));
    if let Some(ref final_url) = content.final_url {
        if *final_url != content.url {
            out.push_str(&format!("最终 URL: {}\n", final_url));
        }
    }
    if let Some(status) = content.status_code {
        out.push_str(&format!("HTTP 状态: {}\n", status));
    }
    out.push_str(&format!("模式: {}\n", content.fetch_mode));
    out.push_str(&format!("字符数: {}\n\n", content.content_length));

    // 将 Markdown 转为纯文本（简单处理）
    let text = content
        .content
        .replace("**", "")
        .replace("__", "")
        .replace("*", "")
        .replace("`", "")
        .replace("#", "")
        .replace("> ", "");

    out.push_str(&text);
    out
}
