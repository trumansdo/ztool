# 测试框架

> 测试不是验证代码没有错误——测试是验证代码在已知条件下表现如预期。写得越多越精，只会减少你半夜被电话叫醒的次数。

## 1. #[test] 基本单元测试

```rust
// 测试函数用 #[test] 标记
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_string_contains() {
    let s = "Hello, Rust!";
    assert!(s.contains("Rust"));
    assert!(!s.contains("Java"));
}

#[test]
fn test_vec_push() {
    let mut v = vec![1, 2, 3];
    v.push(4);
    assert_eq!(v.len(), 4);
    assert_eq!(v[3], 4);
}
```

> 每个 #[test] 函数都是一个独立的小宇宙——测试运行器为每个测试创建独立的线程，互不影响，互不依赖。

## 2. 断言宏

### 2.1 assert! / assert_eq! / assert_ne!

```rust
#[test]
fn test_assertions() {
    // assert!：验证布尔条件
    assert!(true);
    assert!(1 < 2, "条件应为真，但实际为假");

    // assert_eq!：验证相等（需要 PartialEq）
    assert_eq!(4, 2 + 2);
    assert_eq!("hello", "hello");

    // assert_ne!：验证不相等
    assert_ne!(5, 3 + 3);
}

// 自定义类型的 assert_eq 需要 Debug trait
#[derive(Debug, PartialEq)]
struct Point { x: i32, y: i32 }

#[test]
fn test_custom_eq() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 1, y: 2 };
    assert_eq!(p1, p2);
    // 失败时会打印两个值的 Debug 表示
}
```

### 2.2 自定义错误消息

```rust
#[test]
fn test_custom_message() {
    let actual = 42;
    let expected = 100;

    assert_eq!(
        actual, expected,
        "测试失败：期望 {}，实际 {}，差异 {}",
        expected, actual, expected - actual
    );
}
```

> 好的错误消息能让你在半年后一眼看出测试为什么失败——为重要的断言加上自定义消息是一种代码素养。

## 3. #[should_panic] 预期恐慌

```rust
// 验证函数确实会 panic
#[test]
#[should_panic]
fn test_divide_by_zero_panics() {
    let _result = divide(10, 0);
}

fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("除数不能为零");
    }
    a / b
}

// expected 参数验证 panic 消息内容
#[test]
#[should_panic(expected = "除数不能为零")]
fn test_divide_by_zero_message() {
    divide(42, 0);
}

#[test]
#[should_panic(expected = "索引越界")]
fn test_out_of_bounds() {
    let v = vec![1, 2, 3];
    let _x = v[99]; // 这会 panic，但消息不包含"索引越界"
    // 注意：Rust 标准库的 panic 消息可能与 expected 不匹配！
}
```

### 3.1 should_panic 注意事项

```rust
// 以下测试会失败——panic 在另一个线程中发生
#[test]
#[should_panic]
fn test_thread_panic() {
    std::thread::spawn(|| {
        panic!("子线程中的恐慌");
    }).join().unwrap();
    // should_panic 默认只识别当前线程的 panic
}
```

## 4. #[ignore] 忽略测试

```rust
// 被忽略的测试不会在默认 cargo test 中运行
#[test]
#[ignore = "等待 Bug #1234 修复后再启用"]
fn test_unstable_feature() {
    // 正在开发中的功能
}

#[test]
#[ignore]
fn test_very_slow() {
    std::thread::sleep(std::time::Duration::from_secs(60));
}

// 运行被忽略的测试：
// cargo test -- --ignored
// 运行全部（含忽略的）：cargo test -- --include-ignored
```

> #[ignore] 不是垃圾桶——它是"需要后续处理"的标记。每个 ignore 应该有明确的理由注释和恢复计划。

## 5. 单元测试与模块

```rust
// 被测试的函数
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn private_multiply(a: i32, b: i32) -> i32 {
    a * b
}

// 测试模块（惯例放在同一个文件中）
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    // 单元测试可以访问私有函数
    #[test]
    fn test_private_fn() {
        assert_eq!(private_multiply(4, 5), 20);
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 100), 100);
        assert_eq!(add(100, 0), 100);
    }
}
```

### 5.1 测试辅助函数

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 辅助函数不需要 #[test]
    fn setup_test_data() -> Vec<i32> {
        vec![1, 2, 3, 4, 5]
    }

    fn create_test_point() -> Point {
        Point { x: 10, y: 20 }
    }

    #[test]
    fn test_with_helper() {
        let data = setup_test_data();
        let point = create_test_point();
        assert_eq!(data.iter().sum::<i32>(), 15 + point.x + point.y);
    }
}
```

## 6. 集成测试

集成测试放在项目根目录的 `tests/` 文件夹中，每个 `.rs` 文件是一个独立的 crate：

```rust
// tests/integration_test.rs
// 集成测试中需要通过 crate 名称引用被测库
use my_library; // 假设 lib crate 名称是 my_library

#[test]
fn test_public_api() {
    let result = my_library::public_function(42);
    assert!(result.is_ok());
}

#[test]
fn test_end_to_end() {
    let input = "Hello";
    let processed = my_library::process(input);
    assert_eq!(processed, "HELLO");
}

