//! 网页获取模块
//!
//! 提供两种获取策略：
//! - `http`: 轻量级静态 HTTP 请求（reqwest）
//! - `browser`: Headless 浏览器渲染（headless_chrome + Edge）
//!
//! 以及统一的正文提取流水线 `extract`。

pub mod browser;
pub mod extract;
pub mod http;
