//! # 网络数据包解析模块
//!
//! 提供 TCP/UDP/HTTP 数据包解析功能，模拟 Wireshark 的核心解析逻辑。
//!
//! ## Rust 概念 — 模块系统
//! 这个文件对应 `pub mod parser;` 声明在 net_capture.rs 中。
//! Rust 编译器自动查找同名文件 `parser.rs` 或目录 `parser/mod.rs`。
//!
//! ## Rust 概念 — `use std::collections::HashMap;`
//! `std::collections` 是标准库的集合模块，HashMap 是其中的哈希表类型。
//! 与 Vec（连续数组）不同，HashMap 通过键的哈希值进行 O(1) 查找。
//! 类比：Python 的 `dict`、Java 的 `HashMap<K,V>`、C++ 的 `std::unordered_map`。
//!
//! # 调用链 (从深到浅)
//! ```
//! HttpInfo::parse_response / parse_request (第0层 叶子函数)
//!   ↑
//! HttpInfo::summary (第1层)
//! parse_tcp_packet / parse_udp_packet (第1层)
//!   ↑
//! format_packet_info (第2层)
//!   ↑
//! PacketCapture::format_packets (第3层)
//! ```

use std::collections::HashMap;

/// 单个网络数据包的完整信息
///
/// ## Rust 概念 — 结构体字段定义
/// `pub field_name: type` — 每个字段的类型必须显式标注（Rust 不支持类型推断）。
/// `u16` — 16 位无符号整数（0~65535），端口号不可能是负数。
/// `usize` — 指针大小的无符号整数，与平台相关（64位系统为 u64）。
///
/// ## Rust 概念 — Option<T> (可选类型)
/// `pub http_info: Option<HttpInfo>` — 不是每个数据包都是 HTTP，所以用 Option 表示「可能没有」。
/// Option 只有两个变体：`Some(HttpInfo)` 或 `None`。
/// Rust 没有 null，Option 提供了编译期安全的「可选值」机制。
///
/// ## Rust 概念 — String vs &str
/// `pub src_ip: String` — String 是堆分配的、可变的、拥有所有权的字符串。
/// `&str` 是借用的、不可变的字符串切片（后续代码中会见到）。
#[derive(Debug, Clone, Default)]
pub struct PacketInfo {
    /// 时间戳（格式如 "12:34:56.789"）
    pub timestamp: String,
    /// 源 IP 地址（如 "192.168.1.100"）
    pub src_ip: String,
    /// 目标 IP 地址
    pub dst_ip: String,
    /// 源端口 (0-65535，u16 保证不会溢出)
    pub src_port: u16,
    /// 目标端口（80=HTTP, 443=HTTPS, 53=DNS...）
    pub dst_port: u16,
    /// 协议类型 ("TCP", "UDP", "HTTP", "HTTPS", "DNS")
    pub protocol: String,
    /// 数据长度（字节数），usize 在 64 位系统上等同于 u64
    pub length: usize,
    /// HTTP 层信息，仅 HTTP/HTTPS 时非 None
    pub http_info: Option<HttpInfo>,
}

/// HTTP 协议的详细信息
///
/// ## Rust 概念 — HashMap<K, V>
/// `HashMap<String, String>` — 键和值都是 String 的哈希表。
/// 用于存储 HTTP 头信息如 `"Content-Type" -> "application/json"`。
/// 不使用字段而用 HashMap 是因为 HTTP 头的名称是动态的（Host, User-Agent 等）。
#[derive(Debug, Clone, Default)]
pub struct HttpInfo {
    /// 请求方法 ("GET", "POST", "PUT", "DELETE" 等)
    pub method: Option<String>,
    /// 请求的路径 ("/api/users", "/index.html")
    pub uri: Option<String>,
    /// HTTP 版本 ("HTTP/1.1", "HTTP/2.0")
    pub version: Option<String>,
    /// HTTP 头部键值对 (Content-Type, Host, Cookie...)
    pub headers: HashMap<String, String>,
    /// 请求或响应体（POST 的 JSON 数据、响应的 HTML 等）
    pub body: Option<String>,
    /// HTTP 响应状态码 (200=OK, 404=Not Found, 500=Server Error)
    pub response_code: Option<u16>,
}

