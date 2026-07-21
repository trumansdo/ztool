//! 统一错误类型定义
//!
//! 使用 `thiserror` 派生宏定义所有可能的错误变体，
//! 通过 `From` 自动转换实现与 `?` 运算符的无缝集成。

use thiserror::Error;

/// AI CLI 工具的所有错误类型
#[derive(Error, Debug)]
pub enum AiCliError {
    /// HTTP 请求失败
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// 浏览器启动或操作失败
    #[error("Browser error: {0}")]
    Browser(String),

    /// URL 解析失败
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// 内容提取失败
    #[error("Content extraction failed: {0}")]
    Extraction(String),

    /// 文件 I/O 失败
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化失败
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 配置错误
    #[error("Config error: {0}")]
    Config(String),

    /// AI API 错误
    #[error("AI API error: {0}")]
    AiApi(String),

    /// 通用错误
    #[error("{0}")]
    General(String),
}

/// 便捷类型别名
pub type Result<T> = std::result::Result<T, AiCliError>;
