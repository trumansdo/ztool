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
    assert_eq!(empty.capacity(), 0); // 空 String 不分配堆内存

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

    // 边界: 预分配 0 容量
    let zero_cap = String::with_capacity(0);
    assert_eq!(zero_cap.len(), 0);
    assert_eq!(zero_cap.is_empty(), true);

    // 边界: repeat(0) 产生空串
    assert_eq!("x".repeat(0), "");
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

    // 边界: pop 空字符串
    let mut empty_s = String::new();
    assert_eq!(empty_s.pop(), None);

    // 边界: insert_str 在空串位置 0
    let mut empty = String::new();
    empty.insert_str(0, "abc");
    assert_eq!(empty, "abc");

    // 边界: truncate(0) 安全
    let mut t = String::from("hello");
    t.truncate(0);
    assert_eq!(t, "");

    // 边界: remove 最后一个字符
    let mut r = String::from("abc");
    assert_eq!(r.remove(2), 'c');
    assert_eq!(r, "ab");
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

    // 扩展: starts_with 支持 char
    assert!("hello".starts_with('h'));
    assert!(!"hello".starts_with('x'));

    // 扩展: ends_with 支持 char
    assert!("hello".ends_with('o'));

    // 扩展: match_indices
    let matches: Vec<(usize, &str)> = "ababa".match_indices("aba").collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], (0, "aba"));

    // 扩展: matches 返回所有匹配子串
    let all_matches: Vec<&str> = "hello llo world".matches("llo").collect();
    assert_eq!(all_matches, vec!["llo", "llo"]);

    // 扩展: rfind 查找中文
    let cn = "你好世界你好";
    assert_eq!(cn.rfind("你好"), Some(12)); // 第二个"你好"从字节索引 12 开始

    // 扩展: 检查空串的 contains
    assert!("hello".contains("")); // 空串总是包含

    // 扩展: as_bytes()
    let bytes = s.as_bytes();
    assert_eq!(bytes.len(), s.len());
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

    // 扩展: 边界测试 - 空切片
    assert_eq!(s.get(0..0), Some(""));
    assert_eq!(s.get(11..11), Some(""));
    assert_eq!(s.get(100..100), None);

    // 扩展: 从中间开始到末尾
    assert_eq!(s.get(6..), Some("world"));
    assert_eq!(s.get(6..s.len()), Some("world"));

    // 扩展: get_mut 返回 Option<&mut str>
    let mut mut_s = String::from("hello");
    if let Some(slice) = mut_s.get_mut(0..1) {
        assert_eq!(slice, "h");
    }
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

    // 扩展: trim_matches 多字符
    assert_eq!("<>hello</>".trim_matches(&['<', '>', '/'][..]), "hello");

    // 扩展: trim_start_matches / trim_end_matches
    assert_eq!("##hello".trim_start_matches('#'), "hello");
    assert_eq!("hello##".trim_end_matches('#'), "hello");

    // 扩展: rsplit 从右向左分割
    let parts: Vec<&str> = "a.b.c".rsplit('.').collect();
    assert_eq!(parts, vec!["c", "b", "a"]);

    // 扩展: split_inclusive 保留分隔符
    let inclusive: Vec<&str> = "a,b,c".split_inclusive(',').collect();
    assert_eq!(inclusive, vec!["a,", "b,", "c"]);

    // 扩展: split_ascii_whitespace
    let ascii_ws: Vec<&str> = "a\tb\nc d".split_ascii_whitespace().collect();
    assert_eq!(ascii_ws, vec!["a", "b", "c", "d"]);
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

    // 扩展: {:X} 大写十六进制
    assert_eq!(format!("{:X}", 255), "FF");

    // 扩展: {:o} 八进制
    assert_eq!(format!("{:o}", 10), "12");

    // 扩展: 居中对齐
    assert_eq!(format!("{:^7}", "hi"), "  hi   ");

    // 扩展: 动态宽度
    assert_eq!(format!("{:>width$}", "hi", width = 5), "   hi");
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

    // 扩展: 4 字节 emoji 编码验证
    let emoji = "🤖"; // U+1F916
    assert_eq!(emoji.len(), 4);
    assert_eq!(emoji.chars().count(), 1);

    // 扩展: 混合字符编码边界测试
    let mixed = "a世"; // 1 + 3 = 4 字节
    assert_eq!(mixed.len(), 4);
    assert_eq!(mixed.chars().count(), 2);
    assert!(mixed.get(0..1).is_some()); // 'a' — 有效边界
    assert!(mixed.get(1..4).is_some()); // '世' — 有效边界 (1='世'首字节, 4=末尾)
    assert!(mixed.get(0..2).is_none()); // 切在 '世' 的中间字节

    // 扩展: from_utf8_lossy 不产生额外内存分配(在已有有效 UTF-8 时)
    let valid_bytes: &[u8] = b"hello";
    let cow = String::from_utf8_lossy(valid_bytes);
    assert_eq!(cow, "hello");
    // Cow::Borrowed 意味着没有分配
    assert!(matches!(cow, std::borrow::Cow::Borrowed(_)));
}

