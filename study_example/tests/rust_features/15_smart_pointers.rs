// ---------------------------------------------------------------------------
// 3.3 智能指针与内部可变性
// ---------------------------------------------------------------------------

#[test]
/// 测试: Box 智能指针 (堆分配/独占所有权)
fn test_box() {
    // 语法: Box<T> 将数据分配到堆上, 编译期确定大小, 独占所有权
    // 避坑: Box 解引用有运行时开销(间接访问); 递归类型必须用 Box 打破无限大小
    let boxed = Box::new(42);
    assert_eq!(*boxed, 42);
}

#[test]
/// 测试: Box 递归类型 (打破无限大小)
fn test_box_recursive_types() {
    // 语法: 递归枚举必须用 Box 包装递归字段, 否则编译器无法确定大小
    // 避坑: 不用 Box 会导致 "recursive type has infinite size" 编译错误
    enum List<T> {
        Cons(T, Box<List<T>>),
        Nil,
    }

    use List::{Cons, Nil};
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

    // 遍历链表
    let mut cur = &list;
    let mut values = Vec::new();
    loop {
        match cur {
            Cons(val, next) => {
                values.push(*val);
                cur = next;
            }
            Nil => break,
        }
    }
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
/// 测试: Box::leak 永久内存泄漏
fn test_box_leak() {
    // 语法: Box::leak(boxed) 释放 Box 的所有权, 返回 &'static mut T
    // 避坑: 泄漏的内存永远不会被释放; 仅在需要 'static 生命周期时使用
    let s = Box::leak(Box::new(String::from("hello")));
    s.push_str(" world");
    assert_eq!(s, "hello world");
    // s 永远不会被 drop
}

#[test]
/// 测试: Box 与 trait 对象
fn test_box_dyn_trait() {
    // 语法: Box<dyn Trait> 在堆上存储 trait 对象, 支持不同大小的类型
    // 避坑: dyn Trait 是胖指针(数据指针+虚表), Box 本身是单指针
    trait Animal {
        fn sound(&self) -> &str;
    }

    struct Dog;
    impl Animal for Dog {
        fn sound(&self) -> &str {
            "woof"
        }
    }

    struct Cat;
    impl Animal for Cat {
        fn sound(&self) -> &str {
            "meow"
        }
    }

    let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];
    let sounds: Vec<&str> = animals
        .iter()
        .map(|a| a.sound())
        .collect();
    assert_eq!(sounds, vec!["woof", "meow"]);
}

#[test]
/// 测试: Rc 引用计数共享所有权 (单线程)
fn test_rc() {
    // 语法: Rc<T> 引用计数共享所有权, 仅单线程; clone 增加计数
    // 避坑: Rc 不能跨线程使用(编译期阻止); 循环引用导致内存泄漏, 需用 Weak 打破
    use std::rc::Rc;
    let rc1 = Rc::new(42);
    let _rc2 = Rc::clone(&rc1);
    assert_eq!(Rc::strong_count(&rc1), 2);
}

#[test]
/// 测试: Rc + Weak 打破循环引用
fn test_rc_weak() {
    // 语法: Rc::downgrade(&rc) 创建弱引用, 不增加强计数
    // 避坑: Weak::upgrade() 返回 Option<Rc<T>>, 强引用为 0 时返回 None
    //       循环引用是 Rc 内存泄漏的主要原因
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    struct Node {
        value: i32,
        children: RefCell<Vec<Rc<Node>>>,
        parent: RefCell<Weak<Node>>,
    }

    let root = Rc::new(Node {
        value: 1,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Weak::new()),
    });

    let child = Rc::new(Node {
        value: 2,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Rc::downgrade(&root)),
    });

    root.children
        .borrow_mut()
        .push(Rc::clone(&child));

    // 弱引用可以升级
    let parent = child.parent.borrow().upgrade();
    assert!(parent.is_some());
    assert_eq!(parent.unwrap().value, 1);

    // 强计数和弱计数
    assert_eq!(Rc::strong_count(&root), 1); // root itself
    assert_eq!(Rc::weak_count(&root), 1); // child 的 parent 是弱引用
}

#[test]
/// 测试: Arc 线程安全的引用计数
fn test_arc() {
    // 语法: Arc<T> 是线程安全的 Rc, 使用原子操作更新计数
    // 避坑: Arc 的原子操作比 Rc 慢; 跨线程共享数据用 Arc, 单线程用 Rc
    use std::sync::Arc;

    let data = Arc::new(vec![1, 2, 3]);
    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = std::thread::spawn(move || format!("thread {}: {:?}", i, data_clone));
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.contains("1, 2, 3"));
    }
}

