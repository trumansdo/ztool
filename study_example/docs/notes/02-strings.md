# 字符串类型：String 与 &str

> 在 Rust 中，字符串不是一个类型，而是一整个类型家族——理解 String 与 &str 的区别，是 Rust 入门的第一道分水岭。

## 一、核心区别

| 特性 | String | &str |
|------|--------|------|
| 所有权 | 拥有 (Owned)，数据在堆上 | 借用 (Borrowed)，指向别处 |
| 可变性 | 可修改（需 `mut`） | 不可变 |
| 栈占用 | 24 字节（ptr+len+cap） | 16 字节（ptr+len） |
| 增长 | 支持 push/push_str | 固定长度视图 |
| 编码 | UTF-8 | UTF-8 |

```rust
let s = String::from("hello");   // String：拥有堆数据
let slice: &str = &s;             // &str： 借用，不拥有数据
let literal: &str = "你好世界";   // 字面量：&'static str
```

> 所有权不是束缚，而是自由——它让你无需 GC 也能安全地管理内存。

---

## 二、String 内部布局：24 字节

64 位系统上，每个 `String` 在栈上占据固定的 **24 字节**：

| 字段 | 大小 | 说明 |
|------|------|------|
| 指针 (pointer) | 8 字节 | 指向堆上字节缓冲区起始地址 |
| 长度 (length) | 8 字节 | 当前已使用字节数，即 `len()` |
| 容量 (capacity) | 8 字节 | 已分配总字节数，即 `capacity()` |

```
栈 (24 字节)               堆 (capacity 字节)
┌──────────────────┐      ┌─────────────────────┐
│ ptr ────────────────────→│ h  e  l  l  o  ?  ?  │
│ len = 5           │      │─── 有效数据 ────│    │
│ capacity = 8      │      │────── 容量 ────────→ │
└──────────────────┘      └─────────────────────┘
```

`len() <= capacity()` 始终成立；`clear()` 将 `len()` 归零但保留 `capacity()`；`String::new()` 不分配堆内存。

> 每一个 String 都是 24 字节的承诺：指针指向堆上的数据，长度告诉你已走多远，容量告诉你还能走多远。

---

## 三、增长策略

```rust
let mut s = String::new();
for _ in 0..10 {
    s.push('a');
    println!("len={}, cap={}", s.len(), s.capacity());
}
// 典型输出: cap 按 0→4→8→16 翻倍增长，摊销 O(1)
```

| 操作 | 行为 |
|------|------|
| `String::from("xxx")` | 精确分配，`capacity == len` |
| `push('x')` / `push_str("...")` | 容量不足时翻倍：`new_cap = max(old*2, new_len)` |
| `with_capacity(n)` | 预分配 n 字节，避免后续多次扩容 |
| `shrink_to_fit()` | 收缩容量至长度，释放多余内存 |
| `reserve(n)` | 确保至少还有 n 字节可用空间 |

```rust
let mut s = String::with_capacity(1000);
for word in words {
    s.push_str(word);
    s.push(' ');
}
// 全程零次重新分配（假设总长 <= 1000）
```

> String 的增长策略简单而有效：容量不足时翻倍，空间换时间。

---

## 四、UTF-8 编码与字节边界陷阱

UTF-8 变长编码，一个字符占 1~4 字节：

| 范围 | 字节数 | 示例 |
|------|--------|------|
| U+0000 ~ U+007F (ASCII) | 1 | `'H'`, `'a'` |
| U+0080 ~ U+07FF | 2 | `'©'`, `'é'` |
| U+0800 ~ U+FFFF（含中文） | 3 | `'你'`, `'世'` |
| U+10000 ~ U+10FFFF（含 emoji） | 4 | `'🤖'`, `'🚀'` |

**陷阱：切片索引是字节索引，落在字符中间会 panic！**

```rust
let s = String::from("你好");       // 每个中文 3 字节
// let _ = &s[0..1];               // panic! byte index 1 is not a char boundary
assert_eq!(s.get(0..1), None);     // get() 安全返回 None
assert_eq!(s.get(0..3), Some("你")); // 正确

let emoji = String::from("🤖");    // 4 字节: F0 9F A4 96
assert_eq!(emoji.len(), 4);        // 字节数
assert_eq!(emoji.chars().count(), 1); // 字符数
assert_eq!(emoji.get(0..1), None);
assert_eq!(emoji.get(0..4), Some("🤖"));
```