#[test]
/// 测试: str split 分割方法
fn test_str_split() {
    // 语法: split() 按分隔符拆分, 返回迭代器
    // 避坑: split 返回的是 &str 迭代器, 需要 .collect() 才能转为 Vec; 分隔符是 char 或 &str
    let s = "a,b,c";
    let parts: Vec<&str> = s.split(',').collect();
    assert_eq!(parts, vec!["a", "b", "c"]);

    // 扩展: 连续分隔符产生空串
    let empty_parts: Vec<&str> = "a,,b".split(',').collect();
    assert_eq!(empty_parts, vec!["a", "", "b"]);

    // 扩展: split_terminator 忽略尾部空串
    let term_parts: Vec<&str> = "a,b,".split_terminator(',').collect();
    assert_eq!(term_parts, vec!["a", "b"]);

    // 扩展: 按 &str 分割
    let parts2: Vec<&str> = "hello world rust".split(" wo").collect();
    assert_eq!(parts2, vec!["hello", "rld rust"]);
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

    // 扩展: 检查 null 终止符
    let bytes_with_nul = c_str.to_bytes_with_nul();
    assert_eq!(bytes_with_nul.last(), Some(&0));
    assert_eq!(bytes_with_nul.len(), 6); // "hello" + \0

    // 扩展: 不含 null 的字节
    let bytes = c_str.to_bytes();
    assert_eq!(bytes, b"hello");

    // 扩展: 内部包含 \0 字符会失败
    let result = CString::new("he\0llo");
    assert!(result.is_err());

    // 扩展: 空字符串的 CString
    let empty_c = CString::new("").unwrap();
    assert_eq!(empty_c.to_bytes_with_nul(), [0]);
    assert_ne!(empty_c.as_ptr(), std::ptr::null());

    // 扩展: into_string 尝试转换回 String
    let c = CString::new("rust").unwrap();
    let s: String = c.into_string().unwrap();
    assert_eq!(s, "rust");
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

    // 扩展: 基本 raw string r"..."
    let basic = r"C:\Users\truma";
    assert_eq!(basic, "C:\\Users\\truma");

    // 扩展: raw string 的最小形式
    let minimal = r"nothing to escape";
    assert_eq!(minimal, "nothing to escape");

    // 扩展: r### 三层嵌套
    // 注意: raw string 结束于第一个 "# 后跟足够数量的 #
    let triple = r###"He said: "r## and r#" in one string"###;
    assert_eq!(triple, r#"He said: "r## and r#" in one string"#);
}

// ---------------------------------------------------------------------------
// 1.2 新增: String 内部布局与增长策略
// ---------------------------------------------------------------------------

#[test]
/// 测试: String 内部布局 (指针+长度+容量, 24字节)
fn test_string_layout() {
    // 语法: String 在栈上占 24 字节(64位系统): 指针(8) + 长度(8) + 容量(8)
    //
    // 特性:
    //   - String 是 Vec<u8> 的封装, 内部结构完全相同
    //   - size_of::<String>() 在 64 位系统上固定为 24 字节
    //   - 空 String::new() 不分配堆内存, capacity=0, 指针可能为 dangling
    //   - clear() 归零 length, 保留 capacity
    //   - 所有权移动后, 旧变量失效, 但内存布局未改变
    //
    // 避坑:
    //   - String 的栈大小固定, 但堆大小随内容变化
    //   - 不要依赖 capacity 的精确值, 它是实现细节
    //   - with_capacity 的值是建议值, 实际可能分配更多
    //
    use std::mem::size_of;

    // 栈上大小固定为 24 字节 (64 位系统)
    assert_eq!(size_of::<String>(), 24);
    assert_eq!(size_of::<Vec<u8>>(), 24); // 验证 String 和 Vec<u8> 大小相同

    // 空 String 不分配堆内存
    let empty = String::new();
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.capacity(), 0);

    // with_capacity 分配容量但 len 仍为 0
    let pre = String::with_capacity(64);
    assert_eq!(pre.len(), 0);
    assert!(pre.capacity() >= 64);

    // clear() 归零 len, 保留 capacity
    let mut s = String::from("hello world");
    let cap_before = s.capacity();
    s.clear();
    assert_eq!(s.len(), 0);
    assert_eq!(s.capacity(), cap_before);

    // 所有权移动: 旧变量不能被使用
    let s1 = String::from("data");
    let s2 = s1; // s1 移动到 s2
                 // 以下代码取消注释会导致编译错误:
                 // let _ = s1.len(); // error[E0382]: borrow of moved value: `s1`
    assert_eq!(s2, "data");

    // 移动后栈上的三个字段被逐位复制, 堆数据不变
    // 这解释了为什么 String move 的成本很低 (仅复制 24 字节)
}

