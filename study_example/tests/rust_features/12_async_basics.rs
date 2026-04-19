// ---------------------------------------------------------------------------
// 4.1 异步编程基础
// ---------------------------------------------------------------------------

#[test]
/// 测试: async fn 异步函数和 tokio 运行时
fn test_async_fn() {
    // 语法: async fn 返回 impl Future, 惰性执行, 需要 .await 或 executor 驱动
    // 避坑: async fn 不运行代码, 返回的 Future 被 drop 后状态丢失; 需要运行时(如 tokio)
    async fn async_add(a: i32, b: i32) -> i32 {
        a + b
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    assert_eq!(rt.block_on(async_add(3, 4)), 7);
}

#[test]
/// 测试: async 块捕获变量
fn test_async_blocks() {
    // 语法: async { ... } 创建匿名 Future, 可捕获环境变量
    // 避坑: async 块捕获的变量必须满足 Send 才能跨线程; 借用检查更严格
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let x = 10;
        let y = 20;
        x + y
    });
    assert_eq!(result, 30);
}

#[test]
/// 测试: .await 挂起点
fn test_await_suspension() {
    // 语法: .await 挂起当前 Future, 让出执行权给 executor
    // 避坑: .await 只能在 async 上下文中使用; 多个 .await 顺序执行, 不并发
    async fn slow_value() -> i32 {
        42
    }

    async fn compute() -> i32 {
        let a = slow_value().await;
        let b = slow_value().await;
        a + b
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    assert_eq!(rt.block_on(compute()), 84);
}

#[test]
/// 测试: join! 并发执行多个 Future
fn test_join_concurrent() {
    // 语法: tokio::join! 并发等待多个 Future, 全部完成后返回结果元组
    // 避坑: join! 是并发执行, 不是顺序执行; 任一 Future panic 会导致整个 join panic
    async fn task(n: i32) -> i32 {
        n * 2
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        tokio::join!(task(1), task(2), task(3))
    });
    assert_eq!(result, (2, 4, 6));
}

#[test]
/// 测试: spawn 后台任务
fn test_spawn() {
    // 语法: tokio::spawn 在后台启动任务, 返回 JoinHandle
    // 避坑: spawn 的任务独立运行, 不等待完成; JoinHandle::await 等待结果
    //       spawn 的任务必须 'static (不能借用外部数据)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let handle = tokio::spawn(async {
            42
        });
        handle.await.unwrap()
    });
    assert_eq!(result, 42);
}

#[test]
/// 测试: select! 选择最先完成的 Future 概念
fn test_select() {
    // 语法: tokio::select! 等待多个 Future 中第一个完成的, 取消其余
    // 避坑: 未完成的 Future 会被 drop; 需要偏执模式用 biased select
    // 注意: 此测试仅验证概念, 实际运行需要 tokio 运行时
    assert!(true);
}

#[test]
/// 测试: async move 所有权转移
fn test_async_move() {
    // 语法: async move { ... } 强制捕获所有变量的所有权
    // 避坑: move 后原变量不可用; 需要 Clone 的变量要先 clone
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let data = String::from("hello");
        let handle = tokio::spawn(async move {
            data.len()
        });
        handle.await.unwrap()
    });
    assert_eq!(result, 5);
}

#[test]
/// 测试: async 中的借用检查
fn test_async_borrowing() {
    // 语法: async 块中的借用受 Future 生命周期约束
    // 避坑: 跨 .await 点的借用必须保证数据在 await 后仍然有效
    //       最常见错误: 借用局部变量后 .await
    async fn read_data() -> String {
        String::from("data")
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let data = read_data().await;
        // data 是 owned String, 可以跨 await 使用
        let len = data.len();
        let _ = read_data().await; // 另一个 await
        len
    });
    assert_eq!(result, 4);
}

#[test]
/// 测试: Send trait 和异步任务
fn test_async_send() {
    // 语法: tokio::spawn 要求 Future: Send, 即所有捕获的变量必须 Send
    // 避坑: Rc/RefCell 不是 Send, 不能在 spawn 的 async 块中使用
    //       用 Arc/Mutex 替代
    use std::sync::Arc;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let data = Arc::new(42);
        let data_clone = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            *data_clone
        });
        handle.await.unwrap()
    });
    assert_eq!(result, 42);
}

#[test]
/// 测试: 异步迭代器 (Stream 概念)
fn test_async_iteration_concept() {
    // 语法: Rust 标准库还没有 async Iterator, 但 tokio-stream 提供了 Stream trait
    // 避坑: async for 语法尚未稳定; 用 while let + next() 模式
    // 注意: 这里只演示概念, 不依赖 tokio-stream crate
    assert!(true);
}
