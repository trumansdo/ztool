# Rust 错误处理：Option 与 Result

## 一、错误分类总览

> **金句引用**："错误不是异常，是类型——Rust 把错误编入了类型系统，让编译器替你检查。"

Rust 将错误分为两类，界限清晰，不容混淆：

| 分类 | 代表类型 | 恢复性 | 典型场景 |
|------|----------|--------|----------|
| 可恢复错误 | `Result<T, E>` / `Option<T>` | 调用者可决定处理方式 | 文件未找到、网络超时 |
| 不可恢复错误 | `panic!` | 立即终止线程 | 数组越界、除零 |

```rust
// 可恢复：调用者必须面对 None 或 Err
fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

// 不可恢复：违反程序契约，直接中止
fn index_out_of_bounds() {
    let v = vec![1, 2, 3];
    let _x = v[99]; // panic! 下标越界
}
```

---

## 二、Option\<T\>：值可有可无

> **金句引用**："Option 消灭了十亿美元的错误——空指针。"

### 2.1 定义与基本使用

```rust
enum Option<T> {
    None,     // 没有值
    Some(T),  // 有值 T
}

let x: Option<i32> = Some(42);
let y: Option<i32> = None;

// match 解构是最安全的方式
match x {
    Some(v) => println!("值是: {}", v),
    None    => println!("没有值"),
}

// 便捷判断方法
assert!(x.is_some());
assert!(!x.is_none());
```

### 2.2 取出值系列：unwrap / expect / unwrap_or

```rust
let some_val = Some(10);
let none_val: Option<i32> = None;

// unwrap: None 时直接 panic
let a = some_val.unwrap();             // 10

// expect: 带自定义信息的 panic
let b = some_val.expect("必须要有值");  // 10

// unwrap_or: 提供默认值，安全
let c = none_val.unwrap_or(0);         // 0
let d = none_val.unwrap_or_else(|| expensive_fallback());

// 返回引用系列
let e = some_val.as_ref();             // Option<&i32>
let f = some_val.as_mut();             // Option<&mut i32>
let g = some_val.as_deref();           // Option<&<i32 as Deref>::Target>
```

### 2.3 变换系列：map / map_or / map_or_else

> **金句引用**："Option 是容器——用 map 在容器内操作，永远不提前取出。"

```rust
let val = Some(3);

// map: 有值则变换，无值保持 None
let doubled = val.map(|x| x * 2);              // Some(6)
let none_doubled: Option<i32> = None.map(|x| x * 2); // None

// map_or: 变换后用默认值解包
let result = val.map_or(0, |x| x * 2);         // 6

// map_or_else: 惰性默认值
let result = None.map_or_else(
    || compute_default(),
    |x| x * 2,
);
```

### 2.4 组合系列：and / and_then / or / or_else / xor

```rust
let a = Some(1);
let b = Some(2);
let n: Option<i32> = None;

// and: 两者都有则返回第二个
assert_eq!(a.and(b), Some(2));
assert_eq!(a.and(n), None);

// and_then: 有值时链式调用（flat_map）
let result = a.and_then(|x| if x > 0 { Some(x * 10) } else { None });
assert_eq!(result, Some(10));

// or: 前者有值则前者，否则后者
assert_eq!(n.or(Some(99)), Some(99));

// or_else: 惰性备选
let result = n.or_else(|| compute_fallback());

// xor: 恰好一个为 Some 时返回
assert_eq!(a.xor(b), None);    // 两个都有 → None
assert_eq!(a.xor(n), Some(1)); // 只有一个 → 返回它
```

### 2.5 过滤与转换系列