#[test]
/// 测试: String 增长策略 (push/push_str 时 capacity 自动翻倍)
fn test_string_growth() {
    // 语法: push/push_str 触发容量不足时自动扩容, 策略为翻倍 (2x)
    //
    // 特性:
    //   - 从空串开始 push 时, capacity 从 0 跳到某个正数 (实现定义)
    //   - 之后每次容量不足, 新容量 >= 旧容量 * 2
    //   - 扩容涉及堆上新分配、memcpy 旧数据、释放旧内存
    //   - reserve(n) 确保 capacity >= n, 避免后续多次扩容
    //   - 扩容的摊还时间复杂度为 O(1)
    //
    // 避坑:
    //   - 不要在循环中用 format! 累积字符串, 用 push_str + with_capacity
    //   - capacity 的精确值是实现细节, 不要硬编码断言
    //   - reserve_exact 请求精确容量, 但标准库不保证严格精确
    //
    let mut s = String::new();
    assert_eq!(s.capacity(), 0);

    // 第一次 push 触发首次分配
    s.push('a');
    assert_eq!(s.len(), 1);
    let first_cap = s.capacity();
    assert!(first_cap >= 1);

    // 继续 push 直到触发扩容
    let mut growth_count = 0;
    let mut prev_cap = s.capacity();
    for _ in 0..100 {
        s.push('x');
        let curr_cap = s.capacity();
        if curr_cap > prev_cap {
            growth_count += 1;
            // 验证扩容至少是翻倍 (或首次分配)
            assert!(curr_cap >= prev_cap * 2 || prev_cap == 0);
            prev_cap = curr_cap;
        }
    }
    assert!(growth_count >= 1); // 至少触发了一次扩容

    // reserve() 主动扩容
    let mut r = String::with_capacity(10);
    let _cap_before = r.capacity();
    r.reserve(100);
    assert!(r.capacity() >= 100); // reserve(n) 确保 capacity >= len + n

    // with_capacity 预分配避免 push 时扩容
    let mut pre = String::with_capacity(200);
    let cap_start = pre.capacity();
    for _ in 0..200 {
        pre.push('a');
    }
    assert_eq!(pre.capacity(), cap_start); // 没有触发扩容!
    assert_eq!(pre.len(), 200);

    // push_str 批量追加同样触发扩容
    let mut ps = String::with_capacity(5);
    let cap0 = ps.capacity();
    ps.push_str("abcdefghij"); // 10 字节, 远超 5 容量
    assert!(ps.capacity() > cap0);
    assert_eq!(ps, "abcdefghij");
}

#[test]
/// 测试: UTF-8 字节边界切片安全 (panic 演示与 get 安全处理)
fn test_utf8_slicing_safety() {
    // 语法: 字符串切片按字节索引, 必须在 UTF-8 字符边界上; get() 返回 Option 避免 panic
    //
    // 特性:
    //   - &s[start..end] 在非字符边界上会 panic
    //   - s.get(start..end) 安全返回 Option<&str>, 非边界返回 None
    //   - 中文字符占 3 字节, emoji 占 4 字节, ASCII 占 1 字节
    //   - 可以使用 std::panic::catch_unwind 捕获切片 panic
    //
    // 避坑:
    //   - 永远不要假设字符占固定字节数, 总是用 chars() 迭代或 get() 安全切片
    //   - 从用户输入或外部数据取切片前, 先用 get() 测试边界
    //   - catch_unwind 不是正常控制流, 仅用于测试和边界场景
    //
    let s = String::from("你好世界");
    // 安全切片
    assert_eq!(s.get(0..3), Some("你"));
    assert_eq!(s.get(3..6), Some("好"));
    assert_eq!(&s[0..6], "你好");

    // 不安全边界: get 返回 None 而不是 panic
    assert_eq!(s.get(0..1), None); // 落在'你'的中间字节
    assert_eq!(s.get(1..3), None); // 从'你'的中间开始
    assert_eq!(s.get(0..2), None); // 结束在'你'的中间
    assert_eq!(s.get(2..6), None); // 从'你'的中间开始

    // 验证 panic: 直接在非字符边界上切片会 panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let inner = String::from("你好世界");
        let _ = &inner[0..1];
    }));
    assert!(result.is_err());

    // 验证另一个非边界: 索引 1 在字符内部
    let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let inner = String::from("你好世界");
        let _ = &inner[1..6];
    }));
    assert!(result2.is_err());

    // Emoji 测试 (4 字节字符)
    let emoji = String::from("A🤖C"); // A=1字节, 🤖=4字节, C=1字节
    assert_eq!(emoji.len(), 6);
    assert_eq!(emoji.chars().count(), 3);
    assert_eq!(emoji.get(0..1), Some("A"));
    assert_eq!(emoji.get(1..2), None); // 落在 emoji 中间字节
    assert_eq!(emoji.get(1..5), Some("🤖")); // 1='🤖'首字节, 5='C'首字节, 两个有效边界
    assert_eq!(emoji.get(0..5), Some("A🤖")); // 正确的边界
    assert_eq!(emoji.get(5..6), Some("C"));
    assert_eq!(emoji.get(2..5), None); // 从 emoji 中间字节开始

    // catch_unwind 捕获 emoji panic
    let emoji_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let inner = String::from("A🤖C");
        let _ = &inner[0..1]; // 这个其实是 OK 的('A')
    }));
    assert!(emoji_result.is_ok()); // 0..1 不是 panic, 它是有效的

    let emoji_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let inner = String::from("A🤖C");
        let _ = &inner[0..2]; // 从 0 到 emoji 中间字节
    }));
    assert!(emoji_panic.is_err());

    // 混合字符的字节边界遍历
    let mixed = "a你b";
    let boundaries: Vec<usize> = mixed.char_indices().map(|(i, _)| i).collect();
    assert_eq!(boundaries, vec![0, 1, 4]); // 'a'@0, '你'@1, 'b'@4
    // 验证有效边界可以切片
    assert_eq!(&mixed[0..1], "a");
    assert_eq!(&mixed[1..4], "你");
    assert_eq!(&mixed[4..5], "b");
}

