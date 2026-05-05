# 并发编程

> 并发不是魔法，是精心编排的时序——Rust 的类型系统在编译期确保数据竞争不可能发生，让你在运行期只需关心逻辑正确性。

## 1. 线程基础

### 1.1 thread::spawn

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..=5 {
            println!("子线程: {}", i);
            thread::sleep(Duration::from_millis(50));
        }
    });

    for i in 1..=3 {
        println!("主线程: {}", i);
        thread::sleep(Duration::from_millis(50));
    }

    handle.join().unwrap();
}
```

> spawn 就是"分叉"——它创建一条新的执行路径，join 就是"汇合"——它等待分叉的路到达终点。

### 1.2 move 闭包与所有权

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3, 4, 5];

    let handle = thread::spawn(move || {
        println!("子线程收到数据: {:?}", data);
        data.iter().sum::<i32>()
    });

    // data 已移入闭包，此处不可再用
    let sum = handle.join().unwrap();
    println!("总和: {}", sum);
}
```

### 1.3 作用域线程 (scoped threads)

```rust
use std::thread;

fn main() {
    let mut data = vec![1, 2, 3, 4, 5];

    thread::scope(|s| {
        // 作用域线程可以直接借用外部变量
        s.spawn(|| {
            data.push(6);
            println!("第一个线程: {:?}", data);
        });

        s.spawn(|| {
            data.push(7);
            println!("第二个线程: {:?}", data);
        });
    });

    // scope 结束后所有线程都已完成
    println!("最终数据: {:?}", data);
}
```

> scoped threads 解决了"线程内借用外部变量"的经典难题——编译器知道 scope 内所有线程在 scope 结束前都会 join，因此允许安全地借用。

### 1.4 Builder 配置线程

```rust
use std::thread;

fn main() {
    let builder = thread::Builder::new()
        .name("my-thread".into())
        .stack_size(4 * 1024 * 1024); // 4MB 栈

    let handle = builder
        .spawn(|| {
            println!("我是 {}", thread::current().name().unwrap());
            42
        })
        .unwrap();

    let result = handle.join().unwrap();
    println!("结果: {}", result);
}
```

## 2. park/unpark

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        println!("工作线程: 开始等待...");
        thread::park(); // 挂起当前线程
        println!("工作线程: 被唤醒!");
    });

    thread::sleep(Duration::from_millis(100));
    println!("主线程: 唤醒工作线程");
    handle.thread().unpark(); // 唤醒目标线程

    handle.join().unwrap();
}
```

> park/unpark 是最轻量的线程同步原语——它不需要 Mutex、不需要 channel，就像一个"许可令牌"，但要注意 unpark 先于 park 时不会丢失。

## 3. Send 与 Sync

### 3.1 两个自动 trait

| Trait | 语义 | 自动实现条件 |
|-------|------|-------------|
| `Send` | 值可以在线程间安全地转移所有权 | 所有字段都是 Send |
| `Sync` | 值的不可变引用可以在线程间安全地共享 | 所有字段都是 Sync |

```rust
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

fn main() {
    // Rc 不是 Send：不能发送到其他线程
    // let rc = Rc::new(42);
    // thread::spawn(move || { println!("{}", rc); });
    // ↑ 编译错误：Rc<i32> 没有实现 Send

    // Arc 是 Send + Sync：可以安全发送和共享
    let arc = Arc::new(42);
    let arc_clone = Arc::clone(&arc);
    thread::spawn(move || {
        println!("子线程: {}", arc_clone);
    }).join().unwrap();
}
```

### 3.2 RefCell vs Mutex

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Mutex 提供跨线程的内部可变性
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("计数结果: {}", *counter.lock().unwrap()); // 10
}
```

> `Rc<RefCell<T>>` 是单线程的内部可变性方案，`Arc<Mutex<T>>` 是多线程版本——两者结构相似但用途截然不同。

### 3.3 unsafe impl 注意事项

```rust
use std::cell::Cell;

// 包含 Cell 的类型自动为 !Sync
struct NotSync {
    cell: Cell<i32>,
}

// 只有当你知道自己在做什么时才 unsafe impl
// unsafe impl Sync for NotSync {} // 危险！Cell 不是 Sync
```

## 4. 消息传递 (mpsc)