```rust
let val = Some(100);

// filter: 不满足条件变 None
let filtered = val.filter(|&x| x > 50);   // Some(100)
let filtered = val.filter(|&x| x > 200);  // None

// flatten: 展平嵌套 Option
let nested: Option<Option<i32>> = Some(Some(5));
assert_eq!(nested.flatten(), Some(5));

// zip: 将两个 Option 压缩为元组
let x = Some(1);
let y = Some("hello");
assert_eq!(x.zip(y), Some((1, "hello")));

// ok_or: Option → Result 转换
let opt = Some(3);
let res: Result<i32, &str> = opt.ok_or("缺失值");
assert_eq!(res, Ok(3));

// transpose: Option<Result> ↔ Result<Option> 互换
let opt_res: Option<Result<i32, &str>> = Some(Ok(5));
let res_opt: Result<Option<i32>, &str> = opt_res.transpose();
assert_eq!(res_opt, Ok(Some(5)));

// get_or_insert: 原地获取或插入
let mut opt = None;
let val = opt.get_or_insert(42);
assert_eq!(*val, 42);
```

---

## 三、Result\<T, E\>：可恢复的错误

> **金句引用**："Result 是对错误的尊重——它告诉调用者：'我曾失败过，你得知道。'"

### 3.1 定义与基本使用

```rust
enum Result<T, E> {
    Ok(T),   // 成功，包含值
    Err(E),  // 失败，包含错误信息
}

use std::fs::File;
use std::io::ErrorKind;

fn open_file(path: &str) -> Result<File, std::io::Error> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                println!("文件不存在，尝试创建...");
                File::create(path)
            }
            other => Err(e),
        },
    }
}
```

### 3.2 ? 操作符深度解析

? 是 Rust 错误传播的"语法糖之王"，展开逻辑如下：

```rust
// 这行代码：
let file = File::open("data.txt")?;

// 等价于：
let file = match File::open("data.txt") {
    Ok(f)  => f,
    Err(e) => return Err(e.into()),  // 关键：自动调用 .into() ！
};
```

**自动类型转换链**：`?` 通过 `From` trait 自动将源错误类型转换为目标错误类型：

```rust
use std::io;
use std::num::ParseIntError;

// 自定义错误能接收 io::Error 和 ParseIntError
#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self { AppError::Io(e) }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self { AppError::Parse(e) }
}

// ? 自动完成转换
fn read_and_parse() -> Result<i32, AppError> {
    let content = std::fs::read_to_string("num.txt")?; // io::Error → AppError
    let num: i32 = content.trim().parse()?;             // ParseIntError → AppError
    Ok(num)
}
```

**Option 上的 ?**：

```rust
fn first_even(nums: &[i32]) -> Option<i32> {
    let first = nums.first()?;         // None 时提前返回
    if first % 2 == 0 { Some(*first) } else { None }
}
```

**main 返回 Result**：

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("config.toml")?;
    println!("{}", content);
    Ok(())
}
```

### 3.3 Result 方法全集

```rust
let ok_val: Result<i32, &str> = Ok(10);
let err_val: Result<i32, &str> = Err("失败");

// —— 变换 ——
ok_val.map(|x| x * 2);                          // Ok(20)
err_val.map(|x| x * 2);                         // Err("失败")
err_val.map_err(|e| format!("错误: {}", e));     // Err("错误: 失败")

// —— 组合 ——
ok_val.and_then(|x| if x > 5 { Ok(x) } else { Err("太小") });
err_val.or_else(|e| Ok(0));

// —— 兜底 ——
ok_val.or(Ok(99));                               // Ok(10)
err_val.or(Ok(99));                              // Ok(99)
err_val.unwrap_or(0);                            // 0
err_val.unwrap_or_else(|e| e.len() as i32);      // 6

// —— unwrap 家族 ——
ok_val.unwrap();          // 10
ok_val.expect("必须成功"); // 10
ok_val.unwrap_err();      // panic! (因为是 Ok)

// —— flatten ——
let nested: Result<Result<i32, &str>, &str> = Ok(Ok(5));
assert_eq!(nested.flatten(), Ok(5));
```

### 3.4 collect 妙用

```rust
// 收集 Vec<Result<T, E>> → Result<Vec<T>, E>
let results = vec![Ok(1), Ok(2), Ok(3)];
let collected: Result<Vec<i32>, &str> = results.into_iter().collect();
assert_eq!(collected, Ok(vec![1, 2, 3]));