impl HttpInfo {
    /// 解析 HTTP 请求数据
    ///
    /// ## Rust 概念 — `&[u8]` 字节切片
    /// `data: &[u8]` — 对字节数组的不可变引用（切片类型）。
    /// 网络数据以原始字节流形式传输，u8 是 Rust 的字节类型。
    ///
    /// ## Rust 概念 — `String::from_utf8_lossy`
    /// 将字节转换为 UTF-8 字符串。`lossy` 表示遇到非法 UTF-8 字节时
    /// 替换为 U+FFFD（替换字符）而不是报错。适合处理可能非法的网络数据。
    ///
    /// ## Rust 概念 — `Vec<&str>` 与 `.collect()`
    /// `text.lines().collect()` — lines() 返回迭代器，collect() 把迭代器元素收集为 Vec。
    /// `Vec<&str>` 的类型是编译器通过左侧变量类型标注推断的（类型推断）。
    ///
    /// ## Rust 概念 — `if let` 模式匹配
    /// `if let Some(value) = expression { }` — 仅当表达式是 Some 时才执行。
    /// 等价于 match 的单个分支 + 忽略其他。语法更简洁。
    ///
    /// ## Rust 概念 — `split_whitespace()`
    /// 按空白字符（空格、制表符、换行）分割字符串。
    /// 类比 Python 的 `.split()`（无参数版本）。
    ///
    /// ## Rust 概念 — `line.find(':')`
    /// 返回 Option<usize>，找到时返回 Some(位置)，未找到返回 None。
    ///
    /// ## Rust 概念 — 字符串切片 `line[..colon_pos]`
    /// Rust 使用范围语法 `start..end` 对字符串/数组切片。
    /// `..colon_pos` 是从开头到 colon_pos（不含），`colon_pos + 1..` 是从 colon_pos+1 到结尾。
    /// 注意：切片单位是**字节**而非字符，对 ASCII 文本安全，对多字节 Unicode 可能 panic。
    ///
    /// ## Rust 概念 — `&str` vs `String`
    /// `.trim()` 返回 `&str`（切片引用，不分配新内存），
    /// `.to_string()` 创建新的堆分配 String。当需要所有权时使用 to_string()。
    ///
    /// # 解析格式
    /// ```http
    /// GET /api/users HTTP/1.1\r\n       ← 请求行 (方法 URI 版本)
    /// Host: example.com\r\n              ← 请求头 (Key: Value)
    /// Content-Type: application/json\r\n
    /// \r\n                               ← 空行分隔
    /// {"key": "value"}                   ← 请求体
    /// ```
    pub fn parse_request(data: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.lines().collect();
        
        if lines.is_empty() {
            return None;  // return 提前返回，简化控制流
        }
        
        let mut info = HttpInfo::default();
        
        // 解析请求行: "GET /path HTTP/1.1"
        // lines.first() 返回 Option<&&str>，if let Some 解构双层引用
        if let Some(request_line) = lines.first() {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() >= 3 {
                // parts[0] 是 &str，.to_string() 转为 String 以拥有所有权
                info.method = Some(parts[0].to_string());
                info.uri = Some(parts[1].to_string());
                info.version = Some(parts[2].to_string());
            }
        }
        
        // 解析请求头 (跳过请求行，从第 1 行开始)
        // .skip(1) 跳过迭代器的第一个元素
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;  // 空行表示头部结束，后面是请求体
            }
            // 查找冒号位置来分割 key: value
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                // HashMap::insert — 插入键值对，如果键已存在则覆盖旧值
                info.headers.insert(key, value);
            }
        }
        
        // 解析请求体 (HTTP 使用 \r\n 或 \n 作为行分隔符)
        // \r\n\r\n 标记头/体分隔（CRLF = Carriage Return + Line Feed）
        if let Some(body_start) = text.find("\r\n\r\n") {
            let body = text[body_start + 4..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        } else if let Some(body_start) = text.find("\n\n") {
            // 备选：仅 \n 分隔（某些实现不使用 \r）
            let body = text[body_start + 2..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        }
        
        Some(info)
    }
    
    /// 解析 HTTP 响应数据
    ///
    /// ## Rust 概念 — 泛型解析 `parse::<u16>()`
    /// `parts[1].parse::<u16>()` — `::<u16>` 是 turbofish 语法，
    /// 显式指定泛型参数类型。等价于 `let code: u16 = parts[1].parse()?`。
    /// `parse()` 返回 `Result<T, ParseIntError>`。
    ///
    /// ## Rust 概念 — `if let Ok(code) = expr`
    /// 解构 Result 的 Ok 变体，忽略 Err 情况。
    /// 这里用于忽略非数字的状态码文本（极少数情况下会出现）。
    ///
    /// # 解析格式
    /// ```http
    /// HTTP/1.1 200 OK\r\n               ← 状态行 (版本 状态码 原因短语)
    /// Content-Type: text/html\r\n
    /// \r\n
    /// <html>...</html>
    /// ```
    pub fn parse_response(data: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.lines().collect();
        
        if lines.is_empty() {
            return None;
        }
        
        let mut info = HttpInfo::default();
        
        // 解析状态行: "HTTP/1.1 200 OK"
        if let Some(status_line) = lines.first() {
            let parts: Vec<&str> = status_line.split_whitespace().collect();
            if parts.len() >= 2 {
                info.version = Some(parts[0].to_string());
                // turbofish ::<u16> 显式指定解析目标类型
                if let Ok(code) = parts[1].parse::<u16>() {
                    info.response_code = Some(code);
                }
            }
        }
        
        // 解析响应头 (逻辑与 parse_request 相同)
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                info.headers.insert(key, value);
            }
        }
        
        // 解析响应体
        if let Some(body_start) = text.find("\r\n\r\n") {
            let body = text[body_start + 4..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        } else if let Some(body_start) = text.find("\n\n") {
            let body = text[body_start + 2..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        }
        
        Some(info)
    }
    
    /// 生成 HTTP 信息摘要（可读的文本格式）
    ///
    /// ## Rust 概念 — `as_ref()` 与 `unwrap_or()`
    /// `self.uri.as_ref().unwrap_or(&String::new())`:
    /// 1. `as_ref()` — Option<String> → Option<&String>（引用转换，不移动所有权）
    /// 2. `unwrap_or(&String::new())` — 若是 None 则返回空字符串的引用
    /// 这是处理 `Option<String>` 并获取 `&str` 的惯用方式。
    ///
    /// ## Rust 概念 — `ref` 关键字
    /// `if let Some(ref method) = self.method` — `ref` 表示按引用绑定而非移动。
    /// 没有 ref 时 `Some(method)` 会将 String 移出 Option（消耗所有权），
    /// 有了 ref 则 `method` 是 `&String`，所有权保留在 Option 中。
    ///
    /// ## Rust 概念 — HashMap::get()
    /// `self.headers.get("Host")` 返回 `Option<&String>`。
    /// 与 `[]` 索引不同，get() 不 panic，安全地返回 None。
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        // 请求信息
        if let Some(ref method) = self.method {
            parts.push(format!("{} {} {}", method, self.uri.as_ref().unwrap_or(&String::new()), self.version.as_ref().unwrap_or(&String::new())));
        }
        
        // 响应状态
        if let Some(code) = self.response_code {
            parts.push(format!("{} {}", self.version.as_ref().unwrap_or(&String::new()), code));
        }
        
        // Host 头 — HashMap::get() 返回 Option<&V>
        if let Some(host) = self.headers.get("Host") {
            parts.push(format!("Host: {}", host));
        }
        
        // 响应体 (截断前 100 字符，防止超长内容刷屏)
        // `&body[..100]` 是字节切片，对中文等多字节字符可能截断在中间
        if let Some(ref body) = self.body {
            if body.len() > 100 {
                parts.push(format!("Body: {}...", &body[..100]));
            } else {
                parts.push(format!("Body: {}", body));
            }
        }
        
        // Vec::join — 用分隔符连接所有元素
        parts.join(" | ")
    }
}

