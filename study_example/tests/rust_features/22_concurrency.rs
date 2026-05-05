// ---------------------------------------------------------------------------
// 7.1 并发编程
// ---------------------------------------------------------------------------

use std::sync::{Arc, Mutex, mpsc, RwLock, Barrier, Condvar, Once, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::thread;
use std::time::Duration;

// ============================================================================
// 线程基础
// ============================================================================

#[test]
/// 测试: 创建线程 (thread::spawn/join)
fn test_thread_basics() {
    // 语法: thread::spawn(闭包) 创建新线程
    //
    // 操作:
    //   - spawn 返回 JoinHandle
    //   - handle.join() 等待线程结束，可获取返回值
    //   - 主线程结束, 所有子线程被终止
    //
    // 避坑:
    //   - 闭包需要 move 获取变量所有权
    //   - 忘掉 join 会导致主线程先结束
    //   - join 返回 Result (子线程可能 panic)
    //

    let handle = thread::spawn(|| {
        42
    });

    let result = handle.join().unwrap();
    assert_eq!(result, 42);
}

#[test]
/// 测试: move 闭包在线程中传递所有权
fn test_thread_move() {
    // 语法: thread::spawn(move || { ... }) 使用 move 闭包
    //
    // 避坑:
    //   - move 后原作用域不能再用该变量
    //   - 只能 move, 不能借用(生命周期问题)
    //

    let v = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        assert_eq!(v.len(), 3);
    });
    handle.join().unwrap();
    // println!("{:?}", v); // 编译错误: v 已被 move
}

#[test]
/// 测试: 作用域线程 (thread::scope) — 可借用局部变量，无需 move
fn test_thread_scope() {
    // 语法: thread::scope(|s| { s.spawn(...) }) (Rust 1.63+)
    //
    // 特性:
    //   - scope 内的线程可以直接借用外部变量，无需 move 或 Arc
    //   - scope 返回前会自动等待所有子线程结束（隐式 join）
    //   - 子线程 panic 会导致 scope 也 panic
    //
    // 避坑:
    //   - scope 内的闭包不能逃逸到 scope 外部
    //   - 谨慎处理子线程 panic 传播
    //

    let mut data = vec![1, 2, 3, 4, 5];

    thread::scope(|s| {
        // 直接借用 data，无需 move
        s.spawn(|| {
            for item in &data {
                let _ = item * 2; // 读取操作
            }
        });

        s.spawn(|| {
            for item in &data {
                let _ = item + 1; // 读取操作
            }
        });
        // scope 结束时自动 join 所有子线程
    });

    // scope 结束后可以安全地修改 data
    data.push(6);
    assert_eq!(data.len(), 6);
    assert_eq!(data[5], 6);
}

#[test]
/// 测试: 线程 Builder 配置 (名称、栈大小)
fn test_thread_builder() {
    // 语法: thread::Builder::new().name(...).stack_size(...).spawn(...)
    //
    // 操作:
    //   - .name("名称")     设置线程名称，便于调试
    //   - .stack_size(n)    设置栈大小(字节)，默认约 2 MiB
    //   - .spawn(闭包)      返回 io::Result<JoinHandle<T>>
    //
    // 避坑:
    //   - 栈大小有平台最小值限制，设置过小可能失败
    //   - Builder 的 spawn 需要 unwrap() 处理 io::Result
    //

    let builder = thread::Builder::new()
        .name("custom-worker".into())
        .stack_size(2 * 1024 * 1024); // 2 MiB

    let handle = builder.spawn(|| {
        let name = thread::current().name().map(str::to_string);
        (name, 42)
    }).unwrap();

    let (thread_name, result) = handle.join().unwrap();
    assert_eq!(thread_name, Some("custom-worker".to_string()));
    assert_eq!(result, 42);
}

