//! 网页正文提取与格式转换
//!
//! 对标 pi-web-access 的 extractViaHttp 流程：
//! 1. `scraper` 解析 HTML（基于 html5ever，浏览器级解析）
//! 2. 启发式正文提取（模拟 Mozilla Readability 算法）
//! 3. `htmd` 转换 HTML → Markdown
//! 4. JS 渲染页面检测

use crate::error::{AiCliError, Result};
use scraper::{Html, Selector};
use serde::Serialize;
use url::Url;
use tracing::{debug, info, warn};

/// 提取后的内容结构
#[derive(Debug, Clone, Serialize)]
pub struct ExtractedContent {
    /// 页面标题
    pub title: String,
    /// 正文内容（Markdown 格式）
    pub content: String,
    /// 原始 URL
    pub url: String,
    /// 最终 URL（可能经过重定向）
    pub final_url: Option<String>,
    /// HTTP 状态码
    pub status_code: Option<u16>,
    /// Content-Type 响应头
    pub content_type: Option<String>,
    /// 获取模式: "static_http" | "headless_browser"
    pub fetch_mode: String,
    /// 是否为 JS 渲染页面
    pub is_js_rendered: bool,
    /// 内容长度（字符数）
    pub content_length: usize,
}

/// 检测页面是否疑似 JS 渲染（SPA）
///
/// 启发式规则（与 pi-web-access 一致）：
/// - 可见文本 < 500 字符
/// - 且 script 标签 > 3 个
pub fn is_likely_js_rendered(html: &str) -> bool {
    let text_content = strip_tags(html);
    let script_count = html.matches("<script").count();

    let result = text_content.len() < 500 && script_count > 3;
    if result {
        debug!(
            text_len = text_content.len(),
            script_count,
            "Detected JS-rendered page"
        );
    }
    result
}

/// 去除 HTML 标签，提取纯文本
fn strip_tags(html: &str) -> String {
    // 移除 script 和 style 内容
    let no_scripts = regex_lite_strip(html, r"<script[\s\S]*?</script>");
    let no_styles = regex_lite_strip(&no_scripts, r"<style[\s\S]*?</style>");
    // 移除所有标签
    let no_tags = regex_lite_strip(&no_styles, r"<[^>]+>");
    // 压缩空白
    let cleaned = regex_lite_strip(&no_tags, r"\s+");
    cleaned.trim().to_string()
}

/// 简单的正则替换（避免引入 regex crate，使用标准库字符串操作）
fn regex_lite_strip(input: &str, pattern: &str) -> String {
    let mut result = String::with_capacity(input.len());

    // 简化处理：对于已知模式直接处理
    match pattern {
        r"<script[\s\S]*?</script>" => {
            let mut s = input;
            loop {
                let start = s.to_lowercase().find("<script");
                match start {
                    Some(i) => {
                        result.push_str(&s[..i]);
                        let rest = &s[i..];
                        if let Some(end) = rest.to_lowercase().find("</script>") {
                            s = &rest[end + 9..];
                        } else {
                            break;
                        }
                    }
                    None => {
                        result.push_str(s);
                        break;
                    }
                }
            }
        }
        r"<style[\s\S]*?</style>" => {
            let mut s = input;
            loop {
                let start = s.to_lowercase().find("<style");
                match start {
                    Some(i) => {
                        result.push_str(&s[..i]);
                        let rest = &s[i..];
                        if let Some(end) = rest.to_lowercase().find("</style>") {
                            s = &rest[end + 8..];
                        } else {
                            break;
                        }
                    }
                    None => {
                        result.push_str(s);
                        break;
                    }
                }
            }
        }
        r"<[^>]+>" => {
            let mut s = input;
            loop {
                match s.find('<') {
                    Some(i) => {
                        result.push_str(&s[..i]);
                        if let Some(end) = s[i..].find('>') {
                            s = &s[i + end + 1..];
                        } else {
                            break;
                        }
                    }
                    None => {
                        result.push_str(s);
                        break;
                    }
                }
            }
        }
        r"\s+" => {
            let mut prev_space = false;
            for ch in input.chars() {
                if ch.is_whitespace() {
                    if !prev_space {
                        result.push(' ');
                        prev_space = true;
                    }
                } else {
                    result.push(ch);
                    prev_space = false;
                }
            }
        }
        _ => result = input.to_string(),
    }

    result
}

