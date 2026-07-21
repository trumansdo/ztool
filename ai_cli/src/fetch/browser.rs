//! Headless 浏览器网页获取
//!
//! 使用 `headless_chrome` 驱动 Chromium 内核浏览器（Edge/Chrome），
//! 执行 JavaScript 渲染后获取完整 DOM。适用于 SPA 动态页面。
//!
//! # 浏览器检测优先级
//! 1. 用户指定的 `--browser` 路径
//! 2. 环境变量 `CHROME` 或 `EDGE`
//! 3. 自动搜索: Edge → Chrome → Chromium

use crate::error::{AiCliError, Result};
use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use tracing::{debug, info};

/// 浏览器获取结果
#[derive(Debug)]
pub struct BrowserFetchResult {
    /// JS 渲染后的完整 HTML
    pub html: String,
    /// 页面标题
    #[allow(dead_code)]
    pub title: String,
    /// 最终 URL
    pub final_url: String,
}

/// 自动检测系统中可用的浏览器可执行文件路径
///
/// 按优先级搜索: Edge → Chrome → Chromium
fn detect_browser() -> Option<PathBuf> {
    // 检查环境变量
    for env_var in &["CHROME", "EDGE", "BROWSER"] {
        if let Ok(path) = std::env::var(env_var) {
            let p = PathBuf::from(&path);
            if p.exists() {
                debug!(?p, env_var, "Found browser via env var");
                return Some(p);
            }
        }
    }

    // Windows 常见安装路径
    let candidates = [
        // Edge 稳定版
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        // Edge Beta/Dev/Canary
        r"C:\Program Files (x86)\Microsoft\Edge Beta\Application\msedge.exe",
        r"C:\Program Files (x86)\Microsoft\Edge Dev\Application\msedge.exe",
        // Chrome
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        // Chromium
        r"C:\Users\*\AppData\Local\Chromium\Application\chrome.exe",
    ];

    for candidate in &candidates {
        // 展开通配符
        let expanded = if candidate.contains('*') {
            expand_user_path(candidate)
        } else {
            Some(PathBuf::from(candidate))
        };

        if let Some(ref p) = expanded {
            if p.exists() {
                debug!(?p, "Found browser at known path");
                return Some(p.clone());
            }
        }
    }

    // PATH 中搜索
    for name in &["msedge", "chrome", "chromium", "chromium-browser"] {
        if let Ok(path) = which::which(name) {
            debug!(?path, name, "Found browser in PATH");
            return Some(path);
        }
    }

    None
}

/// 展开路径中的 `*` 通配符（简单处理：替换为当前用户名）
fn expand_user_path(path: &str) -> Option<PathBuf> {
    if let Ok(home) = std::env::var("USERPROFILE") {
        let expanded = path.replace("*", &home);
        let p = PathBuf::from(&expanded);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// 使用 headless 浏览器获取网页渲染后的 HTML
///
/// # 参数
/// - `url`: 目标网页 URL
/// - `browser_path`: 自定义浏览器路径，`None` 时自动检测
/// - `timeout_secs`: 页面加载超时（秒）
///
/// # 返回
/// 包含渲染后 HTML 的 `BrowserFetchResult`
pub fn fetch_with_browser(
    url: &str,
    browser_path: Option<&str>,
    timeout_secs: u64,
) -> Result<BrowserFetchResult> {
    let executable = match browser_path {
        Some(p) => {
            let path = PathBuf::from(p);
            if !path.exists() {
                return Err(AiCliError::Browser(format!(
                    "Specified browser not found: {}",
                    p
                )));
            }
            path
        }
        None => detect_browser().ok_or_else(|| {
            AiCliError::Browser(
                "No Chromium-based browser found. Install Edge or Chrome, \
                 or specify path with --browser."
                    .into(),
            )
        })?,
    };

    info!(?executable, %url, "Launching headless browser");

    let options = LaunchOptions {
        path: Some(executable.to_string_lossy().into_owned().into()),
        headless: true,
        sandbox: false, // Windows 上通常需要关闭沙箱
        window_size: Some((1920, 1080)),
        idle_browser_timeout: std::time::Duration::from_secs(timeout_secs),
        ..LaunchOptions::default()
    };

    let browser =
        Browser::new(options).map_err(|e| AiCliError::Browser(format!("Failed to launch browser: {}", e)))?;

    let tab = browser
        .new_tab()
        .map_err(|e| AiCliError::Browser(format!("Failed to create tab: {}", e)))?;

    tab.navigate_to(url)
        .map_err(|e| AiCliError::Browser(format!("Failed to navigate: {}", e)))?;

    tab.wait_until_navigated()
        .map_err(|e| AiCliError::Browser(format!("Navigation timeout: {}", e)))?;

    // 额外等待 JS 渲染完成
    std::thread::sleep(std::time::Duration::from_secs(2));

    let html = tab
        .get_content()
        .map_err(|e| AiCliError::Browser(format!("Failed to get page content: {}", e)))?;

    let title = tab
        .get_title()
        .unwrap_or_default();

    let final_url = tab
        .get_url();

    info!(
        bytes = html.len(),
        %title,
        %final_url,
        "Browser fetch completed"
    );

    Ok(BrowserFetchResult {
        html,
        title,
        final_url,
    })
}