#[test]
/// 测试: Arc + Mutex 线程安全共享和互斥锁
fn test_arc_mutex() {
    // 语法: Arc<T> 线程安全的 Rc; Mutex<T> 互斥锁保护内部数据
    // 避坑: lock() 返回 Result, 线程 panic 时锁会中毒(Poisoned); 避免长时间持有锁
    use std::sync::{Arc, Mutex};
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);
    let mut guard = data_clone.lock().unwrap();
    guard.push(4);
    assert_eq!(*guard, vec![1, 2, 3, 4]);
}

#[test]
/// 测试: Arc + RwLock 读写锁
fn test_arc_rwlock() {
    // 语法: RwLock 允许多个读或一个写, 适合读多写少场景
    // 避坑: 读锁可以并发获取, 写锁独占; 同一线程不能同时获取读锁和写锁(死锁)
    use std::sync::{Arc, RwLock};

    let data = Arc::new(RwLock::new(vec![1, 2, 3]));

    // 多个读锁可以并发
    let read1 = data.read().unwrap();
    let read2 = data.read().unwrap();
    assert_eq!(*read1, vec![1, 2, 3]);
    assert_eq!(*read2, vec![1, 2, 3]);
    drop(read1);
    drop(read2);

    // 写锁独占
    let mut write = data.write().unwrap();
    write.push(4);
    assert_eq!(*write, vec![1, 2, 3, 4]);
}

#[test]
/// 测试: Cell 内部可变性 (Copy 类型)
fn test_cell() {
    // 语法: Cell<T> 通过 get/set 修改内部值, 不需要 &mut
    // 避坑: 只能用于 Copy 类型; get() 复制值, 不返回引用; 不能借用内部数据
    use std::cell::Cell;

    let cell = Cell::new(5);
    cell.set(10);
    assert_eq!(cell.get(), 10);

    // 常见场景: 在不可变上下文中修改状态
    struct Counter {
        count: Cell<u32>,
    }

    impl Counter {
        fn new() -> Self {
            Self { count: Cell::new(0) }
        }
        fn increment(&self) {
            self.count
                .set(self.count.get() + 1);
        }
        fn get(&self) -> u32 {
            self.count.get()
        }
    }

    let counter = Counter::new();
    counter.increment();
    counter.increment();
    counter.increment();
    assert_eq!(counter.get(), 3);
}

#[test]
/// 测试: RefCell 内部可变性 (非 Copy 类型)
fn test_refcell() {
    // 语法: RefCell<T> 运行时借用检查, 允许在不可变引用下获取可变引用
    // 避坑: borrow_mut() 时已有 borrow() 会 panic; 借用是运行时的, 不是编译时的
    use std::cell::RefCell;

    let data = RefCell::new(vec![1, 2, 3]);

    // 不可变借用
    let borrow = data.borrow();
    assert_eq!(*borrow, vec![1, 2, 3]);
    drop(borrow); // 必须先释放不可变借用

    // 可变借用
    let mut borrow_mut = data.borrow_mut();
    borrow_mut.push(4);
    assert_eq!(*borrow_mut, vec![1, 2, 3, 4]);
}

#[test]
/// 测试: RefCell 运行时借用检查 panic
fn test_refcell_borrow_panic() {
    // 语法: RefCell 在运行时检查借用规则, 违反时 panic
    // 避坑: 不要同时持有 borrow() 和 borrow_mut(); 用 try_borrow() 避免 panic
    use std::cell::RefCell;

    let data = RefCell::new(42);

    // try_borrow 不会 panic
    let borrow = data.borrow();
    assert!(data.try_borrow_mut().is_err()); // 已有不可变借用, 无法可变借用
    assert_eq!(*borrow, 42);

    // try_borrow_mut 不会 panic
    drop(borrow);
    let mut borrow_mut = data.borrow_mut();
    *borrow_mut = 100;
    assert!(data.try_borrow().is_err()); // 已有可变借用, 无法不可变借用
}

