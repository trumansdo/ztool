# 13 - async 闭包

## 核心概念

### AsyncFn / AsyncFnMut / AsyncFnOnce

Rust 异步闭包有三种 trait：

```rust
trait AsyncFn<T> { /* */ }
trait AsyncFnMut<T> { /* */ }
trait AsyncFnOnce<T> { /* */ }
```

### async move

```rust
let data = String::from("hello");
let future = async move {
    // 获得 data 所有权
    data.len()
};
```

### 多次调用

```rust
async fn create_counter() -> impl AsyncFnMut() -> i32 {
    let mut count = 0;
    async move || {
        count += 1;
        count
    }
}
```

### 配合 spawn

```rust
tokio::spawn(async move {
    closure().await;
}).await;
```

## 单元测试

详见 `tests/rust_features/13_async_closures.rs`
