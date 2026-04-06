// ---------------------------------------------------------------------------
// 4.2 异步闭包 (1.85+)
// ---------------------------------------------------------------------------

// 语法: async || { ... } 创建异步闭包, 返回 impl Future (RFC 3668, 1.85+)
// 避坑: async 闭包捕获的变量生命周期受 Future 约束; 可变捕获需要闭包为 AsyncFnMut

async fn test_async_closure_basic() {
    let closure = async || {
        let x = 10;
        x * 2
    };
    assert_eq!(closure().await, 20);
}

async fn test_async_closure_capture() {
    let value = 5i32;
    let closure = async || value + 10;
    assert_eq!(closure().await, 15);
}

async fn takes_async_fn<F>(f: F) -> i32
where
    F: AsyncFn() -> i32,
{
    f().await
}

#[test]
/// 测试: async 闭包 (AsyncFn trait, 1.85+)
fn test_async_closure() {
    // 语法: AsyncFn/AsyncFnMut/AsyncFnOnce 三个 trait 对应不同捕获语义
    // 避坑: async 闭包默认推断为 AsyncFn(不可变捕获); 需要可变捕获时用 async move 或 AsyncFnMut
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        test_async_closure_basic().await;
        test_async_closure_capture().await;
        assert_eq!(takes_async_fn(async || 42).await, 42);
    });
}

#[test]
/// 测试: async 闭包多次调用
fn test_async_closure_multiple_calls() {
    // 语法: AsyncFn 闭包可以多次调用, 类似普通 Fn
    // 避坑: 每次调用创建新的 Future, 可以并发 .await
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let closure = async |x: i32| x * 2;
        assert_eq!(closure(5).await, 10);
        assert_eq!(closure(10).await, 20);
        assert_eq!(closure(15).await, 30);
    });
}

#[test]
/// 测试: async 闭包 + tokio::spawn
fn test_async_closure_with_spawn() {
    // 语法: async 闭包可以传给 spawn, 但必须满足 Send + 'static
    // 避坑: 闭包捕获的变量必须 Clone 或 move; 不能借用局部变量
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let data = Arc::new(42);
        let handle = tokio::spawn(async move { *data });
        handle.await.unwrap()
    });
    assert_eq!(result, 42);
}

#[test]
/// 测试: AsyncFnMut 可变捕获
fn test_async_fn_mut() {
    // 语法: async || 默认推断为 AsyncFn, 需要可变捕获时用 AsyncFnMut
    // 避坑: AsyncFnMut 闭包不能并发调用; 需要 &mut self
    use std::cell::Cell;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let counter = Cell::new(0i32);
        let mut closure = async || {
            counter.set(counter.get() + 1);
            counter.get()
        };
        assert_eq!(closure().await, 1);
        assert_eq!(closure().await, 2);
        assert_eq!(closure().await, 3);
    });
}

#[test]
/// 测试: async 闭包作为函数参数
fn test_async_closure_as_parameter() {
    // 语法: 函数可以接受 async 闭包作为参数, 用 impl AsyncFn 约束
    // 避坑: 参数类型需要指定 AsyncFn/AsyncFnMut/AsyncFnOnce
    async fn retry<F, T>(mut f: F, max_attempts: u32) -> Option<T>
    where
        F: AsyncFnMut() -> Option<T>,
    {
        for _ in 0..max_attempts {
            if let Some(result) = f().await {
                return Some(result);
            }
        }
        None
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut attempts = 0;
        retry(
            async || {
                attempts += 1;
                if attempts >= 3 {
                    Some("success")
                } else {
                    None
                }
            },
            5,
        )
        .await
    });
    assert_eq!(result, Some("success"));
}

use std::sync::Arc;