#[test]
/// 测试: 线程 park/unpark (暂停/唤醒)
fn test_thread_park_unpark() {
    // 语法: thread::park() 暂停当前线程, handle.thread().unpark() 唤醒
    //
    // 操作:
    //   - park()        暂停当前线程，直到被 unpark
    //   - park_timeout(d) 暂停最多 d 时长
    //   - unpark()      唤醒目标线程 (可提前于 park 调用)
    //
    // 避坑:
    //   - park 有虚假唤醒风险，应配合条件循环使用
    //   - unpark 可提前调用（有记忆性），下次 park 会立即返回
    //   - 复杂同步场景请用 Condvar 或 channel
    //

    let handle = thread::spawn(|| {
        thread::park(); // 暂停自己
        99
    });

    // 短暂等待确保子线程已进入 park
    thread::sleep(Duration::from_millis(20));

    // 唤醒子线程
    handle.thread().unpark();

    let result = handle.join().unwrap();
    assert_eq!(result, 99);
}

#[test]
/// 测试: 线程睡眠与让步 (sleep / yield_now)
fn test_thread_sleep_and_yield() {
    // 语法:
    //   - thread::sleep(Duration)  睡眠至少指定时长，不占 CPU
    //   - thread::yield_now()     主动让出 CPU 时间片
    //
    // 避坑:
    //   - yield_now 不保证时间，只作提示
    //   - 不要用 sleep 做线程同步
    //

    let start = std::time::Instant::now();
    thread::sleep(Duration::from_millis(50));
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(50));

    // yield_now 测试：主线程让出，让子线程有机会执行
    let flag = Arc::new(AtomicBool::new(false));
    let flag2 = Arc::clone(&flag);
    let handle = thread::spawn(move || {
        flag2.store(true, Ordering::SeqCst);
    });

    // 让出 CPU 让子线程有机会运行
    for _ in 0..100 {
        thread::yield_now();
    }
    handle.join().unwrap();
    assert!(flag.load(Ordering::SeqCst));
}

// ============================================================================
// 消息传递 (Channel)
// ============================================================================

#[test]
/// 测试: 消息传递通道 (mpsc::channel/send/recv)
fn test_message_passing() {
    // 语法: mpsc::channel() 创建多生产者单消费者通道
    //
    // 操作:
    //   - tx.send(val)         发送消息(返回 Result)
    //   - rx.recv()            阻塞接收(返回 Result)
    //   - rx.try_recv()        非阻塞接收
    //   - rx.recv_timeout(d)   超时接收
    //   - tx.clone()           克隆发送者(多生产者)
    //
    // 避坑:
    //   - send 返回 Err 表示接收端已 drop
    //   - recv 阻塞当前线程
    //   - 通道是无界的(默认)，发送永不阻塞
    //

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(42).unwrap();
    });

    assert_eq!(rx.recv().unwrap(), 42);
}

#[test]
/// 测试: 多发送者 (mpsc + clone)
fn test_multiple_senders() {
    // 语法: tx.clone() 创建多发送者
    //

    let (tx, rx) = mpsc::channel();

    let tx1 = tx.clone();
    thread::spawn(move || {
        tx1.send("from thread 1").unwrap();
    });

    let tx2 = tx.clone();
    thread::spawn(move || {
        tx2.send("from thread 2").unwrap();
    });

    // 必须 drop 原始 tx，否则 rx 会永远等待
    drop(tx);

    let mut messages = Vec::new();
    for received in rx {
        messages.push(received);
    }
    assert_eq!(messages.len(), 2);
}

#[test]
/// 测试: 非阻塞接收 (try_recv)
fn test_try_recv() {
    // 语法: try_recv 非阻塞, 立即返回
    //
    // 返回值:
    //   - Ok(msg)         成功接收
    //   - Err(Empty)      无消息
    //   - Err(Disconnected) 通道已关闭
    //
    // 避坑:
    //   - 需要区分 Empty 和 Disconnected
    //

    let (tx, rx) = mpsc::channel();
    tx.send("hello").unwrap();

    match rx.try_recv() {
        Ok(msg) => assert_eq!(msg, "hello"),
        Err(_) => panic!("应该有消息"),
    }

    // 再次 try_recv 应返回 Empty
    match rx.try_recv() {
        Err(mpsc::TryRecvError::Empty) => {} // 预期
        _ => panic!("应为空"),
    }
}

