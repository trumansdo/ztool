// ---------------------------------------------------------------------------
// 5.4 if let 临时作用域 (Edition 2024)
// ---------------------------------------------------------------------------

// 语法: Edition 2024 中 if let 的临时值作用域延长到整个 if-else 块结束
// 避坑: 旧版中临时值在 if 块结束后立即 drop; 迁移后 drop 时机改变, 可能影响依赖 drop 顺序的代码

use std::cell::RefCell;

struct Droppable<'a>(&'a RefCell<i32>);
impl Drop for Droppable<'_> {
    fn drop(&mut self) {
        *self.0.borrow_mut() += 1;
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
