# 23 - 精确捕获 (Precise Capturing)

## 核心概念

RPIT (impl Trait in return position) 可以精确控制捕获哪些生命周期和类型：

### 基本语法

```rust
fn foo() -> impl Trait + use<'a, 'b> {
    // 明确指定捕获的生命周期
}
```

### 排除捕获

```rust
struct Container<'a> {
    data: &'a i32,
}

impl<'a> Container<'a> {
    fn access(&self) -> impl Debug + use<'a> {
        self.data
    }
}
```

### 必须包含 Self

```rust
trait MyTrait {
    fn method(&self) -> impl Debug + use<Self>;
}
```

## RFC 3498

此特性由 RFC 3498 引入。

## 单元测试

详见 `tests/rust_features/23_precise_capturing.rs`
