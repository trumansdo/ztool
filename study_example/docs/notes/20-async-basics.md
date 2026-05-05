# 异步编程基础

> 异步编程的本质是把"等待"的主动权交还给运行时——在 IO 完成之前，当前任务释放 CPU，让其他任务得以执行，这是对计算资源最高效的利用。

## 1. async/await 核心语法

### 1.1 async fn 返回 impl Future

```rust
// async 函数在编译时被转换为返回 Future 的状态机
async fn fetch_data(id: u32) -> String {
    // 模拟异步操作
    format!("数据{}", id)
}

// 类型签名的等价表达
fn fetch_data(id: u32) -> impl std::future::Future<Output = String> {
    async move { format!("数据{}", id) }
}
```

> `async fn` 只是语法糖——编译器把它转换为返回 `impl Future<Output = T>` 的普通函数，真正的魔法在于 `.await` 展开成的状态机代码。

### 1.2 .await 暂停与恢复

```rust
async fn process() {
    println!("步骤1: 开始");

    // .await 暂停当前任务，将控制权交还运行时
    let data = fetch_data(42).await;

    println!("步骤2: 获得 {}", data);
}
```

`.await` 的语义：
- 它不会阻塞线程，而是将当前 `Future` 挂起
- 运行时切换到其他可执行的任务
- 当被等待的 `Future` 就绪时，从来中断处恢复

### 1.3 ? 操作符在 async 中的用法

```rust
async fn safe_fetch(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}
```

> `?` 在异步函数中的行为与同步代码完全一致——遇到错误立即返回，不需要特殊的异步错误处理语法。

## 2. Future trait 内幕

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

### 2.1 状态机变换

编译器将 async 函数展开为一个匿名类型（状态机），包含：
- 所有 `.await` 点之间的局部变量
- 当前执行到哪个阶段的枚举值
- 可能跨越 `.await` 的引用（需要 Pin 保护）

```rust
// 用户编写的 async 代码
async fn example(x: i32) -> i32 {
    let a = async_op1().await;
    let b = a + x;
    async_op2(b).await
}

// 编译器大致展开为（伪代码）
enum ExampleStateMachine {
    Start { x: i32 },
    AfterOp1 { x: i32, _awaiting_fut1: Fut1 },
    AfterOp2 { result: i32 },
    End,
}
```

### 2.2 Waker 唤醒机制

```rust
use std::task::Waker;

// Waker 负责在 Future 就绪时通知执行器
// 执行器在 poll 返回 Poll::Pending 后注册 Waker
// IO 完成或定时器触发时，Waker::wake() 被调用
// 执行器收到唤醒信号，重新将任务放入队列
```

> 理解 Waker 就能理解异步运行时为什么是"事件驱动"的——它不需要轮询检查每个任务，而是由任务主动通知运行时"我准备好了"。

## 3. 异步运行时对比

| 特性 | tokio | async-std | smol |
|------|-------|-----------|------|
| 模型 | 多线程工作窃取 | 多线程工作窃取 | 单线程/轻量 |
| IO 后端 | mio | async-io | polling |
| 定时器 | tokio::time | async_std::stream | async-io |
| 生态 | 最丰富 | 兼容 tokio API | 极简 |
| 适用场景 | 高性能服务 | tokio替代 | 嵌入式/轻量 |

### 3.1 Tokio 核心使用

```rust
#[tokio::main]
async fn main() {
    println!("Tokio 运行时已启动");
}

// #[tokio::main] 宏展开等价于
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        println!("Tokio 运行时已启动");
    });
}
```

### 3.2 单线程 vs 多线程运行时

```rust
// 单线程运行时（适合 IO 密集型）
#[tokio::main(flavor = "current_thread")]
async fn main() { }

// 多线程运行时（适合混合 CPU/IO）
#[tokio::main]
async fn main() { }

// 手动创建自定义运行时
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-worker")
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async { /* ... */ });
}
```

> 单线程运行时并非"性能差"——当任务主要是等待网络 IO 时，单线程已经足够；多线程的优势在于 CPU 密集计算的并行。

### 3.3 spawn 后台任务

```rust
use tokio::task;

async fn background_work(id: u32) -> u32 {
    println!("后台任务 {} 启动", id);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("后台任务 {} 完成", id);
    id * 10
}

#[tokio::main]
async fn main() {
    // spawn 提交到运行时，立即返回 JoinHandle
    let handle1 = task::spawn(background_work(1));
    let handle2 = task::spawn(background_work(2));

    // 等待所有任务完成
    let (r1, r2) = tokio::join!(handle1, handle2);
    println!("结果: {} {}", r1.unwrap(), r2.unwrap());
}
```

## 4. 异步 IO 操作

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TCP 监听
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("新连接: {}", addr);

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                let n = socket.read(&mut buf).await.unwrap();
                if n == 0 { return; }
                socket.write_all(&buf[0..n]).await.unwrap();
            }
        });
    }
}
```

> 异步 IO 与同步 IO 的关键区别：`read` 返回 `Poll::Pending` 时线程不阻塞，运行时去处理其他连接，IO 就绪时再回来继续读。

## 5. async 块与闭包

### 5.1 匿名 async 块

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = async {
            let a = work_a().await;
            let b = work_b().await;
            a + b
        };
        println!("结果: {}", result.await);
    });
}
```

### 5.2 move async

```rust
#[tokio::main]
async fn main() {
    let name = String::from("张三");

    // move async 取得 name 的所有权
    let fut = async move {
        println!("异步块中: {}", name);
    };

    // 此时 name 已被移入 fut，外部不可再使用
    fut.await;
}
```

