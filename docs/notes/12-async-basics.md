# 12 - 异步编程基础

## 核心概念

### async/await

```rust
async fn fetch() -> String {
    let response = reqwest::get("http://example.com").await;
    response.text().await.unwrap()
}
```

### Future

```rust
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}
```

### 运行时

Rust 异步需要运行时，如 tokio:

```rust
#[tokio::main]
async fn main() {
    let result = async_operation().await;
}
```

### spawn

创建并发任务:

```rust
let handle = tokio::spawn(async {
    // 并发执行
});
let result = handle.await;
```

### join!

并行等待多个 Future:

```rust
let (a, b) = tokio::join!(future1(), future2());
```

### select!

等待多个 Future，先完成的那个:

```rust
tokio::select! {
    result1 = future1() => { /* ... */ }
    result2 = future2() => { /* ... */ }
}
```

## Send 约束

- async 函数如果捕获非 Send 数据，返回的 Future 不是 Send
- 跨线程传递需保证安全

## 单元测试

详见 `tests/rust_features/12_async_basics.rs`
