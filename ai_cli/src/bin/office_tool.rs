//! office_tool —— Office 文档统一读写 CLI 工具
//!
//! 自动检测文件类型（magic bytes + 扩展名），分发到 Excel/Word 引擎。
//!
//! 使用方式: office_tool <COMMAND>
//!
//! 子命令:
//!   read  <FILE>  读取 Office 文件（自动识别 .xlsx/.xls/.docx/.doc）
//!   write <FILE>  写入 Office 文件（按扩展名自动判定，--type 可覆盖）

use clap::{Parser, Subcommand, ValueEnum};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(
    name = "office_tool",
    version,
    about = "Office 文档统一读写工具",
    long_about = "自动检测文件类型（magic bytes + 扩展名），统一操作 Excel 和 Word 文档。\n\
                  读取：自动识别 .xlsx/.xls/.docx/.doc\n\
                  写入：按扩展名自动判定，--type 可覆盖"
)]
struct OfficeCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 读取 Office 文件（自动识别类型）
    Read {
        /// 文件路径（.xlsx / .xls / .docx / .doc）
        #[arg(value_name = "FILE")]
        file: String,

        /// 输出格式（Excel: table/json/csv, Word: text/markdown/json）
        #[arg(long = "format", short = 'f', value_name = "FMT")]
        format: Option<String>,

        // ---- Excel 专用 ----
        /// 工作表名称
        #[arg(long, value_name = "NAME")]
        sheet: Option<String>,

        /// 单元格引用 A1 B2
        #[arg(long = "cell", value_name = "CELL", num_args = 1..)]
        cells: Vec<String>,

        /// 行范围 1-50
        #[arg(long = "row-range", value_name = "RANGE")]
        row_range: Option<String>,

        /// 列范围 A-D
        #[arg(long = "col-range", value_name = "RANGE")]
        col_range: Option<String>,
    },

    /// 替换 docx 模板中的占位符文本
    Replace {
        /// 模板文件路径（.docx）
        #[arg(value_name = "FILE")]
        file: String,

        /// 键值对 KEY=VALUE（可多次指定）
        #[arg(long = "kv", value_name = "KEY=VALUE", num_args = 1..)]
        kv: Vec<String>,

        /// 输出文件路径（不指定则自动编号备份）
        #[arg(long = "output", short = 'o', value_name = "FILE")]
        output: Option<String>,

        /// 覆盖原文件
        #[arg(long, default_value_t = false)]
        r#override: bool,
    },

    /// 写入 Office 文件（按扩展名判定类型）
    Write {
        /// 输出文件路径（.xlsx 或 .docx）
        #[arg(value_name = "FILE")]
        file: String,

        /// 强制指定类型（覆盖扩展名判定）
        #[arg(long, value_enum, value_name = "TYPE")]
        r#type: Option<OfficeType>,

        // ---- Excel 写入 ----
        /// 单元格写入 A1=hello B2=42
        #[arg(long = "cell", value_name = "CELL=VALUE", num_args = 1..)]
        cells: Vec<String>,

        /// 工作表名称
        #[arg(long, value_name = "NAME")]
        sheet: Option<String>,

        /// 覆盖原文件
        #[arg(long, default_value_t = false)]
        r#override: bool,

        /// 保留样式
        #[arg(long, default_value_t = false)]
        keep_style: bool,

        // ---- Word 写入 ----
        /// 段落文本（可多次指定）
        #[arg(long = "para", short = 'p', value_name = "TEXT", num_args = 1..)]
        paragraphs: Vec<String>,

        /// 标题文本
        #[arg(long, value_name = "TEXT")]
        title: Option<String>,

        /// 表格数据 header1,header2 row1c1,row1c2
        #[arg(long = "table", value_name = "CSV", num_args = 1..)]
        table: Vec<String>,
    },
}

#[derive(Clone, ValueEnum, Debug)]
enum OfficeType {
    Xlsx,
    Docx,
}

// ============================
// 文件类型检测
// ============================

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetectedType {
    Excel,
    Word,
}