#[test]
/// 测试: chars / bytes / char_indices 方法族对比
fn test_string_methods_chars() {
    // 语法: chars() 迭代 Unicode 标量值, bytes() 迭代原始字节, char_indices() 带索引遍历
    //
    // 特性:
    //   - chars() 返回 char, 每个 char 是一个 Unicode 标量值 (4 字节)
    //   - bytes() 返回 u8, 是原始 UTF-8 字节流
    //   - char_indices() 返回 (字节索引, char), 便于定位
    //   - chars().count() = 字符数, <len()> = 字节数
    //
    // 避坑:
    //   - chars() 不处理组合字符 (如 e + ̀ = è 这种)
    //   - 反转 chars() 集合不等于反转字符串 (会破坏多字节字符)
    //   - 需要字形簇操作时用 unicode-segmentation crate
    //
    let s = "Rust世";

    // chars() — 字符迭代
    let chars: Vec<char> = s.chars().collect();
    assert_eq!(chars, vec!['R', 'u', 's', 't', '世']);
    assert_eq!(s.chars().count(), 5);

    // bytes() — 字节迭代
    let bytes: Vec<u8> = s.bytes().collect();
    assert_eq!(bytes.len(), 7); // 4 ASCII + 3 中文 = 7
    assert_eq!(s.len(), 7); // len() 和 bytes().count() 一致
    assert_eq!(bytes[0], b'R');
    // '世' 的三个字节
    assert_eq!(bytes[4], 0xe4);
    assert_eq!(bytes[5], 0xb8);
    assert_eq!(bytes[6], 0x96);

    // char_indices() — 带索引的字符遍历
    let indices: Vec<(usize, char)> = s.char_indices().collect();
    assert_eq!(indices[0], (0, 'R'));
    assert_eq!(indices[4], (4, '世')); // '世' 从字节索引 4 开始

    // 验证索引间距
    let index_gaps: Vec<usize> = s.char_indices().map(|(_, c)| c.len_utf8()).collect();
    assert_eq!(index_gaps, vec![1, 1, 1, 1, 3]); // 每个字符占的字节数

    // 空字符串的行为
    assert_eq!("".chars().count(), 0);
    assert_eq!("".bytes().count(), 0);
    assert_eq!("".char_indices().count(), 0);

    // 验证 chars() 不是"用户看见的字符"
    // 带变音符号的 e (U+0065 U+0301) 是两个 Unicode 标量值
    let combining = "e\u{0301}"; // e + combining acute accent
    assert_eq!(combining.chars().count(), 2); // 两个标量值, 但人眼看是一个字符
    assert_eq!(combining.len(), 3); // 3 字节 (e=1 + 组合符=2)

    // 经典 emoji 组合: 家庭 emoji 由多个标量值组成
    let family = "👨‍👩‍👧‍👦"; // 4 个人物 + 3 个 ZWJ = 7 个标量值
    assert!(family.chars().count() > 1); // 远超 1

    // last() 方法
    assert_eq!("abc".chars().last(), Some('c'));
    assert_eq!("abc".bytes().last(), Some(b'c'));
    assert_eq!("你好".chars().last(), Some('好'));

    // nth() 方法
    assert_eq!("hello".chars().nth(1), Some('e'));
    assert_eq!("hello".bytes().nth(1), Some(b'e'));
    assert_eq!("hello".char_indices().nth(1), Some((1, 'e')));
}

