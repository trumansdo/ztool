# 22 - 诊断属性

## 核心概念

### #[diagnostic::do_not_recommend] (1.85+)

阻止编译器在错误信息中推荐此实现：

```rust
#[diagnostic::do_not_recommend]
impl InternalTrait for SomeType {}
```

### #[must_use]

强制使用返回值：

```rust
#[must_use]
fn important_result() -> i32 {
    42
}
// 不使用会产生警告
```

### #[deprecated]

标记废弃 API：

```rust
#[deprecated(since = "1.0.0", note = "use new_function")]
fn old_function() {}
```

### Lint 级别

```rust
#[allow(unused)]
#[warn(unused)]
#[deny(unused)]
```

### #[non_exhaustive]

强制 match 必须有通配符：

```rust
#[non_exhaustive]
pub enum Event {
    Start,
    End,
}
```

## 单元测试

详见 `tests/rust_features/22_diagnostic_attributes.rs`

## 参考资料

- [Rust 1.85 公告](https://blog.rust-lang.org/2025/03/20/Rust-1.85.0/)
