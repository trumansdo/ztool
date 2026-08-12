//! JSON-RPC 传输抽象层
//!
//! 提供 `LspTransport` trait 和 stdio 实现，与 lsp-types 解耦。

use anyhow::Result;
use std::io::{BufRead, BufReader, Read, Write};

/// LSP 传输抽象: 读/写 JSON-RPC 消息帧
pub trait LspTransport {
    /// 读取一条完整的 JSON-RPC 消息 (Content-Length 帧)
    fn read_message(&mut self) -> Result<String>;

    /// 写入一条 JSON-RPC 消息 (自动添加 Content-Length 头)
    fn write_message(&mut self, json: &str) -> Result<()>;
}

// ─── Stdio 实现 ───────────────────────────────────────

/// 基于 stdio 的传输实现 (连接外部 LSP 进程)
pub struct StdioTransport {
    reader: BufReader<Box<dyn Read + Send>>,
    writer: Box<dyn Write + Send>,
}

impl StdioTransport {
    pub fn new(reader: impl Read + Send + 'static, writer: impl Write + Send + 'static) -> Self {
        Self {
            reader: BufReader::new(Box::new(reader)),
            writer: Box::new(writer),
        }
    }
}

impl LspTransport for StdioTransport {
    fn read_message(&mut self) -> Result<String> {
        // 读 Header: "Content-Length: N\r\n\r\n"
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            if line == "\r\n" {
                break;
            }
            if let Some(len) = line
                .strip_prefix("Content-Length: ")
                .or_else(|| line.strip_prefix("content-length: "))
            {
                content_length = Some(len.trim().parse()?);
            }
        }
        let len = content_length.ok_or_else(|| anyhow::anyhow!("缺少 Content-Length 头"))?;

        // 读 Body
        let mut body = vec![0u8; len];
        self.reader.read_exact(&mut body)?;
        Ok(String::from_utf8(body)?)
    }

    fn write_message(&mut self, json: &str) -> Result<()> {
        let bytes = json.as_bytes();
        let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
        self.writer.write_all(header.as_bytes())?;
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }
}