/// 解析 TCP 数据包 → 识别是否为 HTTP/HTTPS
///
/// ## Rust 概念 — 结构体更新语法 `..Default::default()`
/// ```rust
/// PacketInfo {
///     src_port,          // 简写，等价于 src_port: src_port
///     dst_port,
///     protocol: "TCP".to_string(),
///     length: data.len(),
///     ..Default::default()  // 其余字段使用 Default 值（timestamp "", src_ip "", http_info None 等）
/// }
/// ```
/// `..` 表示「其余字段」，这样可以只设置关心的字段，剩下的用默认值填充。
///
/// ## Rust 概念 — `||` 逻辑或运算符
/// `src_port == 80 || dst_port == 80` — 源端口或目标端口为 80，都可能承载 HTTP。
/// 端口扫描使用 `||` 而不是 `|`（`||` 是短路求值，`|` 是位运算）。
///
/// ## Rust 概念 — Nested if let
/// `if let Some(http_info) = HttpInfo::parse_request(data)` — 在条件判断中解析 HTTP，
/// 如果数据不是 HTTP 格式则静默忽略（不报错、不 panic）。
pub fn parse_tcp_packet(data: &[u8], src_port: u16, dst_port: u16) -> PacketInfo {
    let mut info = PacketInfo {
        src_port,
        dst_port,
        protocol: "TCP".to_string(),
        length: data.len(),  // data.len() 返回字节数，类型为 usize
        ..Default::default()  // 结构体更新语法：其余字段使用默认值
    };
    
    // HTTP 端口探测（80=HTTP, 8080=HTTP 备用）
    if src_port == 80 || dst_port == 80 || src_port == 8080 || dst_port == 8080 {
        if let Some(http_info) = HttpInfo::parse_request(data) {
            info.protocol = "HTTP".to_string();
            info.http_info = Some(http_info);
        }
    } else if src_port == 443 || dst_port == 443 {
        // 443 = HTTPS，内容已加密无法解析
        info.protocol = "HTTPS".to_string();
    }
    
    info
}