### 4.1 基本通道

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send("第一条消息").unwrap();
        tx.send("第二条消息").unwrap();
        // tx 离开作用域时通道关闭
    });

    // recv 阻塞等待
    println!("收到: {}", rx.recv().unwrap());

    // 迭代器方式接收所有消息
    for msg in rx {
        println!("收到: {}", msg);
    }
}
```

### 4.2 多生产者

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    thread::spawn(move || {
        tx.send("来自 A").unwrap();
    });

    thread::spawn(move || {
        tx2.send("来自 B").unwrap();
    });

    // 两个 tx 都已 drop 后 recv 才会返回 Err
    for msg in rx {
        println!("{}", msg);
    }
}
```

### 4.3 recv_timeout

```rust
use std::sync::mpsc;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        tx.send("等了很久").unwrap();
    });

    match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(msg) => println!("收到了: {}", msg),
        Err(mpsc::RecvTimeoutError::Timeout) => println!("超时了"),
        Err(mpsc::RecvTimeoutError::Disconnected) => println!("通道关闭"),
    }
}
```

> mpsc 遵循"用类型共享内存，用消息共享知识"的哲学——它让并发通信不再是"共享内存加锁"，而是"发送消息通信"。

## 5. 共享内存同步

### 5.1 Mutex

```rust
use std::sync::{Arc, Mutex};

fn main() {
    let shared = Arc::new(Mutex::new(vec![]));

    let mut handles = vec![];
    for i in 0..5 {
        let shared = Arc::clone(&shared);
        handles.push(std::thread::spawn(move || {
            let mut data = shared.lock().unwrap();
            data.push(i);
        }));
    }

    for h in handles { h.join().unwrap(); }

    println!("{:?}", shared.lock().unwrap()); // 乱序的 [0,1,2,3,4]
}
```

### 5.2 RwLock

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let data = Arc::new(RwLock::new(0));

    let readers: Vec<_> = (0..5).map(|_| {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            let val = data.read().unwrap();
            println!("读取: {}", *val);
        })
    }).collect();

    let writer = {
        let data = Arc::clone(&data);
        thread::spawn(move || {
            let mut val = data.write().unwrap();
            *val = 42;
            println!("写入: {}", *val);
        })
    };

    for r in readers { r.join().unwrap(); }
    writer.join().unwrap();
}
```

> RwLock 适合"读多写少"的场景——多个读者可以同时持有读锁，而写锁是独占的。但要注意获取写锁时可能被饥饿。

### 5.3 parking_lot 替代

```rust
// Cargo.toml: parking_lot = "0.12"
use parking_lot::Mutex;
use std::sync::Arc;

fn main() {
    let m = Arc::new(Mutex::new(0));
    // parking_lot 的 Mutex 不需要 unwrap()
    // 不会中毒(Poisoning)
    // 性能更好（更小的内存占用，零分配）
    let mut guard = m.lock();
    *guard += 1;
}
```

## 6. Condvar 条件变量

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;

fn main() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);

    let handle = thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap(); // 自动释放锁并等待
        }
        println!("工作线程: 收到启动信号!");
        *started = false; // 重置信号
    });

    thread::sleep(std::time::Duration::from_millis(100));

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one(); // 唤醒一个等待者
    println!("主线程: 发送启动信号");

    handle.join().unwrap();
}
```

> Condvar 解决了"等待特定条件成立"的问题——线程在条件未满足时自动挂起，条件满足时被唤醒，避免了忙等待的 CPU 浪费。

## 7. Barrier 同步屏障

```rust
use std::sync::{Arc, Barrier};
use std::thread;

fn main() {
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];

    for i in 0..3 {
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            println!("线程 {}: 第一阶段工作", i);
            b.wait(); // 在此等待所有线程到达
            println!("线程 {}: 第二阶段工作", i);
        }));
    }

    for h in handles { h.join().unwrap(); }
}
```

> Barrier 实现了"到齐了就走"的批量同步——它在并行算法的分阶段计算中不可或缺。

## 8. Once/OnceLock

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<String> = OnceLock::new();

fn get_config() -> &'static String {
    CONFIG.get_or_init(|| {
        println!("初始化配置（仅一次）");
        String::from("app_config_value")
    })
}

