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

/// hover 内容格式化（MarkupContent / string / 数组）
pub fn format_hover(v: &Value) -> String {
    let contents = v.get("contents");
    let mut out = String::new();
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
            _ => out.push_str(&c.to_string()),
        }
    }
    append(&contents.unwrap_or(&Value::Null), &mut out);
    out
}

/// Location/LocationLink 格式化: `uri:line:char [sel=...]`
pub fn format_locations(v: &Value) -> String {
    fn loc_str(l: &Value) -> String {
        let uri = l.get("uri").and_then(Value::as_str).unwrap_or("?");
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
        format!("{}:{}:{}{}", uri, sl + 1, sc + 1, sel_info)
    }
    match v {
        Value::Array(a) => a.iter().map(loc_str).collect::<Vec<_>>().join("\n"),
        Value::Object(_) => loc_str(v),
        _ => String::new(),
    }
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