// 集成测试只能访问 public API，无法测试私有函数
```

### 6.1 tests/ 目录结构

```
my_project/
├── src/
│   └── lib.rs
├── tests/
│   ├── integration_test.rs      # 独立的测试文件
│   ├── api_tests.rs             # 另一个测试文件
│   └── common/                  # 测试辅助模块（目录方式）
│       └── mod.rs               # pub mod common { ... }
```

```rust
// tests/common/mod.rs
pub fn setup() -> String {
    String::from("shared setup data")
}

// tests/api_tests.rs
use my_library;
mod common;

#[test]
fn test_with_common() {
    let data = common::setup();
    assert!(!data.is_empty());
}
```

> 集成测试站在用户的角度检验你的库——它只能调用公开 API，这迫使你思考"用户是否能方便地使用我的公共接口"。

## 7. 文档测试

```rust
/// 计算两个数字的和
///
/// ```
/// // 文档测试既是文档也是测试
/// let result = my_library::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 这个例子不应该被运行（需要网络连接）
///
/// ```no_run
/// let response = fetch_remote_data("https://example.com");
/// // ...
/// ```

/// 这个例子编译会失败（演示错误用法）
///
/// ```compile_fail
/// let result: i32 = "not a number".parse(); // 编译错误
/// ```

/// 隐藏不必要的前置代码
///
/// ```
/// # use my_library::hidden_setup;
/// # let context = hidden_setup();
/// // 用户只看到下面这行
/// let result = my_library::process("input");
/// assert!(result > 0);
/// ```

/// 测试 panic 预期
///
/// ```should_panic
/// let v = vec![1, 2, 3];
/// let _ = v[99]; // panic!
/// ```
```

### 7.1 文档测试属性对照

| 属性 | 效果 |
|------|------|
| (无) | 编译并运行 |
| `no_run` | 仅编译不运行 |
| `compile_fail` | 必须编译失败（测试才能通过） |
| `should_panic` | 必须 panic（测试才能通过） |
| `# 代码行` | 隐藏该行（不出现在文档中） |
| `ignore` | 跳过 |
| `edition2015` 等 | 指定 Rust edition |

> 文档测试让示例代码永远保持正确——它是 Rust 社区"文档即测试"哲学的基石，也是与其他语言生态拉开差距的标志之一。

## 8. Result<T, E> 作为测试返回类型

```rust
#[test]
fn test_with_result() -> Result<(), String> {
    if 2 + 2 == 4 {
        Ok(())
    } else {
        Err(String::from("2+2 != 4? 这不可能！"))
    }
}

#[test]
fn test_with_io() -> std::io::Result<()> {
    let content = std::fs::read_to_string("Cargo.toml")?;
    assert!(content.contains("[package]"));
    Ok(())
}

// 使用 anyhow 简化错误处理
/*
#[test]
fn test_with_anyhow() -> anyhow::Result<()> {
    let data = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&data)?;
    assert!(config.port > 0);
    Ok(())
}
*/
```

> 返回 `Result` 的测试函数让错误处理更加优雅——可以用 `?` 操作符传播错误，不必在测试中使用 `unwrap()` 伪装世界永远正确。

## 9. 测试组织最佳实践

```rust
// 组织测试的推荐结构
#[cfg(test)]
mod tests {
    use super::*;

    // 1. 简单单元测试
    mod unit {
        use super::*;

        #[test]
        fn test_basic() { /* ... */ }
    }

    // 2. 边界条件测试
    mod edge_cases {
        use super::*;

        #[test]
        fn test_empty_input() { /* ... */ }
        #[test]
        fn test_max_values() { /* ... */ }
    }

    // 3. 回归测试（来自已修复的 bug）
    mod regression {
        use super::*;

        #[test]
        fn test_issue_123() { /* ... */ }
    }

    // 4. 性能基准测试 (nightly)
    // mod bench {
    //     use super::*;
    //     #[bench]
    //     fn bench_large_input(b: &mut test::Bencher) { /* ... */ }
    // }
}
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 测试函数没有 `#[test]` 属性 | 缺少属性运行器不会执行 | 确保每个测试函数前都有 `#[test]` |
| 测试依赖执行顺序 | 测试默认并行运行，顺序不确定 | 每个测试独立设置/清理状态 |
| should_panic 匹配不到 panic 消息 | 实际 panic 消息与 expected 不匹配 | 使用 `expected = "关键片段"` 而非完整消息 |
| 文档测试中的隐藏代码语法错误 | `# ` 开头的行虽然隐藏但仍会被编译 | 确保隐藏行的代码正确生效 |
| 集成测试访问不到私有 API | 集成测试是外部 crate | 测试私有逻辑时使用单元测试 |
| 多个测试函数共享可变状态 | 测试并行执行可能产生竞态条件 | 避免共享状态，或使用串行执行 `cargo test -- --test-threads=1` |
| `use super::*` 污染测试命名空间 | 被测试模块的函数与测试辅助函数重名 | 测试函数使用明确的前缀，或将测试放在子模块中 |

> **详见测试**: `tests/rust_features/25_testing.rs`