#[test]
/// 测试: split / lines / split_whitespace 分割方法族
fn test_string_split_methods() {
    // 语法: split 系列方法按各种条件分割字符串, 返回 &str 迭代器
    //
    // 特性:
    //   - split(pat) 按分隔符分割, 连续分隔符产生空串
    //   - splitn(n, pat) 限制分割段数
    //   - rsplit(pat) 从右向左分割
    //   - split_terminator(pat) 忽略末尾空串
    //   - split_inclusive(pat) 分隔符包含在前一段
    //   - split_whitespace() 按 Unicode 空白分割, 跳过连续空白
    //   - split_ascii_whitespace() 仅按 ASCII 空白分割, 更快
    //   - lines() 按 \n 和 \r\n 分割
    //
    // 避坑:
    //   - split 产生空串符合预期时直接用; 不需要空串时用 split_whitespace 或 filter
    //   - splitn 产生的最后一段可能包含分隔符
    //   - lines() 不包含行尾换行符
    //
    // --- split 基本用法 ---
    let parts: Vec<&str> = "a,b,c".split(',').collect();
    assert_eq!(parts, vec!["a", "b", "c"]);

    // 连续分隔符产生空串
    let empty_parts: Vec<&str> = "a,,b".split(',').collect();
    assert_eq!(empty_parts, vec!["a", "", "b"]);

    // 前后都有分隔符
    let edge_parts: Vec<&str> = ",a,".split(',').collect();
    assert_eq!(edge_parts, vec!["", "a", ""]);

    // --- splitn ---
    let limited: Vec<&str> = "a,b,c,d".splitn(2, ',').collect();
    assert_eq!(limited, vec!["a", "b,c,d"]);

    // splitn(n=1) 不分割
    let no_split: Vec<&str> = "hello".splitn(1, ',').collect();
    assert_eq!(no_split, vec!["hello"]);

    // --- rsplit ---
    let rev: Vec<&str> = "a.b.c".rsplit('.').collect();
    assert_eq!(rev, vec!["c", "b", "a"]);

    // --- split_terminator ---
    let term: Vec<&str> = "a,b,".split_terminator(',').collect();
    assert_eq!(term, vec!["a", "b"]); // 末尾的空串被忽略

    // 中间的空串仍然保留
    let mid_empty: Vec<&str> = "a,,b".split_terminator(',').collect();
    assert_eq!(mid_empty, vec!["a", "", "b"]);

    // --- split_inclusive ---
    let inclusive: Vec<&str> = "a,b,c".split_inclusive(',').collect();
    assert_eq!(inclusive, vec!["a,", "b,", "c"]);

    // --- split_whitespace ---
    let ws: Vec<&str> = "  hello\t\n  world   ".split_whitespace().collect();
    assert_eq!(ws, vec!["hello", "world"]);

    // 全角空格 (U+3000) 也是 Unicode 空白, split_whitespace 也能处理
    let fullwidth_space = "hello\u{3000}world";
    let fw: Vec<&str> = fullwidth_space.split_whitespace().collect();
    assert_eq!(fw, vec!["hello", "world"]);

    // --- split_ascii_whitespace ---
    let ascii_ws: Vec<&str> = "a\tb\nc d".split_ascii_whitespace().collect();
    assert_eq!(ascii_ws, vec!["a", "b", "c", "d"]);

    // 全角空格不是 ASCII 空白, 不会被分割
    let ascii_fw: Vec<&str> = fullwidth_space.split_ascii_whitespace().collect();
    assert_eq!(ascii_fw, vec!["hello\u{3000}world"]); // 未被分割

    // --- lines ---
    let ls: Vec<&str> = "line1\nline2\r\nline3".lines().collect();
    assert_eq!(ls, vec!["line1", "line2", "line3"]);

    // UTF-8 行分隔符 (U+2028) 等不是 lines() 的分隔符
    let non_newline = "hello\u{2028}world";
    assert_eq!(non_newline.lines().count(), 1); // U+2028 不算换行

    // 按多字符分隔符分割
    let multi: Vec<&str> = "hello||world||rust".split("||").collect();
    assert_eq!(multi, vec!["hello", "world", "rust"]);

    // 空字符串的分割行为
    assert_eq!("".split(',').collect::<Vec<_>>(), vec![""]);
    assert_eq!("".split_whitespace().collect::<Vec<_>>(), Vec::<&str>::new());
    assert_eq!("".lines().collect::<Vec<_>>(), vec![] as Vec<&str>);
}

#[test]
/// 测试: trim / starts_with / ends_with / contains / replace / replacen 方法族
fn test_string_trim_replace() {
    // 语法: trim 系列返回 &str (零成本), replace 系列返回 String (新分配)
    //
    // 特性:
    //   - trim / trim_start / trim_end 去除 Unicode 空白, 返回引用
    //   - trim_matches / trim_start_matches / trim_end_matches 按字符集裁剪
    //   - starts_with / ends_with 检查前后缀 (支持 char 和 &str)
    //   - contains 检查是否包含子串
    //   - replace / replacen 生成新 String, 原串不变
    //
    // 避坑:
    //   - trim 返回 &str, 生命周期跟随原串; 原串被释放后 trim 结果也不可用
    //   - trim_matches 的参数是 Pattern, 字符串会被理解为字符集 (SlicePattern)
    //   - replace 不修改原 String, 需要手动接收返回值
    //   - trim 不会原地修改, 因为 &str 是不可变的
    //
    // --- trim 系列 ---
    let s = "  \t hello 世界 \n  ";
    assert_eq!(s.trim(), "hello 世界");
    assert_eq!(s.trim_start(), "hello 世界 \n  ");
    assert_eq!(s.trim_end(), "  \t hello 世界");

    // 全由空白构成的字符串
    assert_eq!("   ".trim(), "");
    assert_eq!("\t\n\r".trim(), "");

    // --- trim_matches 系列 ---
    assert_eq!("--hello--".trim_matches('-'), "hello");
    // 多字符集裁剪
    assert_eq!("<>hello</>".trim_matches(&['<', '>', '/'][..]), "hello");

    // trim_start_matches / trim_end_matches
    assert_eq!("##hello".trim_start_matches('#'), "hello");
    assert_eq!("hello##".trim_end_matches('#'), "hello");

    // 无匹配字符时不裁剪
    assert_eq!("hello".trim_matches('-'), "hello");

    // --- starts_with / ends_with / contains ---
    assert!("hello world".starts_with("hello"));
    assert!("hello world".ends_with("world"));
    assert!("hello world".contains("lo wo"));

    // 支持 char 参数
    assert!("hello".starts_with('h'));
    assert!("hello".ends_with('o'));
    assert!("hello".contains('e'));

    // 边界情况
    assert!("hello".starts_with("")); // 空串匹配一切
    assert!("hello".ends_with(""));
    assert!("hello".contains(""));
    assert!(!"hello".starts_with("x"));
    assert!(!"hello".ends_with("x"));
    assert!(!"hello".contains("x"));

    // --- replace / replacen ---
    let s = "hello world hello";
    // replace 返回新 String
    let replaced = s.replace("hello", "hi");
    assert_eq!(replaced, "hi world hi");
    assert_eq!(s, "hello world hello"); // 原串未变

    // replacen 限制次数
    let replaced_once = s.replacen("hello", "hi", 1);
    assert_eq!(replaced_once, "hi world hello");

    // replacen(0) 不替换
    assert_eq!(s.replacen("hello", "hi", 0), "hello world hello");

    // replace 支持 char
    assert_eq!("a_b_c".replace('_', "-"), "a-b-c");

    // --- 组合使用: trim + replace ---
    let dirty = "  hello_world  ";
    let cleaned = dirty.trim().replace('_', " ");
    assert_eq!(cleaned, "hello world");
}