fn detect_type(path: &std::path::Path) -> Result<DetectedType> {
    // 1. magic bytes 优先
    if let Ok(Some(kind)) = infer::get_from_path(path) {
        let mime = kind.mime_type();
        if mime.contains("spreadsheet") || mime.contains("excel") {
            return Ok(DetectedType::Excel);
        }
        if mime.contains("wordprocessing") || mime.contains("msword") {
            return Ok(DetectedType::Word);
        }
    }

    // 2. 扩展名 fallback
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" => return Ok(DetectedType::Excel),
            "docx" | "doc" | "docm" | "odt" => return Ok(DetectedType::Word),
            _ => {}
        }
    }

    anyhow::bail!("无法识别文件类型: {}", path.display())
}

// ============================
// main
// ============================

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()))
        .with_target(false)
        .without_time()
        .init();

    let cli = OfficeCli::parse();

    match &cli.command {
        Commands::Read { file, format, sheet, cells, row_range, col_range } => {
            run_read(file, format.as_deref(), sheet.as_deref(), cells, row_range.as_deref(), col_range.as_deref())?;
        }
        Commands::Replace { file, kv, output, r#override } => {
            run_replace(file, kv, output.as_deref(), *r#override)?;
        }
        Commands::Write { file, r#type, cells, sheet, r#override, keep_style, paragraphs, title, table } => {
            run_write(file, r#type.as_ref(), cells, sheet.as_deref(), *r#override, *keep_style, paragraphs, title, table)?;
        }
    }

    Ok(())
}

// ============================
// 读取分发
// ============================

fn run_read(
    file: &str,
    format: Option<&str>,
    sheet: Option<&str>,
    cells: &[String],
    row_range: Option<&str>,
    col_range: Option<&str>,
) -> Result<()> {
    use std::path::Path;

    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("文件不存在: {file}");
    }

    let detected = detect_type(path)?;

    match detected {
        DetectedType::Excel => read_excel(path, format, sheet, cells, row_range, col_range),
        DetectedType::Word => read_word(path, format),
    }
}

