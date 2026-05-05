// ---------------------------------------------------------------------------
// 4.1 异步编程基础
// ---------------------------------------------------------------------------

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::{self, FuturesUnordered, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, timeout};

// ============================================================================
// 已有测试增强
// ============================================================================

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
    // async fn 返回的是 impl Future, 不是具体类型
    // 可以用 impl Future 明确表述
    fn returns_future() -> impl Future<Output = i32> {
        async_add(10, 20)
    }
    assert_eq!(rt.block_on(returns_future()), 30);
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

    // async 块可以捕获外部变量(引用或所有权)
    let base = 100;
    let result2 = rt.block_on(async {
        let x = base; // 捕获 base 的所有权
        x + 1
    });
    assert_eq!(result2, 101);
}

#[test]
/// 测试: .await 挂起点
fn test_await_suspension() {
    // 语法: .await 挂起当前 Future, 让出执行权给 executor
    // 避坑: .await 只能在 async 上下文中使用; 多个 .await 顺序执行, 不并发
    //       每个 .await 点都是状态机的分界线
    async fn slow_value() -> i32 {
        42
    }

    async fn compute() -> i32 {
        let a = slow_value().await; // 挂起点 1: 状态机从 start 切换到 after_a
        let b = slow_value().await; // 挂起点 2: 状态机从 after_a 切换到 after_b
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

    // join! 支持最多 64 个 Future
    let result2 = rt.block_on(async {
        tokio::join!(task(5), task(10))
    });
    assert_eq!(result2, (10, 20));
}

#[test]
/// 测试: spawn 后台任务
fn test_spawn() {
    // 语法: tokio::spawn 在后台启动任务, 返回 JoinHandle
    // 避坑: spawn 的任务独立运行, 不等待完成; JoinHandle::await 等待结果
    //       spawn 的任务必须 'static (不能借用外部数据)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let handle = tokio::spawn(async { 42 });
        handle.await.unwrap()
    });
    assert_eq!(result, 42);
}

#[test]
/// 测试: select! 选择最先完成的 Future
fn test_select() {
    // 语法: tokio::select! 等待多个 Future 中第一个完成的, 取消其余
    // 避坑: 未完成的 Future 会被 drop; 需要偏执模式用 biased select
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        tokio::select! {
            v = fast_task() => v,
            v = slow_task() => v,
        }
    });
    assert_eq!(result, "fast");

    async fn fast_task() -> &'static str {
        sleep(Duration::from_millis(10)).await;
        "fast"
    }
    async fn slow_task() -> &'static str {
        sleep(Duration::from_millis(100)).await;
        "slow"
    }
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
    // 语法: Rust 标准库还没有 async Iterator, 但 futures crate 提供了 Stream trait
    // 避坑: async for 语法尚未稳定; 用 while let + next() 模式
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut stream = stream::iter(vec![1, 2, 3, 4, 5]);
        let mut sum = 0;
        while let Some(item) = stream.next().await {
            sum += item;
        }
        sum
    });
    assert_eq!(result, 15);
}

// ============================================================================
// 新增测试函数
// ============================================================================

#[test]
/// 测试: async fn 返回 Future 类型
fn test_async_fn_returns_future() {
    // 语法: async fn 的返回类型是 impl Future<Output = T>
    // 避坑: impl Future 是 opaque 类型, 不能用具体类型标注; 但可以装箱为 Pin<Box<dyn Future>>
    async fn compute(x: i32) -> i32 {
        x * 2
    }
    // compute 的类型是 fn(i32) -> impl Future<Output = i32>
    let future = compute(21);
    // future 的类型是 impl Future<Output = i32>, 惰性不执行
    let rt = tokio::runtime::Runtime::new().unwrap();
    assert_eq!(rt.block_on(future), 42);

    // async 块的类型也是 impl Future
    let block_future = async {
        let x = 10;
        let y = 3;
        x * y
    };
    assert_eq!(rt.block_on(block_future), 30);
}