/// 从 HTML 中提取标题
fn extract_title(_html: &str, document: &Html) -> String {
    // 优先从 <title> 标签获取
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = document.select(&sel).next() {
            let title = el.text().collect::<String>();
            // 去除站点名后缀（如 " - SiteName"）
            let cleaned = title
                .split(" | ")
                .next()
                .unwrap_or(&title)
                .split(" - ")
                .next()
                .unwrap_or(&title)
                .trim()
                .to_string();
            if !cleaned.is_empty() {
                return cleaned;
            }
        }
    }

    // 回退到 <h1>
    if let Ok(sel) = Selector::parse("h1") {
        if let Some(el) = document.select(&sel).next() {
            let text = el.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }
    }

    // 最终回退：URL 路径最后一段
    "Untitled".to_string()
}

/// ── 正文提取───────────────
/// Readability 风格的两阶段算法：
/// 1. 快速路径：CSS 选择器探测常见容器
/// 2. 评分路径：对 `<p>` 元素逐段打分，找内容簇

fn extract_main_content(document: &Html) -> String {
    // Phase 1: 快速路径 — 命中常见语义容器直接返回
    if let Some(html) = try_common_selectors(document) {
        if html.len() > 200 {
            debug!("Fast path: found content by selector");
            return html;
        }
    }

    // Phase 2: Readability 式段落评分
    let result = extract_by_scoring(document);
    if !result.is_empty() {
        debug!("Used scoring path, content length: {}", result.len());
        return result;
    }

    // Phase 3: 兜底 — body 全部内容
    if let Ok(sel) = Selector::parse("body") {
        if let Some(el) = document.select(&sel).next() {
            let html = el.inner_html();
            if !html.trim().is_empty() {
                warn!("Falling back to full body content");
                return html;
            }
        }
    }

    String::new()
}

/// 快速路径：常见正文容器 CSS 选择器探测
fn try_common_selectors(document: &Html) -> Option<String> {
    let selectors = [
        "article",
        "[role=\"main\"]",
        "main",
        ".post-content",
        ".article-content",
        ".entry-content",
        ".content",
        ".post",
        ".article",
        "#content",
        "#main",
        "#article",
        ".markdown-body",
        ".prose",
    ];

    for s in &selectors {
        if let Ok(sel) = Selector::parse(s) {
            if let Some(el) = document.select(&sel).next() {
                let html = el.inner_html();
                if !html.trim().is_empty() && html.len() > 100 {
                    return Some(html);
                }
            }
        }
    }
    None
}

