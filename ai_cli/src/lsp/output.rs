//! LSP 结果格式化（简洁不丢信息）

use serde_json::Value;

/// 符号树格式化（缩进层级）
pub fn format_symbols(v: &Value) -> String {
    fn walk(syms: &[Value], depth: usize, out: &mut String) {
        for s in syms {
            let name = s.get("name").and_then(Value::as_str).unwrap_or("?");
            let kind = s.get("kind").and_then(Value::as_u64).unwrap_or(0);
            let detail = s.get("detail").and_then(Value::as_str).unwrap_or("");
            let indent = "  ".repeat(depth);
            if let Some(children) = s.get("children").and_then(Value::as_array) {
                if !children.is_empty() {
                    out.push_str(&format!(
                        "{}{} {} {}\n",
                        indent,
                        symbol_kind_name(kind),
                        name,
                        detail
                    ));
                    walk(children, depth + 1, out);
                    continue;
                }
            }
            out.push_str(&format!("{}{} {} {}\n", indent, symbol_kind_name(kind), name, detail));
        }
    }
    let mut out = String::new();
    if let Some(arr) = v.as_array() {
        walk(arr, 0, &mut out);
    }
    out
}

/// SymbolKind 数字 → 名称（LSP 规范）
pub fn symbol_kind_name(kind: u64) -> &'static str {
    match kind {
        1 => "File",
        2 => "Module",
        3 => "Namespace",
        4 => "Package",
        5 => "Class",
        6 => "Method",
        7 => "Property",
        8 => "Field",
        9 => "Constructor",
        10 => "Enum",
        11 => "Interface",
        12 => "Function",
        13 => "Variable",
        14 => "Constant",
        23 => "Struct",
        24 => "Event",
        25 => "Operator",
        26 => "TypeParameter",
        _ => "Symbol",
    }
}

/// hover 内容格式化（对 AI 精确: 签名与文档分离, 去掉 markdown 噪音）
/// jdtls 返回 MarkupContent(markdown), 典型结构: ```java\n<签名>\n``` + 文档段落
/// 输出: `签名` + 分隔线 + `文档`(首行简述), 让大模型直接可读
pub fn format_hover(v: &Value) -> String {
    let contents = v.get("contents");
    let mut raw = String::new();
    fn append(c: &Value, out: &mut String) {
        match c {
            Value::String(s) => out.push_str(s),
            Value::Object(o) => {
                if let Some(v) = o.get("value").and_then(Value::as_str) {
                    out.push_str(v);
                }
            }
            Value::Array(a) => {
                for x in a {
                    append(x, out);
                    out.push('\n');
                }
            }
            // Null/数字等无文本内容, 静默跳过 (避免把 null 当作 hover 文本输出)
            _ => {}
        }
    }
    append(&contents.unwrap_or(&Value::Null), &mut raw);

    // 解析 markdown: 提取首个代码块作为签名, 其余作为文档
    let mut signature = String::new();
    let mut doc = String::new();
    let mut in_code = false;
    for line in raw.lines() {
        if line.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            if signature.is_empty() {
                signature.push_str(line.trim());
            } else {
                signature.push(' ');
                signature.push_str(line.trim());
            }
        } else if !line.trim().is_empty() {
            doc.push_str(line.trim());
            doc.push(' ');
        }
    }
    let sig = signature.trim();
    let docs = doc.trim();
    let mut out = String::new();
    if !sig.is_empty() {
        out.push_str(sig);
        if !docs.is_empty() {
            out.push_str("\n---\n");
            out.push_str(docs);
        }
    } else if !docs.is_empty() {
        out.push_str(docs);
    }
    out
}

/// Location/LocationLink 格式化: `路径:line:col [sel=...]`
/// 优化: 去掉 file:/// 前缀(省 token, AI 直接可读路径); 行号 1-based
pub fn format_locations(v: &Value) -> String {
    fn loc_str(l: &Value) -> String {
        // 坑: Location 用 uri 字段, LocationLink 用 targetUri 字段(jdtls/rust-analyzer 均有此差异),
        //     只取 uri 会得到空导致输出 "?" 无法定位
        let uri = l
            .get("uri")
            .or_else(|| l.get("targetUri"))
            .and_then(Value::as_str)
            .unwrap_or("?");
        // 统一路径: file:///D:/x -> D:/x; 保留 Windows 盘符可读性
        let path = uri.strip_prefix("file:///").unwrap_or(uri).to_string();
        let (rng, sel) = match l.get("targetRange") {
            Some(tr) => (tr, l.get("targetSelectionRange")),
            None => (l.get("range").unwrap_or(&Value::Null), None),
        };
        let (sl, sc) = rng
            .get("start")
            .map(|s| {
                (
                    s.get("line").and_then(Value::as_u64).unwrap_or(0),
                    s.get("character").and_then(Value::as_u64).unwrap_or(0),
                )
            })
            .unwrap_or((0, 0));
        let sel_info = sel
            .and_then(|s| s.get("start"))
            .map(|s| {
                format!(
                    " sel={}:{}",
                    s.get("line").and_then(Value::as_u64).unwrap_or(0),
                    s.get("character").and_then(Value::as_u64).unwrap_or(0)
                )
            })
            .unwrap_or_default();
        format!("{}:{}:{}{}", path, sl + 1, sc + 1, sel_info)
    }
    match v {
        Value::Array(a) => a.iter().map(loc_str).collect::<Vec<_>>().join("\n"),
        Value::Object(_) => loc_str(v),
        _ => String::new(),
    }
}

/// 诊断信息格式化: `[ERROR] file:line:col message`
pub fn format_diagnostics(diags: &[Value]) -> String {
    if diags.is_empty() {
        return "(无诊断信息)".into();
    }
    let mut out = String::new();
    for d in diags {
        let severity = d.get("severity").and_then(Value::as_u64).unwrap_or(0);
        let sev = match severity {
            1 => "ERROR",
            2 => "WARN ",
            3 => "INFO ",
            _ => "HINT ",
        };
        let msg = d.get("message").and_then(Value::as_str).unwrap_or("?");
        let range = d.get("range");
        let (line, col) = range
            .and_then(|r| r.get("start"))
            .map(|s| {
                (
                    s.get("line").and_then(Value::as_u64).unwrap_or(0) + 1,
                    s.get("character").and_then(Value::as_u64).unwrap_or(0) + 1,
                )
            })
            .unwrap_or((0, 0));
        out.push_str(&format!("[{}] {}:{}:{} {}\n", sev, line, col, line, msg));
    }
    out
}

/// 补全项格式化: `kind label detail`
pub fn format_completions(v: &Value) -> String {
    let items = match v {
        Value::Array(a) => a.clone(),
        Value::Object(o) => o
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    items
        .iter()
        .map(|i| {
            let label = i.get("label").and_then(Value::as_str).unwrap_or("?");
            let kind = i.get("kind").and_then(Value::as_u64).unwrap_or(0);
            let detail = i.get("detail").and_then(Value::as_str).unwrap_or("");
            format!("{:<4} {} {}", kind, label, detail)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