fn read_excel(
    path: &std::path::Path,
    format: Option<&str>,
    sheet: Option<&str>,
    cells: &[String],
    row_range: Option<&str>,
    col_range: Option<&str>,
) -> Result<()> {
    use ai_cli::excel::{
        engine::resolve_engine,
        reader::{self, read_excel},
    };

    let engine = resolve_engine(None, false, true, false, false);

    let rr = row_range.map(|s| reader::parse_row_range(s)).transpose()?;
    let cr = col_range.map(|s| reader::parse_col_range(s)).transpose()?;

    let cell_refs = if !cells.is_empty() {
        let refs: Vec<(u32, u32)> = cells.iter()
            .map(|c| reader::parse_cell_ref(c))
            .collect::<Result<Vec<_>>>()?;
        Some(refs)
    } else {
        None
    };

    let result = read_excel(path, sheet, engine, rr, cr, cell_refs.as_deref(), 10)?;

    let fmt = format.unwrap_or("table");
    match fmt {
        "json" => {
            let rows: Vec<Vec<&str>> = result.rows.iter()
                .map(|r| r.iter().map(|s| s.as_str()).collect())
                .collect();
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        "csv" => {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{}", result.columns.join(","))?;
            for row in &result.rows {
                let escaped: Vec<String> = row.iter()
                    .map(|v| ai_cli::output::format::escape_csv(v))
                    .collect();
                writeln!(handle, "{}", escaped.join(","))?;
            }
        }
        _ => {
            if result.columns.is_empty() || result.rows.is_empty() {
                println!("(无数据)");
                return Ok(());
            }
            let rows: Vec<Vec<String>> = result.rows.clone();
            ai_cli::output::format::print_table(&result.columns, &rows, result.row_count)?;
        }
    }

    Ok(())
}

fn read_word(path: &std::path::Path, format: Option<&str>) -> Result<()> {
    use ai_cli::doc::reader::read_doc;

    let result = read_doc(path)?;
    let fmt = format.unwrap_or("text");

    match fmt {
        "markdown" | "md" => println!("{}", result.markdown),
        "json" => {
            let json = serde_json::json!({
                "format": result.format,
                "text": result.text,
                "markdown": result.markdown,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => println!("{}", result.text),
    }

    Ok(())
}

// ============================
// 替换分发
// ============================

fn run_replace(
    file: &str,
    kv: &[String],
    output: Option<&str>,
    do_override: bool,
) -> Result<()> {
    use std::path::Path;

    let input = Path::new(file);
    if !input.exists() {
        anyhow::bail!("文件不存在: {file}");
    }

    // 解析 KEY=VALUE（Vec 支持同一 key 多次出现，用于按序替换）
    let mut replacements: Vec<(String, String)> = Vec::new();
    for pair in kv {
        let (k, v) = pair.split_once('=')
            .ok_or_else(|| anyhow::anyhow!("格式错误，应为 KEY=VALUE: {pair}"))?;
        replacements.push((k.to_string(), v.to_string()));
    }

    if replacements.is_empty() {
        anyhow::bail!("至少需要一对 --kv KEY=VALUE");
    }

    // 确定输出路径
    let output_path = if let Some(o) = output {
        Path::new(o).to_path_buf()
    } else if do_override {
        input.to_path_buf()
    } else {
        auto_numbered_path(input)
    };

    ai_cli::doc::replacer::replace_in_docx(input, &replacements, &output_path)?;

    let action = if do_override { "覆盖" } else { "新建" };
    println!("✓ {action}: {}", output_path.display());
    Ok(())
}

/// 自动序号生成文件名
fn auto_numbered_path(path: &std::path::Path) -> std::path::PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(std::path::Path::new("."));

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

// ============================
// 写入分发
// ============================

fn run_write(
    file: &str,
    r#type: Option<&OfficeType>,
    cells: &[String],
    sheet: Option<&str>,
    do_override: bool,
    keep_style: bool,
    paragraphs: &[String],
    title: &Option<String>,
    table: &[String],
) -> Result<()> {
    use std::path::Path;

    let path = Path::new(file);

    // 判定目标类型
    let target = if let Some(t) = r#type {
        match t {
            OfficeType::Xlsx => DetectedType::Excel,
            OfficeType::Docx => DetectedType::Word,
        }
    } else {
        detect_type(path)?
    };

    match target {
        DetectedType::Excel => write_excel(path, cells, sheet, do_override, keep_style),
        DetectedType::Word => write_word(path, paragraphs, title, table),
    }
}

fn write_excel(
    path: &std::path::Path,
    cells: &[String],
    sheet: Option<&str>,
    do_override: bool,
    keep_style: bool,
) -> Result<()> {
    use ai_cli::excel::{
        engine::{resolve_engine, auto_numbered_path},
        writer::{self, write_excel, parse_cell_set},
    };

    let file_exists = path.exists();
    let engine = resolve_engine(None, true, file_exists, keep_style, do_override);

    let final_path = if do_override || !file_exists {
        path.to_path_buf()
    } else {
        auto_numbered_path(path)
    };

    let cell_sets: Vec<writer::CellSet> = cells.iter()
        .map(|c| parse_cell_set(c))
        .collect::<Result<Vec<_>>>()?;

    write_excel(&final_path, sheet, engine, &cell_sets, None)?;
    let action = if do_override { "覆盖" } else { "新建" };
    println!("✓ {action}: {}", final_path.display());
    Ok(())
}

fn write_word(
    path: &std::path::Path,
    paragraphs: &[String],
    title: &Option<String>,
    table: &[String],
) -> Result<()> {
    use ai_cli::doc::writer::{self, ParaData, TableData};

    if !table.is_empty() {
        if table.len() < 2 {
            anyhow::bail!("表格至少需要表头行和一行数据");
        }
        let headers: Vec<String> = table[0].split(',').map(|s| s.trim().to_string()).collect();
        let rows: Vec<Vec<String>> = table[1..].iter()
            .map(|r| r.split(',').map(|s| s.trim().to_string()).collect())
            .collect();
        let tbl = TableData { headers, rows };
        writer::write_docx_with_table(path, title.as_deref(), &tbl)?;
    } else {
        let paras: Vec<ParaData> = if paragraphs.is_empty() {
            vec![ParaData {
                text: title.clone().unwrap_or_default(),
                bold: true,
                italic: false,
                size: Some(28),
            }]
        } else {
            paragraphs.iter().map(|t| ParaData {
                text: t.clone(),
                bold: false,
                italic: false,
                size: None,
            }).collect()
        };
        writer::write_docx(path, &paras)?;
    }

    println!("✓ 写入: {}", path.display());
    Ok(())
}
