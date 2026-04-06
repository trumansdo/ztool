// ---------------------------------------------------------------------------
// 1.1 字符串基础操作
// ---------------------------------------------------------------------------

#[test]
/// 测试: String 创建方式 (from/to_string/format!/with_capacity/repeat)
fn test_string_creation() {
    // 语法: Rust 有两种字符串类型 - String(owned, 堆分配) 和 &str(borrowed, 切片)
    //
    // String 创建方式:
    //   - String::new()                        空字符串
    //   - String::from("hello")                从 &str 转换
    //   - "hello".to_string()                  从 &str 转换(同上)
    //   - format!("{} {}", a, b)               格式化创建(最常用)
    //   - String::with_capacity(32)            预分配容量, 避免多次重新分配
    //   - "x".repeat(5)                        重复字符串 "xxxxx"
    //
    // &str 来源:
    //   - 字符串字面量: let s: &str = "hello";  (类型为 &'static str)
    //   - String 切片: let s: &str = &my_string[..];
    //   - 函数参数: fn foo(s: &str) { ... }    (推荐用 &str 而非 &String)
    //
    // 避坑:
    //   - String 是 Vec<u8> 的包装, 保证有效 UTF-8; &str 是 &[u8] 的包装, 也保证 UTF-8
    //   - 不要用 &String 作参数, 用 &str (Deref 自动转换, &str 更灵活)
    //   - String::with_capacity 只分配容量, 长度为 0
    //   - format! 宏功能强大但比直接拼接稍慢, 性能敏感场景注意
    //
    let empty = String::new();
    assert!(empty.is_empty());

    let from_str = String::from("hello");
    assert_eq!(from_str, "hello");

    let to_string = "hello".to_string();
    assert_eq!(to_string, from_str);

    let formatted = format!("{} {}", "hello", "world");
    assert_eq!(formatted, "hello world");

    let with_cap = String::with_capacity(100);
    assert_eq!(with_cap.len(), 0);
    assert!(with_cap.capacity() >= 100);

    let repeated = "ab".repeat(3);
    assert_eq!(repeated, "ababab");
}

#[test]
/// 测试: String 修改操作 (push/insert/pop/truncate/clear/拼接)
fn test_string_modification() {
    // 语法: String 可变, &str 不可变
    //
    // 修改方法:
    //   - push(c: char)         追加单个字符
    //   - push_str(s: &str)     追加字符串切片
    //   - insert(idx, c)        在指定位置插入字符
    //   - insert_str(idx, s)    在指定位置插入字符串
    //   - pop() -> Option<char> 移除并返回最后一个字符
    //   - remove(idx) -> char   移除并返回指定位置字符(O(n), 需移动后续字节)
    //   - truncate(len)         截断到指定长度
    //   - clear()               清空字符串(保留容量)
    //   - replace(pat, to)      替换所有匹配(返回新 String)
    //   - replacen(pat, to, n)  替换前 n 个匹配
    //
    // 拼接方式:
    //   - s1 + &s2              (s1 被移动, s2 借用)
    //   - s1 += &s2             (s1 必须 mut, 等价于 s1.push_str)
    //   - format!("{s1}{s2}")   (不移动原值, 最灵活)
    //
    // 避坑:
    //   - + 操作符左边的 String 会被移动(所有权转移), 右边是 &str
    //   - remove/insert 按字符索引不是字节索引, 时间复杂度 O(n)
    //   - 在无效字符边界调用 remove/insert 会 panic
    //   - replace 返回新 String, 不修改原串
    //
    let mut s = String::from("hello");
    s.push('!');
    assert_eq!(s, "hello!");

    s.push_str(" world");
    assert_eq!(s, "hello! world");

    s.insert(5, ',');
    assert_eq!(s, "hello,! world");

    let popped = s.pop();
    assert_eq!(popped, Some('d'));
    assert_eq!(s, "hello,! worl");

    s.truncate(5);
    assert_eq!(s, "hello");

    s.clear();
    assert!(s.is_empty());
    assert!(s.capacity() > 0); // 容量保留

    // 拼接
    let s1 = String::from("hello");
    let s2 = String::from(" world");
    let s3 = s1 + &s2; // s1 被移动, s2 借用
    assert_eq!(s3, "hello world");
    // assert_eq!(s1, "hello"); // 编译错误: s1 已被移动
}