// 遇到第一个错误即短路
let mixed = vec![Ok(1), Err("失败"), Ok(3)];
let collected: Result<Vec<i32>, &str> = mixed.into_iter().collect();
assert_eq!(collected, Err("失败"));
```

---

## 四、自定义错误类型

> **金句引用**："好的错误类型是 API 的说明书——看错误枚举就知道会出什么岔子。"

### 4.1 thiserror：属性驱动的错误定义

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),              // #[from] 自动生成 From 实现

    #[error("解析错误: 在第 {line} 行第 {col} 列")]
    Parse { line: usize, col: usize },        // 具名字段

    #[error("数据验证失败")]
    Validation(#[source] anyhow::Error),      // #[source] 标记底层错误源

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>), // 透明转发 Display + source()

    #[error("未知错误")]
    Unknown,
}
```

**属性表速查**：

| 属性 | 作用 |
|------|------|
| `#[error("...{0}...")]` | 定义 Display 输出模板 |
| `#[from]` | 自动生成 `From<Inner> for Self` |
| `#[source]` | 标记 `source()` 返回的底层错误 |
| `#[error(transparent)]` | 完全透明委托给内部错误 |

### 4.2 anyhow：应用层的错误报告

```rust
use anyhow::{bail, anyhow, ensure, Context, Result};

// bail! — 立即返回错误
fn validate_age(age: i32) -> Result<()> {
    if age < 0 {
        bail!("年龄不能为负: {}", age);
    }
    Ok(())
}

// anyhow! — 从字符串构建错误
fn get_user(id: u32) -> Result<String> {
    if id == 0 {
        return Err(anyhow!("无效的用户 ID: {}", id));
    }
    Ok(format!("用户_{}", id))
}

// ensure! — 断言式校验
fn withdraw(balance: f64, amount: f64) -> Result<f64> {
    ensure!(amount <= balance, "余额不足: {} < {}", balance, amount);
    Ok(balance - amount)
}

// context / with_context — 添加上下文信息
fn read_config() -> Result<String> {
    std::fs::read_to_string("config.toml")
        .context("读取配置文件失败")?;
    std::fs::read_to_string("config.toml")
        .with_context(|| format!("路径: {}", "config.toml"))?;
    Ok("内容".into())
}

// anyhow::Result<T> = Result<T, anyhow::Error>
fn app_main() -> anyhow::Result<()> {
    let config = read_config()?;
    println!("{}", config);
    Ok(())
}
```

---

## 五、Error Trait 与错误链

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct AppError {
    msg: String,
    source: Option<Box<dyn Error + 'static>>,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref()
    }
}

// 遍历错误链
fn print_error_chain(err: &dyn Error) {
    eprintln!("错误: {}", err);
    let mut source = err.source();
    while let Some(e) = source {
        eprintln!("  原因: {}", e);
        source = e.source();
    }
}

// Box<dyn Error> 用于擦除具体错误类型
fn dynamic_error() -> Result<(), Box<dyn Error>> {
    let _file = std::fs::File::open("不存在.txt")?;
    Ok(())
}
```

---

## 六、panic! 家族宏

> **金句引用**："panic 是开发期的警报器，`Result` 是运行期的安全带。"

| 宏 | 用途 | 示例 |
|----|------|------|
| `panic!()` | 立即终止当前线程 | `panic!("严重错误: {}", msg)` |
| `todo!()` | 标记未实现的功能 | `todo!("实现数据库连接")` |
| `unimplemented!()` | 同 todo! | `unimplemented!("网络层")` |
| `unreachable!()` | 标记不可能到达的分支 | `unreachable!("Option 已确保有值")` |

```rust
fn unfinished_feature() -> i32 {
    todo!("下周实现这个特性")  // 编译通过，运行触发 panic
}

fn assert_variant() -> i32 {
    match Some(5) {
        Some(x) => x,
        None    => unreachable!(), // 编译器优化提示
    }
}
```

### 6.1 catch_unwind：捕获 panic

```rust
use std::panic;