#[test]
/// 测试: 字符串连接的六种方式 (+, +=, format!, push_str, join, concat!)
fn test_string_concat() {
    // 语法: Rust 提供多种字符串连接方式, 各有不同的所有权和性能特征
    //
    // 特性:
    //   - + 运算符: 移动左侧 String, 借用右侧 &str, 可能触发 realloc
    //   - += 运算符: 原地追加 (等价于 push_str), 左侧需 mut
    //   - push_str: 原地追加, 最高效
    //   - format!: 创建新 String, 最灵活, 不移动原值
    //   - [s1, s2].join(""): 预计算总长度后一次分配
    //   - concat!: 编译期常量拼接, 零运行时开销
    //
    // 避坑:
    //   - + 左边的 String 被移动了, 之后不能再用
    //   - format! 每次分配新内存, 热路径避免频繁调用
    //   - concat! 只能拼接字符串字面量, 不能用于运行时值
    //   - push_str 是最快的追加方式, 配合 with_capacity 最优
    //
    // --- + 运算符 ---
    let s1 = String::from("hello");
    let s2 = String::from(" world");
    let s3 = s1 + &s2;
    assert_eq!(s3, "hello world");
    // s1 被移动, 不能再使用; s2 仍可用 (仅借用)
    assert_eq!(s2, " world");

    // + 可以链式调用
    let a = String::from("a");
    let c = a + "b" + "c";
    assert_eq!(c, "abc");

    // --- += 运算符 ---
    let mut s = String::from("hello");
    s += " world";
    s += "!";
    assert_eq!(s, "hello world!");

    // --- push_str ---
    let mut ps = String::with_capacity(32);
    ps.push_str("hello");
    ps.push_str(" ");
    ps.push_str("world");
    assert_eq!(ps, "hello world");

    // --- format! ---
    let hello = String::from("hello");
    let world = String::from("world");
    let formatted = format!("{} {}", hello, world);
    assert_eq!(formatted, "hello world");
    // 原值仍然可用
    assert_eq!(hello, "hello");
    assert_eq!(world, "world");

    // format! 不移动任何原值的所有权
    let f2 = format!("{hello} {world}!");
    assert_eq!(f2, "hello world!");

    // --- join ---
    let parts = vec![
        String::from("hello"),
        String::from(" "),
        String::from("world"),
    ];
    let joined = parts.join("");
    assert_eq!(joined, "hello world");

    // join 的分隔符
    let words = vec!["hello", "world", "rust"];
    assert_eq!(words.join(", "), "hello, world, rust");
    assert_eq!(words.join(""), "helloworldrust");

    // --- concat! (编译期) ---
    let compiled = concat!("hello", " ", "world");
    assert_eq!(compiled, "hello world");
    // concat! 返回 &'static str
    let static_str: &'static str = concat!("a", "b", "c");
    assert_eq!(static_str, "abc");

    // --- 性能对比: push_str 原地追加不会移动所有权 ---
    let base = String::from("base");
    let mut fast = base;
    fast.push_str("-extended");
    assert_eq!(fast, "base-extended");

    // --- 边界: 空字符串拼接 ---
    assert_eq!(String::from("") + "", "");
    assert_eq!(format!("{}{}", "", ""), "");
    assert_eq!(["", ""].join(","), ",");
}