#[test]
/// 测试: mpsc 通道完整用法 (send/recv/try_recv/recv_timeout/通道关闭)
fn test_mpsc_channel_full() {
    // 语法: mpsc 通道的完整 API 演练
    //
    // 场景:
    //   - 多生产者发送消息
    //   - 超时接收、非阻塞接收
    //   - 通道关闭语义
    //

    let (tx, rx) = mpsc::channel();

    // 多生产者发送
    let tx1 = tx.clone();
    let tx2 = tx.clone();

    thread::spawn(move || {
        tx1.send(10).unwrap();
        tx1.send(20).unwrap();
    });

    thread::spawn(move || {
        tx2.send(30).unwrap();
        // tx2 在此 drop
    });

    // 必须 drop 原始 tx，否则接收端永远阻塞
    drop(tx);

    let mut results = Vec::new();
    // 通过 for 循环接收全部消息（通道关闭时自动停止）
    for received in rx {
        results.push(received);
    }
    results.sort();
    assert_eq!(results, vec![10, 20, 30]);
}

#[test]
/// 测试: recv_timeout 超时接收
fn test_recv_timeout() {
    // 语法: rx.recv_timeout(Duration) 阻塞等待最多指定时间
    //
    // 返回值:
    //   - Ok(val)       成功收到消息
    //   - Err(Timeout)  超时
    //   - Err(Disconnected) 通道关闭
    //

    let (tx, rx) = mpsc::channel();

    // 延迟发送
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        tx.send("delayed").unwrap();
    });

    // 第一次可能超时(因消息尚未发送)
    let mut got_message = false;
    for _ in 0..20 {
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(msg) => {
                assert_eq!(msg, "delayed");
                got_message = true;
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // 继续等待
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("通道不应关闭");
            }
        }
    }
    assert!(got_message, "应收到消息");
}

// ============================================================================
// 共享内存同步
// ============================================================================

#[test]
/// 测试: 互斥锁 Mutex (数据保护)
fn test_mutex_basics() {
    // 语法: Mutex<T> 提供互斥访问
    //
    // 操作:
    //   - Mutex::new(val)   创建
    //   - lock().unwrap()   获取锁(阻塞)，返回 MutexGuard
    //   - MutexGuard 自动释放锁(Drop)
    //
    // 避坑:
    //   - lock 可能因中毒返回 Err
    //   - 不要在 .await 期间持有锁
    //   - guard 在作用域结束自动解锁
    //

    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap();
        *num = 6;
    }
    assert_eq!(*m.lock().unwrap(), 6);
}

#[test]
/// 测试: Arc + Mutex (线程安全共享)
fn test_arc_mutex() {
    // 语法: Arc 提供线程安全引用计数, Mutex 提供互斥
    //
    // 模式:
    //   - Arc::clone(&counter) 增加引用计数
    //   - counter.lock().unwrap() 获取锁
    //
    // 避坑:
    //   - Arc 是引用计数, 不是深拷贝
    //   - 避免锁持有时间过长
    //

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

    assert_eq!(*counter.lock().unwrap(), 10);
}

#[test]
/// 测试: 读写锁 RwLock (读共享/写独占)
fn test_rwlock() {
    // 语法: RwLock 允许多个读锁或一个写锁
    //
    // 适用场景:
    //   - 读多写少的场景
    //   - 比 Mutex 有更好的并发性能
    //
    // 避坑:
    //   - 可能 writer starvation (写者饥饿)
    //   - 持有读锁时不能升级为写锁
    //

    let lock = RwLock::new(5);

    // 多个读锁共存
    {
        let r1 = lock.read().unwrap();
        let r2 = lock.read().unwrap();
        assert_eq!(*r1, 5);
        assert_eq!(*r2, 5);
    }

    // 写锁独占
    {
        let mut w = lock.write().unwrap();
        *w += 1;
    }

    assert_eq!(*lock.read().unwrap(), 6);
}

