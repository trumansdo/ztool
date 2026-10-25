//! pdf_tool —— PDF 文本提取 CLI 工具
//!
//! 基于 pdf_oxide 纯 Rust 库，零外部 C 依赖。
//!
//! 使用方式: pdf_tool <COMMAND>
//!
//! 子命令:
//!   read  <FILE>  提取 PDF 文本内容
//!   meta  <FILE>  查看 PDF 元信息

use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

// ============================
// CLI 定义
// ============================

#[derive(Parser, Debug)]
#[command(
    name = "pdf_tool",
    version,
    about = "PDF 文本提取工具",
    long_about = "基于 pdf_oxide 纯 Rust 库的 PDF 读取工具。\n\
                  支持文本提取和 Markdown 转换，页码范围选择。"
)]
struct PdfCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 提取 PDF 文本内容
    Read {
        /// PDF 文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// 页码范围，如 1-5 或 1,3,5-8
        #[arg(long = "pages", short = 'p', value_name = "RANGE")]
        pages: Option<String>,

        /// 输出到文件（默认 stdout）
        #[arg(long = "output", short = 'o', value_name = "PATH")]
        output: Option<PathBuf>,
    },

    /// 查看 PDF 元信息（页数、版本、作者等）
    Meta {
        /// PDF 文件路径
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

// ============================
// main
// ============================

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()),
        )
        .with_target(false)
        .without_time()
        .init();

    let cli = PdfCli::parse();

    match &cli.command {
        Commands::Read {
            file,
            pages,
            output,
        } => run_read(file, pages.as_deref(), output.as_deref())?,
        Commands::Meta { file } => run_meta(file)?,
    }

    Ok(())
}

// ============================
// read 子命令
// ============================

fn run_read(
    file: &PathBuf,
    pages: Option<&str>,
    output: Option<&std::path::Path>,
) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("文件不存在: {}", file.display());
    }

    // 按页提取文本
    let all_pages = pdf_extract::extract_text_by_pages(file)
        .with_context(|| format!("无法读取 PDF: {}", file.display()))?;

    let total_pages = all_pages.len();

    // 解析页码范围
    let page_indices = parse_page_range(pages, total_pages)?;

    // 组装输出
    let mut result = String::new();
    for &idx in &page_indices {
        if page_indices.len() > 1 {
            result.push_str(&format!("\n=== 第 {} 页 ===\n\n", idx + 1));
        }
        result.push_str(&all_pages[idx]);
        result.push('\n');
    }

    // 输出
    if let Some(out_path) = output {
        std::fs::write(out_path, &result)
            .with_context(|| format!("写入文件失败: {}", out_path.display()))?;
        eprintln!(
            "\x1b[90m✓ 已写入: {} ({} 页)\x1b[0m",
            out_path.display(),
            page_indices.len()
        );
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle
            .write_all(result.as_bytes())
            .context("写入 stdout 失败")?;
    }

    Ok(())
}

// ============================
// meta 子命令
// ============================

fn run_meta(file: &PathBuf) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("文件不存在: {}", file.display());
    }

    let all_pages = pdf_extract::extract_text_by_pages(file)
        .with_context(|| format!("无法读取 PDF: {}", file.display()))?;

    let page_count = all_pages.len();
    let file_size = std::fs::metadata(file)
        .map(|m| m.len())
        .unwrap_or(0);

    println!("文件: {}", file.display());
    println!("页数: {page_count}");
    println!("大小: {} bytes ({:.1} KB)", file_size, file_size as f64 / 1024.0);

    Ok(())
}

// ============================
// 页码范围解析
// ============================

/// 解析页码范围字符串，返回 0-based 页面索引列表。
///
/// 支持格式:
///   - 不指定 → 所有页
///   - "1-5"  → 第 1 到第 5 页
///   - "1,3,5" → 第 1、3、5 页
///   - "1-3,7-9" → 混合
fn parse_page_range(spec: Option<&str>, total: usize) -> Result<Vec<usize>> {
    let spec = match spec {
        Some(s) => s.trim(),
        None => return Ok((0..total).collect()),
    };

    if spec.is_empty() {
        return Ok((0..total).collect());
    }

    let mut indices = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut range_parts = part.splitn(2, '-');
            let start_str = range_parts.next().unwrap_or("").trim();
            let end_str = range_parts.next().unwrap_or("").trim();

            let start: usize = start_str
                .parse()
                .with_context(|| format!("无效页码: {start_str}"))?;
            let end: usize = end_str
                .parse()
                .with_context(|| format!("无效页码: {end_str}"))?;

            if start < 1 || end < 1 || start > total || end > total {
                anyhow::bail!(
                    "页码范围 {}-{} 超出文档范围 (1-{})",
                    start,
                    end,
                    total
                );
            }
            if start > end {
                anyhow::bail!("页码范围 {}-{} 起始页不能大于结束页", start, end);
            }

            for p in start..=end {
                indices.push(p - 1); // 转 0-based
            }
        } else {
            let page: usize = part
                .parse()
                .with_context(|| format!("无效页码: {part}"))?;
            if page < 1 || page > total {
                anyhow::bail!("页码 {} 超出文档范围 (1-{})", page, total);
            }
            indices.push(page - 1);
        }
    }

    // 去重并排序
    indices.sort_unstable();
    indices.dedup();

    Ok(indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all_pages() {
        assert_eq!(parse_page_range(None, 10).unwrap(), (0..10).collect::<Vec<_>>());
        assert_eq!(parse_page_range(Some(""), 10).unwrap(), (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn test_parse_single_page() {
        assert_eq!(parse_page_range(Some("3"), 10).unwrap(), vec![2]);
    }

    #[test]
    fn test_parse_range() {
        assert_eq!(
            parse_page_range(Some("1-3"), 10).unwrap(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn test_parse_mixed() {
        assert_eq!(
            parse_page_range(Some("1,3-5,8"), 10).unwrap(),
            vec![0, 2, 3, 4, 7]
        );
    }

    #[test]
    fn test_parse_out_of_range() {
        assert!(parse_page_range(Some("11"), 10).is_err());
        assert!(parse_page_range(Some("1-11"), 10).is_err());
    }

    #[test]
    fn test_parse_invalid_order() {
        assert!(parse_page_range(Some("5-1"), 10).is_err());
    }
}