#[test]
/// 测试: to_lowercase/to_uppercase Unicode 陷阱 (德语 ß / 长度变化)
fn test_string_case() {
    // 语法: to_lowercase/to_uppercase 正确处理 Unicode 大小写映射
    //
    // Unicode 大小写陷阱:
    //   - 德语 ß (Eszett, U+00DF) 转大写 = "SS" (两个字符)
    //   - 某些字符的大小写形式长度不同, 字符串长度可能变化
    //   - to_ascii_lowercase/to_ascii_uppercase 仅处理 A-Z/a-z, 不做 Unicode 映射
    //   - 不同语言环境的大小写规则可能不同 (如土耳其语 İ/i)
    //
    // 避坑:
    //   - 不要假设 to_lowercase/to_uppercase 后 len() 不变
    //   - 处理德语文本时特别注意 ß 的转换
    //   - ASCII-only 场景用 to_ascii_* 可以获得更好性能
    //   - 比较字符串大小写不敏感时, 用 to_lowercase() 而非 to_ascii_lowercase()
    //
    // --- 德语 ß 陷阱 ---
    let sharp_s = "ß";
    assert_eq!(sharp_s.len(), 2); // UTF-8 编码占 2 字节
    assert_eq!(sharp_s.chars().count(), 1); // 1 个字符

    let upper = sharp_s.to_uppercase();
    assert_eq!(upper, "SS");
    assert_eq!(upper.len(), 2); // 变成 2 个字符, 2 字节!
    assert_eq!(upper.chars().count(), 2);

    // 实际德国街道名称例子
    let street = "Straße";
    let street_upper = street.to_uppercase();
    assert_eq!(street_upper, "STRASSE");
    assert_eq!(street.len(), 7); // ß 占 2 字节 UTF-8
    assert_eq!(street_upper.len(), 7); // 长度不变 (2 字节 ß → 2 字节 SS)
    assert_eq!(street.chars().count(), 6); // 6 个字符
    assert_eq!(street_upper.chars().count(), 7); // 但字符数增加了! ß(1个) → SS(2个)

    // --- 其他大小写长度变化的例子 ---
    // 某些 Unicode 字符转大写后长度也不同
    // 例如 U+00FF (ÿ) 转大写 = U+0178 (Ÿ), 但 Ÿ 和 ÿ 长度相同 (2字节)
    // 重点是不要假设长度不变

    // --- ASCII 版本: 不处理非 ASCII 字符 ---
    assert_eq!("Hello ß".to_ascii_lowercase(), "hello ß"); // ß 不变!
    assert_eq!("Hello ß".to_lowercase(), "hello ß"); // ß 本身就是小写, 也不变
    assert_eq!("ß".to_ascii_lowercase(), "ß"); // 不变, 因为 ß 不是 ASCII
    assert_eq!("ß".to_uppercase(), "SS"); // Unicode 正确转换

    // ASCII 大写
    assert_eq!("hello".to_ascii_uppercase(), "HELLO");
    assert_eq!("HELLO".to_ascii_lowercase(), "hello");

    // --- 混合字符 ---
    let mixed = "AßC";
    assert_eq!(mixed.to_lowercase(), "aßc"); // 小写: ß 不变
    assert_eq!(mixed.to_uppercase(), "ASSC"); // 大写: ß → SS, 长度 +1
    // 因此: to_lowercase 可能是无损的 (ß 无更小写), 而 to_uppercase 可能增加长度

    // --- 空字符串 ---
    assert_eq!("".to_lowercase(), "");
    assert_eq!("".to_uppercase(), "");

    // --- 数字和符号不受影响 ---
    assert_eq!("123!@#".to_lowercase(), "123!@#");
    assert_eq!("123!@#".to_uppercase(), "123!@#");
}

#[test]
/// 测试: raw string 和 byte string (r#/#/b""/br"")
fn test_raw_and_byte_strings() {
    // 语法: r"..." 原始字面量(免转义), b"..." 字节字面量(&[u8; N]), br"..." 两者结合
    //
    // 特性:
    //   - r"..." 中 \ 保持原样, 适合 Windows 路径和正则表达式
    //   - r#"..."# 可在内容中包含双引号, # 数量可叠加
    //   - b"..." 只能包含 ASCII 字符 (\x00-\x7F), 类型为 &[u8; N]
    //   - br"..." / br#"..."# 将 raw 和 byte 结合
    //
    // 避坑:
    //   - b"你好" 编译错误! 中文 > 0x7F, 不能用字节字面量直接写
    //   - raw string 不能包含末尾单独的 \ (如 r"abc\" 编译错误)
    //   - r#" 中的 # 定界符数量: 内容含 "# 时需 r##...##
    //
    // --- 基本 raw string ---
    let raw = r"hello\nworld"; // \n 不会变成换行
    assert_eq!(raw, "hello\\nworld");

    // Windows 路径
    let path = r"C:\Users\truma\Documents";
    assert_eq!(path, "C:\\Users\\truma\\Documents");

    // 正则表达式
    let regex = r"\d{3}-\d{4}-\d{4}";
    assert_eq!(regex, "\\d{3}-\\d{4}-\\d{4}");

    // --- raw string 含双引号 ---
    let quoted = r#"He said: "hello world""#;
    assert_eq!(quoted, "He said: \"hello world\"");

    // 两层嵌套
    let nested = r##"This contains "# symbols"##;
    assert_eq!(nested, "This contains \"# symbols");

    // 三层嵌套
    let triple = r###"r##" and r#" together"###;
    assert_eq!(triple, "r##\" and r#\" together");

    // --- raw string 的边界限制 ---
    // raw string 中不能直接以奇数的 \ 结尾 (无法表示 r"abc\" 这种)
    // 因为最后的 " 会被 \ 转义掉
    // 可以用 r#"abc\"# 变通

    // --- 字节字符串 b"..." ---
    let bs: &[u8; 5] = b"hello";
    assert_eq!(bs, &[104, 101, 108, 108, 111]);

    // 字节字符串中的转义
    let bs_escaped: &[u8; 6] = b"\x48\x65\x6c\x6c\x6f\x21";
    assert_eq!(bs_escaped, b"Hello!");

    // 字节字符串中的 \n 转义
    let bs_newline: &[u8; 5] = b"a\nb\nc";
    assert_eq!(bs_newline, &[b'a', b'\n', b'b', b'\n', b'c']);

    // --- raw byte string br"..." ---
    let br: &[u8; 5] = br"x\n\y"; // \ 不转义
    assert_eq!(br, &[b'x', b'\\', b'n', b'\\', b'y']);

    // raw byte string 含引号
    let br_quoted: &[u8; 5] = br#"x"y"z"#;
    assert_eq!(br_quoted, b"x\"y\"z");

    // --- 验证类型 ---
    let _explicit_byte: &[u8; 5] = b"hello"; // 类型: &[u8; 5]
    let _explicit_raw_byte: &[u8; 5] = br"hello"; // 类型: &[u8; 5]

    // 字节字符串不能包含非 ASCII
    // let _ = b"你好"; // 编译错误

    // 但可以用 \x 转义表示任意字节
    // "你" 的 UTF-8 编码: e4 bd a0
    let chinese_bytes: &[u8; 3] = b"\xe4\xbd\xa0";
    let decoded = std::str::from_utf8(chinese_bytes).unwrap();
    assert_eq!(decoded, "你");
}