#[test]
/// 测试: 锁中毒 (Mutex Poisoning) — 持有锁的线程 panic 后锁被污染
fn test_mutex_poisoning() {
    // 语法: 线程 panic 时持有的 Mutex 被标记为"中毒"
    //
    // 行为:
    //   - lock() 返回 Err(PoisonError)
    //   - into_inner() 可强行获取数据
    //   - get_ref() 获取中毒数据的引用
    //
    // 避坑:
    //   - 生产代码应正确处理 PoisonError
    //   - 可选择 restore 或替换中毒数据
    //

    let mutex = Arc::new(Mutex::new(0));
    let m_clone = Arc::clone(&mutex);

    let handle = thread::spawn(move || {
        let _guard = m_clone.lock().unwrap();
        panic!("持有锁的线程崩溃了！");
    });

    let _ = handle.join();

    // 锁已中毒
    match mutex.lock() {
        Ok(_) => panic!("锁应为中毒状态"),
        Err(poisoned) => {
            // 通过 into_inner 恢复数据
            let mut data = poisoned.into_inner();
            *data = 42;
            assert_eq!(*data, 42);
        }
    }
}

#[test]
/// 测试: Mutex 和 RwLock 综合测试 (MutexGuard 作用域 + 读/写锁共存)
fn test_mutex_and_rwlock() {
    // 测试 MutexGuard 在作用域结束时自动释放
    let m = Mutex::new(0);
    {
        let mut guard = m.lock().unwrap();
        *guard = 100;
    } // guard 在此 drop，锁释放
    assert_eq!(*m.lock().unwrap(), 100);

    // 测试 RwLock 读锁与写锁的互斥关系
    let rw = RwLock::new(10);
    {
        let r1 = rw.read().unwrap();
        let r2 = rw.read().unwrap();  // 多个读锁可共存
        assert_eq!(*r1, 10);
        assert_eq!(*r2, 10);
        drop(r1);
        drop(r2);
    }

    // 写锁修改
    {
        let mut w = rw.write().unwrap();
        *w = 20;
    }

    // 验证修改结果
    assert_eq!(*rw.read().unwrap(), 20);
}

// ============================================================================
// 原子操作
// ============================================================================

#[test]
/// 测试: 原子操作 (AtomicBool/AtomicI32/Ordering)
fn test_atomic_operations() {
    // 语法: std::sync::atomic 提供无锁原子操作
    //
    // 常用原子类型:
    //   - AtomicBool / AtomicI32 / AtomicUsize
    //   - AtomicPtr<T>
    //
    // Ordering:
    //   - Relaxed: 无顺序保证(性能最好)
    //   - Acquire: 读操作，后续操作不能乱序到此之前
    //   - Release: 写操作，之前操作不能乱序到此之后
    //   - AcqRel: 读-改-写
    //   - SeqCst: 全局顺序一致性(最安全)
    //
    // 避坑:
    //   - Ordering 选择错误可能引入 bug
    //   - 不熟悉时用 SeqCst
    //   - 原子操作不是免费的(尤其 SeqCst)
    //

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = Arc::clone(&flag);

    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        flag_clone.store(true, Ordering::SeqCst);
    });

    // 自旋等待 flag 设为 true
    while !flag.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(1));
    }
    assert!(flag.load(Ordering::SeqCst));
    handle.join().unwrap();
}

#[test]
/// 测试: 原子类型和内存排序 (Release-Acquire 配对)
fn test_atomic_types() {
    // 测试 Release-Acquire 配对确保数据可见性
    let flag = Arc::new(AtomicBool::new(false));
    let value = Arc::new(AtomicI32::new(0));

    let flag2 = Arc::clone(&flag);
    let value2 = Arc::clone(&value);

    let handle = thread::spawn(move || {
        // 写入数据（Relaxed 即可，配合 Release 保证可见性）
        value2.store(42, Ordering::Relaxed);
        // Release 确保之前的所有写入对 Acquire 侧可见
        flag2.store(true, Ordering::Release);
    });

    // 等待 flag 被设置
    while !flag.load(Ordering::Acquire) {
        thread::yield_now();
    }
    // Acquire 确保能看到 Release 前的 value 写入
    assert_eq!(value.load(Ordering::Relaxed), 42);
    handle.join().unwrap();
}

#[test]
/// 测试: 原子 fetch_add/fetch_sub (无锁计数器)
fn test_atomic_fetch_ops() {
    // 语法: fetch_add/fetch_sub 原子地执行加减操作
    //
    // 使用多线程并发递增原子计数器

    let counter = Arc::new(AtomicI32::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                c.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::Relaxed), 1000);
}

// ============================================================================
// 同步原语
// ============================================================================