#[test]
/// 测试: async 状态机 —— 跨 await 点的状态保持
fn test_async_state_machine() {
    // 语法: async 函数编译为状态机, 每个 .await 点是一个状态转换
    // 避坑: 跨 .await 点的局部变量保留, 但它们被保存在状态机中
    //       不可跨 .await 持有对局部变量的引用(自引用问题)
    async fn multi_step() -> i32 {
        let mut total = 0;
        // --- 状态 0: 初始化, 进入状态 1 ---
        sleep(Duration::from_millis(1)).await;
        total += 10;
        // --- 状态 1: total=10, 进入状态 2 ---
        sleep(Duration::from_millis(1)).await;
        total += 20;
        // --- 状态 2: total=30, 进入状态 3 ---
        sleep(Duration::from_millis(1)).await;
        total += 30;
        // --- 状态 3: 完成 ---
        total
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(multi_step());
    assert_eq!(result, 60);
}

#[test]
/// 测试: Pin 的基本用法和自引用陷阱
fn test_pin_basics() {
    // 语法: Pin<P> 保证指针指向的值不会被移动
    //       Pin::new(x) 对于 Unpin 类型是安全的
    //       Box::pin(x) 在堆上创建 Pin<Box<T>>
    // 避坑: 自引用结构体必须 Pin 住, 否则移动后悬垂指针导致 UB
    use std::marker::PhantomPinned;

    // 演示: Pin<Box<T>> 在堆上固定值
    let value = 42i32;
    let pinned_box = Box::pin(value);
    assert_eq!(*pinned_box, 42);

    // 演示: Pin<&mut T> 在栈上固定值(仅对 Unpin 类型安全)
    let mut value = String::from("hello");
    {
        let pinned_ref = Pin::new(&mut value);
        assert_eq!(pinned_ref.as_ref().get_ref(), "hello");
    }
    // value 仍可安全使用, 因为 String 是 Unpin
    value.push_str(" world");
    assert_eq!(value, "hello world");

    // 演示: 非 Unpin 类型(PhantomPinned)只能用 unsafe 或 pin!/Box::pin
    struct SelfRef {
        data: String,
        _pin: PhantomPinned,
    }
    let sr = SelfRef { data: "test".into(), _pin: PhantomPinned };
    // Pin::new(&mut sr) 对 !Unpin 类型是 unsafe —— 这里不演示
    // 正确做法是用 Box::pin
    let pinned = Box::pin(sr);
    assert_eq!(pinned.data, "test");
}

#[test]
/// 测试: tokio::spawn 和 JoinHandle —— 多任务并发与结果收集
fn test_tokio_spawn_and_join() {
    // 语法: tokio::spawn(async { ... }) -> JoinHandle<T>
    //       handle.await 获取结果, handle.abort() 取消任务
    // 避坑: spawn 要求 Future: Send + 'static
    //       JoinHandle 不 await 也不 abort 会导致任务泄露
    let rt = tokio::runtime::Runtime::new().unwrap();
    let results = rt.block_on(async {
        let handle1 = tokio::spawn(async {
            sleep(Duration::from_millis(10)).await;
            100
        });
        let handle2 = tokio::spawn(async {
            sleep(Duration::from_millis(20)).await;
            200
        });
        let handle3 = tokio::spawn(async { 300 });

        // 使用 join! 并发等待所有结果
        let (r1, r2, r3) = tokio::join!(
            async { handle1.await.unwrap() },
            async { handle2.await.unwrap() },
            async { handle3.await.unwrap() },
        );
        vec![r1, r2, r3]
    });
    assert_eq!(results, vec![100, 200, 300]);
}

#[test]
/// 测试: tokio::select! 的选择等待 —— 多路复用与超时控制
fn test_tokio_select_macro() {
    // 语法: tokio::select! { pattern = future => handler, ... }
    // 避坑: 未选中的分支被 cancel(drop); 循环中使用 select! 需要 pin!
    //       biased 模式按声明顺序优先而不是随机
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        tokio::select! {
            v = async {
                sleep(Duration::from_millis(10)).await;
                "task_a"
            } => v,
            v = async {
                sleep(Duration::from_millis(5)).await;
                "task_b"
            } => v,
        }
    });
    // select! 随机选择就绪分支，验证结果是有效值而非具体顺序
    assert!(result == "task_a" || result == "task_b");

    // biased 模式: 按顺序检查, 同等就绪时选前者
    let result2 = rt.block_on(async {
        tokio::select! {
            biased;
            _ = async { sleep(Duration::from_millis(5)).await } => "biased_first",
            _ = async { sleep(Duration::from_millis(5)).await } => "biased_second",
        }
    });
    // biased 模式下优先检查第一个分支
    assert_eq!(result2, "biased_first");

    // select! 配合超时
    let result3 = rt.block_on(async {
        tokio::select! {
            v = async {
                sleep(Duration::from_millis(500)).await;
                "done"
            } => v,
            _ = sleep(Duration::from_millis(10)) => "timeout",
        }
    });
    assert_eq!(result3, "timeout");
}