fn main() {
    let mut handles = vec![];
    for _ in 0..10 {
        handles.push(std::thread::spawn(|| {
            let config = get_config();
            println!("{}", config);
        }));
    }
    for h in handles { h.join().unwrap(); }
}
```

## 9. Atomic 原子操作

```rust
use std::sync::atomic::{AtomicI32, AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                c.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles { h.join().unwrap(); }
    println!("最终计数: {}", counter.load(Ordering::SeqCst)); // 1000
}
```

### 9.1 Ordering 内存序

| 内存序 | 保证 | 适用场景 |
|--------|------|----------|
| `Relaxed` | 仅保证原子性 | 简单计数器 |
| `Release` | 之前的所有写对后续 Acquire 可见 | 锁释放 |
| `Acquire` | 后续所有读/写看到 Release 前的状态 | 锁获取 |
| `AcqRel` | Release + Acquire 组合 | CAS 操作 |
| `SeqCst` | 全局顺序一致性，最强保证 | 需要严格顺序 |

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let ready = Arc::new(AtomicBool::new(false));
    let data = Arc::new(AtomicI32::new(0));

    let r = Arc::clone(&ready);
    let d = Arc::clone(&data);
    let producer = thread::spawn(move || {
        d.store(42, Ordering::Release); // 写入数据
        r.store(true, Ordering::Release); // 通知就绪
    });

    let r = Arc::clone(&ready);
    let d = Arc::clone(&data);
    let consumer = thread::spawn(move || {
        while !r.load(Ordering::Acquire) {} // 等待就绪信号
        let val = d.load(Ordering::Acquire);
        println!("读取到: {}", val); // 保证是 42
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

> 选择 Relaxed 而非 SeqCst 不是为了炫技——对于高性能计数器等场景，SeqCst 的全局同步开销可能成为瓶颈。但不知道用什么时，SeqCst 永远是正确的。

## 10. thread_local! 线程局部存储

```rust
use std::cell::RefCell;
use std::thread;

thread_local! {
    static THREAD_DATA: RefCell<u32> = RefCell::new(0);
}

fn main() {
    THREAD_DATA.with(|d| {
        *d.borrow_mut() = 42;
    });

    thread::spawn(|| {
        // 每个线程有独立的值
        THREAD_DATA.with(|d| {
            println!("子线程的值: {}", *d.borrow()); // 0（不是 42）
        });
    }).join().unwrap();
}
```

## 11. 并发模式

### 生产者-消费者

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::collections::VecDeque;

struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    not_empty: Condvar,
}

impl<T> Channel<T> {
    fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()), not_empty: Condvar::new() }
    }

    fn send(&self, item: T) {
        self.queue.lock().unwrap().push_back(item);
        self.not_empty.notify_one();
    }

    fn recv(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        loop {
            if let Some(item) = queue.pop_front() {
                return item;
            }
            queue = self.not_empty.wait(queue).unwrap();
        }
    }
}
```

> 从零实现一个 Channel 是理解并发通信的最佳方式——你需要协调 Mutex、Condvar 和所有权三个核心概念。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 忘记 join 导致主线程提前退出 | 守护线程在主线程结束后被强制终止 | 对所有 JoinHandle 调用 join |
| Rc 跨线程使用 | Rc 不是 Send | 改用 Arc |
| RefCell 跨线程使用 | RefCell 不是 Sync | 改用 Mutex 或 RwLock |
| Mutex 死锁 | 同一线程多次 lock 同一 Mutex | 仔细规划锁顺序；使用 try_lock 或 parking_lot |
| 锁中毒(PoisonError) | 持锁线程 panic 导致 Mutex 被标记为已中毒 | 使用 `lock().unwrap_or_else(|e| e.into_inner())` 处理中毒情况 |
| Condvar 虚假唤醒 | wait 可能在未收到通知时返回 | 始终在 while 循环中使用 wait，检查条件 |
| mpsc 通道关闭后发送 | 接收端已 drop | 使用 send 的 Result 返回值检查，或使用 sync_channel |
| Atomic Ordering 使用 Relaxed 但需要同步 | 编译器/CPU 可能重排序指令 | 对同步场景至少使用 Release/Acquire 对 |

> **详见测试**: `tests/rust_features/22_concurrency.rs`
