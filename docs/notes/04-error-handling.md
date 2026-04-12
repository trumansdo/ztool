# 04 - 错误处理：Option 与 Result

## 概述

Rust 以其无与伦比的可靠性著称，而这背后最大的功臣就是其独特的错误处理机制。它在编译时就强制我们思考并处理潜在的失败，从根源上消除了大量的运行时错误。

Rust 使用类型系统处理错误，有两种主要方式：

### Option<T> - 表示可能存在或不存在的值

```rust
enum Option<T> {
    Some(T),  // 值存在
    None,     // 值不存在
}
```

**使用场景**：
- 函数返回值可能为空
- 查找操作可能找不到结果
- 可选配置或参数

### Result<T, E> - 表示操作成功或失败

```rust
enum Result<T, E> {
    Ok(T),   // 操作成功，返回值 T
    Err(E),  // 操作失败，返回错误 E
}
```

**使用场景**：
- 文件 I/O 操作
- 网络请求
- 数据解析
- 任何可能失败的操作

## 一、Option<T> 详解

### 创建 Option

```rust
// 常见创建方式
let some_value: Option<i32> = Some(5);
let absent_value: Option<i32> = None;

// 从数组中查找元素
let numbers = vec![1, 2, 3, 4, 5];
let result = numbers.iter().find(|&&x| x == 10);
// result 类型是 Option<&i32>
```

### Option 的核心方法

#### 1. unwrap 系列 - 简单但危险

```rust
let some_value = Some(5);
let unwrapped = some_value.unwrap(); // 返回 5

let none_value: Option<i32> = None;
let unwrapped = none_value.unwrap(); // panic!
```

**避坑**：不要在生产代码中使用 `unwrap()` 处理 `None`，除非你确定值一定存在。

#### 2. unwrap_or / unwrap_or_else - 安全取值

```rust
// unwrap_or: None 时返回默认值
let value = Some(5).unwrap_or(0); // 5
let value = None.unwrap_or(0);    // 0

// unwrap_or_else: None 时调用闭包（惰性求值）
let value = None.unwrap_or_else(|| {
    println!("执行了闭包");
    42
});
```

**适用场景**：当默认值计算成本较低时用 `unwrap_or`，成本较高时用 `unwrap_or_else`。

#### 3. map - 转换内部值

```rust
let number = Some(5);
let doubled = number.map(|x| x * 2); // Some(10)

let number: Option<i32> = None;
let doubled = number.map(|x| x * 2); // None（不会调用闭包）
```

#### 4. and_then - 链式处理

```rust
fn get_username(id: u32) -> Option<String> {
    if id == 1 {
        Some("alice".to_string())
    } else {
        None
    }
}

fn get_email(username: &str) -> Option<String> {
    if username == "alice" {
        Some("alice@example.com".to_string())
    } else {
        None
    }
}

// 链式调用
let email = Some(1)
    .and_then(|id| get_username(id))
    .and_then(|name| get_email(&name));

// email = Some("alice@example.com") 或 None
```

#### 5. ok_or / ok_or_else - Option 转 Result

```rust
let some_value: Option<i32> = Some(5);
let result: Result<i32, &str> = some_value.ok_or("错误信息");
// Ok(5)

let none_value: Option<i32> = None;
let result: Result<i32, &str> = none_value.ok_or("错误信息");
// Err("错误信息")
```

### Option 匹配模式

```rust
let value = Some(42);

match value {
    Some(x) => println!("值为: {}", x),
    None => println!("值为空"),
}

// 使用 if let 简化
if let Some(x) = value {
    println!("值为: {}", x);
} else {
    println!("值为空");
}
```

## 二、Result<T, E> 详解

### 处理 Result 的方式

#### 1. match 表达式（最基础）

```rust
use std::fs::File;

let f = File::open("hello.txt");

let file = match f {
    Ok(file) => file,
    Err(error) => {
        panic!("打开文件失败: {:?}", error)
    }
};
```

#### 2. 匹配不同错误类型

```rust
use std::fs::File;
use std::io::ErrorKind;

let f = File::open("hello.txt");

let file = match f {
    Ok(file) => file,
    Err(error) => match error.kind() {
        ErrorKind::NotFound => {
            // 文件不存在，尝试创建
            match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("创建文件失败: {:?}", e),
            }
        }
        ErrorKind::PermissionDenied => {
            panic!("权限不足: {:?}", error)
        }
        other_error => {
            panic!("其他错误: {:?}", other_error)
        }
    },
};
```

#### 3. unwrap 和 expect