#[test]
/// 测试: String 查询方法 (len/contains/find/chars/char_indices)
fn test_string_query() {
    // 语法: 字符串查询和检查方法
    //
    // 查询方法:
    //   - len() -> usize            字节长度(不是字符数!)
    //   - is_empty() -> bool        是否为空
    //   - contains(pat) -> bool     是否包含子串
    //   - starts_with(pat) -> bool  是否以某前缀开头
    //   - ends_with(pat) -> bool    是否以某后缀结尾
    //   - find(pat) -> Option<usize> 查找子串首次出现的字节索引
    //   - rfind(pat) -> Option<usize> 查找子串最后一次出现的字节索引
    //   - matches(pat)              返回所有匹配的迭代器
    //   - match_indices(pat)        返回 (字节索引, 匹配子串) 的迭代器
    //   - char_indices()            返回 (字节索引, char) 的迭代器
    //   - chars()                   返回 char 迭代器
    //   - bytes()                   返回 u8 字节迭代器
    //   - as_bytes() -> &[u8]       获取底层字节切片
    //   - as_str() -> &str          获取 &str 切片
    //
    // 避坑:
    //   - len() 返回字节数, 非 ASCII 字符的字节数 != 字符数!
    //   - find 返回的是字节索引, 不是字符索引
    //   - chars() 按 Unicode 标量值迭代, 不等于用户感知的"字形簇"
    //   - 中文/emoji 等多字节字符需要特别注意 len() 和 chars().count() 的区别
    //
    let s = String::from("hello Rust 世界");

    assert_eq!(s.len(), 17); // 11 ASCII(含空格) + 2*3 中文 = 17 字节
    assert_eq!(s.chars().count(), 13); // 11 ASCII(含空格) + 2 中文字符 = 13 字符
    assert!(!s.is_empty());
    assert!(s.contains("Rust"));
    assert!(s.starts_with("hello"));
    assert!(s.ends_with("世界"));

    assert_eq!(s.find("Rust"), Some(6)); // 字节索引 6
    assert_eq!(s.rfind("l"), Some(3)); // 最后一次出现的 'l'
    assert_eq!(s.find("不存在"), None);

    // 字符遍历
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars[0], 'h');
    assert_eq!(chars[11], '世');

    // 带索引遍历
    let indices: Vec<(usize, char)> = s.char_indices().collect();
    assert_eq!(indices[0], (0, 'h'));
    assert_eq!(indices[6], (6, 'R')); // 'R' 在字节索引 6 处
}

#[test]
/// 测试: String 切片操作 (字节索引切片/get安全切片/UTF-8边界)
fn test_string_slicing() {
    // 语法: 字符串切片 s[start..end] 按字节索引, 必须在字符边界上
    //
    // 切片方式:
    //   - &s[..]          整个字符串
    //   - &s[start..]     从 start 到末尾
    //   - &s[..end]       从开头到 end
    //   - &s[start..end]  指定范围
    //   - s.get(start..end) 安全切片, 返回 Option<&str>
    //
    // 避坑:
    //   - 切片索引必须是有效的 UTF-8 字符边界, 否则 panic!
    //   - 对 "你好" 用 &s[0..1] 会 panic, 因为中文字符占 3 字节
    //   - 不确定边界时用 .get() 返回 Option, 避免 panic
    //   - 切片返回 &str, 不是 String
    //
    let s = String::from("hello world");
    assert_eq!(&s[0..5], "hello");
    assert_eq!(&s[6..], "world");
    assert_eq!(&s[..5], "hello");
    assert_eq!(&s[6..11], "world");

    // 安全切片
    assert_eq!(s.get(0..5), Some("hello"));
    assert_eq!(s.get(0..6), Some("hello ")); // 0..6 包含空格, 是有效 UTF-8 边界
    assert_eq!(s.get(0..100), None); // 越界返回 None

    // 中文字符切片 - 必须在字符边界
    let chinese = String::from("你好世界");
    assert_eq!(&chinese[0..3], "你"); // '你' 占 3 字节
    assert_eq!(&chinese[3..6], "好"); // '好' 占 3 字节
    assert_eq!(&chinese[0..6], "你好"); // 两个字符共 6 字节
    // &chinese[0..1] 会 panic! 不是有效的 UTF-8 边界
    assert_eq!(chinese.get(0..1), None); // 安全方式返回 None
}