#[test]
/// 测试: Stream 的基本用法(消费异步流)
fn test_stream_basics() {
    // 语法: Stream trait 是异步版 Iterator, 用 poll_next 驱动
    //       StreamExt 提供 map/filter/fold 等适配器方法
    // 避坑: Stream 消费需 .await next() 而非 for 循环
    //       Stream 只能消费一次(类似 Iterator)
    let rt = tokio::runtime::Runtime::new().unwrap();

    // 测试: stream::iter 创建 Stream
    let items = rt.block_on(async {
        let mut stream = stream::iter(vec![10, 20, 30, 40, 50]);
        let mut collected = Vec::new();
        while let Some(item) = stream.next().await {
            collected.push(item);
        }
        collected
    });
    assert_eq!(items, vec![10, 20, 30, 40, 50]);

    // 测试: StreamExt::fold 折叠消费
    let sum = rt.block_on(async {
        stream::iter(1..=5)
            .fold(0, |acc, x| async move { acc + x })
            .await
    });
    assert_eq!(sum, 15);

    // 测试: StreamExt::filter 过滤(嵌套 async 块导致 !Unpin, 需 Box::pin)
    let evens = rt.block_on(async {
        let mut stream = Box::pin(
            stream::iter(1..=10).filter(|&x| async move { x % 2 == 0 })
        );
        let mut result = Vec::new();
        while let Some(item) = stream.next().await {
            result.push(item);
        }
        result
    });
    assert_eq!(evens, vec![2, 4, 6, 8, 10]);

    // 测试: StreamExt::then 映射(将 Future 立即执行, 产生 Output)
    let doubled = rt.block_on(async {
        let items: Vec<_> = stream::iter(vec![1, 2, 3])
            .then(|x| async move { x * 2 })
            .collect()
            .await;
        items
    });
    assert_eq!(doubled, vec![2, 4, 6]);
}

#[test]
/// 测试: 异步 TCP echo 服务(客户端-服务端通信)
fn test_async_tcp_echo() {
    // 语法: tokio::net::TcpListener bind 端口 + accept 连接
    //       TcpStream 实现 AsyncRead + AsyncWrite
    // 避坑: 服务端必须 spawn 每个连接, 否则串行处理阻塞后续连接
    //       read 返回 0 表示对端关闭; write_all 保证完整写入
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // 启动 echo 服务
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // spawn 服务端处理
        let server_handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 128];
            let n = socket.read(&mut buf).await.unwrap();
            socket.write_all(&buf[..n]).await.unwrap(); // echo 回写
        });

        // 客户端连接并发送数据
        let mut client = TcpStream::connect(addr).await.unwrap();
        client.write_all(b"hello echo").await.unwrap();

        // 等待服务端处理完成
        server_handle.await.unwrap();

        // 读取回显
        let mut buf = vec![0u8; 128];
        let n = client.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello echo");
    });
}

#[test]
/// 测试: join! 并发执行多个异步任务
fn test_join_all_concurrent() {
    // 语法: tokio::join! 并发等待多个 Future, 返回各结果的元组
    //       futures::future::join_all 等待 Vec<Future> 全部完成
    // 避坑: join! 的并发不是并行(多线程); 需要用 spawn 才能多线程
    let rt = tokio::runtime::Runtime::new().unwrap();

    // join! 并发执行
    let (a, b, c) = rt.block_on(async {
        tokio::join!(
            async { sleep(Duration::from_millis(20)).await; 10 },
            async { sleep(Duration::from_millis(10)).await; 20 },
            async { sleep(Duration::from_millis(30)).await; 30 },
        )
    });
    assert_eq!((a, b, c), (10, 20, 30));

    // join_all 批量等待(用 spawn 统一类型, 不同 async 块类型不同)
    let results = rt.block_on(async {
        let h1 = tokio::spawn(async { 1 });
        let h2 = tokio::spawn(async { 2 });
        let h3 = tokio::spawn(async { 3 });
        let (r1, r2, r3) = tokio::join!(
            async { h1.await.unwrap() },
            async { h2.await.unwrap() },
            async { h3.await.unwrap() },
        );
        vec![r1, r2, r3]
    });
    assert_eq!(results, vec![1, 2, 3]);

    // 验证 join! 是并发的(总时间取最长, 不是累加)
    let start = std::time::Instant::now();
    let _ = rt.block_on(async {
        tokio::join!(
            async { sleep(Duration::from_millis(50)).await; 1 },
            async { sleep(Duration::from_millis(50)).await; 2 },
        )
    });
    let elapsed = start.elapsed();
    // 并发执行时间应接近 50ms 而不是 100ms
    assert!(elapsed < Duration::from_millis(120));
}