#[test]
/// 测试: 屏障 Barrier (多线程同步点)
fn test_barrier() {
    // 语法: Barrier::new(n) 创建一个屏障，等 n 个线程到齐后同时放行
    //
    // 操作:
    //   - wait()            阻塞直到所有线程到达，返回 BarrierWaitResult
    //   - is_leader()       返回 true 表示当前线程是"领头线程"
    //
    // 适用场景:
    //   - 多线程需要在同一时刻开始执行
    //   - 多阶段计算，每阶段末尾同步
    //
    // 避坑:
    //   - 线程数必须与 Barrier 参数匹配，否则 deadlock
    //   - Barrier 可复用（所有线程通过后自动重置）
    //

    const N: usize = 4;
    let barrier = Arc::new(Barrier::new(N));
    let mut handles = vec![];

    for id in 0..N {
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            // 第一阶段
            let result = id * 2;
            // 等待所有线程完成第一阶段
            let leader_info = b.wait();
            let is_leader = leader_info.is_leader();
            (result, is_leader, id)
        }));
    }

    let mut results = vec![];
    for h in handles {
        let (res, _is_leader, id) = h.join().unwrap();
        results.push((id, res));
    }
    results.sort_by_key(|(id, _)| *id);
    let values: Vec<_> = results.into_iter().map(|(_, v)| v).collect();
    assert_eq!(values, vec![0, 2, 4, 6]);
}

#[test]
/// 测试: 条件变量 Condvar (等待通知)
fn test_condvar() {
    // 语法: Condvar 配合 Mutex 实现"等待-通知"模式
    //
    // 操作:
    //   - cvar.wait(mutex_guard)  原子性释放锁并等待通知
    //   - cvar.notify_one()        唤醒一个等待线程
    //   - cvar.notify_all()        唤醒所有等待线程
    //
    // 避坑:
    //   - 必须用 while 而非 if 检查条件(防范虚假唤醒)
    //   - wait 返回的 guard 可能不是原来的（被重新获取）
    //   - notify 在锁内锁外均可调用
    //

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);

    let handle = thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        let mut ready = lock.lock().unwrap();
        *ready = true;
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut ready = lock.lock().unwrap();
    // 使用 while 防范虚假唤醒
    while !*ready {
        ready = cvar.wait(ready).unwrap();
    }
    assert!(*ready);

    handle.join().unwrap();
}

