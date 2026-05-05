# 诊断属性 (Diagnostic Attributes)

## 目录
1. [弃用标注 #[deprecated]](#弃用标注-deprecated)
2. [未使用返回值警告 #[must_use]](#未使用返回值警告-must_use)
3. [外部穷举性限制 #[non_exhaustive]](#外部穷举性限制-non_exhaustive)
4. [自定义 trait 错误信息](#自定义-trait-错误信息)
5. [跟踪调用位置 #[track_caller]](#跟踪调用位置-track_caller)
6. [冷路径优化 #[cold]](#冷路径优化-cold)
7. [lint 级别属性](#lint-级别属性)
8. [条件诊断 cfg_attr](#条件诊断-cfg_attr)
9. [属性继承与作用域](#属性继承与作用域)
10. [避坑指南](#避坑指南)

---

## 弃用标注 #[deprecated]

`#[deprecated]` 属性用于标记一个 API 已过时，编译器会在使用者代码中发出警告。可以附带 `since` 和 `note` 参数说明更替信息。

> 过时的代码就像旧地图——标注清楚才能让人找到新路。

基本用法：

```rust
#[deprecated(since = "1.2.0", note = "请使用 new_process 函数代替")]
fn old_process(data: &str) {
    println!("旧处理: {data}");
}

fn new_process(data: &str) {
    println!("新处理: {data}");
}

fn caller() {
    old_process("test"); // 编译警告: use of deprecated function `old_process`
}
```

多层弃用——标注模块和类型：

```rust
#[deprecated(since = "2.0.0", note = "此模块已迁移至 crate::v2")]
mod old_api {
    #[deprecated(since = "2.0.0", note = "请使用 v2::Handler")]
    pub struct Handler;
}

// 弃用枚举的特定变体（使用单独的常量或函数）
struct Api;
impl Api {
    #[deprecated(since = "3.0.0", note = "请使用 Api::v2_get")]
    pub fn get(&self) {}
    pub fn v2_get(&self) {}
}
```

支持的类型级弃用：

```rust
#[deprecated(since = "4.1.0", note = "使用 NewConfig 替代")]
pub struct OldConfig {
    pub timeout: u32,
}

pub struct NewConfig {
    pub timeout_ms: u64,
}
```

---

## 未使用返回值警告 #[must_use]

`#[must_use]` 标注在函数或类型上，当返回值未被使用时编译器会发出警告。这是防范疏忽型 bug 的重要手段。

> 沉默是金，但被忽略的返回值是定时炸弹。

函数级 `#[must_use]`：

```rust
#[must_use]
fn compute_checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
}

fn demo() {
    compute_checksum(b"hello"); // 编译警告: unused return value
    let _ = compute_checksum(b"hello"); // 显式忽略，无警告
}
```

类型级 `#[must_use]`——所有返回该类型的位置都会触发警告：

```rust
#[must_use]
struct Transaction {
    id: u64,
    committed: bool,
}

impl Transaction {
    fn begin(id: u64) -> Self {
        Transaction { id, committed: false }
    }

    fn commit(mut self) {
        self.committed = true;
        println!("事务 {} 已提交", self.id);
    }
}

fn risk() {
    Transaction::begin(1); // 警告: unused Transaction that must be used
    Transaction::begin(2).commit(); // 正确使用
}
```

带自定义信息的 `#[must_use]`：

```rust
#[must_use = "此 Future 若未被轮询则不会执行任何工作"]
async fn lazy_work() -> u32 { 42 }

#[must_use = "Result 不处理可能导致静默失败"]
fn fallible_op() -> Result<u32, &'static str> {
    Ok(100)
}
```

---

## 外部穷举性限制 #[non_exhaustive]

`#[non_exhaustive]` 标注在枚举或结构体上，限制**外部 crate** 对其进行穷举匹配或结构体字面量构造，为将来增加变体/字段保留兼容性空间。

> 永远不要说"只有这么多"——`#[non_exhaustive]` 是 API 设计中的远见。

枚举上的 `#[non_exhaustive]`：

```rust
// crate A (库)
#[non_exhaustive]
pub enum Error {
    NotFound,
    PermissionDenied,
    Timeout,
}

// crate B (使用者)
fn handle_error(e: Error) {
    match e {
        Error::NotFound => println!("未找到"),
        Error::PermissionDenied => println!("权限不足"),
        Error::Timeout => println!("超时"),
        _ => println!("其他错误"), // 必须有通配分支，否则编译错误
    }
}
```

结构体上的 `#[non_exhaustive]`：

```rust
// 库侧定义
#[non_exhaustive]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
}

// 使用者不能用 struct literal 创建
// let cfg = WindowConfig { width: 800, height: 600 }; // 编译错误！

// 库必须提供构造器
impl WindowConfig {
    pub fn new(width: u32, height: u32) -> Self {
        WindowConfig { width, height }
    }
}

// 使用者正确创建
fn create_window() {
    let cfg = WindowConfig::new(800, 600);
}
```

枚举与结构体联合使用：

```rust
#[non_exhaustive]
pub struct Request {
    pub url: String,
    pub method: String,
}

#[non_exhaustive]
pub enum Response {
    Ok { body: String },
    Err { code: u16 },
}

// 未来可以安全地给 Request 添加字段、给 Response 添加变体
// 而不会破坏外部使用者的代码
```

---

## 自定义 trait 错误信息

`#[diagnostic::on_unimplemented]` 属性允许为 trait 定制当类型未实现该 trait 时的编译器错误信息，提升诊断可读性。

> 好的错误信息胜过十行代码注释——它直接告诉使用者"差哪一步"。

基本用法：

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` 必须实现序列化",
    label = "此类型缺少 Serialize 实现",
    note = "考虑使用 #[derive(Serialize)] 自动实现"
)]
pub trait Serialize {
    fn serialize(&self) -> String;
}

struct MyType;
// 当 MyType 未实现 Serialize 却被要求时，
// 编译器会显示上述自定义消息而非默认的 "the trait bound ... is not satisfied"
```

条件化消息——根据缺失的 trait 参数展现不同信息：

```rust
use std::fmt::Display;

#[diagnostic::on_unimplemented(
    message = "`{Self}` 不能转换为 `{Into}`",
    label = "缺少 From<{Self}> for {Into} 的实现",
    note = "尝试实现 `impl From<{Self}> for {Into}`"
)]
pub trait Convertible<Into> {
    fn convert(self) -> Into;
}
```

多层 trait 约束的友好提示：

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` 不能作为键，因为它未实现 Hash + Eq",
    label = "此类型缺少 HashMap 键所需的能力",
    note = "为 `{Self}` 实现 Hash 和 Eq trait"
)]
pub trait MapKey: std::hash::Hash + Eq {}

impl<T: std::hash::Hash + Eq> MapKey for T {}
```

---

## 跟踪调用位置 #[track_caller]

`#[track_caller]` 标注在函数上时，函数内部的 `panic!()` 和 `std::panic::Location::caller()` 会报告调用者的位置，而不是被调用函数自身的位置。

> 甩锅要甩得精准——#[track_caller] 让 panic 信息指向真正的肇事者。

基本对比：

```rust
// 没有 #[track_caller]——panic 指向 helper 内部
fn helper_bad(index: usize, slice: &[i32]) -> &i32 {
    &slice[index]
}

// 有 #[track_caller]——panic 指向 caller_bad() 的调用位置
#[track_caller]
fn helper_good(index: usize, slice: &[i32]) -> &i32 {
    &slice[index]
}

fn caller_good() {
    let data = vec![1, 2, 3];
    let _ = helper_good(10, &data); // panic 信息指向此处，而非 helper 内部
}
```

自定义断言函数：

```rust
#[track_caller]
pub fn assert_non_empty(s: &str) {
    if s.is_empty() {
        panic!("字符串不能为空");
    }
}

fn validate() {
    assert_non_empty(""); // panic 定位到这一行
}
```

类型构造器中的 `#[track_caller]`：

```rust
pub struct SafeSlice<'a> {
    data: &'a [u8],
}

impl<'a> SafeSlice<'a> {
    #[track_caller]
    pub fn new(data: &'a [u8], min_len: usize) -> Self {
        assert!(data.len() >= min_len, "数据长度不足");
        SafeSlice { data }
    }
}
```

---

## 冷路径优化 #[cold]

`#[cold]` 标注在函数上，告诉编译器此函数不太可能被执行，帮助优化器将热路径和冷路径分离，提升指令缓存效率。

> 把异常路线标记为冷路径，就是为正常路线清扫跑道。

基本用法：

```rust
// 错误处理函数通常执行概率低，适合标记为冷
#[cold]
fn allocation_failed() -> ! {
    eprintln!("致命错误：内存分配失败");
    std::process::abort();
}

fn allocate(size: usize) -> Vec<u8> {
    if size > 10_000_000 {
        allocation_failed();
    }
    vec![0u8; size]
}
```

与 `#[inline]` 的配合：

```rust
// 热路径——尽量内联
#[inline]
fn process_fast(data: &[u8]) -> u32 {
    data.iter().map(|&b| b as u32).sum()
}

// 冷路径——展开失败处理
#[cold]
fn process_slow_fallback(data: &[u8]) -> u32 {
    // 复杂但极少执行的逻辑
    data.iter().map(|&b| b as u32 * 2).sum()
}

fn process(data: &[u8]) -> u32 {
    if data.len() < 1000 {
        process_fast(data)
    } else {
        process_slow_fallback(data)
    }
}
```

`#[cold]` 的原理：编译器将标记为 cold 的函数放置在代码段的末尾区域，减少指令缓存污染并让分支预测器默认走向热路径。

---

## lint 级别属性

Rust 提供四个 lint 级别属性，允许在模块/函数/语句级别控制编译器检查的严格程度。

> lint 是你雇的代码审查团队——allow 让他们闭嘴，deny 让他们拍桌子。

四个级别：

```rust
// 1. allow —— 允许，不报告
#[allow(dead_code)]
fn unused_but_ok() {}

// 2. warn —— 警告（默认级别）
#[warn(missing_docs)]
pub mod documented_module {
    /// 有文档的函数
    pub fn documented() {}
}

// 3. deny —— 禁止，产生编译错误
#[deny(unsafe_code)]
mod safe_only {
    pub fn add(a: i32, b: i32) -> i32 { a + b }
    // unsafe { ... } // 编译错误！
}

// 4. forbid —— 禁止且不可被覆盖
#[forbid(unsafe_code)]
mod absolutely_safe {
    // 即使子模块用 #[allow(unsafe_code)] 也无法恢复
    pub fn safe_fn() {}
}
```

常用 lint 名称：

```rust
// 在 crate 根级声明 lint 策略
#![deny(
    missing_docs,
    rust_2018_idioms,
    unsafe_code,
    clippy::all,
    clippy::pedantic,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
)]
```

作用域局部覆盖：

```rust
#[allow(clippy::all)]
mod legacy_code {
    pub fn old_fn(x: i32) -> i32 { x + 1 }
}

// 仅对特定表达式
fn precise_control() {
    #[allow(unused_variables)]
    let temp = 42;
}
```

---

## 条件诊断 cfg_attr

`cfg_attr` 允许根据编译配置条件性地应用属性，常用于跨平台或 feature-gated 代码。

> 诊断也要审时度势——cfg_attr 让属性随编译环境而变。

基本用法：

```rust
// 仅在测试环境下启用特定 lint
#[cfg_attr(test, allow(dead_code))]
fn test_helper() -> u32 { 42 }

// 根据 feature 切换 deprecation
#[cfg_attr(
    feature = "new_api",
    deprecated(since = "1.5.0", note = "请使用新 API")
)]
pub fn conditional_deprecated() {}
```

跨平台的 lint 控制：

```rust
#[cfg_attr(target_os = "windows", allow(unused_imports))]
#[cfg_attr(target_os = "linux", deny(missing_docs))]
mod platform_specific {
    // Windows 上允许未使用导入，Linux 上要求文档
}
```

组合条件属性：

```rust
#[cfg_attr(
    all(debug_assertions, not(feature = "no_tracking")),
    track_caller
)]
pub fn debug_tracked_assert(condition: bool, msg: &str) {
    if !condition {
        panic!("断言失败: {msg}");
    }
}
```

---

## 属性继承与作用域

诊断属性的作用域从声明点向下级联，但可被更内层的作用域覆盖。

> 属性像家族的规矩——祖先定的可以被儿孙辈重新商议。

继承规则：

```rust
#![deny(unsafe_code)] // crate 级：整个 crate 不能有 unsafe

mod parent {
    // 继承了 crate 的 deny(unsafe_code)

    mod child {
        // 同样继承 deny(unsafe_code)
    }
}

#[allow(unsafe_code)] // 此模块的 allow 可以覆盖上级的 deny
mod exception {
    pub unsafe fn low_level_op() {
        // allow 可以覆盖 deny，但无法覆盖 forbid
    }
}
```

forbid 的不可覆盖性：

```rust
#![forbid(unsafe_code)]

// 以下都无法绕过 forbid：
// #[allow(unsafe_code)] mod nope1 { }
// fn inner() { #[allow(unsafe_code)] { } }
// 这就是 forbid 和 deny 的关键区别
```

诊断属性继承速查表：

| 属性 | crate 级 | 模块级 | 函数级 | 可被覆盖 |
|------|----------|--------|--------|----------|
| `allow` | 是 | 是 | 是 | 是 |
| `warn` | 是 | 是 | 是 | 是 |
| `deny` | 是 | 是 | 是 | 是 |
| `forbid` | 是 | 是 | 是 | **否** |
| `deprecated` | — | 是 | 是 | 不适用 |
| `must_use` | — | 是 | 是 | 不适用 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `forbid` 后试图用 `allow` 覆盖 | `forbid` 级别不可被覆盖，`allow` 无效 | 如需局部豁免，不要在上级用 `forbid`，改用 `deny` |
| `deprecated` 标注的函数自身内部使用了另一个 deprecated | 产生级联警告，干扰诊断 | 在实现内部添加 `#[allow(deprecated)]` 抑制 |
| `#[must_use]` 类型被用于泛型约束时产生噪音 | 泛型代码中可能需要丢弃该类型 | 使用 `let _ = value;` 显式忽略 |
| `#[non_exhaustive]` 结构体未提供构造器 | 外部完全无法创建实例 | 必须提供 `new()` 或其他构造方法 |
| `#[track_caller]` 用于 trait 方法 | trait 方法不允许标注 `#[track_caller]` | 在 impl 块的方法上标注，而非 trait 定义上 |
| `#[cold]` 标注在递归函数上 | 可能干扰尾递归优化和栈帧分配 | 评估是否真的极少执行，避免滥用 |
| cfg_attr 条件表达式过于复杂 | 难以理解和维护 | 将复杂条件拆分为独立的 feature flag |

---

> **详见测试**: `tests/rust_features/30_diagnostic_attributes.rs`
