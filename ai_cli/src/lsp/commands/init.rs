//! 初始化序列: initialize + initialized + didOpen + classpath 自动生成
//!
//! # javac-lsp classpath 避坑指南
//!
//! 1. javac-lsp 不像 jdtls 能自动从 pom.xml 推断完整 classpath。
//!    必须通过 `workspace/didChangeConfiguration` 发送 `java.classPath` 才能解析类型。
//!
//! 2. 如果手动设置了 classPath，javac-ls 会跳过 InferConfig 自动推断（包括 JDK 类）。
//!    所以 classPath 必须包含 JDK 核心 jar，否则连 String 都解析不了。
//!
//! 3. 如果不设 classPath，InferConfig 会自动从 pom.xml / BUILD 推断，
//!    但仅限 Maven/Gradle 项目，裸 Java 文件无法推断。
//!
//! 4. Windows 下 mvn 是 POSIX shell 脚本，Rust Command 无法直接执行，
//!    必须用 `mvn.cmd` 而非 `mvn`。
//!
//! 5. classpath 文件缓存到 target/classpath.txt，依赖不变就不重新生成。

use crate::lsp::client::LspClient;
use crate::lsp::transport::LspTransport;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// 打开文件并返回 URI (初始化已在 LspClient::start 中完成)
pub fn open_file(client: &mut LspClient<impl LspTransport>, file: &Path) -> Result<String> {
    client.open_file(file)
}

/// 自动生成 Maven classpath 文件 (如果不存在)
/// 返回 classpath 条目列表
pub fn auto_classpath(project_dir: &Path) -> Option<Vec<String>> {
    let pom = project_dir.join("pom.xml");
    if !pom.exists() {
        return None;
    }

    let cp_file = project_dir.join("target").join("classpath.txt");

    // 已存在则直接读
    if cp_file.exists() {
        return read_classpath_file(&cp_file);
    }

    // 不存在则生成
    eprintln!("[javac-lsp] 正在生成 classpath (mvn dependency:build-classpath)...");
    let mvn_cmd = if cfg!(windows) { "mvn.cmd" } else { "mvn" };
    let result = Command::new(mvn_cmd)
        .args([
            "dependency:build-classpath",
            "-q",
            "-Dmdep.outputFile=target/classpath.txt",
        ])
        .current_dir(project_dir)
        .output();

    match result {
        Ok(output) if output.status.success() => {
            eprintln!("[javac-lsp] classpath 生成成功");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("[javac-lsp] mvn 失败. stderr: {} stdout: {}", stderr.trim(), stdout.trim());
            return None;
        }
        Err(e) => {
            eprintln!("[javac-lsp] 无法启动 mvn: {}", e);
            return None;
        }
    }

    read_classpath_file(&cp_file)
}

fn read_classpath_file(path: &Path) -> Option<Vec<String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let entries: Vec<String> = content
        .trim()
        .split(';')
        .filter(|p| !p.is_empty())
        .map(|p| p.trim().to_string())
        .collect();
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}