> `move async` 和 `move ||` 闭包一样道理——async 块默认按需借用，加 `move` 则强制转移所有权，这在 spawn 时几乎是必需的。

## 6. Pin 与 Unpin

### 6.1 自引用结构问题

async 生成的 Future 可能包含自引用指针（因为局部变量可以引用同一状态机中的其他变量），移动这些结构会导致悬垂指针。

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(Self {
            data,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });
        // 安全：boxed 已被固定，不会移动
        let self_ptr: *const String = &boxed.data;
        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).ptr = self_ptr;
        }
        boxed
    }
}
```

### 6.2 Pin 工具宏

```rust
use tokio::pin;
use futures::pin_mut;

async fn demo() {
    let future = async { 42 };
    tokio::pin!(future);      // tokio 提供的 pin 宏
    // future 现在被固定，可以安全地 poll

    let future2 = async { 100 };
    futures::pin_mut!(future2); // futures crate 提供的版本
}
```

> Pin 是自引用结构的"护身符"——被 Pin 包裹的值保证不会在内存中移动，这是 async 状态机正常运行的必要条件。

## 7. Stream trait

Stream 是异步迭代器：

```rust
use tokio_stream::{self as stream, StreamExt};

#[tokio::main]
async fn main() {
    // 从迭代器创建 Stream
    let mut stream = stream::iter(vec![1, 2, 3, 4, 5]);

    // 类似迭代器的链式操作
    let result: Vec<_> = stream
        .filter(|x| { let val = *x; async move { val % 2 == 0 } })
        .map(|x| async move { x * 10 })
        .collect()
        .await;

    println!("{:?}", result); // [20, 40]
}
```

### 7.1 Stream 常用方法

```rust
// fold 异步折叠
let sum = stream
    .fold(0, |acc, x| async move { acc + x })
    .await;

// next 逐个取出
while let Some(val) = stream.next().await {
    println!("{}", val);
}
```

> Stream 之于 Future，正如 Iterator 之于值——它是"时间上的序列"，每个元素在未来的某个时刻就绪。

## 8. Tokio 核心并发模式

### 8.1 join!：并发等待全部

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (r1, r2, r3) = tokio::join!(
        async { sleep(Duration::from_secs(2)).await; 1 },
        async { sleep(Duration::from_secs(1)).await; 2 },
        async { sleep(Duration::from_secs(3)).await; 3 },
    );
    println!("全部完成: {} {} {}", r1, r2, r3);
    // 总耗时约 3 秒（最长的那个）
}
```

### 8.2 select!：竞态等待第一个

```rust
#[tokio::main]
async fn main() {
    tokio::select! {
        result = async { sleep(Duration::from_secs(2)).await; "慢" } => {
            println!("慢操作完成: {}", result);
        }
        result = async { sleep(Duration::from_millis(100)).await; "快" } => {
            println!("快操作完成: {}", result);
        }
    }
    // 输出: 快操作完成: 快
}
```

### 8.3 select! 的 biased 模式与超时

```rust
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    tokio::select! {
        biased; // biased 模式：按顺序检查分支（非随机）
        result = long_running_task() => {
            println!("任务完成");
        }
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            println!("超时！");
        }
    }
}

// 或使用 timeout 包装器
async fn with_timeout() -> Result<(), tokio::time::error::Elapsed> {
    let result = timeout(
        Duration::from_secs(5),
        long_running_task()
    ).await?;
    Ok(())
}
```

> select! 实现了"竞态"语义——如果有分支取消，剩下的分支仍在运行。这是实现超时、取消等模式的基础操作符。

### 8.4 FuturesUnordered：批量异步

```rust
use futures::stream::FuturesUnordered;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let mut tasks = FuturesUnordered::new();

    for i in 0..10 {
        tasks.push(async move {
            tokio::time::sleep(Duration::from_millis(i * 100)).await;
            i
        });
    }

    // 谁先完成就先处理谁
    while let Some(result) = tasks.next().await {
        println!("任务 {} 完成", result);
    }
}
```

> FuturesUnordered 实现了"完成即处理"的模式——它不是 FIFO，而是按实际完成时间排序，对批量 API 调用之类场景效率极高。

## 9. 手动实现 Future

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

struct TimerFuture {
    deadline: Instant,
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            // 未就绪，注册 waker（实际需要全局定时器）
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
```

> 手动实现 Future 是理解异步的最深方式——你会发现 .await 点本质上就是"poll 返回 Pending 并注册 waker"的位置。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| async fn 忘记 .await | 不 .await 的 Future 什么也不做 | 编译器 warning 会提醒，务必加上 .await |
| 在同步代码中调用 .await | .await 只能在 async 上下文 | 使用 `block_on` 启动运行时 |
| spawn 闭包未使用 move | 借用可能超过闭包生命周期 | 对 spawn 的闭包使用 `async move` |
| select! 中取消分支后资源泄漏 | 取消分支时 DROP 仍会正常执行 | 确保取消安全的清理逻辑在 Drop 中 |
| 阻塞操作放入 async | 阻塞代码阻塞整个工作线程 | 使用 `spawn_blocking` 将 CPU/阻塞任务移到阻塞线程池 |
| 忘记 Pin | 自引用 Future 移动后悬垂指针 | 使用 `Box::pin` 或 `pin_mut!` 宏 |
| join! 全部等待 vs 独立 spawn | join! 要求全部完成才继续 | 独立任务用 spawn，需要结果聚合用 join! |

> **详见测试**: `tests/rust_features/20_async_basics.rs`