> 在 Rust 中切字符串不是切字符，而是切字节——忘记这一点，panic 就会找上门。

---

## 五、chars() vs bytes() vs char_indices()

| 方法 | 迭代项 | 适用场景 |
|------|--------|----------|
| `chars()` | `char` | 按字符遍历，人类可读的文本处理 |
| `bytes()` | `u8` | 底层字节操作、网络协议、编码转换 |
| `char_indices()` | `(usize, char)` | 需要每个字符的字节偏移量 |

```rust
let s = "Rust世界";

let chars: Vec<char> = s.chars().collect();
assert_eq!(chars, vec!['R', 'u', 's', 't', '世', '界']);

let bytes: Vec<u8> = s.bytes().collect();
assert_eq!(bytes.len(), 10);  // 4 ASCII + 2×3 中文 = 10

for (idx, ch) in s.char_indices() {
    println!("字符 '{}' 从字节索引 {} 开始", ch, idx);
}
// 输出: R→0, u→1, s→2, t→3, 世→4, 界→7

assert_eq!(s.chars().count(), 6);  // 6 个字符
assert_eq!(s.len(), 10);           // 10 个字节——二者天差地别
```

> Rust 的字符串方法族命名一致、语义清晰——学会一个，就能推演出十个。

---

## 六、分割方法族

```rust
let csv = "a,b,,c,d";
assert_eq!(csv.split(',').collect::<Vec<_>>(), vec!["a", "b", "", "c", "d"]);
assert_eq!(csv.splitn(3, ',').collect::<Vec<_>>(), vec!["a", "b", ",c,d"]);
assert_eq!("a.b.c".rsplit('.').collect::<Vec<_>>(), vec!["c", "b", "a"]);
assert_eq!("a,b,".split_terminator(',').collect::<Vec<_>>(), vec!["a", "b"]);
assert_eq!("a,b,c".split_inclusive(',').collect::<Vec<_>>(), vec!["a,", "b,", "c"]);
assert_eq!(
    "  hello   world\n\t".split_whitespace().collect::<Vec<_>>(),
    vec!["hello", "world"]
);
assert_eq!(
    "line1\nline2\r\nline3".lines().collect::<Vec<_>>(),
    vec!["line1", "line2", "line3"]
);
```

| 方法 | 分隔依据 | 连续分隔符行为 |
|------|----------|----------------|
| `split(pat)` | 指定分隔符 | 产生空串 |
| `splitn(n, pat)` | 指定分隔符，最多 n 段 | 同 split |
| `rsplit(pat)` | 指定分隔符，从右向左 | 产生空串 |
| `split_terminator(pat)` | 指定分隔符 | 忽略尾部空串 |
| `split_inclusive(pat)` | 指定分隔符 | 分隔符包含在前一段中 |
| `split_whitespace()` | Unicode 空白字符 | 连续空白被跳过 |
| `lines()` | `\n` 或 `\r\n` | 不产生空串 |

> 分割操作返回的都是迭代器，惰性求值、零内存分配——直到你调用 collect。

---

## 七、修剪与查询替换

### trim 系列（返回 &str 切片，零拷贝）

```rust
let s = "  \t hello 世界 \n  ";
assert_eq!(s.trim(), "hello 世界");
assert_eq!(s.trim_start(), "hello 世界 \n  ");
assert_eq!(s.trim_end(), "  \t hello 世界");

// trim_matches 按字符集裁剪
assert_eq!("--hello--".trim_matches('-'), "hello");
assert_eq!("<>hello</>".trim_matches(&['<', '>', '/'][..]), "hello");
```

### 查询与替换

```rust
let s = "hello world";
assert!(s.starts_with("hello"));
assert!(s.ends_with("world"));
assert!(s.contains("lo wo"));
assert!("hello".starts_with('h'));   // 也支持 char

let s2 = s.replace("o", "0");
assert_eq!(s2, "hell0 w0rld");
assert_eq!(s, "hello world");        // 原串未改变

let s3 = s.replacen("o", "0", 1);
assert_eq!(s3, "hell0 world");
assert_eq!("a_b_c".replace('_', "-"), "a-b-c");
```