```rust
use std::fs::File;

// unwrap: 错误信息不可自定义
let f = File::open("hello.txt").unwrap();

// expect: 可以自定义错误信息（推荐）
let f = File::open("hello.txt")
    .expect("无法打开配置文件 hello.txt");
```

#### 4. unwrap_or 和 unwrap_or_else

```rust
use std::fs::File;

// unwrap_or: 返回默认值
let f = File::open("hello.txt").unwrap_or_else(|_| {
    File::create("hello.txt").expect("无法创建文件")
});
```

### Result 的核心方法

```rust
// map: 转换 Ok 值
let result: Result<i32, &str> = Ok(5);
let doubled = result.map(|x| x * 2); // Ok(10)

let result: Result<i32, &str> = Err("error");
let doubled = result.map(|x| x * 2); // Err("error")

// map_err: 转换错误值
let result: Result<i32, &str> = Err("error");
let converted = result.map_err(|e| format!("错误: {}", e));

// and_then: 链式处理 Ok 值
let result = Ok("42")
    .and_then(|s| s.parse::<i32>())
    .and_then(|n| Ok(n * 2));

// or_else: 处理错误后返回新的 Result
let result: Result<i32, &str> = Err("error");
let recovered = result.or_else(|_| Ok(0));
```

## 三、? 操作符详解

### 基本用法

`?` 是错误传播的快捷方式，让函数将错误返回给调用者处理。

```rust
use std::fs::File;
use std::io;
use std::io::Read;

// 传统方式：手动处理错误
fn read_username_from_file_v1() -> Result<String, io::Error> {
    let f = File::open("hello.txt");
    
    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    };
    
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

// 使用 ? 操作符（简洁）
fn read_username_from_file_v2() -> Result<String, io::Error> {
    let mut f = File::open("hello.txt")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

// 链式调用更简洁
fn read_username_from_file_v3() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("hello.txt")?.read_to_string(&mut s)?;
    Ok(s)
}
```

### ? 操作符的工作原理

```rust
// ? 操作符等同于以下代码：
// 如果 Result 是 Ok，提取值继续执行
// 如果 Result 是 Err，立即返回该错误
let value = result?;
// 等同于:
let value = match result {
    Ok(v) => v,
    Err(e) => return Err(e.into()),
};
```

### ? 与 From trait

`?` 操作符会自动调用 `From` trait 进行错误类型转换：

```rust
use std::fs::File;
use std::io;

// 假设我们有自定义错误类型
#[derive(Debug)]
struct MyError(String);

impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self {
        MyError(format!("IO错误: {}", error))
    }
}

fn read_file() -> Result<String, MyError> {
    // io::Error 会自动转换为 MyError
    let content = File::open("hello.txt")?.read_to_string()?;
    Ok(content)
}
```

**避坑**：如果遇到 "the trait `From<X>` is not implemented for `Y`" 错误，需要手动实现转换或使用合适的错误类型。

### ? 操作符的限制

```rust
use std::fs::File;

fn main() {
    // 错误: ? 只能用于返回 Result 或 Option 的函数
    let f = File::open("hello.txt")?; 
}

// 正确: main 函数返回 Result
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("hello.txt")?;
    Ok(())
}
```

### 在 main 函数中使用 ?

```rust
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("config.txt")?;
    println!("文件打开成功");
    Ok(())
}
```

`Box<dyn Error>` 是 trait 对象，可以容纳任何实现了 `Error` trait 的错误类型。

## 四、自定义错误类型

### 使用 thiserror（适合库）

`thiserror` 提供了便捷的 derive 宏来定义自定义错误类型。

**添加依赖**：
```toml
[dependencies]
thiserror = "2.0"
```

**定义错误类型**：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    // 简单错误消息
    #[error("配置缺失")]
    ConfigMissing,
    
    // 格式化错误消息
    #[error("无效的值: {0}")]
    InvalidValue(String),
    
    // 包含额外字段的错误
    #[error("IO错误: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    
    // 自定义错误码
    #[error("解析错误: code={code}, message={message}")]
    ParseError {
        code: i32,
        message: String,
    },
}
```

### 使用 anyhow（适合应用程序）

`anyhow` 简化了应用程序的错误处理，提供更灵活的错误链。

**添加依赖**：
```toml
[dependencies]
anyhow = "2.0"
```

**使用示例**：

```rust
use anyhow::{Context, Result};

