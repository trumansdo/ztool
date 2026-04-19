# 19 - 条件编译

## 核心概念

### cfg 属性

```rust
#[cfg(target_os = "linux")]
fn linux_only() {}

#[cfg(feature = "debug")]
fn debug_feature() {}
```

### cfg! 宏

```rust
if cfg!(debug_assertions) {
    // 调试模式
}
```

### cfg 布尔字面量 (1.88+)

```rust
#[cfg(true)]  // 始终启用
#[cfg(false)] // 始终禁用
```

### cfg_attr

条件应用属性:

```rust
#[cfg_attr(feature = "trace", log::warn, allow(dead_code))]
fn some_function() {}
```

### all / any

```rust
#[cfg(all(target_os = "linux", feature = "unix"))]
#[cfg(any(target_os = "linux", target_os = "macos"))]
```

### target_family / target_arch

```rust
#[cfg(target_family = "unix")]
#[cfg(target_arch = "x86_64")]
```

## 单元测试

详见 `tests/rust_features/19_conditional_compilation.rs`

## 参考资料

- [Rust Reference - Conditional Compilation](https://doc.rust-lang.org/1.88.0/reference/conditional-compilation.html)