#[test]
/// 测试: Cell::update 原子读取-修改-写入 (1.88+)
fn test_cell_update() {
    // 语法: Cell::update(f) 原子地读取-修改-写入, 返回旧值 (1.88+)
    // 避坑: 闭包参数是 T(需要 Copy), 不是 &T; 适用于计数器场景
    use std::cell::Cell;
    let cell = Cell::new(5);
    cell.update(|x: i32| x * 2);
    assert_eq!(cell.get(), 10);
}

#[test]
/// 测试: MutexGuard 生命周期和作用域
fn test_mutex_guard_lifetime() {
    // 语法: MutexGuard 是 RAII 锁守卫, drop 时自动释放锁
    // 避坑: Guard 持有可变借用, 作用域要尽量小; 不要用 let 在函数级别持有
    use std::sync::Mutex;

    let mutex = Mutex::new(0);

    // 好的做法: 小作用域
    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    } // guard 在这里 drop, 锁释放

    {
        let mut guard = mutex.lock().unwrap();
        *guard += 1;
    }

    assert_eq!(*mutex.lock().unwrap(), 2);
}

#[test]
/// 测试: Deref 强制转换 (智能指针自动解引用)
fn test_deref_coercion() {
    // 语法: 实现了 Deref<Target=T> 的类型可以自动 &Box<T> → &T 转换
    // 避坑: Deref 只作用于不可变引用; DerefMut 作用于可变引用; 不要滥用 Deref 做隐式转换
    let s = String::from("hello");

    // String 实现 Deref<Target=str>, 所以 &String 可以当 &str 用
    fn takes_str(s: &str) -> usize {
        s.len()
    }

    assert_eq!(takes_str(&s), 5); // &String → &str 自动转换

    // Vec<T> 实现 Deref<Target=[T]>
    let v = vec![1, 2, 3];
    fn takes_slice(s: &[i32]) -> usize {
        s.len()
    }
    assert_eq!(takes_slice(&v), 3); // &Vec<i32> → &[i32] 自动转换
}

#[test]
/// 测试: Cow (Clone on Write) 写时复制
fn test_cow() {
    // 语法: Cow<str> 可以是借用(&str)或 owned(String), 修改时才克隆
    // 避坑: 不修改时零拷贝; 一旦修改就变成 owned; 适合 "大多数情况不需要修改" 的场景
    use std::borrow::Cow;

    fn abs_all(input: &mut Cow<[i32]>) {
        for i in 0..input.len() {
            if input[i] < 0 {
                // 第一次修改时, 从借用转为 owned
                input.to_mut()[i] *= -1;
            }
        }
    }

    // 没有负数, 不克隆
    let borrowed = Cow::Borrowed(&[1, 2, 3][..]);
    let mut b = borrowed.clone();
    abs_all(&mut b);
    assert!(matches!(b, Cow::Borrowed(_)));

    // 有负数, 克隆为 owned
    let borrowed = Cow::Borrowed(&[-1, 2, -3][..]);
    let mut b = borrowed.clone();
    abs_all(&mut b);
    assert!(matches!(b, Cow::Owned(_)));
    assert_eq!(b.as_ref(), &[1, 2, 3]);
}

#[test]
/// 测试: NonZero 整数类型内存优化
fn test_nonzero_integers() {
    // 语法: NonZeroU8/NonZeroUsize 等保证值不为 0, 用于 Option 内存优化
    // 避坑: Option<NonZeroU8> 和 u8 大小相同(1字节), 因为 None 用 0 表示
    use std::num::NonZeroU8;

    assert_eq!(std::mem::size_of::<u8>(), 1);
    assert_eq!(std::mem::size_of::<NonZeroU8>(), 1);
    assert_eq!(std::mem::size_of::<Option<NonZeroU8>>(), 1); // 优化!
    assert_eq!(std::mem::size_of::<Option<u8>>(), 2); // 未优化

    let nz = NonZeroU8::new(42).unwrap();
    assert_eq!(nz.get(), 42);
    assert!(NonZeroU8::new(0).is_none());
}

#[test]
/// 测试: NonZero<char> 内存优化 (1.89+)
fn test_nonzero_char() {
    // 语法: NonZero<char> 利用 char 永不为零的特性做内存优化 (1.89+)
    // 避坑: NonZero::new('\0') 返回 None, 因为 null char 是零值; 用于 Option<NonZero<char>> 优化
    use std::num::NonZero;
    let nz = NonZero::new('A').expect("NonZero<char> 创建失败");
    assert_eq!(nz.get(), 'A');
    assert!(NonZero::<char>::new('\0').is_none());
}

