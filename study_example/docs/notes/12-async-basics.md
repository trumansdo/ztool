# 12 - 异步编程基础

## 概述

Rust 的异步编程模型是一种高效的并发处理方式，它允许在单线程中并发执行多个任务，而无需创建大量线程。Rust 的异步是零成本抽象——不需要运行时也能工作，但通常配合 tokio 或 async-std 等运行时使用。

## async/await

`async` 将函数变为返回 Future 的函数，`await` 等待 Future 完成：

```rust
async fn fetch() -> String {
    let response = reqwest::get("http://example.com").await;
    response.text().await.unwrap()
}

// 等价于：
fn fetch() -> impl Future<Output = String> {
    async {
        let response = reqwest::get("http://example.com").await;
        response.text().await.unwrap()
    }
}
```

## Future Trait

```rust
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),    // Future 完成
    Pending,   // 仍在等待
}
```

## 运行时

Rust 异步需要运行时，如 tokio：

```rust
#[tokio::main]
async fn main() {
    let result = async_operation().await;
    println!("Result: {}", result);
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // 多线程运行时
}
```

### 内置运行时

```rust
use std::future::Future;
use std::pin::Pin;

fn block_on<F: Future>(future: F) -> F::Output {
    // 简单的阻塞实现
}
```

## 并发操作

### spawn - 创建并发任务

```rust
let handle = tokio::spawn(async {
    // 并发执行
    compute_heavy().await
});
let result = handle.await.unwrap();
```

### join! - 并行等待

```rust
let (a, b) = tokio::join!(future1(), future2());
```

### select! - 先完成优先

```rust
tokio::select! {
    result1 = future1() => {
        println!("First: {}", result1);
    }
    result2 = future2() => {
        println!("Second: {}", result2);
    }
}

// biased select! 优先检查顺序
tokio::select! {
    biased;
    result1 = future1() => { /* ... */ }
    result2 = future2() => { /* ... */ }
}
```

### spawn 与借用

```rust
let data = String::from("hello");

tokio::spawn(async move {
    // move 转移所有权
    println!("{}", data);
});
```

## Send 约束

- async 函数如果捕获非 Send 数据，返回的 Future 不是 Send
- 跨线程传递需保证安全
- 使用 `std::marker::Send` 和 `std::marker::Sync` 约束

```rust
async fn not_send() {
    // RefCell 不是 Send
    let cell = std::cell::RefCell::new(1);
    // async 函数返回不是 Send 的 Future
}

async fn is_send() {
    let data = String::from("hello");
    // 数据是 Send，Future 也是 Send
}
```

## Stream

类似迭代器，用于异步产生多个值：

```rust
use futures::stream::StreamExt;

async fn read_items() {
    let mut stream = /* async stream */;
    while let Some(item) = stream.next().await {
        process(item).await;
    }
}
```

## 避坑指南

1. **运行时依赖**：异步代码需要 tokio 等运行时
2. **Send 要求**：跨线程的 async 函数必须实现 Send
3. **await 链**：避免在循环中 await 多次
4. **spawn 生命周期**：spawn 的任务可能比主函数活得久

## 单元测试

详见 `tests/rust_features/12_async_basics.rs`

## 参考资料

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/tokio)
- [Async Rust](https://doc.rust-lang.org/std/keyword.async.html)