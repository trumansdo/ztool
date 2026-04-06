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
    let rc2 = Rc::clone(&rc1);
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
    use std::ops::Deref;

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