#[test]
/// 测试: 超时控制模式 —— tokio::time::timeout
fn test_timeout_pattern() {
    // 语法: timeout(duration, future).await 返回 Result
    //       Ok(val): 在期限内完成; Err(_): 超时
    // 避坑: 超时后 future 被 cancel; 不要对关键任务用 timeout 而不做清理
    let rt = tokio::runtime::Runtime::new().unwrap();

    // 正常完成
    let result = rt.block_on(async {
        timeout(
            Duration::from_millis(100),
            async {
                sleep(Duration::from_millis(10)).await;
                42
            },
        )
        .await
    });
    assert_eq!(result.unwrap(), 42);

    // 超时
    let result = rt.block_on(async {
        timeout(
            Duration::from_millis(10),
            async {
                sleep(Duration::from_millis(500)).await;
                "never"
            },
        )
        .await
    });
    assert!(result.is_err());

    // timeout 配合 select! 实现带默认值的超时
    let result = rt.block_on(async {
        tokio::select! {
            v = async {
                sleep(Duration::from_millis(500)).await;
                "done"
            } => v,
            _ = sleep(Duration::from_millis(10)) => "default_value",
        }
    });
    assert_eq!(result, "default_value");
}

#[test]
/// 测试: FuturesUnordered 批量并发(完成即处理模式)
fn test_futures_unordered() {
    // 语法: FuturesUnordered<F> 实现 Stream, 任意 Future 完成就能立即处理
    //       push(future) 添加任务; next().await 消费结果
    // 避坑: 内部是堆分配; 不适合无限增长的场景
     //       FuturesUnordered 不能共享, 只用于单线程消费
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut results = rt.block_on(async {
        let mut tasks = FuturesUnordered::new();
        // 使用 tokio::spawn 统一类型, 避免不同 async 块类型不匹配
        tasks.push(tokio::spawn(async { sleep(Duration::from_millis(30)).await; 1 }));
        tasks.push(tokio::spawn(async { sleep(Duration::from_millis(10)).await; 2 }));
        tasks.push(tokio::spawn(async { sleep(Duration::from_millis(20)).await; 3 }));

        let mut results = Vec::new();
        while let Some(val) = tasks.next().await {
            results.push(val.unwrap());
        }
        results
    });
    // 按完成顺序收集，排序后验证值集合
    results.sort();
    assert_eq!(results, vec![1, 2, 3]);

    // 通过 buffer_unordered 实现类似效果
    let results2 = rt.block_on(async {
        let items: Vec<_> = stream::iter(vec![30u64, 10, 20])
            .map(|ms| async move {
                sleep(Duration::from_millis(ms)).await;
                ms
            })
            .buffer_unordered(3) // 最多 3 个并发
            .collect()
            .await;
        items
    });
    assert_eq!(results2, vec![10, 20, 30]);
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 演示: 手动实现 Future, 理解 poll 机制
struct TimerFuture {
    duration: Duration,
    elapsed: bool,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.elapsed {
            Poll::Ready(())
        } else {
            self.elapsed = true;
            // 简单实现: 第一次 poll 立即返回 Ready
            // 实际 timer 需要用 Waker 在时间到达时通知执行器
            Poll::Ready(())
        }
    }
}

#[test]
/// 测试: 手动实现 Future trait, 理解 poll 与状态机
fn test_manual_future() {
    let future = TimerFuture {
        duration: Duration::from_millis(10),
        elapsed: false,
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(future); // TimerFuture 第一次 poll 即 Ready
}
