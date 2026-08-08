//! LSP JSON-RPC 帧编解码
//!
//! 协议格式: `Content-Length: <N>\r\n\r\n<JSON body>`（字节精确，UTF-8）

use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout};

/// 发送一条 JSON-RPC 消息
pub fn write_msg(w: &mut ChildStdin, msg: &Value) -> anyhow::Result<()> {
    let body = serde_json::to_vec(msg)?;
    w.write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())?;
    w.write_all(&body)?;
    w.flush()?;
    Ok(())
}

/// 读取一条 JSON-RPC 消息（阻塞；连接关闭返回 Err）
pub fn read_msg(r: &mut BufReader<ChildStdout>) -> anyhow::Result<Value> {
    read_msg_from(r)
}

/// 从任意 Read 读取一条消息（内部实现，供 BufReader<ChildStdout> 使用）
pub fn read_msg_from(r: &mut impl BufRead) -> anyhow::Result<Value> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = r.read_line(&mut line)?;
        if n == 0 {
            anyhow::bail!("LSP 服务器连接已关闭 (EOF)");
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            content_length = Some(
                v.trim()
                    .parse::<usize>()
                    .map_err(|e| anyhow::anyhow!("Content-Length 解析失败: {} ({})", v, e))?,
            );
        }
    }
    let len = content_length.ok_or_else(|| anyhow::anyhow!("消息缺少 Content-Length 头"))?;
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body)?)
}
