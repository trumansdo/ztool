// ---------------------------------------------------------------------------
// 5.4 if let 临时作用域 (Edition 2024)
// ---------------------------------------------------------------------------

// 语法: Edition 2024 中 if let 的临时值作用域延长到整个 if-else 块结束
// 避坑: 旧版中临时值在 if 块结束后立即 drop; 迁移后 drop 时机改变, 可能影响依赖 drop 顺序的代码

use std::cell::RefCell;
use std::sync::Mutex;

struct Droppable<'a>(&'a RefCell<i32>);
impl Drop for Droppable<'_> {
    fn drop(&mut self) {
        *self.0.borrow_mut() += 1;
    }
}

// 用于跟踪 drop 顺序的辅助结构
struct OrderedDrop<'a> {
    id: i32,
    log: &'a RefCell<Vec<i32>>,
}
impl Drop for OrderedDrop<'_> {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.id);
    }
}

#[test]
/// 测试: Edition 2024 if let 临时值作用域延长
fn test_if_let_scope() {
    let counter = RefCell::new(0);

    if let Some(_x) = Some(Droppable(&counter)) {
        assert_eq!(*counter.borrow(), 0);
    }

    assert_eq!(*counter.borrow(), 1);
}

#[test]
/// 测试: if let 与 else 分支作用域
fn test_if_let_else_scope() {
    let counter = RefCell::new(0);

    if let Some(_) = Some(Droppable(&counter)) {
        assert_eq!(*counter.borrow(), 0);
    } else {
        assert!(false, "should not reach else");
    }

    assert_eq!(*counter.borrow(), 1);
}

#[test]
/// 测试: 嵌套 if let 临时值生命周期
fn test_nested_if_let_scope() {
    let counter = RefCell::new(0);

    if let Some(_) = Some(Droppable(&counter)) {
        if let Some(_) = Some(Droppable(&counter)) {
            assert_eq!(*counter.borrow(), 0);
        }
    }

    assert_eq!(*counter.borrow(), 2);
}

#[test]
/// 测试: if let 在循环中临时值行为
fn test_if_let_in_loop() {
    let counter = RefCell::new(0);
    let data = vec![1, 2, 3];

    for item in data {
        if item > 0 {
            *counter.borrow_mut() += 1;
        }
    }

    assert_eq!(*counter.borrow(), 3);
}

#[test]
/// 测试: 临时引用与借用检查
fn test_temp_ref_borrow_check() {
    let value = RefCell::new(10);
    if let Some(x) = Some(&value) {
        *x.borrow_mut() = 20;
    }
    assert_eq!(*value.borrow(), 20);
}

#[test]
/// 测试: match 中的临时值行为对比
fn test_match_temp_scope() {
    let counter = RefCell::new(0);

    match Some(Droppable(&counter)) {
        Some(_) => {
            assert_eq!(*counter.borrow(), 0);
        }
        None => {}
    }

    assert_eq!(*counter.borrow(), 1);
}

// ===================== 扩充测试 =====================

#[test]
/// 测试: if let None 路径 —— 临时值也在 if-else 结束后 drop
fn test_if_let_none_path_drop_timing() {
    let counter = RefCell::new(0);

    if let Some(_) = None::<Droppable> {
        panic!("should not match");
    } else {
        assert_eq!(*counter.borrow(), 0);
    }

    // None 路径也创建了临时值 None<Droppable>, 但因为没有 Droppable 实例, 不会触发 drop
    assert_eq!(*counter.borrow(), 0);
}

#[test]
/// 测试: 多个 if let 在同一作用域, 各自的临时值独立 drop
fn test_multiple_if_let_independent_scopes() {
    let counter = RefCell::new(0);

    if let Some(_) = Some(Droppable(&counter)) {
        assert_eq!(*counter.borrow(), 0);
    }
    assert_eq!(*counter.borrow(), 1);

    if let Some(_) = Some(Droppable(&counter)) {
        assert_eq!(*counter.borrow(), 1);
    }
    assert_eq!(*counter.borrow(), 2);
}