fn read_config() -> Result<Config> {
    let content = std::fs::read_to_string("config.json")
        .context("无法读取配置文件")?;
    
    let config: Config = serde_json::from_str(&content)
        .context("配置文件格式错误")?;
    
    Ok(config)
}

fn main() -> Result<()> {
    let config = read_config()?;
    println!("配置: {:?}", config);
    Ok(())
}
```

### thiserror vs anyhow: 如何选择？

| 特性 | thiserror | anyhow |
|------|-----------|--------|
| 适用场景 | 库代码 | 应用程序 |
| 错误定义 | 明确枚举所有错误变体 | 动态错误链 |
| 类型安全 | 高（编译时确定） | 中（运行时） |
| 堆栈追踪 | 需手动添加 | 自动包含 |

**通用建议**：
- **库**：使用 `thiserror` 定义精确的错误类型，调用者可以精确匹配和处理
- **应用程序**：使用 `anyhow`，快速构建和传播错误

## 五、避坑指南

### 1. 不要滥用 unwrap

```rust
// 糟糕：生产代码中使用 unwrap
fn process_user_input(input: &str) -> i32 {
    input.parse().unwrap() // 如果输入不是数字，程序崩溃
}

// 更好：返回 Result
fn process_user_input(input: &str) -> Result<i32, ParseIntError> {
    input.parse()
}

// 最好：提供友好的错误处理
fn process_user_input(input: &str) -> Result<i32, String> {
    input.parse().map_err(|e| format!("无效的数字: {}", e))
}
```

### 2. ? 操作符必须在返回 Result/Option 的函数中使用

```rust
// 错误：main 函数返回 ()
fn main() {
    let f = File::open("test.txt")?; // 编译错误！
}

// 正确：main 函数返回 Result
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("test.txt")?;
    Ok(())
}
```

### 3. 避免错误类型嵌套

```rust
// 不好：Result<Result<T, E1>, E2>
fn bad_example() -> Result<Result<i32, ParseError>, IoError> {
    let content = File::open("data.txt")??; // 别这样做！
}

// 好：扁平化错误类型
fn good_example() -> Result<i32, AppError> {
    let content = File::open("data.txt")?
        .read_to_string()?;
    Ok(content.parse()?)
}
```

### 4. 正确使用 Option 和 Result

```rust
// 使用 Option：值可能"不存在"
fn find_user(id: u32) -> Option<User> {
    database.find(id)
}

// 使用 Result：操作可能"失败"
fn save_user(user: &User) -> Result<(), DatabaseError> {
    database.save(user)
}
```

### 5. 闭包惰性求值

```rust
// 错误：每次都创建默认向量，即使 Some 值存在
let value = Some(vec![1, 2, 3]).unwrap_or_else(Vec::new);

// 正确：只在 None 时调用闭包
let value = Some(vec![1, 2, 3]).unwrap_or_else(Vec::new);
```

### 6. 错误转换要完整

```rust
use std::num::ParseIntError;

#[derive(Debug)]
struct AppError(String);

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError(format!("解析失败: {}", e))
    }
}

fn parse_number(s: &str) -> Result<i32, AppError> {
    Ok(s.parse()?)
}
```

## 六、何时应该用 panic

### 适用 panic 的场景

1. **原型/测试代码**：快速验证概念
2. **确定不会发生的错误**：使用 `unwrap()` 表明你的代码不可能失败
3. **不可能的状态**：程序状态不应该到达的位置

```rust
use std::net::IpAddr;

fn main() {
    let home: IpAddr = "127.0.0.1".parse().unwrap();
}
```

### 使用 panic vs 返回 Result 的指导原则

| 场景 | 建议 |
|------|------|
| 示例代码/原型 | `unwrap()` |
| 测试代码 | `unwrap()` / `expect()` |
| 外部输入验证 | `panic!` 或返回 `Result` |
| I/O 操作 | `Result` |
| 你可控的代码 | `Result` |
| 不可控的外部代码返回非法状态 | `panic!` |

## 单元测试

详见 `tests/rust_features/04_error_handling_basics.rs`

## 参考资料

- [Rust Error Handling: Complete Guide to Result, Option, and ? in 2026](https://rustify.rs/articles/rust-error-handling-result-option)
- [Rust 错误处理终极指南：从 panic! 到 Result 的优雅之道](https://juejin.cn/post/7525701918035902505)
- [How to Design Error Types with thiserror and anyhow](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view)
- [Beginner's Guide to anyhow and thiserror](https://zeljic.com/blog/anyhow-thiserror-beginners-guide)
- [thiserror crate documentation](https://docs.rs/thiserror/latest/thiserror/)