#[test]
/// 测试: OsString / CString / PathBuf 三种特殊字符串类型基本用法
fn test_os_c_path_strings() {
    // 语法: OsString/OsStr, CString/CStr, PathBuf/Path 是三个特殊的字符串类型对
    //
    // 类型对照:
    //   - String / &str            通用 UTF-8 字符串
    //   - OsString / &OsStr        操作系统原生字符串 (平台相关)
    //   - CString / &CStr          C 兼容的 null 结尾字符串
    //   - PathBuf / &Path          文件系统路径 (封装 OsString/OsStr)
    //
    // 特性:
    //   - OsString 可以存储非 UTF-8 的有效字节 (如 Linux 文件名)
    //   - CString 末尾自动加 \0, 内部不允许 \0
    //   - PathBuf 和 OsString 几乎可以零成本互转
    //   - 三对类型都遵循 owned/borrowed 模式
    //
    // 避坑:
    //   - OsString → String 可能失败 (非 UTF-8 字节)
    //   - PathBuf → &str 可能失败 (非 UTF-8 路径)
    //   - CString 的 as_ptr() 返回的指针生命周期与 CString 绑定
    //
    use std::ffi::{CString, OsString};
    use std::path::PathBuf;

    // --- String → OsString (总是成功) ---
    let os = OsString::from("hello");
    assert_eq!(&os, "hello"); // OsString 实现了 PartialEq<&str>

    // OsString → String (可能失败)
    let os_ok = OsString::from("world");
    let s = os_ok.into_string().unwrap();
    assert_eq!(s, "world");

    // --- String → PathBuf (总是成功) ---
    let p = PathBuf::from("src/main.rs");
    assert!(p.ends_with("main.rs"));

    // PathBuf → &str (可能失败)
    let p_str = PathBuf::from("test.txt");
    assert_eq!(p_str.to_str(), Some("test.txt"));
    // to_string_lossy() 总是成功, 但可能替换无效字节
    assert_eq!(p_str.to_string_lossy(), "test.txt");

    // --- PathBuf 和 OsString 互转 (零成本) ---
    let os2 = OsString::from("file.txt");
    let path: PathBuf = os2.into(); // OsString → PathBuf
    assert_eq!(path.file_name().unwrap(), "file.txt");

    let path_parts = PathBuf::from("foo").join("bar");
    assert_eq!(path_parts, PathBuf::from("foo/bar").canonicalize().unwrap_or_else(|_| PathBuf::from("foo/bar")));
    // 注: 在 Windows 上 join 使用 \, 在 Unix 上使用 /

    // --- CString ---
    let c = CString::new("hello").unwrap();
    // 指针非空
    assert_ne!(c.as_ptr(), std::ptr::null());
    // 转回 &str
    assert_eq!(c.to_str().unwrap(), "hello");
    // 检查 null 终止符
    assert_eq!(c.to_bytes_with_nul().last(), Some(&0));

    // OsString → CString (如果包含 \0 会失败)
    let os3 = OsString::from("no_nulls_here");
    let c_str = CString::new(os3.to_str().unwrap()).unwrap();
    assert_eq!(c_str.to_str().unwrap(), "no_nulls_here");

    // --- 类型大小验证 ---
    // OsString, CString, PathBuf 都是堆分配类型, 大小不固定
    // 它们的内存结构类似 String (指针+长度+其他)

    // --- CString 不允许内部 null ---
    let c_result = CString::new("hello\0world");
    assert!(c_result.is_err());

    // 已废弃的 CString::from_raw 用法: 需要用 into_raw 先释放所有权
    // 这里仅演示基本 API, 不深入 unsafe

    // --- OsString 可以存储非 UTF-8 (Linux) ---
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::ffi::OsStringExt;
        // Linux 上 OsString 可以存储任意字节
        let non_utf8 = OsString::from_vec(vec![0xFF, 0xFE]);
        assert!(non_utf8.to_str().is_none()); // 不是有效 UTF-8
    }
    // Windows 上 OsString 使用 WTF-8 编码 (UTF-16 的变种)

    // --- 空 OsString / CString / PathBuf ---
    let empty_os = OsString::new();
    assert_eq!(empty_os.len(), 0);

    let empty_c = CString::new("").unwrap();
    assert_eq!(empty_c.to_bytes_with_nul(), [0]);
    assert_eq!(empty_c.to_bytes().len(), 0);

    let empty_p = PathBuf::new();
    assert_eq!(empty_p, PathBuf::from(""));
}