#[test]
/// 测试: Condvar 多消费者唤醒
fn test_condvar_notify_all() {
    // 语法: notify_all() 唤醒所有等待线程

    let pair = Arc::new((Mutex::new(0u32), Condvar::new()));
    let mut handles = vec![];
    let num_waiters = 3;

    for _ in 0..num_waiters {
        let p = Arc::clone(&pair);
        handles.push(thread::spawn(move || {
            let (lock, cvar) = &*p;
            let mut val = lock.lock().unwrap();
            while *val == 0 {
                val = cvar.wait(val).unwrap();
            }
            assert!(*val > 0);
        }));
    }

    // 短暂等待确保所有等待线程就绪
    thread::sleep(Duration::from_millis(20));

    {
        let (lock, cvar) = &*pair;
        let mut val = lock.lock().unwrap();
        *val = 42;
        cvar.notify_all(); // 唤醒所有等待者
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
/// 测试: 一次性初始化 (Once)
fn test_once_initialization() {
    // 语法: Once::call_once() 保证闭包只执行一次（线程安全）
    //
    // 适用场景:
    //   - 全局初始化
    //   - 惰性单例
    //

    static INIT: Once = Once::new();
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let c = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            // 多个线程调用 call_once，但闭包只执行一次
            INIT.call_once(|| {
                let mut count = c.lock().unwrap();
                *count += 1;
            });
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 1, "call_once 应只执行一次");
}

#[test]
/// 测试: OnceLock 一次性写入锁
fn test_once_lock() {
    // 语法: OnceLock<T> 是"只写一次"的线程安全容器 (Rust 1.70+)
    //
    // 操作:
    //   - OnceLock::new()              创建空容器
    //   - get_or_init(|| val)          获取或初始化（惰性）
    //   - set(val)                     设置值（只成功一次）
    //   - get()                        获取已设置的值
    //
    // 避坑:
    //   - set 只能成功一次，第二次返回 Err
    //   - get_or_init 不会阻塞（不同于 Mutex）
    //

    let lock = Arc::new(OnceLock::new());
    let mut handles = vec![];

    for _ in 0..5 {
        let l = Arc::clone(&lock);
        handles.push(thread::spawn(move || {
            let val = l.get_or_init(|| {
                String::from("initialized_once")
            });
            assert_eq!(val, "initialized_once");
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(lock.get(), Some(&String::from("initialized_once")));

    // 测试 set 只能成功一次
    let cell = OnceLock::new();
    assert!(cell.set(42).is_ok());
    assert!(cell.set(99).is_err()); // 第二次 set 失败
    assert_eq!(cell.get(), Some(&42));
}

// ============================================================================
// Send 与 Sync
// ============================================================================

#[test]
/// 测试: Send 与 Sync trait 静态检查
fn test_send_sync() {
    // 语法: Send 允许跨线程转移所有权, Sync 允许跨线程共享引用
    //
    // 规则:
    //   - 几乎所有类型是 Send
    //   - Rc<T> 不是 Send (引用计数非原子)
    //   - Cell/RefCell 不是 Sync (内部可变性非线程安全)
    //   - Mutex<T> 是 Sync (如果 T 是 Send)
    //   - Arc<T> 是 Send + Sync (如果 T 是 Send + Sync)
    //
    // 避坑:
    //   - 实现 Send/Sync 是 unsafe
    //   - 编译器自动为组合类型推导 Send/Sync
    //

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    is_send::<String>();
    is_sync::<String>();
    is_sync::<Mutex<i32>>();

    is_send::<Arc<i32>>();
    is_sync::<Arc<i32>>();

    // Rc 不是 Send/Sync
    // is_send::<Rc<i32>>();  // 编译错误
    // is_sync::<Rc<i32>>();  // 编译错误
}

#[test]
/// 测试: Send/Sync 自动推导规则深入
fn test_send_sync_traits() {
    // Send/Sync 是自动 trait:
    //   - 一个类型是 Send 当且仅当其所有字段都是 Send
    //   - 一个类型是 Sync 当且仅当其所有字段都是 Sync
    //
    // 关键区分:
    //   - Rc 不是 Send：跨线程时 clone/drop 操作非原子
    //   - RefCell 不是 Sync：运行时借用检查不线程安全
    //   - Arc 是 Send+Sync：原子引用计数保证
    //   - Mutex<T:Send> 是 Sync：内部同步保护

    struct AllSend {
        name: String,       // 是 Send + Sync
        value: i32,         // 是 Send + Sync
        arc: Arc<i32>,      // 是 Send + Sync
    }
    // AllSend 自动实现 Send 和 Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<AllSend>();
    assert_sync::<AllSend>();

    // 注意：如果结构体包含 *mut u8 或 Rc，则自动失去 Send/Sync
    // struct WithPtr { ptr: *const u8 }  // 不是 Send
}

// ============================================================================
// 线程局部存储
// ============================================================================

#[test]
/// 测试: 线程局部变量 (thread_local!)
fn test_thread_local() {
    // 语法: thread_local! 宏定义线程局部变量
    //
    // 特性:
    //   - 每个线程有独立副本
    //   - 初始化惰性执行
    //   - 线程结束时自动 drop
    //

    thread_local! {
        static COUNTER: std::cell::Cell<u32> = std::cell::Cell::new(0);
    }

    COUNTER.with(|c| {
        c.set(42);
        assert_eq!(c.get(), 42);
    });

    // 另一个线程的副本不受影响
    let handle = thread::spawn(|| {
        COUNTER.with(|c| {
            assert_eq!(c.get(), 0); // 独立的初始值
        });
    });
    handle.join().unwrap();
}

// ============================================================================
// 并发模式
// ============================================================================

#[test]
/// 测试: 线程池模式 (通过 channel 分发任务)
fn test_thread_pool_pattern() {
    // 语法: 使用 channel + 多个工作线程模拟线程池
    //
    // 模式:
    //   - 创建多个工作线程
    //   - 通过 channel 分发任务
    //   - 主线程收集结果
    //

    let (tx, rx) = mpsc::channel::<i32>();
    let rx = Arc::new(Mutex::new(rx));
    let mut handles = vec![];

    for _ in 0..3 {
        let rx = Arc::clone(&rx);
        let handle = thread::spawn(move || {
            loop {
                let rx = rx.lock().unwrap();
                match rx.recv() {
                    Ok(task) => {
                        // 处理任务
                        let _result = task * 2;
                    }
                    Err(_) => break,
                }
            }
        });
        handles.push(handle);
    }

    // 发送任务
    for i in 0..5 {
        tx.send(i).unwrap();
    }
    drop(tx); // 关闭通道，通知工作线程退出

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
/// 测试: 生产者-消费者模式 (含同步屏障确保任务全部完成后验证)
fn test_producer_consumer() {
    // 模式: 多生产者 -> channel -> 单消费者，用 Barrier 同步
    //
    // 架构:
    //   - 多个生产者线程生成任务
    //   - 一个消费者线程处理并收集结果
    //   - Barrier 确保所有任务完成后再验证
    //

    let (tx, rx) = mpsc::channel::<i32>();
    let results = Arc::new(Mutex::new(Vec::<i32>::new()));
    let results_for_consumer = Arc::clone(&results);
    let barrier = Arc::new(Barrier::new(5)); // 4 个生产者 + 1 个消费者

    // 消费者线程
    let consumer_barrier = Arc::clone(&barrier);
    let consumer = thread::spawn(move || {
        for task in rx {
            results_for_consumer.lock().unwrap().push(task * 2);
        }
        consumer_barrier.wait(); // 消费者完成任务
    });

    // 多个生产者
    let mut producers = vec![];
    for thread_id in 0..4 {
        let tx = tx.clone();
        let b = Arc::clone(&barrier);
        producers.push(thread::spawn(move || {
            for i in 0..5 {
                tx.send(thread_id * 10 + i).unwrap();
            }
            drop(tx); // 每个生产者完成时 drop 自己的 tx
            b.wait(); // 等待其他线程
        }));
    }
    drop(tx); // 原始发送端已在生产者中克隆，可安全释放

    // 等待所有生产者线程完成
    for p in producers {
        p.join().unwrap();
    }
    consumer.join().unwrap();

    let collected = results.lock().unwrap();
    assert_eq!(collected.len(), 20);
    // 验证所有值为偶数(每个值 * 2)
    assert!(collected.iter().all(|&v| v % 2 == 0));
}

#[test]
/// 测试: 生产者-消费者模式 (完整示例：多生产者 -> 单消费者 + 结果收集)
fn test_producer_consumer_full() {
    // 正确实现：使用 channel 的生产者-消费者模式
    //
    // 架构:
    //   生产者1 ─┐
    //   生产者2 ─┤──> channel ──> 消费者 ──> 结果收集
    //   生产者3 ─┘
    //

    let (tx, rx) = mpsc::channel::<i32>();
    let results = Arc::new(Mutex::new(Vec::<i32>::new()));
    let results_for_consumer = Arc::clone(&results);

    // 消费者线程
    let consumer = thread::spawn(move || {
        for task in rx {
            // 模拟处理
            let processed = task * task;
            results_for_consumer.lock().unwrap().push(processed);
        }
    });

    // 多个生产者线程
    let mut producers = vec![];
    for thread_id in 0..4 {
        let tx = tx.clone();
        producers.push(thread::spawn(move || {
            for i in 0..5 {
                let value = thread_id * 10 + i;
                tx.send(value).unwrap();
            }
        }));
    }

    // 等待所有生产者完成
    for p in producers {
        p.join().unwrap();
    }
    // 关闭通道，通知消费者结束
    drop(tx);

    // 等待消费者完成
    consumer.join().unwrap();

    let collected = results.lock().unwrap();
    assert_eq!(collected.len(), 20); // 4 个生产者 * 5 个任务
    // 验证所有值都被处理（平方）
    for &val in collected.iter() {
        let sqrt = (val as f64).sqrt() as i32;
        assert_eq!(sqrt * sqrt, val, "值 {} 应为平方数", val);
    }
}