/// 段落评分提取（Readability 核心算法简化版）
///
/// 给每个 `<p>` 打分 → 找最高分段落 → 扩展为连续内容簇
fn extract_by_scoring(document: &Html) -> String {
    let p_sel = match Selector::parse("p") {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let a_sel = match Selector::parse("a") {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    struct Para {
        score: f64,
        html: String,
    }

    let mut paras: Vec<Para> = Vec::new();

    for p in document.select(&p_sel) {
        let text: String = p.text().collect();
        let text = text.trim();

        // 过滤过短段落
        if text.len() < 25 {
            continue;
        }

        let inner = p.inner_html();

        // 链接密度：链接文本占段落总 HTML 的比例
        let link_density = a_sel.as_ref().and_then(|a_sel| {
            let link_len: usize = p.select(a_sel).flat_map(|a| a.text()).map(|t| t.len()).sum();
            if inner.is_empty() { None } else { Some(link_len as f64 / inner.len() as f64) }
        }).unwrap_or(1.0);

        // ═══ 核心评分公式 ═══
        // 1. 文本长度分：ln(text_len) * 10，长段落加分
        let length_score = (text.len() as f64).ln_1p() * 10.0;

        // 2. 标点密度分：逗号/句号/问号/感叹号是自然语言正文特征
        let punct_count = text.chars().filter(|c| matches!(c, '，' | '。' | ',' | '.' | '！' | '？' | '!' | '?' | '；' | ';' | '、')).count() as f64;
        let punct_score = punct_count * 8.0;

        // 3. 链接密度罚分：链接越多越可能是导航/广告
        let density_penalty = link_density * 80.0;

        // 4. 平均词长分（可选）：中英文混合更可能是正文
        let avg_word_len = text.len() as f64 / text.split_whitespace().count().max(1) as f64;
        let word_score = if avg_word_len > 2.0 { 5.0 } else { 0.0 };

        let score = length_score + punct_score + word_score - density_penalty;

        if score > 0.0 {
            paras.push(Para { score, html: inner });
        }
    }

    if paras.is_empty() {
        return String::new();
    }

    // 找到最高分段落
    let best_idx = paras.iter()
        .enumerate()
        .max_by(|a, b| a.1.score.partial_cmp(&b.1.score).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    // 从最高分向两侧扩展，纳入相近分数的相邻段落
    let threshold = paras[best_idx].score * 0.25;
    let mut start = best_idx;
    while start > 0 && paras[start - 1].score >= threshold {
        start -= 1;
    }
    let mut end = best_idx;
    while end < paras.len() - 1 && paras[end + 1].score >= threshold {
        end += 1;
    }

    let joined: String = paras[start..=end].iter().map(|p| p.html.as_str()).collect::<Vec<_>>().join("\n");

    if joined.len() > 50 {
        format!("<div>{}</div>", joined)
    } else {
        String::new()
    }
}

/// 提取内容构建参数
#[derive(Debug, Default)]
pub struct ExtractOptions<'a> {
    /// 最终请求 URL（重定向后）
    pub final_url: Option<String>,
    /// HTTP 状态码
    pub status_code: Option<u16>,
    /// Content-Type 响应头
    pub content_type: Option<String>,
    /// 获取模式
    pub fetch_mode: Option<&'a str>,
}

/// 从原始 HTML 提取结构化内容（带可选元数据）
///
/// 流水线: readability（主） → scraper（回退） → htmd（转 Markdown）
pub fn extract_content(html: &str, url: &str, opts: &ExtractOptions) -> Result<ExtractedContent> {
    info!(%url, bytes = html.len(), "Starting content extraction");

    let (title, main_html, used_readability) = try_readability(html, url)
        .unwrap_or_else(|| {
            info!("Readability failed, falling back to scraper extraction");
            let document = Html::parse_document(html);
            let title = extract_title(html, &document);
            let main_html = extract_main_content(&document);
            (title, main_html, false)
        });

    if main_html.is_empty() {
        return Err(AiCliError::Extraction(
            "Could not extract readable content from HTML structure".into(),
        ));
    }

    // HTML → Markdown
    let markdown = htmd::convert(&main_html)
        .unwrap_or_else(|_| {
            warn!("htmd conversion failed, falling back to plain text");
            strip_tags(&main_html)
        });

    let is_js = is_likely_js_rendered(html);
    let content_length = markdown.len();

    info!(
        %title,
        content_length,
        is_js,
        used_readability = used_readability,
        "Content extraction completed"
    );

    Ok(ExtractedContent {
        title,
        content: markdown,
        url: url.to_string(),
        final_url: opts.final_url.clone().or_else(|| Some(url.to_string())),
        status_code: opts.status_code,
        content_type: opts.content_type.clone(),
        fetch_mode: opts.fetch_mode.unwrap_or("static_http").to_string(),
        is_js_rendered: is_js,
        content_length,
    })
}

/// 尝试用 Readability 算法提取正文
/// 返回 (title, html_content, used_readability)
fn try_readability(html: &str, url: &str) -> Option<(String, String, bool)> {
    let parsed_url = Url::parse(url).ok()?;
    let mut bytes = html.as_bytes();

    let product = readability::extractor::extract(&mut bytes, &parsed_url).ok()?;

    let content = product.content.trim();
    if content.is_empty() || content.len() < 50 {
        return None;
    }

    debug!("Readability extracted: title={}, content_len={}", product.title, content.len());
    Some((product.title, content.to_string(), true))
}