#[test]
/// 测试: String 转换和分割 (trim/case/replace/split/lines)
fn test_string_transform() {
    // 语法: 字符串转换和格式化方法
    //
    // 转换方法:
    //   - to_lowercase() -> String    转小写
    //   - to_uppercase() -> String    转大写
    //   - to_ascii_lowercase()        仅 ASCII 转小写(更快)
    //   - to_ascii_uppercase()        仅 ASCII 转大写
    //   - trim() -> &str              去除两端空白
    //   - trim_start() -> &str        去除开头空白
    //   - trim_end() -> &str          去除末尾空白
    //   - trim_matches(pat)           去除两端匹配字符
    //   - replace(pat, to) -> String  替换所有匹配
    //   - replacen(pat, to, n)        替换前 n 个匹配
    //   - split_whitespace()          按空白分割的迭代器
    //   - lines()                     按行分割的迭代器(处理 \n 和 \r\n)
    //   - split(pat)                  按分隔符分割
    //   - splitn(n, pat)              最多分割 n 部分
    //   - split_terminator(pat)       分割, 末尾分隔符不产生空串
    //   - rsplit(pat)                 从右向左分割
    //   - split_ascii_whitespace()    仅按 ASCII 空白分割(更快)
    //
    // 避坑:
    //   - to_lowercase/to_uppercase 返回新 String, 不修改原串
    //   - trim 系列返回 &str, 不是 String
    //   - to_lowercase 对德语 ß 等特殊字符处理与预期可能不同
    //   - split 连续分隔符会产生空串, 用 split_whitespace 自动跳过空白
    //
    let s = String::from("  Hello WORLD  ");
    assert_eq!(s.trim(), "Hello WORLD");
    assert_eq!(s.trim_start(), "Hello WORLD  ");
    assert_eq!(s.trim_end(), "  Hello WORLD");

    assert_eq!(s.to_lowercase(), "  hello world  ");
    assert_eq!(s.to_uppercase(), "  HELLO WORLD  ");
    assert_eq!(s.to_ascii_lowercase(), "  hello world  ");

    assert_eq!("--hello--".trim_matches('-'), "hello");
    assert_eq!("hello world".replace("o", "0"), "hell0 w0rld");
    assert_eq!("hello world".replacen("o", "0", 1), "hell0 world");

    // split 相关
    let csv = "a,b,,c";
    assert_eq!(
        csv.split(',')
            .collect::<Vec<_>>(),
        vec!["a", "b", "", "c"]
    );
    assert_eq!(
        csv.splitn(3, ',')
            .collect::<Vec<_>>(),
        vec!["a", "b", ",c"]
    );

    let text = "hello\nworld\r\nrust";
    assert_eq!(text.lines().collect::<Vec<_>>(), vec!["hello", "world", "rust"]);

    let spaced = "  hello   world  ";
    assert_eq!(
        spaced
            .split_whitespace()
            .collect::<Vec<_>>(),
        vec!["hello", "world"]
    );
}

