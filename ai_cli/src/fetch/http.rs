//! 静态 HTTP 网页获取
//!
//! 使用 `reqwest` 发送 HTTP GET 请求，模拟真实浏览器请求头，
//! 获取原始 HTML 内容。适用于服务端渲染 (SSR) 的静态页面。

use crate::error::{AiCliError, Result};
use reqwest::header;
use std::time::Duration;
use tracing::{debug, info};

/// 浏览器级默认请求头
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36 Edg/122.0.0.0";

/// HTTP 获取结果
#[derive(Debug)]
pub struct FetchResult {
    /// 原始 HTML 内容
    pub html: String,
    /// 最终 URL（可能经过重定向）
    pub final_url: String,
    /// HTTP 状态码
    pub status: u16,
    /// Content-Type 响应头
    pub content_type: Option<String>,
}

/// 通过 HTTP GET 获取网页原始 HTML
///
/// # 参数
/// - `url`: 目标网页 URL
/// - `timeout_secs`: 请求超时时间（秒）
///
/// # 返回
/// 包含原始 HTML 和元数据的 `FetchResult`
///
/// # 错误
/// - URL 解析失败
/// - 网络请求超时
/// - HTTP 非 2xx 状态码
/// - 响应体过大（> 5MB）
pub async fn fetch_static(url: &str, timeout_secs: u64) -> Result<FetchResult> {
    info!(%url, "Starting static HTTP fetch");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(USER_AGENT)
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::ACCEPT,
                header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"),
            );
            headers.insert(
                header::ACCEPT_LANGUAGE,
                header::HeaderValue::from_static("zh-CN,zh;q=0.9,en;q=0.8"),
            );
            headers.insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("no-cache"),
            );
            headers
        })
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let response = client.get(url).send().await?;

    let status = response.status().as_u16();
    let final_url = response.url().to_string();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    debug!(%status, %final_url, ?content_type, "Received HTTP response");

    if !response.status().is_success() {
        return Err(AiCliError::General(format!(
            "HTTP {}: {}",
            status,
            response.status().canonical_reason().unwrap_or("Unknown")
        )));
    }

    // 检查内容类型
    if let Some(ref ct) = content_type {
        if ct.contains("application/octet-stream")
            || ct.contains("image/")
            || ct.contains("audio/")
            || ct.contains("video/")
            || ct.contains("application/zip")
        {
            return Err(AiCliError::General(format!(
                "Unsupported content type: {}",
                ct.split(';').next().unwrap_or(ct)
            )));
        }
    }

    // 限制响应大小 5MB
    let html = response.text().await?;
    if html.len() > 5 * 1024 * 1024 {
        return Err(AiCliError::General(format!(
            "Response too large: {} bytes",
            html.len()
        )));
    }

    info!(bytes = html.len(), "Static fetch completed");

    Ok(FetchResult {
        html,
        final_url,
        status,
        content_type,
    })
}