#[test]
/// 测试: Pin 固定指针 (自引用结构体)
fn test_pin() {
    // 语法: Pin<P> 保证数据不会被移动, 用于自引用结构体和 async/await
    // 避坑: Pin 不是智能指针, 是包装器; 不能从 Pin<&mut T> 获取 &mut T
    use std::pin::Pin;

    let mut x = 42;
    let pin = Pin::new(&mut x);
    // 通过 Pin 访问需要特殊方法, 不能直接解引用为 &mut
    assert_eq!(*pin, 42);
}

#[test]
/// 测试: Drop trait 自定义清理
fn test_drop() {
    // 语法: impl Drop for T { fn drop(&mut self) } 在值离开作用域时调用
    // 避坑: 不能手动调用 drop 方法, 用 std::mem::drop(value); drop 顺序是字段逆序
    use std::sync::atomic::{AtomicBool, Ordering};

    static DROPPED: AtomicBool = AtomicBool::new(false);

    struct Cleanup;
    impl Drop for Cleanup {
        fn drop(&mut self) {
            DROPPED.store(true, Ordering::SeqCst);
        }
    }

    {
        let _c = Cleanup;
    } // Cleanup 在这里 drop

    assert!(DROPPED.load(Ordering::SeqCst));
}

// ===========================================================================
// 深入增强测试（笔记增强配套）
// ===========================================================================

#[test]
/// 测试: Rc 强引用计数和弱引用计数详细操作
fn test_rc_reference_count() {
    // 语法: strong_count 决定数据释放, weak_count 决定 RcBox 释放
    // 避坑: 强计数为 0 时数据释放但 RcBox 可能留存(弱计数>0); clone 增加强计数不深拷贝
    use std::rc::Rc;

    let rc = Rc::new(42);

    // 初始计数
    assert_eq!(Rc::strong_count(&rc), 1);
    assert_eq!(Rc::weak_count(&rc), 0);

    // clone 增加强计数
    let rc2 = Rc::clone(&rc);
    assert_eq!(Rc::strong_count(&rc), 2);
    assert_eq!(Rc::strong_count(&rc2), 2);

    // downgrade 增加弱计数, 强计数不变
    let weak1 = Rc::downgrade(&rc);
    assert_eq!(Rc::strong_count(&rc), 2);
    assert_eq!(Rc::weak_count(&rc), 1);

    let weak2 = Rc::downgrade(&rc);
    assert_eq!(Rc::weak_count(&rc), 2);

    // 弱引用可升级
    let upgraded = weak1.upgrade().unwrap();
    assert_eq!(*upgraded, 42);

    // 释放所有强引用后数据 drop
    drop(rc);
    drop(rc2);
    drop(upgraded);

    // 弱引用升级失败
    assert!(weak1.upgrade().is_none());
    assert!(weak2.upgrade().is_none());
}

#[test]
/// 测试: Rc 循环引用导致内存泄漏 (仅供教学演示)
fn test_rc_cycle_problem() {
    // 语法: 两个 Rc 互相持有对方 → 强计数永不归零 → 内存泄漏
    // 避坑: 打破循环用 Weak; 此测试仅演示危险, 实际数据会在测试结束后由 OS 回收
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct CycleNode {
        _value: i32,
        _next: RefCell<Option<Rc<CycleNode>>>,
    }

    // 标记是否被 drop 过
    static DROPPED_A: AtomicBool = AtomicBool::new(false);
    static DROPPED_B: AtomicBool = AtomicBool::new(false);

    struct DropTag {
        flag: &'static AtomicBool,
    }
    impl Drop for DropTag {
        fn drop(&mut self) {
            self.flag.store(true, Ordering::SeqCst);
        }
    }

    let a = Rc::new(RefCell::new(Some(DropTag { flag: &DROPPED_A })));
    let b = Rc::new(RefCell::new(Some(DropTag { flag: &DROPPED_B })));

    // 形成循环引用
    let node_a = Rc::new(CycleNode { _value: 1, _next: RefCell::new(None) });
    let node_b = Rc::new(CycleNode { _value: 2, _next: RefCell::new(Some(Rc::clone(&node_a))) });
    *node_a._next.borrow_mut() = Some(Rc::clone(&node_b));
    // node_a → node_b → node_a (循环)

    // 此时两个节点的强引用计数都 >= 2
    assert!(Rc::strong_count(&node_a) >= 2);
    assert!(Rc::strong_count(&node_b) >= 2);

    // 释放局部变量, 但强计数仍为 1 (互相持有), 不会触发 drop
    drop(node_a);
    drop(node_b);
    // 注意: 此测试不会因为泄漏而失败, 仅做教学展示

    // DropTag 的 drop 测试仍在作用域内, 确保正常 drop
    std::mem::drop(a);
    std::mem::drop(b);
}

