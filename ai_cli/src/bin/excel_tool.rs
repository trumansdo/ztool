//! excel_tool —— Excel 文件读写 CLI 工具
//!
//! 使用方式: excel_tool <COMMAND>
//!
//! 子命令:
//!   read  <FILE>  读取 Excel 文件
//!   write <FILE>  写入 Excel 文件

use clap::{Parser, Subcommand, ValueEnum};
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(
    name = "excel_tool",
    version,
    about = "Excel 文件读写工具",
    long_about = "基于 calamine / rust_xlsxwriter / umya-spreadsheet 三引擎的 Excel 操作工具。\n\
                  按需选择引擎：读默认 calamine（最快），写新文件默认 xlsxwriter（功能最全），\n\
                  覆盖原文件默认 umya（保留样式）。"
)]
struct ExcelCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 读取 Excel 文件（默认 calamine）
    Read {
        /// Excel 文件路径
        #[arg(value_name = "FILE")]
        file: String,

        /// 工作表名称（默认第一个）
        #[arg(long, value_name = "NAME")]
        sheet: Option<String>,

        /// 单元格引用，如 A1 B2（精确读取），有 = 时表示写入
        #[arg(long = "cell", value_name = "CELL", num_args = 1..)]
        cells: Vec<String>,

        /// 行范围：1-50 / 1- / -100，默认 1-10
        #[arg(long = "row-range", value_name = "RANGE")]
        row_range: Option<String>,

        /// 列范围：A-D / A- / -Z
        #[arg(long = "col-range", value_name = "RANGE")]
        col_range: Option<String>,

        /// 输出格式
        #[arg(long = "format", short = 'f', value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,

        /// 强制指定引擎
        #[arg(long, value_enum)]
        engine: Option<ExcelEngineArg>,
    },

    /// 写入 Excel 文件（默认 xlsxwriter / umya）
    Write {
        /// Excel 文件路径
        #[arg(value_name = "FILE")]
        file: String,

        /// 单元格写入：A1=hello B2=42（必须）
        #[arg(long = "cell", value_name = "CELL=VALUE", num_args = 1.., required = true)]
        cells: Vec<String>,

        /// 工作表名称（默认 Sheet1）
        #[arg(long, value_name = "NAME")]
        sheet: Option<String>,

        /// 覆盖原文件（不加则自动序号新建）
        #[arg(long, default_value_t = false)]
        r#override: bool,

        /// 保留样式（= --engine umya 快捷方式）
        #[arg(long, default_value_t = false)]
        keep_style: bool,

        /// 强制指定引擎
        #[arg(long, value_enum)]
        engine: Option<ExcelEngineArg>,
    },
}

#[derive(Clone, ValueEnum, Debug)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Clone, ValueEnum, Debug)]
enum ExcelEngineArg {
    Calamine,
    Xlsxwriter,
    Umya,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "ai_cli=warn".into()))
        .with_target(false)
        .without_time()
        .init();

    let cli = ExcelCli::parse();

    match &cli.command {
        Commands::Read { file, sheet, cells, row_range, col_range, format, engine } => {
            run_read(file, sheet.as_deref(), cells, row_range.as_deref(), col_range.as_deref(), format, engine.as_ref())?;
        }
        Commands::Write { file, cells, sheet, r#override, keep_style, engine } => {
            run_write(file, sheet.as_deref(), cells, *r#override, *keep_style, engine.as_ref())?;
        }
    }

    Ok(())
}

fn map_engine(arg: Option<&ExcelEngineArg>) -> Option<ai_cli::excel::engine::ExcelEngine> {
    arg.map(|e| match e {
        ExcelEngineArg::Calamine => ai_cli::excel::engine::ExcelEngine::Calamine,
        ExcelEngineArg::Xlsxwriter => ai_cli::excel::engine::ExcelEngine::Xlsxwriter,
        ExcelEngineArg::Umya => ai_cli::excel::engine::ExcelEngine::Umya,
    })
}