> trim 调用零成本不分配，replace 返回新 String 有代价——选对方法，性能天差地别。

---

## 八、to_lowercase / to_uppercase — Unicode 陷阱

大小写转换不只是 ASCII 的加减 32，特定 Unicode 字符会改变长度：

```rust
// 德语 ß 转大写：长度从 6 变成 7！
let german = "Straße";
let upper = german.to_uppercase();
assert_eq!(upper, "STRASSE");        // ß → SS

assert_eq!("ß".to_uppercase(), "SS");
assert_eq!("ß".to_uppercase().len(), 2); // 一个字符变成两个

// ASCII 版本只处理 A-Z/a-z，更快但狭窄
assert_eq!("Hello ß".to_ascii_lowercase(), "hello ß");  // ß 不变
```

> Unicode 大小写转换不是简单的字节加减 32——德语 ß 转大写后变成两个字符 SS，字符串长度也随之改变。

---

## 九、六种字符串连接方式性能对比

**性能排序（从快到慢）**：`concat!` > `push_str`/`+=` > `+` > `join` > `format!`

| 方式 | 内存分配 | 所有权影响 | 适用场景 |
|------|----------|------------|----------|
| `push_str(&s2)` | 可能扩容 | 仅需 `&mut self` | 高效追加，推荐 |
| `s1 + &s2` | 可能扩容 | s1 被移动 | 少量拼接 |
| `s1 += &s2` | 可能扩容 | s1 保留（需 mut） | 连续追加 |
| `format!("{}{}", a, b)` | 每次新建 String | 原值全部保留 | 灵活组合 |
| `[s1, s2].join("")` | 新建 String | 元素被移动 | 多片段一遍拼接 |
| `concat!(s1, s2)` | 编译期完成 | 仅字面量 | 编译期常量拼接 |

```rust
let mut s = String::from("hello");
s.push_str(" world");      // 推荐方式

let s2 = String::from("hello");
let s3 = s2 + " world";    // s2 已失效

let a = String::from("hello");
let b = String::from("world");
let s4 = format!("{} {}", a, b);  // a, b 仍可用
```

> 拼字符串的方法有六种，选择哪种取决于你是否需要保留原值、是否在意性能。

---

## 十、format! 宏 / to_string / to_owned / into

```rust
let name = "Alice";
let msg = format!("我叫{}，今年{}岁", name, 30);

let s1: String = "hello".to_string();    // 调用 fmt::Display
let s2 = String::from("hello");          // 从 &str 构造
let s3: String = "hello".to_owned();     // 语义与 to_string 等效
let s4: String = "hello".into();         // 类型推断驱动

// &String 自动转为 &str（Deref 强制多态）
fn greet(name: &str) { println!("你好，{}", name); }
greet(&String::from("张三"));  // &String → &str 零成本
greet("李四");                  // 字面量直接传入
```

> 从 String 到 &str 只需借用（零成本）；从 &str 到 String 则需要克隆（有代价）。

---

## 十一、原始字符串与字节字符串

```rust
// 原始字符串：反斜杠不需要转义
let path = r"C:\Users\truma\Documents";
let regex = r"\d{3}-\d{4}-\d{4}";

// 包含双引号：r#"..."#
let json = r#"{"name": "Rust", "version": "1.70"}"#;

// 内容含 "# 时增加 # 数量
let tricky = r##"这里包含 "# 字面量"##;
let deeply = r###"他说："r###嵌套"###在这里"###;

// 字节字符串：产出 &[u8; N]，不含 UTF-8 约束
let bs: &[u8; 5] = b"hello";
let bs_hex = b"\x48\x65\x6c\x6c\x6f\x21";  // \xNN 转义

// raw 字节字符串
let raw_bytes = br"C:\Users";             // br"..."
let raw_json = br#"{"key": "value"}"#;    // br#"..."#
```

> 原始字符串让转义地狱变成历史，字节字符串让网络协议不再需要 UTF-8 担保。

---

## 十二、OsString / CString / PathBuf 对照表