#[test]
/// 测试: Rc<RefCell<T>> 经典模式 —— 多个所有者共享可变数据
fn test_rc_refcell_pattern() {
    // 语法: Rc<RefCell<T>> 既共享所有权又允许内部可变, 单线程最常用组合
    // 避坑: 运行时借用检查: 不能同时 borrow_mut; RefCell 非 Sync, 不能跨线程
    use std::cell::RefCell;
    use std::rc::Rc;

    // 共享的计数器模拟
    let counter = Rc::new(RefCell::new(0));

    let c1 = Rc::clone(&counter);
    let c2 = Rc::clone(&counter);
    let c3 = Rc::clone(&counter);

    // 多个所有者各自修改
    *c1.borrow_mut() += 1;
    *c2.borrow_mut() += 2;
    *c3.borrow_mut() += 3;

    assert_eq!(*counter.borrow(), 6);
    assert_eq!(Rc::strong_count(&counter), 4); // counter + c1 + c2 + c3

    // 释放 clone 后计数恢复
    drop(c1);
    drop(c2);
    drop(c3);
    assert_eq!(Rc::strong_count(&counter), 1);
}

#[test]
/// 测试: Arc::make_mut 写时复制 (CoW) 语义
fn test_arc_make_mut() {
    // 语法: Arc::make_mut(&mut arc) 获取 &mut T, 多共享时自动克隆
    // 避坑: 唯一引用时直接返回, 无分配; 多共享时隐式克隆, 注意性能
    use std::sync::Arc;

    // 唯一引用: make_mut 不克隆
    let mut arc = Arc::new(vec![1, 2, 3]);
    {
        let v = Arc::make_mut(&mut arc);
        v.push(4);
    }
    assert_eq!(*arc, vec![1, 2, 3, 4]);

    // 多个共享者: make_mut 会克隆
    let mut arc1 = Arc::new(vec![10, 20]);
    let arc2 = Arc::clone(&arc1);

    assert_eq!(Arc::strong_count(&arc1), 2);

    {
        let v = Arc::make_mut(&mut arc1); // CoW: 先克隆再修改
        v.push(30);
    }

    // arc1 指向新克隆的数据, arc2 不变
    assert_eq!(*arc1, vec![10, 20, 30]);
    assert_eq!(*arc2, vec![10, 20]);

    // 克隆后 arc1 成为唯一所有者
    assert_eq!(Arc::strong_count(&arc1), 1);
    assert_eq!(Arc::strong_count(&arc2), 1); // arc2 独自持有旧数据
}

#[test]
/// 测试: Cell 与 RefCell 全面对比
fn test_cell_vs_refcell() {
    // 语法: Cell 零开销但仅限 Copy 类型; RefCell 支持任意类型但有运行时检查
    // 避坑: Cell.get() 复制值不返回引用; Cell 不能用于 String/Vec 等非 Copy 类型
    use std::cell::{Cell, RefCell};

    // Cell: 零开销, 仅 Copy 类型
    let cell = Cell::new(0u32);
    assert_eq!(std::mem::size_of_val(&cell), std::mem::size_of::<u32>()); // 大小等于 u32

    // 多次 set 无检查开销
    for i in 1..=100 {
        cell.set(i);
    }
    assert_eq!(cell.get(), 100);

    // RefCell: 有运行时检查, 支持任意类型
    let refcell = RefCell::new(vec![1, 2, 3]);

    // 作用域限制避免借用冲突
    {
        let r1 = refcell.borrow();
        let r2 = refcell.borrow(); // 多个不可变借用 OK
        assert_eq!(*r1, vec![1, 2, 3]);
        assert_eq!(*r2, vec![1, 2, 3]);
    } // r1, r2 在此释放

    {
        let mut w = refcell.borrow_mut();
        w.push(4);
        w.push(5);
    }

    assert_eq!(*refcell.borrow(), vec![1, 2, 3, 4, 5]);

    // Cell 不能包装非 Copy 类型
    // 下面这行会编译错误:
    // let _bad = Cell::new(vec![1, 2, 3]); // Vec<i32> 不是 Copy

    // RefCell 可包装 Vec
    let _good = RefCell::new(vec![1, 2, 3]);
}

