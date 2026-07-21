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