let result = panic::catch_unwind(|| {
    // 可能 panic 的代码
    let v = vec![1, 2, 3];
    v[99]
});

match result {
    Ok(_)  => println!("未发生 panic"),
    Err(e) => {
        if let Some(msg) = e.downcast_ref::<&str>() {
            println!("捕获 panic: {}", msg);
        }
    }
}

// AssertUnwindSafe 包装——告诉编译器你的闭包"对 unwind 安全"
use std::panic::AssertUnwindSafe;
let result = panic::catch_unwind(AssertUnwindSafe(|| {
    // 包含 &mut 引用的闭包，手动声明为 unwind 安全
}));
```

**catch_unwind 限制**：
- 不保证 unwind 安全性（可能留下不一致状态）
- 不捕获 `abort` 导致的终止
- 闭包必须实现 `UnwindSafe` trait
- 谨慎用于 FFI 边界隔离

---

## 七、四种错误处理模式总结

> **金句引用**："错误处理没有银弹，只有合适模式——场景决定策略。"

| 模式 | 方法 | 适用场景 |
|------|------|----------|
| **传播** | `?` 操作符 | 调用者应该知道并处理错误 |
| **转换** | `map_err` | 需要统一错误类型向上传递 |
| **兜底** | `unwrap_or` / `unwrap_or_else` | 有合理默认值的场景 |
| **报告** | `anyhow` + `context` | 应用顶层，用户可读的错误信息 |

```rust
// 四种模式实战对比
fn process(path: &str) -> anyhow::Result<String> {
    // 模式1 + 模式2: 传播 + 转换
    let raw = std::fs::read_to_string(path)
        .map_err(|e| anyhow!("读取失败: {}", e))?;

    // 模式3: 兜底（解析失败用空字符串）
    let trimmed = raw.trim().parse::<i32>().unwrap_or(0);

    // 模式4: 报告（上下文包装）
    ensure!(trimmed > 0, "解析出的值必须大于0");

    Ok(format!("处理结果: {}", trimmed))
}
```

---

## 八、Result 优于 panic 的设计决策树

> **金句引用**："能用 `Result` 就别用 `panic`——把选择的权力交给调用者。"

```
遇到错误状况
├─ 是程序员 bug（数组越界、除零等）
│  └─ 用 panic! / unreachable!
├─ 是外部环境不可控（文件、网络、用户输入）
│  ├─ 调用者能恢复？
│  │  ├─ 是 → 用 Result<T, E>
│  │  └─ 否 → 用 panic!（罕见）
│  └─ 不确定？
│     └─ 用 Result<T, E>（宁可过度也不疏漏）
└─ 值可能为空？
   └─ 用 Option<T>
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 滥用 `unwrap()` 造成生产崩溃 | `unwrap` 在 `None`/`Err` 时直接 panic | 使用 `?` 传播或 `unwrap_or_else` 兜底 |
| `?` 在 `Option` 和 `Result` 间混用 | 两种类型之间没有 `From` 转换 | 使用 `ok_or()` / `ok_or_else()` 桥接 |
| `catch_unwind` 捕获后继续使用状态 | panic 可能破坏数据结构的不变性 | 捕获后立即终止该操作，不恢复状态 |
| `Box<dyn Error>` 无法向下转型 | 类型擦除后丢失具体错误信息 | 需要具体处理时使用枚举 + `#[from]` |
| `anyhow` 用于库而非应用 | anyhow 的错误是**不透明**的，调用者无法匹配 | 库用 `thiserror` 定义明确错误枚举 |
| 忽略 `Result` 返回值 | 未使用 `must_use` 的 Result 会被静默忽略 | 至少用 `let _ = ...` 或 `?` 显式处理 |
| `map_err` 后丢失原始错误 | 只转换了外层，丢弃了 `source()` 链 | 使用 `#[error(transparent)]` 保持错误链 |
| 用 `panic!` 处理用户输入错误 | panic 不是面向用户的错误报告机制 | 校验输入返回 `Result::Err`，给出友好提示 |

> **详见测试**: `tests/rust_features/09_error_handling.rs`