| 类型对 (Owned / Borrowed) | 用途 | 编码 | 典型来源 |
|---------------------------|------|------|----------|
| `String` / `&str` | 通用 Rust 字符串 | UTF-8 | 字面量、文本处理 |
| `OsString` / `&OsStr` | 操作系统原生字符串 | 平台相关 | `std::env::args()` |
| `CString` / `&CStr` | C 兼容 null 结尾字符串 | 任意字节 | FFI 交互 |
| `PathBuf` / `&Path` | 文件系统路径 | 封装 OsStr | `std::fs` API |

```rust
use std::ffi::{CString, OsString};
use std::path::PathBuf;

let os = OsString::from("hello");
let s = os.into_string().unwrap();      // 反向可能失败

let c = CString::new("hello").unwrap(); // 检查无内部 \0

let p = PathBuf::from("src/main.rs");
let s = p.to_str().unwrap();            // Path 可能非 UTF-8

let os_str = OsString::from("test.txt");
let path: PathBuf = os_str.into();      // OsString ↔ PathBuf 零成本
```

> 与操作系统对话用 OsString，与 C 通信用 CString，处理文件路径有 PathBuf。

---

## 十三、&str 的生命周期与函数参数黄金法则

```rust
// 字符串字面量 → 'static 生命周期
let literal: &'static str = "hello";

// 函数参数黄金法则：优先使用 &str（不推荐 &String）
fn process(s: &str) { println!("{}", s); }
process(&String::from("hello")); // &String → &str（Deref 多态）
```

> &str 是 Rust 最巧妙的零成本抽象——一个指针加一个长度，就构成了对任意 UTF-8 数据的不可变视图。

---

## 十四、创建与修改 String 常用方法

```rust
// 创建
let s1 = String::new();                     // 空
let s2 = String::from("hello");             // 从 &str
let s3 = "hello".to_string();               // to_string
let s4 = String::with_capacity(64);         // 预分配

// 修改（需 mut）
let mut s = String::from("foo");
s.push('b');              // 追加单个 char
s.push_str("ar");         // 追加 &str
s.insert(0, '#');         // 插入 char（小心字节边界！）
s.insert_str(0, ">>");    // 插入 &str
s.pop();                  // 删除末尾字符，返回 Option<char>
s.remove(0);              // 删除指定字节索引处字符
s.truncate(3);            // 截断到指定字节长度
s.clear();                // 清空内容，保留容量
s.shrink_to_fit();        // 收缩容量至长度
```

> 函数参数用 &str，存储数据用 String——这条规则能解决 90% 的决策问题。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 直接字节索引 `s[i]` | Rust 不允许，UTF-8 变长编码 | 使用 `.chars().nth(i)` 或按字符迭代 |
| 非法 UTF-8 边界切片 panic | 切片索引落在多字节字符中间 | 使用 `.get(..)` 安全切片，或用 `char_indices()` |
| 混淆 `len()` 与 `chars().count()` | 中文/emoji 字节数远大于字符数 | 字符数用 `chars().count()`，字节数用 `len()` |
| `+` 运算符移动左侧 String | `s1 + &s2` 后 s1 失效 | 多数场景用 `format!` 或 `push_str` |
| 函数参数用 `&String` | 无法接收 `&str` 字面量 | 始终用 `&str` 作为参数类型 |
| 大小写转换改变长度 | `"Straße".to_uppercase()` 从 6 变 7 | 不假设大小写转换前后长度相同 |
| 循环中反复 `format!` 拼接 | 每次分配新内存，性能差 | 预分配 `with_capacity`，用 `push_str` 累积 |
| `replace` 返回新 String 但原串没变 | 非原地修改 | `let new = s.replace(...)` 接收返回值 |
| 非字符边界使用 `insert`/`remove` | 运行时 panic | 索引必须来自 `char_indices()` 或 `find()` |
| 组合 emoji 被 `chars()` 拆分 | `"👨‍👩‍👧‍👦".chars().count()` = 7 | 字形簇拆分需 `unicode-segmentation` crate |

> 字符串的坑，归根到底只有两个：忘记 UTF-8 字节边界，和混淆所有权。记住这两点，你就避开了 95% 的陷阱。

> **详见测试**: `tests/rust_features/02_strings.rs`
