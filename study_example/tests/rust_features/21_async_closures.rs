// ---------------------------------------------------------------------------
// 4.2 异步闭包 (Rust 1.85+, RFC 3668)
// ---------------------------------------------------------------------------
// 语法: async || { ... } 创建异步闭包, 调用后返回 impl Future
// 避坑: async 闭包捕获的变量生命周期受 Future 约束
//       可变捕获需要 AsyncFnMut; 所有权捕获需要 AsyncFnOnce

use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// 辅助函数与类型定义
// ============================================================================

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

/// 接受异步闭包作为参数的函数
async fn takes_async_fn<F: AsyncFn() -> i32>(f: F) -> i32 {
    f().await
}

/// 接受 AsyncFnMut 闭包的函数 —— 可多次调用并携带可变状态
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

// ============================================================================
// 测试用例
// ============================================================================

#[test]
/// 测试: async 闭包基础 (AsyncFn trait, 1.85+)
fn test_async_closure() {
    // 语法: AsyncFn/AsyncFnMut/AsyncFnOnce 三个 trait 对应不同捕获语义
    // 避坑: async 闭包默认推断为 AsyncFn(不可变捕获)
    //       需要可变捕获时用 AsyncFnMut
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        test_async_closure_basic().await;
        test_async_closure_capture().await;
        assert_eq!(takes_async_fn(async || 42).await, 42);
    });
}

#[test]
/// 测试: async 闭包多次调用 (AsyncFn 可复用)
fn test_async_closure_multiple_calls() {
    // 语法: AsyncFn 闭包可以多次调用, 类似普通 Fn, 每次生成新的 Future
    // 避坑: 每次调用创建新的 Future, 可以并发 .await
    //       AsyncFn 以 &self 调用, 不可变借用
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let closure = async |x: i32| x * 2;
        assert_eq!(closure(5).await, 10);
        assert_eq!(closure(10).await, 20);
        assert_eq!(closure(15).await, 30);

        // 验证: 多次调用的 Future 彼此独立
        let f1 = closure(100);
        let f2 = closure(200);
        let (r1, r2) = tokio::join!(f1, f2);
        assert_eq!(r1, 200);
        assert_eq!(r2, 400);
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
/// 测试: AsyncFnMut 可变捕获(有状态的异步闭包)
fn test_async_fn_mut() {
    // 语法: async || 默认推断为 AsyncFn; 需要可变捕获时用 AsyncFnMut
    // 避坑: AsyncFnMut 闭包不能并发调用(需要 &mut self 独占)
    //       AsyncFnMut 以 &mut self 调用
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut counter = 0i32;
        let mut closure = async || {
            counter += 1;
            counter
        };
        // 每次调用修改内部状态(counter 被 &mut 捕获, 闭包为 AsyncFnMut)
        assert_eq!(closure().await, 1);
        assert_eq!(closure().await, 2);
        assert_eq!(closure().await, 3);
    });
}

#[test]
/// 测试: async 闭包作为函数参数 (AsyncFnMut 约束)
fn test_async_closure_as_parameter() {
    // 语法: 函数可以接受 async 闭包作为参数, 用 impl AsyncFn 约束
    // 避坑: 参数类型需要明确指定 AsyncFn/AsyncFnMut/AsyncFnOnce
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut attempts = 0;
        retry(
            async || {
                attempts += 1;
                if attempts >= 3 { Some("success") } else { None }
            },
            5,
        )
        .await
    });
    assert_eq!(result, Some("success"));
}

#[test]
/// 测试: async move 闭包 —— 强制所有权捕获
fn test_async_move_closure() {
    // 语法: async move || { ... } 强制捕获所有权, 类似 move || { ... }
    // 避坑: async move 闭包 = FnOnce + Future 的组合, 调用后闭包本身仍可用
    //       (不同于普通 move 闭包是 FnOnce)
    //       但捕获的变量所有权已转移, 不能在闭包外使用
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let name = String::from("alice");

        // async move 捕获 name 的所有权
        let closure = async move || {
            let greeting = format!("hello, {name}");
            greeting
        };
        // name 已移入闭包, 不能再使用

        let result = closure().await;
        assert_eq!(result, "hello, alice");

        // async move 闭包可以多次调用 (因为 AsyncFn 以 &self 调用)
        let result2 = closure().await;
        assert_eq!(result2, "hello, alice");
    });
}

#[test]
/// 测试: 异步闭包的并发调用 —— 多个 Future 同时 await
fn test_async_closure_concurrent() {
    // 语法: AsyncFn 闭包(不可变捕获)可以并发多次调用
    // 避坑: 如果是 AsyncFnMut 则不能并发——编译器会报借用冲突
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let factor = 10;
        let closure = async |x: i32| {
            tokio::time::sleep(Duration::from_millis(5)).await;
            x * factor
        };

        // 并发调用: 3 个 Future 同时执行
        let (a, b, c) = tokio::join!(
            closure(1),
            closure(2),
            closure(3),
        );
        assert_eq!((a, b, c), (10, 20, 30));
    });
}

#[test]
/// 测试: 异步闭包的生命周期 —— 捕获引用的语义
fn test_async_closure_lifetimes() {
    // 语法: async 闭包可以捕获引用(不可变借用), 闭包的 Future 生命周期受限于引用
    // 避坑: async 闭包捕获的引用必须在 .await 期间仍然有效
    //       spawn 的闭包必须 'static, 不能借用局部变量
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let data = String::from("borrowed");

        // 捕获不可变引用 —— 闭包类型是 AsyncFn
        let closure = async || data.len();
        let len = closure().await;
        assert_eq!(len, 8);

        // data 仍然可用 (不可变借用已释放)
        assert_eq!(data, "borrowed");
    });
}

#[test]
/// 测试: async 闭包组合 —— 闭包作为构建块
fn test_async_closure_composition() {
    // 语法: 异步闭包可以像普通闭包一样组合使用
    // 避坑: 每个闭包的 Future 类型是 opaque 的, 不能用具体类型标注
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let base_closure = async |x: i32| x + 1;
        let composed = async |x: i32| base_closure(x).await * 2;

        let result = composed(5).await;
        assert_eq!(result, 12); // (5+1)*2

        let result2 = composed(10).await;
        assert_eq!(result2, 22); // (10+1)*2
    });
}

#[test]
/// 测试: 带参数的 async 闭包数组/向量
fn test_async_closure_vec() {
    // 语法: 异步闭包可以放入 Vec, 但每个闭包的类型不同(opaque)
    //       放入 Vec 需要 Box::pin 或使用 trait object
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let c1 = async |x: i32| x * 2;
        let c2 = async |x: i32| x + 10;

        // 直接调用
        assert_eq!(c1(5).await, 10);
        assert_eq!(c2(5).await, 15);

        // 务实做法: 组合调用
        let results = tokio::join!(
            c1(100),
            c2(100),
        );
        assert_eq!(results, (200, 110));
    });
}