/// 解析 UDP 数据包 → 识别 DNS 协议
///
/// ## Rust 概念 — 协议识别策略
/// 这是基于端口的启发式协议识别：DNS 使用 53 端口。
/// 现实中的深度包检测（DPI）会更复杂（分析数据特征、签名匹配等），
/// 但基于端口的识别简单高效，适合大多数情况。
pub fn parse_udp_packet(data: &[u8], src_port: u16, dst_port: u16) -> PacketInfo {
    let mut info = PacketInfo {
        src_port,
        dst_port,
        protocol: "UDP".to_string(),
        length: data.len(),
        ..Default::default()
    };
    
    // DNS 标准端口 53
    if src_port == 53 || dst_port == 53 {
        info.protocol = "DNS".to_string();
    }
    
    info
}

/// 将数据包信息格式化为多行字符串（供 UI 显示）
///
/// ## Rust 概念 — `format!` 宏
/// `format!("[{}] {}:{} -> {}:{} ({} bytes)", ...)` — 与 println! 相同语法，
/// 但返回 String 而非打印到控制台。类比 Python 的 f-string / format()。
///
/// ## Rust 概念 — `ref` 模式
/// `if let Some(ref http) = info.http_info` — 同样是 ref 引用绑定，
/// 避免将 HttpInfo 从 Option 中移出。
pub fn format_packet_info(info: &PacketInfo) -> String {
    let mut lines = Vec::new();
    
    // 第一行：时间戳 + 源 → 目标 + 字节数
    lines.push(format!(
        "[{}] {}:{} -> {}:{} ({} bytes)",
        info.timestamp,
        info.src_ip,
        info.src_port,
        info.dst_ip,
        info.dst_port,
        info.length
    ));
    
    // 第二行：协议类型（缩进 2 空格）
    lines.push(format!("  Protocol: {}", info.protocol));
    
    // 第三行：HTTP 摘要（如果有 HTTP 信息）
    if let Some(ref http) = info.http_info {
        let summary = http.summary();
        if !summary.is_empty() {
            lines.push(format!("  {}", summary));  // 缩进 2 空格
        }
    }
    
    // 用换行符连接所有行
    lines.join("\n")
}