#[test]
/// 测试: Cow 在字符串处理中的经典用法
fn test_cow_string_processing() {
    // 语法: Cow<str> 延迟克隆, 只有需要修改时才分配新字符串
    // 避坑: to_mut() 将 Borrowed 转为 Owned; into_owned() 无条件提取 Owned
    use std::borrow::Cow;

    // 场景1: 净化文本 —— 包含敏感词时替换, 否则零分配
    fn censor(word: &str) -> Cow<str> {
        if word.contains("bad") {
            Cow::Owned(word.replace("bad", "***"))
        } else {
            Cow::Borrowed(word)
        }
    }

    let clean = censor("hello world");
    assert!(matches!(clean, Cow::Borrowed(_)));
    assert_eq!(clean, "hello world");

    let dirty = censor("this is bad content");
    assert!(matches!(dirty, Cow::Owned(_)));
    assert_eq!(dirty, "this is *** content");

    // 场景2: to_mut —— 运行时按需克隆
    let mut cow: Cow<str> = Cow::Borrowed("hello");
    assert!(matches!(cow, Cow::Borrowed(_)));

    // 第一次获取可变引用 → 自动转换
    let s: &mut String = cow.to_mut();
    s.push_str(" world");
    assert!(matches!(cow, Cow::Owned(_)));
    assert_eq!(cow, "hello world");

    // 场景3: 大写转换 —— 只有包含小写字母才分配
    fn upper_if_has_lower(s: &str) -> Cow<str> {
        if s.chars().any(|c| c.is_ascii_lowercase()) {
            Cow::Owned(s.to_ascii_uppercase())
        } else {
            Cow::Borrowed(s)
        }
    }

    let r1 = upper_if_has_lower("ABC");
    assert!(matches!(r1, Cow::Borrowed(_)));

    let r2 = upper_if_has_lower("Hello");
    assert!(matches!(r2, Cow::Owned(_)));
    assert_eq!(r2, "HELLO");

    // 场景4: into_owned 无条件提取
    let cow: Cow<str> = Cow::Borrowed("rust");
    let owned: String = cow.into_owned(); // Borrowed 也会转为 Owned
    assert_eq!(owned, "rust");
}

#[test]
/// 测试: Weak 弱引用打破循环引用 —— 树形结构
fn test_weak_reference() {
    // 语法: Rc::downgrade → Weak, Weak::upgrade → Option<Rc<T>>
    // 避坑: 树/图结构: 父持有子用 Rc(强), 子引用父用 Weak(弱); upgrade 需检查 None
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    struct TreeNode {
        value: i32,
        children: RefCell<Vec<Rc<TreeNode>>>,
        parent: RefCell<Weak<TreeNode>>,
    }

    impl Drop for TreeNode {
        fn drop(&mut self) {
            eprintln!("释放节点: {}", self.value);
        }
    }

    // 构建树:  root(1) → child(2)
    let root = Rc::new(TreeNode {
        value: 1,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Weak::new()),
    });

    let child = Rc::new(TreeNode {
        value: 2,
        children: RefCell::new(vec![]),
        parent: RefCell::new(Rc::downgrade(&root)),
    });

    // 建立父子关系
    root.children
        .borrow_mut()
        .push(Rc::clone(&child));

    // 计数验证: 只有 root → child 一条强引用链
    assert_eq!(Rc::strong_count(&root), 1);   // 仅 root 自身
    assert_eq!(Rc::strong_count(&child), 2);   // child 自身 + root.children
    assert_eq!(Rc::weak_count(&root), 1);      // child 的 parent 弱引用

    // 子节点可通过 Weak 找到父节点
    {
        let parent_opt = child.parent.borrow().upgrade();
        assert!(parent_opt.is_some());
        let p = parent_opt.unwrap();
        assert_eq!(p.value, 1);
    }

    // 释放 root: 没有循环引用, 所有节点正常释放
    drop(root);
    // child 仍然存在 (被 root.children 持有... 等等 root 已经没了)
    // 实际上 root 被释放后 root.children 也被释放, child 强计数变为 1
    // child 的 parent 弱引用 upgrade 失效
    assert!(child.parent.borrow().upgrade().is_none());

    // 最终 child 也能正常释放
    assert_eq!(Rc::strong_count(&child), 1);
}
