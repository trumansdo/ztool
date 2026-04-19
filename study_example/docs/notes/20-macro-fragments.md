# 20 - 宏片段说明符

## 核心概念

宏的片段说明符指定匹配 Rust 代码的哪部分：

| 说明符 | 匹配内容 |
|--------|----------|
| `expr` | 表达式 |
| `stmt` | 语句 |
| `pat` / `pat_param` | 模式 |
| `ty` | 类型 |
| `ident` | 标识符 |
| `path` | 路径 |
| `block` | 代码块 |
| `tt` | Token Tree |
| `meta` | 属性内容 |
| `item` | 项(函数、结构体等) |
| `literal` | 字面量 |

### expr 扩展 (Edition 2024)

Edition 2024 中 `expr` 也能匹配 const 块:

```rust
macro_rules! test {
    ($e:expr) => { $e };
}

let x = test!(const { 42 });  // 有效
```

## 重复修饰符

```rust
$($item:expr),*   // 逗号分隔
$($item:expr);*   // 分号分隔
$($item:expr)+*   // 至少一个
```

## 单元测试

详见 `tests/rust_features/20_macro_fragment_specifiers.rs`
