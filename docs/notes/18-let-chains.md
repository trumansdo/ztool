# 18 - Let Chains (Rust 1.88+)

## 概述

Let Chains 是 Rust 1.88 和 Edition 2024 引入的重要特性，它允许在 `if` 和 `while` 条件中链式组合多个 `let` 绑定和布尔条件。这一特性极大地改善了代码可读性，解决了长期以来嵌套 `if let` 带来的"右向漂移"问题。

## 核心概念

Let Chains 允许在条件中链式组合多个 let 绑定和布尔条件：

```rust
if let Some(x) = value && x > 0 {
    // x 在这里可用
}
```

### 语法

- `if let Pat = expr && cond { ... }`
- `while let Pat = expr && cond { ... }`

### 多重 let

```rust
if let Some(x) = opt1 && let Some(y) = opt2 && x + y > 0 {
    // x 和 y 都可用
}
```

### 短路求值

Let Chains 使用短路求值，前面的模式匹配失败会立即终止后续表达式的求值。

```rust
// 等价于嵌套
if let Some(x) = opt1 {
    if let Some(y) = opt2 {
        if x + y > 0 {
            // ...
        }
    }
}
```

### 与元组的区别

Let Chains 会短路求值，但元组形式总是执行两个函数：

```rust
// Let Chains - 短路
if let Some(x) = foo() && let Some(y) = bar() { }

// 元组 - 不短路
if let (Some(x), Some(y)) = (foo(), bar()) { }
```

## 版本要求

- Rust 1.88+ 或 Edition 2024

## 限制

- 只能用 `&&` 连接，不能用 `||`
- let 绑定必须在前
- 不能将 `let` 用作表达式

## 单元测试

详见 `tests/rust_features/18_let_chains.rs`

## 参考资料

- [Rust 1.88 公告](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/)
- [Let Chains RFC 2497](https://rust-lang.github.io/rfcs/2497-if-let-chains.html)
- [Edition Guide - Let Chains](https://doc.rust-lang.org/edition-guide/rust-2024/let-chains.html)