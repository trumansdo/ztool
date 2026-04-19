# 09 - 智能指针

## 概述

智能指针是拥有数据并提供额外功能的指针。除了像常规引用一样访问数据外，它们还拥有数据的所有权。在 Rust 中，智能指针通常通过结构体实现，并实现了 `Deref` 和 `Drop` trait。

## Box<T>

堆分配：

```rust
let boxed = Box::new(42);
let value = *boxed;

// 递归类型必须使用 Box
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

**Box 特点**：
- 堆分配，大小为指针大小（8字节）
- 适合递归类型
- 所有权转移时开销低

## Rc<T>

引用计数（单线程）：

```rust
use std::rc::Rc;

let rc = Rc::new(vec![1, 2, 3]);
let clone = Rc::clone(&rc);
println!("{}", Rc::strong_count(&rc)); // 2
```

**Rc 特点**：
- 引用计数，非线程安全
- 允许多重所有权
- 适合单线程场景

### 打破循环引用

```rust
use std::rc::{Rc, Weak};

let inner = Rc::new(42);
let outer = Rc::new(Rc::downgrade(&inner));

let weak: Weak<i32> = Rc::downgrade(&inner);
println!("{:?}", weak.upgrade()); // Some(42)
```

## Arc<T>

原子引用计数（多线程）：

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);
let mut handles = vec![];

for _ in 0..3 {
    let data = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        println!("{:?}", data);
    }));
}

for h in handles { h.join().unwrap(); }
```

### Arc + Mutex

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        *counter.lock().unwrap() += 1;
    }));
}

for h in handles { h.join().unwrap(); }
println!("{}", *counter.lock().unwrap()); // 10
```

## 内部可变性 Cell<T>

```rust
use std::cell::Cell;

let cell = Cell::new(10);
cell.set(20);
println!("{}", cell.get()); // 20
```

**Cell 特点**：
- 适用于 Copy 类型
- 通过 `get()`/`set()` 操作

## RefCell<T>

```rust
use std::cell::RefCell;

let refcell = RefCell::new(vec![1, 2, 3]);

// 不可变借用
let borrow = refcell.borrow();
println!("{:?}", borrow);

// 可变借用
let mut borrow = refcell.borrow_mut();
borrow.push(4);
```

**RefCell 特点**：
- 运行时 borrow 检查
- 适合在 trait 对象中使用
- `borrow()` 返回 `Ref<T>`，`borrow_mut()` 返回 `RefMut<T>`

### 运行时借用检查

```rust
let refcell = RefCell::new(1);

// 不可变借用成功
let a = refcell.borrow();

// 不可变借用期间尝试可变借用 - panic!
let b = refcell.borrow_mut(); // panic: already borrowed
```

## Cow<T>

Copy-on-Write，惰性克隆：

```rust
use std::borrow::Cow;

fn process(text: &str) -> Cow<str> {
    if text.contains("bad") {
        Cow::Owned(text.replace("bad", "good"))
    } else {
        Cow::Borrowed(text)
    }
}

let text = "hello world";
let result = process(text);
// result 是 Borrowed

let text = "bad word";
let result = process(text);
// result 是 Owned
```

## 性能对比

| 类型 | 内存开销 | 线程安全 | 适用场景 |
|------|----------|----------|----------|
| Box<T> | 8字节 | 否 | 堆分配、递归类型 |
| Rc<T> | 8字节 + 计数 | 否 | 共享所有权 |
| Arc<T> | 8字节 + 计数 | 是 | 多线程共享 |
| Cell<T> | T | 否 | Copy类型内部可变性 |
| RefCell<T> | 8字节 + 状态 | 否 | 运行时借用检查 |

## 避坑指南

1. **Rc 循环引用**：使用 Weak 打破循环
2. **RefCell panic**：运行时借用检查失败会 panic
3. **Arc 性能**：有额外原子操作开销
4. **Cow 选择**：读取多写入少时优先使用 Borrowed

## 单元测试

详见 `tests/rust_features/09_smart_pointers.rs`

## 参考资料

- [Rust Smart Pointers](https://doc.rust-lang.org/book/ch15-smart-pointers.html)
- [Rust Rc and Arc](https://medium.com/@ali_alachkar/rust-arc-and-rc)
- [Interior Mutability in Rust](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)