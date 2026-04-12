# 13 - async 闭包

## 概述

async 闭包是 Rust 中处理异步操作的重要工具，结合了闭包的灵活性和异步编程的能力。它们可以捕获环境并在异步上下文中执行。

## 三种 Async 闭包 Trait

```rust
trait AsyncFn<T> {       // 不可变借用
    type Output;
    async fn call(&self, arg: T) -> Self::Output;
}

trait AsyncFnMut<T> {    // 可变借用
    type Output;
    async fn call_mut(&mut self, arg: T) -> Self::Output;
}

trait AsyncFnOnce<T> {   // 获取所有权
    type Output;
    async fn call_once(self, arg: T) -> Self::Output;
}
```

## async move 闭包

```rust
let data = String::from("hello");

let future = async move {
    // 获得 data 所有权
    data.len()
};

let result = future.await;
```

## 多次调用

```rust
fn create_counter() -> impl AsyncFnMut() -> i32 {
    let mut count = 0;
    async move || {
        count += 1;
        count
    }
}
```

## 配合 spawn

```rust
tokio::spawn(async move {
    closure().await;
}).await;
```

## 单元测试

详见 `tests/rust_features/13_async_closures.rs`

## 参考资料

- [Async Closures RFC](https://rust-lang.github.io/rfcs/2996-async-closures.html)
- [Tokio async closures](https://tokio.rs/tokio/topics/bridging)