# 24 - 保留关键字

## 核心概念

Rust 保留一些关键字为未来语法做准备：

### gen 关键字

为未来生成器语法准备：

```rust
// gen 不能用作标识符
// gen { } / gen || 未来可用
```

### async / await

已经是关键字，不能用作标识符：

```rust
let r#async = 42;  // 使用 raw identifier
```

### dyn

用于 trait 对象：

```rust
let obj: &dyn Trait = &some_type;
```

### raw identifier r#

允许使用关键字作为标识符：

```rust
let r#type = "string";
let r#impl = "implementation";
```

## 关键字列表

- 2021 edition+: `async`, `await`, `dyn`
- 2024 edition+: `gen`
- 特殊: `r#` 前缀

## 单元测试

详见 `tests/rust_features/24_reserved_keywords.rs`