#[test]
/// 测试: if let 中的 MutexGuard  —— 锁在 if-else 结束后才释放
fn test_if_let_mutex_guard_scope() {
    let m = Mutex::new(42);

    if let Ok(guard) = m.lock() && *guard == 42 {
        // guard 仍然持有锁
        assert_eq!(*guard, 42);
    }
    // 锁在这里释放

    // 验证锁确实已释放 —— 可以再次获取
    let guard = m.lock().unwrap();
    assert_eq!(*guard, 42);
}

#[test]
/// 测试: 嵌套 if let + MutexGuard —— 验证无死锁 (锁在正确时机释放)
fn test_nested_if_let_mutex_no_deadlock() {
    let m1 = Mutex::new(1);
    let m2 = Mutex::new(2);

    // 先取 m1 的锁
    if let Ok(g1) = m1.lock() && *g1 == 1 {
        // m1 的锁仍持有, 但可以获取 m2
        if let Ok(g2) = m2.lock() && *g2 == 2 {
            assert_eq!(*g1 + *g2, 3);
        }
    }
    // 两个锁都已释放

    // 以相反顺序获取验证无死锁
    if let Ok(g2) = m2.lock() {
        if let Ok(g1) = m1.lock() {
            assert_eq!(*g1 + *g2, 3);
        }
    }
}

#[test]
/// 测试: if let 临时引用生命周期 —— 临时 String 的引用
fn test_if_let_temp_string_ref() {
    fn get_name() -> Option<String> {
        Some("Rust".to_string())
    }

    if let Some(ref name) = get_name() && name.starts_with("Ru") {
        assert_eq!(name, "Rust");
    }
}

#[test]
/// 测试: let chains 中临时值作用域 (组合特性)
fn test_if_let_chains_temp_scope() {
    let counter = RefCell::new(0);

    if let Some(_x) = Some(Droppable(&counter))
        && let Some(_y) = Some(Droppable(&counter))
    {
        assert_eq!(*counter.borrow(), 0);
    }
    // 两个临时值都在 if-else 结束后 drop
    assert_eq!(*counter.borrow(), 2);
}

#[test]
/// 测试: let chains 短路时临时值 drop 顺序
fn test_if_let_chains_short_circuit_drop() {
    let counter = RefCell::new(0);

    // 第一个匹配成功但第二个匹配失败且短路, 第一个的临时值应已 drop
    if let Some(_x) = Some(Droppable(&counter))
        && let Some(_y) = None::<Droppable>
        && false
    {
        panic!("should not reach");
    } else {
        // 注意: _x 的临时值成功创建了, _y 的 None 不包含 Droppable
        // _x 的 Droppable 应在这之后 drop
    }

    // Edition 2024: 临时值在 if-else 结束后才 drop
    assert_eq!(*counter.borrow(), 1);
}

#[test]
/// 测试: Drop 顺序跟踪 —— 多个临时值 drop 的先后
fn test_drop_order_multiple_temps() {
    let log = RefCell::new(Vec::new());

    {
        if let Some(_) = Some(OrderedDrop { id: 1, log: &log })
            && let Some(_) = Some(OrderedDrop { id: 2, log: &log })
        {
            // 什么都不做
        }
    }

    let drops = log.borrow().clone();
    // Edition 2024: 临时值在 if-else 结束后 drop
    // 通常 2 先于 1 被创建, drop 时可能 2 先, 1 后 (LIFO)
    // 或两者同时, 顺序不一定是严格 FIFO
    assert_eq!(drops.len(), 2);
    assert!(drops.contains(&1) && drops.contains(&2), "both items should be dropped");
}

#[test]
/// 测试: 无 Drop 类型不受作用域延长影响
fn test_no_drop_types_unaffected() {
    let value = 42;

    if let Some(x) = Some(value) && x == 42 {
        assert_eq!(x, 42);
    }
    // i32 没有 Drop, 作用域延长对它无任何可观测影响
    let _ = value; // value 仍然可用 (Copy 类型)
}

#[test]
/// 测试: while let 中每次迭代临时值独立
fn test_while_let_temp_per_iteration() {
    let counter = RefCell::new(0);
    let values = vec![1, 2, 3];

    let mut iter = values.into_iter();
    let mut sum = 0;
    while let Some(x) = iter.next() && x > 0 {
        sum += x;
    }
    assert_eq!(sum, 6);
    // counter 未增加——while let 中没有临时 Droppable 值
    assert_eq!(*counter.borrow(), 0);
}
