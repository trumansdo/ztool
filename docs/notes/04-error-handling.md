# 04 - 错误处理：Option 与 Result

## 核心概念

Rust 使用类型系统处理错误，有两种主要方式：

### Option<T>

表示可能存在或不存在的值：

```rust
enum Option<T> {
    Some(T),
    None,
}
```

### Result<T, E>

表示操作成功或失败：

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

## 主要操作

### Option 方法

- `unwrap()`: 获取值，None 时 panic
- `unwrap_or(default)`: None 时返回默认值
- `unwrap_or_else(f)`: None 时调用闭包
- `map()`: 转换内部值
- `and_then()`: 链式处理
- `ok_or()`: 转为 Result

### Result 方法

- `unwrap()` / `unwrap_err()`
- `map()` / `map_err()`
- `and_then()` / `or_else()`
- `?` 操作符

### ? 操作符

```rust
fn read_file() -> Result<String, io::Error> {
    let content = fs::read_to_string("file.txt")?;
    Ok(content)
}
```

## 避坑指南

1. **不要滥用 unwrap**: 生产代码应优雅处理错误
2. **? 只能在函数返回 Result/Option 时使用**
3. **错误类型**: 建议定义自定义错误类型

## 单元测试

详见 `tests/rust_features/04_error_handling_basics.rs`

## 参考资料

- [Rust Error Handling](https://oneuptime.com/blog/post/2026-02-03-rust-result-option/view)