fn run_read(
    file: &str,
    sheet: Option<&str>,
    cells: &[String],
    row_range: Option<&str>,
    col_range: Option<&str>,
    format: &OutputFormat,
    engine_arg: Option<&ExcelEngineArg>,
) -> Result<()> {
    use ai_cli::excel::{
        engine::resolve_engine,
        reader::{self, read_excel},
    };
    use std::path::Path;

    let path = Path::new(file);
    if !path.exists() {
        anyhow::bail!("文件不存在: {file}");
    }

    let explicit_engine = map_engine(engine_arg);
    let engine = resolve_engine(explicit_engine, false, true, false, false);

    // 解析行范围
    let rr = row_range.map(|s| reader::parse_row_range(s)).transpose()?;
    // 解析列范围
    let cr = col_range.map(|s| reader::parse_col_range(s)).transpose()?;

    // 解析单元格
    let cell_refs = if !cells.is_empty() {
        let refs: Vec<(u32, u32)> = cells.iter()
            .map(|c| reader::parse_cell_ref(c))
            .collect::<Result<Vec<_>>>()?;
        Some(refs)
    } else {
        None
    };

    let result = read_excel(path, sheet, engine, rr, cr, cell_refs.as_deref(), 10)?;
    print_read_result(&result, format)
}

fn run_write(
    file: &str,
    sheet: Option<&str>,
    cells: &[String],
    do_override: bool,
    keep_style: bool,
    engine_arg: Option<&ExcelEngineArg>,
) -> Result<()> {
    use ai_cli::excel::{
        engine::{ExcelEngine, resolve_engine, auto_numbered_path},
        writer::{self, write_excel, parse_cell_set},
    };
    use std::path::Path;

    let path = Path::new(file);
    let file_exists = path.exists();
    let explicit_engine = map_engine(engine_arg);
    let engine = resolve_engine(explicit_engine, true, file_exists, keep_style, do_override);

    let final_path = if do_override || !file_exists {
        path.to_path_buf()
    } else {
        auto_numbered_path(path)
    };

    // 如果有原文件且为覆盖模式，先读取（传给 umya 保留结构）
    let source = if file_exists && do_override {
        match engine {
            ExcelEngine::Umya | ExcelEngine::Xlsxwriter => None,
            ExcelEngine::Calamine => None,
        }
    } else {
        None
    };

    let cell_sets: Vec<writer::CellSet> = cells.iter()
        .map(|c| parse_cell_set(c))
        .collect::<Result<Vec<_>>>()?;

    write_excel(&final_path, sheet, engine, &cell_sets, source)?;
    let action = if do_override { "覆盖" } else { "新建" };
    println!("✓ {action}: {}", final_path.display());
    Ok(())
}

fn print_read_result(result: &ai_cli::excel::reader::ReadResult, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let rows: Vec<Vec<&str>> = result.rows.iter()
                .map(|r| r.iter().map(|s| s.as_str()).collect())
                .collect();
            let json = serde_json::to_string_pretty(&rows)?;
            println!("{json}");
        }
        OutputFormat::Csv => {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{}", result.columns.join(","))?;
            for row in &result.rows {
                let escaped: Vec<String> = row.iter().map(|v| {
                    if v.contains(',') || v.contains('"') || v.contains('\n') {
                        format!("\"{}\"", v.replace('"', "\"\""))
                    } else {
                        v.clone()
                    }
                }).collect();
                writeln!(handle, "{}", escaped.join(","))?;
            }
        }
        OutputFormat::Table => {
            if result.columns.is_empty() || result.rows.is_empty() {
                println!("(无数据)");
                return Ok(());
            }
            // 计算列宽
            let mut widths: Vec<usize> = result.columns.iter().map(|c| c.len()).collect();
            for row in &result.rows {
                for (i, val) in row.iter().enumerate() {
                    if i < widths.len() {
                        widths[i] = widths[i].max(val.len());
                    }
                }
            }
            for w in &mut widths {
                *w = (*w).min(60);
            }

            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();

            let sep: String = widths.iter().map(|w| "-".repeat(*w + 2)).collect::<Vec<_>>().join("+");

            // 表头
            write!(handle, "| ")?;
            for (i, col) in result.columns.iter().enumerate() {
                write!(handle, "{:width$} | ", truncate(col, widths[i]), width = widths[i])?;
            }
            writeln!(handle)?;
            writeln!(handle, "|{sep}|")?;

            // 数据
            for row in &result.rows {
                write!(handle, "| ")?;
                for (i, val) in row.iter().enumerate() {
                    let w = widths.get(i).copied().unwrap_or(10);
                    write!(handle, "{:width$} | ", truncate(val, w), width = w)?;
                }
                writeln!(handle)?;
            }
            writeln!(handle, "{} 行", result.row_count)?;
        }
    }
    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}
