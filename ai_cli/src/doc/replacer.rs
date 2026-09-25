//! Docx 文本替换逻辑
//!
//! 核心策略：对象层合并跨 Run 文本后替换。
//! 借鉴 docxtpl 的跨 Run 合并思路，但在 docx-rs 对象层实现，
//! 避免直接操作 XML 字符串。
//!
//! 算法：
//! 1. 遍历段落，拼接所有 Run 的文本
//! 2. 检查拼接文本是否含占位符
//! 3. 有匹配 → 合并所有 Run 为一个，替换文本，继承首个 Run 格式
//! 4. 无匹配 → 保持原样（零副作用）

use anyhow::{anyhow, Result};
use docx_rs::*;
use std::fs;
use std::path::Path;

/// 替换 docx 中的占位符文本，输出到新文件。
///
/// `replacements` 为 `Vec<(key, value)>`，支持同一 key 多次出现：
/// - key 出现 1 次 → 全局替换（所有匹配处替换为同一 value）
/// - key 出现 N 次 → 按序替换（第 i 个匹配处替换为第 i 个 value，
///   超出部分用最后一个 value）
pub fn replace_in_docx(
    input: &Path,
    replacements: &[(String, String)],
    output: &Path,
) -> Result<()> {
    let buf = fs::read(input)
        .map_err(|e| anyhow!("读取文件失败 '{}': {e}", input.display()))?;

    let mut doc = read_docx(&buf)
        .map_err(|e| anyhow!("解析 docx 失败: {e}"))?;

    // 构建 key → values 映射
    let mut key_values: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for (k, v) in replacements {
        key_values.entry(k.as_str()).or_default().push(v.as_str());
    }

    // 全局计数器：跟踪每个 key 已替换到第几个匹配
    let mut counters: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    for child in &mut doc.document.children {
        match child {
            DocumentChild::Paragraph(p) => replace_in_paragraph(p, &key_values, &mut counters),
            DocumentChild::Table(t) => replace_in_table(t, &key_values, &mut counters),
            _ => {}
        }
    }

    let file = std::fs::File::create(output)
        .map_err(|e| anyhow!("创建文件失败 '{}': {e}", output.display()))?;

    doc.build().pack(file)
        .map_err(|e| anyhow!("写入 docx 失败: {e}"))?;

    Ok(())
}

/// 在段落中替换占位符
fn replace_in_paragraph<'a>(
    p: &mut Paragraph,
    key_values: &std::collections::HashMap<&'a str, Vec<&'a str>>,
    counters: &mut std::collections::HashMap<&'a str, usize>,
) {
    // 1. 收集所有 Run 中的纯文本
    let texts: Vec<&str> = p.children.iter()
        .filter_map(|child| {
            if let ParagraphChild::Run(run) = child {
                Some(run.children.iter().filter_map(|rc| {
                    if let RunChild::Text(t) = rc {
                        Some(t.text.as_str())
                    } else {
                        None
                    }
                }).collect::<Vec<_>>())
            } else {
                None
            }
        })
        .flatten()
        .collect();

    if texts.is_empty() {
        return;
    }

    let full_text = texts.concat();

    // 2. 检查是否有匹配的占位符
    let mut has_match = false;
    for key in key_values.keys() {
        if full_text.contains(key) {
            has_match = true;
            break;
        }
    }

    if !has_match {
        return;
    }

    // 3. 执行替换：对每个 key，按全局出现顺序依次替换
    let mut replaced = full_text.clone();
    for (key, values) in key_values {
        let is_single = values.len() == 1;
        if is_single {
            // 单值 → 全局替换
            replaced = replaced.replace(key, values[0]);
        } else {
            // 多值 → 按全局序替换
            let counter = counters.entry(key).or_insert(0);
            let last = values.last().copied().unwrap_or("");
            while let Some(pos) = replaced.find(key) {
                let val = if *counter < values.len() { values[*counter] } else { last };
                let end = pos + key.len();
                replaced.replace_range(pos..end, val);
                *counter += 1;
            }
        }
    }

    // 5. 克隆首个 Run 以继承其格式属性
    let first_run = p.children.iter()
        .filter_map(|child| {
            if let ParagraphChild::Run(run) = child {
                Some(run.clone())
            } else {
                None
            }
        })
        .next()
        .unwrap_or_default();

    // 6. 合并所有 Run 为一个，保留格式
    let mut new_run = first_run;
    new_run.children = vec![RunChild::Text(Text::new(replaced))];

    // 7. 替换段落子元素
    p.children = vec![ParagraphChild::Run(new_run)];
}

