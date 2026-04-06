# 18 - Let Chains (Rust 1.88+)

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

## 版本要求

- Rust 1.88+ 或 Edition 2024

## 限制

- 只能用 `&&` 连接，不能用 `||`
- let 绑定必须在前

## 单元测试

详见 `tests/rust_features/18_let_chains.rs`

## 参考资料

- [Rust 1.88 公告](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/)
