# 21 - if let 临时作用域 (Edition 2024)

## 核心概念

Edition 2024 改变了 if let 的临时值生命周期：

### 旧版行为

临时值在 if 块结束后立即 drop：

```rust
if let Some(x) = get_temp_value() {
    // x 可用
}
// x 在这里已经被 drop
```

### Edition 2024 行为

临时值作用域延长到整个 if-else 块：

```rust
if let Some(x) = get_temp_value() {
    // x 可用
} else {
    // x 仍然可用
}
```

## 影响

- 改变 drop 顺序
- 某些依赖旧行为的代码可能受影响
- 迁移时需注意

## 单元测试

详见 `tests/rust_features/21_if_let_temporary_scope.rs`
