//! 完整 LSP 能力测试套件（对齐 jdtls_lsp_full_test.py 的 23 项）
//!
//! 场景分层（模拟 AI 大模型检索代码，简单→复杂）：
//! - L1 读: hover / documentSymbol / foldingRange / semanticTokens
//! - L2 找: completion / signatureHelp / definition / declaration / typeDefinition
//! - L3 追: references / implementation / documentHighlight / callHierarchy / typeHierarchy
//! - L4 搜: workspace/symbol
//! - L5 改: formatting / prepareRename / rename / inlayHint / codeLens
//! - L6 深挖: diagnostic(推送) / codeAction / didChange 增量诊断

use crate::lsp::client::LspClient;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// 单项测试报告
pub struct TestReport {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

impl TestReport {
    fn new(name: &str, ok: bool, detail: String) -> Self {
        Self {
            name: name.into(),
            ok,
            detail,
        }
    }
}

fn as_list(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(a) => a.clone(),
        Value::Null => Vec::new(),
        other => vec![other.clone()],
    }
}

/// 运行完整测试套件（在已启动的 client 上执行，返回逐项报告）
pub fn run_full_test(client: &mut LspClient, file: &Path) -> Vec<TestReport> {
    let mut reports: Vec<TestReport> = Vec::new();
    let base = file.parent().unwrap_or(Path::new("."));
    let broken = base.join("__LspBroken.java");
    let test_file = base.join("__LspTest.java");

    macro_rules! check {
        ($name:expr, $ok:expr, $detail:expr) => {{
            reports.push(TestReport::new($name, $ok, $detail));
            println!(
                "[{}] {} {}",
                if $ok { "PASS" } else { "FAIL" },
                $name,
                $detail
            );
        }};
    }

    // 准备测试文件（真实调用场景, 模拟 AI 典型查询锚点）
    let test_src = format!(
        "package com.trigram.zero.flow;\n\
         import java.util.Arrays;\n\
         public class __LspTest {{\n\
         \x20 void demo() {{\n\
         \x20   ZeroFlow<Integer> f = ZeroFlow.of(1, 2, 3).map(x -> x * 2);\n\
         \x20   String s = f.map(Object::toString).join(\",\");\n\
         \x20   int localVar = 42;\n\
         \x20   f.consume(System.out::println);\n\
         \x20 }}\n\
         }}\n"
    );
    let _ = fs::write(&test_file, test_src);
    let _ = fs::write(
        &broken,
        "package com.trigram.zero.flow;\npublic class __LspBroken { void x() { Integer i = \"str\"; } }\n",
    );

    // 预先打开文件并缓存 uri (避免双重可变借用)
    let arr = base.join("ArrayListZeroFlow.java");
    let test_uri = client.open_file(&test_file).unwrap_or_default();
    let arr_uri = client.open_file(&arr).unwrap_or_default();
    let broken_uri = client.open_file(&broken).unwrap_or_default();

    // 新文件需通知 jdtls 重新扫描 (didChangeWatchedFiles, type=1 Created)
    // 否则文件不在项目源码集内, JDT 类型模型缺失 (findPrimaryType null)
    // 坑: initialize 之后才写入磁盘的新文件不会被项目导入快照包含,
    //     直接 didOpen 它只是"外部文件", hover/definition 等类型查询全部 NPE
    let _ = client.notify(
        "workspace/didChangeWatchedFiles",
        json!({"changes": [
            {"uri": test_uri.clone(), "type": 1},
            {"uri": broken_uri.clone(), "type": 1}
        ]}),
    );
    std::thread::sleep(Duration::from_secs(2));

    // 等待类型模型就绪: 轮询 test_file 的 hover 直到返回非空 (首次构建需时间)
    // documentSymbol 是语法级(立即可用), hover/completion 依赖类型构建
    // 坑: 全新 workspace 首次导入后类型模型未构建时查询会 NPE
    //     (ICompilationUnit.findPrimaryType() is null), 必须等构建完成
    println!("[test] 等待 jdtls 类型模型构建完成...");
    let mut ready = false;
    for i in 0..30 {
        std::thread::sleep(Duration::from_secs(3));
        if let Ok(v) = client.hover(&test_file, 5, 49) {
            let txt = crate::lsp::output::format_hover(&v);
            if !txt.trim().is_empty() {
                ready = true;
                println!("[test] 类型模型就绪 ({}s): {}", (i + 1) * 3, txt.chars().take(50).collect::<String>());
                break;
            }
        }
    }
    if !ready {
        println!("[warn] 90s 内类型模型未就绪, 继续测试(结果可能不稳定)");
    }
    // 额外等待测试文件编译稳定
    std::thread::sleep(Duration::from_secs(3));

    // ===== L1 读 =====
    let r = client.hover(&test_file, 5, 49);
    match r {
        Ok(v) => {
            let txt = crate::lsp::output::format_hover(&v);
            check!(
                "L1 hover(map调用点)",
                txt.contains("ZeroFlow") || txt.contains("map"),
                format!("{}", txt.chars().take(60).collect::<String>())
            );
        }
        Err(e) => check!("L1 hover(map调用点)", false, e.to_string()),
    }

    match client.document_symbols(&test_file) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L1 documentSymbol", n >= 2, format!("{} symbols", n));
        }
        Err(e) => check!("L1 documentSymbol", false, e.to_string()),
    }

    match client.request(
        "textDocument/foldingRange",
        json!({"textDocument": {"uri": test_uri.clone()}}),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L1 foldingRange", n > 0, format!("{} ranges", n));
        }
        Err(e) => check!("L1 foldingRange", false, e.to_string()),
    }

    match client.request(
        "textDocument/semanticTokens/full",
        json!({"textDocument": {"uri": test_uri.clone()}}),
    ) {
        Ok(v) => {
            let n = v
                .get("data")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            check!("L1 semanticTokens", n > 0, format!("{} tokens", n));
        }
        Err(e) => check!("L1 semanticTokens", false, e.to_string()),
    }

    // ===== L2 找 =====
    match client.completion(&test_file, 5, 60) {
        Ok(v) => {
            let items = match &v {
                Value::Array(a) => a.len(),
                Value::Object(o) => o
                    .get("items")
                    .and_then(Value::as_array)
                    .map(|a| a.len())
                    .unwrap_or(0),
                _ => 0,
            };
            check!("L2 completion(链式调用点)", items > 0, format!("{} items", items));
        }
        Err(e) => check!("L2 completion(链式调用点)", false, e.to_string()),
    }

    match client.request(
        "textDocument/signatureHelp",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 4, "character": 51}
        }),
    ) {
        Ok(v) => {
            let n = v
                .get("signatures")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            check!("L2 signatureHelp(map调用)", n > 0, format!("{} sigs", n));
        }
        Err(e) => check!("L2 signatureHelp(map调用)", false, e.to_string()),
    }

    match client.definition(&test_file, 5, 49) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L2 definition(map)", n > 0, format!("{} locs", n));
        }
        Err(e) => check!("L2 definition(map)", false, e.to_string()),
    }

    // Java 无独立 declaration 概念(不像 C++), 空属正常
    match client.request(
        "textDocument/declaration",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 4, "character": 49}
        }),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L2 declaration(Java特性)", n <= 1, format!("{} locs(空属正常)", n));
        }
        Err(e) => check!("L2 declaration(Java特性)", false, e.to_string()),
    }

    match client.request(
        "textDocument/typeDefinition",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 4, "character": 22}
        }),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L2 typeDefinition(f)", n > 0, format!("{} locs", n));
        }
        Err(e) => check!("L2 typeDefinition(f)", false, e.to_string()),
    }

    // ===== L3 追 =====
    match client.references(&test_file, 5, 49, true) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L3 references(map)", n > 0, format!("{} refs", n));
        }
        Err(e) => check!("L3 references(map)", false, e.to_string()),
    }

    match client.request(
        "textDocument/implementation",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 4, "character": 49}
        }),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L3 implementation(map)", n > 0, format!("{} impls", n));
        }
        Err(e) => check!("L3 implementation(map)", false, e.to_string()),
    }

    match client.request(
        "textDocument/documentHighlight",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 4, "character": 49}
        }),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L3 documentHighlight(map)", n > 0, format!("{} hits", n));
        }
        Err(e) => check!("L3 documentHighlight(map)", false, e.to_string()),
    }

    match client.request(
        "textDocument/prepareCallHierarchy",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 4, "character": 49}
        }),
    ) {
        Ok(v) => {
            let items = as_list(&v);
            if items.is_empty() {
                check!("L3 callHierarchy(map)", false, String::from("prepare 空"));
            } else {
                let item = items[0].clone();
                let inc = client
                    .request("callHierarchy/incomingCalls", json!({"item": item}))
                    .map(|r| as_list(&r).len())
                    .unwrap_or(0);
                let out = client
                    .request("callHierarchy/outgoingCalls", json!({"item": item}))
                    .map(|r| as_list(&r).len())
                    .unwrap_or(0);
                check!("L3 callHierarchy(map)", inc + out > 0, format!("in={} out={}", inc, out));
            }
        }
        Err(e) => check!("L3 callHierarchy(map)", false, e.to_string()),
    }

    match client.request(
        "textDocument/prepareTypeHierarchy",
        json!({
            "textDocument": {"uri": arr_uri.clone()},
            "position": {"line": 11, "character": 20}
        }),
    ) {
        Ok(v) => {
            let items = as_list(&v);
            if items.is_empty() {
                check!("L3 typeHierarchy", false, String::from("prepare 空"));
            } else {
                let item = items[0].clone();
                let sup = client
                    .request("typeHierarchy/supertypes", json!({"item": item}))
                    .map(|r| as_list(&r).len())
                    .unwrap_or(0);
                let sub = client
                    .request("typeHierarchy/subtypes", json!({"item": item}))
                    .map(|r| as_list(&r).len())
                    .unwrap_or(0);
                check!("L3 typeHierarchy", sup + sub > 0, format!("super={} sub={}", sup, sub));
            }
        }
        Err(e) => check!("L3 typeHierarchy", false, e.to_string()),
    }

    // ===== L4 搜 =====
    match client.request("workspace/symbol", json!({"query": "ZeroFlow"})) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L4 workspace/symbol(ZeroFlow)", n >= 5, format!("{} symbols", n));
        }
        Err(e) => check!("L4 workspace/symbol(ZeroFlow)", false, e.to_string()),
    }

    // ===== L5 改 =====
    match client.request(
        "textDocument/formatting",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "options": {"tabSize": 2, "insertSpaces": true}
        }),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L5 formatting", n > 0, format!("{} edits", n));
        }
        Err(e) => check!("L5 formatting", false, e.to_string()),
    }

    match client.request(
        "textDocument/prepareRename",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 5, "character": 8}
        }),
    ) {
        Ok(v) => check!(
            "L5 prepareRename(局部变量)",
            !v.is_null(),
            v.to_string().chars().take(60).collect::<String>()
        ),
        Err(e) => check!("L5 prepareRename(局部变量)", false, e.to_string()),
    }

    match client.request(
        "textDocument/rename",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "position": {"line": 5, "character": 8},
            "newName": "renamedVar"
        }),
    ) {
        Ok(v) => {
            let n = v
                .get("changes")
                .and_then(Value::as_object)
                .map(|o| o.len())
                .unwrap_or(0);
            check!("L5 rename(局部变量)", n > 0, format!("{} files", n));
        }
        Err(e) => check!("L5 rename(局部变量)", false, e.to_string()),
    }

    match client.request(
        "textDocument/inlayHint",
        json!({
            "textDocument": {"uri": test_uri.clone()},
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 20, "character": 0}}
        }),
    ) {
        Ok(v) => check!("L5 inlayHint", v.is_array(), format!("{} hints", as_list(&v).len())),
        Err(e) => check!("L5 inlayHint", false, e.to_string()),
    }

    match client.request(
        "textDocument/codeLens",
        json!({"textDocument": {"uri": test_uri.clone()}}),
    ) {
        Ok(v) => check!("L5 codeLens", v.is_array(), format!("{} lens", as_list(&v).len())),
        Err(e) => check!("L5 codeLens", false, e.to_string()),
    }

    // ===== L6 深挖 =====
    // diagnostic: 累计推送可能已被 L6 之前的查询驱动消费, 此处 didChange 触发新推送 + 请求式兜底
    // 坑1: publishDiagnostics 是异步推送, 必须靠 request 驱动读循环才捕获
    // 坑2: 若在测试中途 clear 掉 pushed_diagnostics, 早已推送的诊断会丢失,
    //      应对比 before/after 累计值而非清零后只看新增
    let before: usize = client.pushed_diagnostics.iter().map(|d| d.len()).sum();
    let _ = client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": broken_uri.clone(), "version": 2},
            "contentChanges": [{"text": "package com.trigram.zero.flow;\npublic class __LspBroken { void x() { Integer i = \"str\"; } }\n"}]
        }),
    );
    let mut req_problems = 0;
    for _ in 0..3 {
        std::thread::sleep(Duration::from_secs(4));
        if let Ok(v) = client.request(
            "textDocument/diagnostic",
            json!({"textDocument": {"uri": broken_uri.clone()}}),
        ) {
            req_problems = v
                .get("items")
                .and_then(Value::as_array)
                .map(|a| a.len())
                .unwrap_or(0);
            if req_problems > 0 {
                break;
            }
        }
    }
    let after: usize = client.pushed_diagnostics.iter().map(|d| d.len()).sum();
    let total = req_problems + after;
    check!(
        "L6 diagnostic(类型错误)",
        total > before && total > 0,
        format!("累计推送 {} + 请求 {} problems", after, req_problems)
    );

    // codeAction
    let items = client.pushed_diagnostics.last().cloned().unwrap_or_default();
    let diag = items.first().cloned().unwrap_or(Value::Null);
    match client.request(
        "textDocument/codeAction",
        json!({
            "textDocument": {"uri": broken_uri.clone()},
            "range": {"start": {"line": 1, "character": 30}, "end": {"line": 1, "character": 40}},
            "context": {"diagnostics": [diag]}
        }),
    ) {
        Ok(v) => {
            let n = as_list(&v).len();
            check!("L6 codeAction(修复建议)", n > 0, format!("{} actions", n));
        }
        Err(e) => check!("L6 codeAction(修复建议)", false, e.to_string()),
    }

    // didChange + diagnostic (修复后清零)
    client.pushed_diagnostics.clear();
    let _ = client.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": broken_uri.clone(), "version": 2},
            "contentChanges": [{"text": "package com.trigram.zero.flow;\npublic class __LspBroken { void x() { Integer i = 1; } }\n"}]
        }),
    );
    std::thread::sleep(Duration::from_secs(4));
    let last = client.pushed_diagnostics.last().cloned().unwrap_or_default();
    check!("L6 didChange+diagnostic(修复后)", last.is_empty(), format!("{} problems", last.len()));

    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&broken);

    reports
}