#[test]
/// 测试: 字符串格式化 (Display/Debug/对齐/精度/命名参数)
fn test_string_formatting() {
    // 语法: format! / print! / println! 使用相同的格式化语法
    //
    // 格式化占位符:
    //   - {}            默认格式化 (Display trait)
    //   - {:?}          调试格式化 (Debug trait)
    //   - {:#?}         美化调试格式化(多行缩进)
    //   - {:x} / {:X}   十六进制(小写/大写)
    //   - {:b}          二进制
    //   - {:o}          八进制
    //   - {:e} / {:E}   科学计数法
    //   - {:p}          指针地址
    //   - {:.*}         动态精度
    //
    // 格式化选项:
    //   - {:>10}        右对齐, 宽度 10
    //   - {:<10}        左对齐, 宽度 10
    //   - {:^10}        居中对齐
    //   - {:0>10}       右对齐, 0 填充
    //   - {:.2}         保留 2 位小数
    //   - {name}        命名参数
    //   - {0} {1}       位置参数
    //
    // 避坑:
    //   - 类型必须实现对应 trait (Display/Debug 等), 否则编译失败
    //   - {:?} 需要类型实现 Debug (可用 #[derive(Debug)] 自动实现)
    //   - 格式化不改变原值, 只生成新的 String
    //
    assert_eq!(format!("{} {}", "hello", 42), "hello 42");
    assert_eq!(format!("{1} {0}", "world", "hello"), "hello world");
    assert_eq!(format!("{name} is {age}", name = "Rust", age = 15), "Rust is 15");

    assert_eq!(format!("{:x}", 255), "ff");
    assert_eq!(format!("{:b}", 10), "1010");
    assert_eq!(format!("{:0>5}", 42), "00042");
    assert_eq!(format!("{:<5}", "hi"), "hi   ");
    assert_eq!(format!("{:.2}", 3.14159), "3.14");

    // Debug 格式化
    let s = "hello\nworld";
    assert_eq!(format!("{}", s), "hello\nworld");
    assert_eq!(format!("{:?}", s), "\"hello\\nworld\"");
}

#[test]
/// 测试: String UTF-8 编码 (字节操作/from_utf8/lossy转换)
fn test_string_utf8() {
    // 语法: Rust 字符串保证有效 UTF-8, 这是核心设计
    //
    // UTF-8 规则:
    //   - ASCII 字符(0-127): 1 字节
    //   - 拉丁/希腊/西里尔: 2 字节
    //   - 中文/日文/韩文: 3 字节
    //   - Emoji/罕见字符: 4 字节
    //
    // 字节操作(绕过 UTF-8 检查):
    //   - String::from_utf8(vec) -> Result<String, FromUtf8Error>
    //   - String::from_utf8_lossy(bytes) -> Cow<str>  (无效字节替换为 )
    //   - String::from_utf8_unchecked(bytes) -> String (unsafe, 不检查!)
    //   - s.as_bytes() -> &[u8]  (只读, 安全)
    //   - s.as_mut_vec() -> &mut Vec<u8> (unsafe)
    //
    // 避坑:
    //   - 不能通过索引访问单个字符: s[0] 编译错误! (因为 UTF-8 变长)
    //   - chars() 返回 Unicode 标量值, 不等于"用户看到的字符"(字形簇)
    //   - Emoji "👨‍👩‍👧‍👦" 由多个标量值组成, chars().count() > 1
    //   - 字符串反转不能简单用 .rev(), 会破坏 UTF-8 和字形簇
    //   - from_utf8_unchecked 传入无效 UTF-8 是 UB(未定义行为)
    //
    let s = String::from("你好");
    assert_eq!(s.len(), 6); // 每个中文字符 3 字节
    assert_eq!(s.chars().count(), 2); // 2 个字符

    // 字节层面
    let bytes = s.as_bytes();
    assert_eq!(bytes.len(), 6);
    assert_eq!(bytes[0], 0xe4); // '你' 的第一个字节

    // 从字节创建字符串
    let valid_utf8 = vec![0xe4, 0xbd, 0xa0]; // "你" 的 UTF-8 编码
    let s = String::from_utf8(valid_utf8).unwrap();
    assert_eq!(s, "你");

    // 无效 UTF-8 处理
    let invalid_utf8 = vec![0xff, 0xfe, 0x00];
    assert!(String::from_utf8(invalid_utf8.clone()).is_err());

    let lossy = String::from_utf8_lossy(&invalid_utf8);
    assert!(lossy.contains('\u{FFFD}')); // 替换字符
}