/// 在表格中递归替换占位符
fn replace_in_table<'a>(
    t: &mut Table,
    key_values: &std::collections::HashMap<&'a str, Vec<&'a str>>,
    counters: &mut std::collections::HashMap<&'a str, usize>,
) {
    for row in &mut t.rows {
        let TableChild::TableRow(row) = row;
        for cell in &mut row.cells {
            let TableRowChild::TableCell(cell) = cell;
            for content in &mut cell.children {
                match content {
                    TableCellContent::Paragraph(p) => replace_in_paragraph(p, key_values, counters),
                    TableCellContent::Table(inner) => replace_in_table(inner, key_values, counters),
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kv<'a>(pairs: &[(&'a str, &'a str)]) -> (std::collections::HashMap<&'a str, Vec<&'a str>>, std::collections::HashMap<&'a str, usize>) {
        let mut kv: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
        for (k, v) in pairs {
            kv.entry(*k).or_default().push(*v);
        }
        (kv, std::collections::HashMap::new())
    }

    #[test]
    fn test_replace_simple() {
        let (kv, mut counters) = make_kv(&[
            ("{{name}}", "张三"),
            ("{{date}}", "2024-01-01"),
        ]);

        let mut p = Paragraph::default();
        p.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("您好{{name}}，日期：{{date}}")
        )));

        replace_in_paragraph(&mut p, &kv, &mut counters);

        let text = extract_paragraph_text(&p);
        assert_eq!(text, "您好张三，日期：2024-01-01");
    }

    #[test]
    fn test_replace_cross_run() {
        let (kv, mut counters) = make_kv(&[
            ("{{name}}", "张三"),
        ]);

        let mut p = Paragraph::default();
        p.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("{{na")
        )));
        p.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("me}}")
        )));

        replace_in_paragraph(&mut p, &kv, &mut counters);

        let text = extract_paragraph_text(&p);
        assert_eq!(text, "张三");
    }

    #[test]
    fn test_no_match_unchanged() {
        let (kv, mut counters) = make_kv(&[
            ("{{name}}", "张三"),
        ]);

        let mut p = Paragraph::default();
        p.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("普通文本无占位符")
        )));

        let original_text = extract_paragraph_text(&p);
        replace_in_paragraph(&mut p, &kv, &mut counters);
        let new_text = extract_paragraph_text(&p);

        assert_eq!(original_text, new_text);
        assert_eq!(p.children.len(), 1);
    }

    #[test]
    fn test_replace_sequential() {
        let (kv, mut counters) = make_kv(&[
            ("XXX", "{{name}}"),
            ("XXX", "{{phone}}"),
            ("XXX", "{{email}}"),
        ]);

        let mut p = Paragraph::default();
        p.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("姓名XXX，电话XXX，邮箱XXX")
        )));

        replace_in_paragraph(&mut p, &kv, &mut counters);

        let text = extract_paragraph_text(&p);
        assert_eq!(text, "姓名{{name}}，电话{{phone}}，邮箱{{email}}");
    }

    #[test]
    fn test_replace_sequential_overflow() {
        let (kv, mut counters) = make_kv(&[
            ("XXX", "{{a}}"),
            ("XXX", "{{b}}"),
        ]);

        let mut p = Paragraph::default();
        p.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("XXX XXX XXX XXX")
        )));

        replace_in_paragraph(&mut p, &kv, &mut counters);

        let text = extract_paragraph_text(&p);
        assert_eq!(text, "{{a}} {{b}} {{b}} {{b}}");
    }

    #[test]
    fn test_replace_sequential_cross_paragraph() {
        // 跨段落全局计数
        let (kv, mut counters) = make_kv(&[
            ("XXX", "{{a}}"),
            ("XXX", "{{b}}"),
            ("XXX", "{{c}}"),
        ]);

        let mut p1 = Paragraph::default();
        p1.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("XXX")
        )));

        let mut p2 = Paragraph::default();
        p2.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("XXX")
        )));

        let mut p3 = Paragraph::default();
        p3.children.push(ParagraphChild::Run(Box::new(
            Run::new().add_text("XXX")
        )));

        replace_in_paragraph(&mut p1, &kv, &mut counters);
        replace_in_paragraph(&mut p2, &kv, &mut counters);
        replace_in_paragraph(&mut p3, &kv, &mut counters);

        assert_eq!(extract_paragraph_text(&p1), "{{a}}");
        assert_eq!(extract_paragraph_text(&p2), "{{b}}");
        assert_eq!(extract_paragraph_text(&p3), "{{c}}");
    }

    fn extract_paragraph_text(p: &Paragraph) -> String {
        p.children.iter()
            .filter_map(|child| {
                if let ParagraphChild::Run(run) = child {
                    Some(run.children.iter().filter_map(|rc| {
                        if let RunChild::Text(t) = rc {
                            Some(t.text.as_str())
                        } else {
                            None
                        }
                    }).collect::<Vec<_>>().concat())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .concat()
    }
}
