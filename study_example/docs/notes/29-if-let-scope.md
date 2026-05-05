# if let 临时作用域 (Edition 2024)

## 目录
1. [Edition 2021 vs 2024 临时值作用域变化](#edition-2021-vs-2024-临时值作用域变化)
2. [旧版行为：临时值存活至整个表达式](#旧版行为临时值存活至整个表达式)
3. [新版行为：临时值在分支结束时析构](#新版行为临时值在分支结束时析构)
4. [MutexGuard 锁持有时间缩短](#mutexguard-锁持有时间缩短)
5. [match 对比：行为不变](#match-对比行为不变)
6. [Drop 顺序与析构副作用](#drop-顺序与析构副作用)
7. [借用检查影响](#借用检查影响)
8. [迁移策略](#迁移策略)
9. [避坑指南](#避坑指南)

---

## Edition 2021 vs 2024 临时值作用域变化

Edition 2024 引入了对 `if let` 表达式中临时值作用域的重大调整。在旧版（2021 及之前）中，`if let` 表达式求值时创建的临时值会存活到**整个 if-else 链结束**才析构；而在新版中，临时值会在**其所属分支结束**时立即析构。

> 一条锁链的长短，决定了你的程序是跑起来还是卡在那里。

核心变化示意图：

```rust
// 假设有以下类型，其 Drop 实现会打印信息
struct Tracker(&'static str);
impl Drop for Tracker {
    fn drop(&mut self) {
        println!("释放: {}", self.0);
    }
}

fn get_tracker() -> Option<Tracker> {
    Some(Tracker("临时追踪器"))
}
```

从用户角度看，最大的受益者是 `MutexGuard`、`RefMut` 等 RAII 守卫类型——它们的持有时间显著缩短，降低了死锁风险。

---

## 旧版行为：临时值存活至整个表达式

在 Edition 2021 中，`if let` 表达式内的临时值在整个 `if let ... else ...` 表达式的持续期间都保持存活，直到表达式完全求值完毕。

> 旧版的临时值像一个迟迟不肯退场的演员，即使他的戏份早就结束了。

旧版行为演示：

```rust
// Edition 2021 行为
struct LoudDrop(&'static str);
impl Drop for LoudDrop {
    fn drop(&mut self) {
        println!("Dropped: {}", self.0);
    }
}

fn produce() -> Option<LoudDrop> {
    Some(LoudDrop("guard"))
}

fn old_edition_demo() {
    // Edition 2021: guard 在整个 if-else 链结束后才 drop
    if let Some(g) = produce() {
        println!("进入 if 分支，持有: {}", g.0);
    } else {
        println!("进入 else 分支");
    }
    // 此处才 drop "guard"
    println!("if-else 表达式结束");
}
// 输出 (Edition 2021):
//   进入 if 分支，持有: guard
//   if-else 表达式结束
//   Dropped: guard
```

对锁的影响——锁会一直持有到整个 if-else 链结束：

```rust
use std::sync::Mutex;

fn old_lock_behavior() {
    let data = Mutex::new(42);

    if let Some(val) = data.lock().ok() {
        // Edition 2021: 锁在此仍被持有
        println!("读取值: {}", *val);
        // 执行其它耗时操作——锁一直被占用
        heavy_work();
    }
    // Edition 2021: 锁在此才释放！

    // 如果 heavy_work() 内部尝试获取同一把锁 -> 死锁！
}

fn heavy_work() {}
```

---

## 新版行为：临时值在分支结束时析构

Edition 2024 将临时值的析构点提前到**每个分支的结尾**。即 if 块结束时析构 if 块的临时值，else 块结束时析构 else 块的临时值。

> 及时放手，是对资源最大的尊重——Edition 2024 教会了 if let 这个道理。

新版行为演示：

```rust
struct LoudDrop(&'static str);
impl Drop for LoudDrop {
    fn drop(&mut self) {
        println!("Dropped: {}", self.0);
    }
}

fn produce() -> Option<LoudDrop> {
    Some(LoudDrop("guard"))
}

fn new_edition_demo() {
    // Edition 2024: guard 在 if 块结束时就 drop
    if let Some(g) = produce() {
        println!("进入 if 分支，持有: {}", g.0);
        // 此处 drop "guard" —— 在 if 块结束前
    } else {
        println!("进入 else 分支");
    }
    println!("if-else 表达式结束");
}
// 输出 (Edition 2024):
//   进入 if 分支，持有: guard
//   Dropped: guard         <-- 注意：提前到这里了
//   if-else 表达式结束
```

--- ---

## MutexGuard 锁持有时间缩短

锁持有时间的缩短是 Edition 2024 这一变更最显著的实用收益。

> 锁不是用来持有的，是用来短暂通过后立即归还的——Edition 2024 让这个理念落地。

Edition 2024 下锁自动提前释放：

```rust
use std::sync::Mutex;

fn safe_pattern() {
    let shared = Mutex::new(Vec::<i32>::new());

    if let Some(mut guard) = shared.lock().ok() {
        guard.push(1);
        println!("插入数据，长度: {}", guard.len());
        // Edition 2024: guard 在此析构，锁释放
    }

    // 此处可以安全地再次获取锁
    if let Some(guard) = shared.lock().ok() {
        println!("第二次读取，长度: {}", guard.len());
    }
}
```

在旧版中可能死锁的场景：

```rust
use std::sync::Mutex;

fn old_edition_deadlock_prone() {
    let m = Mutex::new(());

    if let Some(_g1) = m.lock().ok() {
        // Edition 2021: _g1 仍存活
        // 如果此处递归或间接尝试获取 m —— 死锁
        // Edition 2024: _g1 在 if 块结尾已析构，安全
    }

    // Edition 2024: 可安全地再次加锁
    let _g2 = m.lock().unwrap();
}
```

---

## match 对比：行为不变

**重要**：`match` 表达式的临时值作用域**不受 Edition 2024 影响**，保持原有行为不变。临时值在 match 的整个臂（arm）求值期间存活。

> if let 在变，match 岿然不动——二者本就不是同一场戏的演员。

match 的临时值行为（各 Edition 一致）：

```rust
struct Tracked(&'static str);
impl Drop for Tracked {
    fn drop(&mut self) { println!("Drop: {}", self.0); }
}

fn match_temp_scope() {
    let val = Some(Tracked("A"));

    match val {
        Some(ref t) => {
            println!("匹配到: {}", t.0);
            // Tracked("A") 在此仍未析构
        }
        None => {}
    }
    // Tracked("A") 在此析构（match 表达式结束后）
}

// 原因：match 的临时值受外层表达式作用域约束，
// 而不是单个分支，这和 if let 的 Edition 2024 行为形成对比。
```

行为对比表：

| 表达式 | 临时值析构时机 (Edition 2021) | 临时值析构时机 (Edition 2024) |
|--------|------------------------------|-------------------------------|
| `if let`  | 整个 if-else 结束后 | 各自分支结束后 |
| `match`   | 整个 match 结束后 | 整个 match 结束后（不变） |
| `while let` | 每次迭代结束后 | 每次迭代结束后（不变） |

---

## Drop 顺序与析构副作用

临时值提前析构意味着副作用（如文件 flush、网络关闭、引用计数递减）发生得更早。必须注意依赖旧析构时序的代码。

> Drop 不是打扫卫生，是关门——关得太早或太晚都会出问题。

析构时序变化示例：

```rust
use std::cell::RefCell;

struct Gate {
    closed: RefCell<bool>,
}
impl Gate {
    fn new() -> Self { Gate { closed: RefCell::new(false) } }
    fn is_closed(&self) -> bool { *self.closed.borrow() }
}

struct Guard<'a> {
    gate: &'a Gate,
}
impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        *self.gate.closed.borrow_mut() = true;
        println!("门已关闭");
    }
}

fn drop_order_demo() {
    let gate = Gate::new();

    // Edition 2024: guard 在 if 块结束时析构
    if let Some(_g) = Some(Guard { gate: &gate }) {
        println!("持有守卫");
        // Edition 2021: gate.is_closed() == false
        // Edition 2024: gate.is_closed() == false (还没出 if 块)
    }
    // Edition 2021: gate.is_closed() == false
    // Edition 2024: gate.is_closed() == true （Guard 已析构）
}
```

---

## 借用检查影响

临时值提前析构对借用检查器的影响是**积极的**：锁或可变引用提前释放后，可以在后续代码中重新获取。

> 被锁住的资源就像被借用的大脑——早还早轻松。

借用检查改善示例：

```rust
use std::cell::RefCell;

fn borrow_checker_benefit() {
    let data = RefCell::new(String::from("hello"));

    // Edition 2024: borrow_mut 的临时守卫在 if 块结束时析构
    if let Some(mut r) = data.try_borrow_mut().ok() {
        r.push_str(" world");
        // r 在此析构，可变借用结束
    }

    // 立即可获取不可变借用（在 Edition 2021 中可能编译失败）
    if let Some(r) = data.try_borrow().ok() {
        println!("读取: {}", *r);
    }
}
```

同一作用域内连续借用的安全性：

```rust
use std::sync::Mutex;

fn multiple_locks() {
    let a = Mutex::new(1);
    let b = Mutex::new(2);

    // Edition 2024: 可以自然地在不同 if let 中轮流获取两把锁
    if let Some(g) = a.lock().ok() {
        println!("A = {}", *g);
    } // A 锁释放

    if let Some(g) = b.lock().ok() {
        println!("B = {}", *g);
    } // B 锁释放
}
```

---

## 迁移策略

从 Edition 2021 迁移到 2024 时，针对临时作用域变化需要注意：

> 迁移不是搬家，而是一次全面的健康体检——查明所有依赖旧行为的代码。

检查清单：

1. **审查所有 `if let` 中创建的 RAII 守卫**（MutexGuard, RefMut, File 等）
2. **检查是否有代码依赖旧的析构时序**（如在 if-else 链结束后再操作被守护的资源）
3. **对有意延迟析构的场景显式绑定变量**：

```rust
// Edition 2024 下如需旧行为：显式将守卫提升到外部作用域
fn keep_guard_longer() {
    let guard = some_lock().ok(); // 显式 let 绑定
    if let Some(ref g) = guard {
        // guard 将存活到当前函数作用域结束
        println!("{}", *g);
    }
    // guard 在此才析构
}

fn some_lock<T>() -> Result<T, ()> { todo!() }
```

4. **利用 cargo fix --edition 自动迁移**

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 在 if 块外访问已被析构的守卫 | Edition 2024 中 if 块结束时守卫已析构 | 将需要跨块存活的守卫提升为显式 `let` 绑定 |
| 依赖旧析构时序的副作用 | 原来在 if-else 结束后才发生 flush/close/dec-ref | 重构代码使得副作用时机明确化，或用显式作用域块 |
| 混淆 if let 和 match 行为 | 仅 if let 的临时作用域在 Edition 2024 中改变，match 不变 | 如需控制精确析构点，统一使用显式 `let` 绑定或块作用域 |
| 在 else if let 中依赖前一分支的临时值 | 旧版中可能意外存活，新版中不再存活 | 每个分支独立管理资源，不跨分支依赖临时值 |
| mut 守卫在析构前发生 panic | 守卫析构顺序可能改变，panic unwind 与 drop 交织更复杂 | 在守卫存活区间内避免可能 panic 的操作 |
| 多重 if let 嵌套中的析构点混淆 | 内层和外层的析构点不同 | 始终将每个 if let 视为独立的资源边界 |

---

> **详见测试**: `tests/rust_features/29_if_let_temporary_scope.rs`