#[test]
/// 测试: str split 分割方法
fn test_str_split() {
    // 语法: split() 按分隔符拆分, 返回迭代器
    // 避坑: split 返回的是 &str 迭代器, 需要 .collect() 才能转为 Vec; 分隔符是 char 或 &str
    let s = "a,b,c";
    let parts: Vec<&str> = s.split(',').collect();
    assert_eq!(parts, vec!["a", "b", "c"]);
}

#[test]
/// 测试: CString FFI C字符串 (new/to_str/as_ptr/内部null检查)
fn test_cstring() {
    // 语法: CString 是 std::ffi 中的 owned C 字符串类型, 用于 Rust 与 C/C++ 的 FFI 交互
    //
    // 核心特性:
    //   1. 末尾自动追加 \0 (null terminator), 符合 C 字符串约定
    //   2. 不允许内部包含 \0 字符(检查在创建时进行)
    //   3. 实现了 Deref<Target=CStr>, 可像 &CStr 一样使用
    //   4. 与 CStr 的关系 = String 与 &str 的关系(owned vs borrowed)
    //
    // 常用方法:
    //   - CString::new(s) -> Result<CString, NulError>  // 从 &str/&[u8] 创建
    //   - c_str.as_ptr() -> *const c_char               // 获取 C 指针, 传给 C 函数
    //   - c_str.to_str() -> Result<&str, Utf8Error>     // 转回 Rust &str
    //   - c_str.to_bytes() -> &[u8]                     // 获取字节切片(不含末尾\0)
    //   - c_str.to_bytes_with_nul() -> &[u8]            // 获取字节切片(含末尾\0)
    //   - c_str.into_bytes() -> Vec<u8>                 // 消费 CString, 返回 Vec
    //   - CString::from_raw(ptr) -> CString             // 从 C 侧拿回所有权(危险!)
    //
    // 避坑:
    //   - CString::new 会扫描整个字符串查找 \0, 长字符串有性能开销
    //   - 内部有 \0 时返回 NulError, 包含错误位置 (nul_position())
    //   - as_ptr() 返回的指针生命周期不能超过 CString 本身
    //   - into_raw() 转移所有权给 C 后, 必须由 Rust 侧用 from_raw() 释放, 否则内存泄漏
    //   - C 函数如果修改了字符串内容, 不能安全转回 CString
    //   - Windows API 用 UTF-16, 需用 OsString/OsStr + encode_wide(), 不是 CString
    //
    // 典型使用场景:
    //   unsafe { some_c_function(c_str.as_ptr()) }
    //
    use std::ffi::CString;
    let c_str = CString::new("hello").unwrap();
    assert_eq!(c_str.to_str().unwrap(), "hello");
}

#[test]
/// 测试: 原始字符串字面量 (r#/r## 免转义)
fn test_raw_strings() {
    // 语法: r#"..."# 原始字符串, 不需要转义; # 数量决定定界符
    // 避坑: 如果字符串内容包含 "#, 需用 r##"..."## 增加 # 数量
    let raw = r#"hello "world""#;
    assert_eq!(raw, "hello \"world\"");

    // 多层 # 示例
    let nested = r##"contains "# not a problem"##;
    assert_eq!(nested, "contains \"# not a problem");
}
