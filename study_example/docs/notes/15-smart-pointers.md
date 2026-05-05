# Rust 智能指针：所有权之上的精细控制

## 一、Box\<T\>：堆分配的起点

> **金句引用**："Box 是 Rust 托管指针的最小单位——数据上堆，指针留栈，所有权不丢。"

### 1.1 基础用法

```rust
// 最简单的堆分配
let b = Box::new(42);        // 栈上8字节指针，指向堆上4字节整数
println!("{}", *b);         // 解引用访问

// Box 离开作用域时自动释放堆内存（调用 drop）
{
    let b = Box::new(String::from("临时"));
    // 离开 → String 的 drop → 堆内存释放
}
```

### 1.2 递归类型

```rust
// 递归枚举：Node 包含 Node，大小无限
// Box 打破递归——指针大小固定（8字节）

enum List {
    Cons(i32, Box<List>),  // Box<List> 大小 = 8字节（指针）
    Nil,
}

use List::{Cons, Nil};
let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

// 遍历
fn print_list(list: &List) {
    match list {
        Cons(val, next) => {
            println!("{}", val);
            print_list(next);
        }
        Nil => {}
    }
}
// 不借 Box 走不了——1→2→3→Nil

// 二叉树示例
struct BinTree<T: Ord> {
    root: Option<Box<Node<T>>>,
}
struct Node<T: Ord> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}
```

### 1.3 Box\<dyn Trait\>：动态分发

```rust
trait Animal {
    fn speak(&self);
}
struct Dog;
struct Cat;
impl Animal for Dog { fn speak(&self) { println!("汪汪"); } }
impl Animal for Cat { fn speak(&self) { println!("喵喵"); } }

let animals: Vec<Box<dyn Animal>> = vec![
    Box::new(Dog),
    Box::new(Cat),
];
for animal in &animals {
    animal.speak(); // 动态分发：运行时查虚表
}
```

### 1.4 Box\<[T]\> vs Vec\<T\>

| 特性 | `Box<[T]>` | `Vec<T>` |
|------|-----------|----------|
| 大小可变 | 不可变（定长） | 可用 `push`/`pop` |
| 内存占用 | 8字节指针 | 24字节（指针+长度+容量） |
| 典型场景 | `String::into_boxed_str()` | 需要动态长度的序列 |

```rust
let vec = vec![1, 2, 3, 4, 5];
let boxed_slice: Box<[i32]> = vec.into_boxed_slice();
// 节省内存：Vec 24字节 → Box<[i32]> 8字节
// 代价：不能再 push
```

---

## 二、Rc\<T\>：单线程引用计数

> **金句引用**："Rc 是共享的代价——多指针指向同一数据，计数归零才释放。"

### 2.1 基础使用

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);

let a = Rc::clone(&data); // 引用计数 +1，不深拷贝数据
let b = Rc::clone(&data); // 引用计数 +1

println!("引用计数: {}", Rc::strong_count(&data)); // 3
println!("数据: {:?}", *data); // 解引用访问

// data、a、b 全部释放后，Vec 才释放
```

**内存布局**：

```
Rc<T> 8字节（栈）
   ↓
┌──────────────┐
│ 强引用计数(usize) │
│ 弱引用计数(usize) │
│ T 值          │
└──────────────┘
全部在堆上
```

### 2.2 Rc<RefCell\<T\>>：内部可变性 + 共享

```rust
use std::rc::Rc;
use std::cell::RefCell;

// 经典组合：多个所有者共享可变数据
let shared = Rc::new(RefCell::new(vec![1, 2, 3]));

let owner1 = Rc::clone(&shared);
let owner2 = Rc::clone(&shared);

owner1.borrow_mut().push(4);
owner2.borrow_mut().push(5);

let borrowed = shared.borrow();
assert_eq!(*borrowed, vec![1, 2, 3, 4, 5]);
```

### 2.3 循环引用问题

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

// ❌ 循环引用 → 内存泄漏
#[derive(Debug)]
struct Node {
    next: Option<Rc<RefCell<Node>>>,
    data: i32,
}
let a = Rc::new(RefCell::new(Node { next: None, data: 1 }));
let b = Rc::new(RefCell::new(Node { next: Some(Rc::clone(&a)), data: 2 }));
a.borrow_mut().next = Some(Rc::clone(&b)); // a→b→a 环形引用
// a 和 b 永远无法释放，RC 无法归零
```

---

## 三、Weak\<T\>：打破循环

> **金句引用**："Weak 是 Rc 的弱引用——只观察，不计入——循环从此断开。"

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct TreeNode {
    value: i32,
    parent: RefCell<Weak<TreeNode>>,  // 弱引用：不增加引用计数
    children: RefCell<Vec<Rc<TreeNode>>>,
}

let leaf = Rc::new(TreeNode {
    value: 3,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![]),
});

println!("leaf 父节点 = {:?}", leaf.parent.borrow().upgrade()); // None

let branch = Rc::new(TreeNode {
    value: 1,
    parent: RefCell::new(Weak::new()),
    children: RefCell::new(vec![Rc::clone(&leaf)]),
});

*leaf.parent.borrow_mut() = Rc::downgrade(&branch); // 建立弱引用

println!("leaf 父节点 = {:?}", leaf.parent.borrow().upgrade()); // Some(branch)
// drop(branch) 之后 leaf.parent.upgrade() 返回 None
```

---

## 四、Arc\<T\>：多线程引用计数

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3, 4, 5]);

let mut handles = vec![];
for i in 0..3 {
    let data_clone = Arc::clone(&data);  // 原子引用计数 +1
    handles.push(thread::spawn(move || {
        println!("线程{}: {:?}", i, *data_clone);
    }));
}
for h in handles { h.join().unwrap(); }
```

### Rc vs Arc 对照

| 特性 | Rc\<T\> | Arc\<T\> |
|------|---------|----------|
| 线程安全 | 仅单线程 | 多线程 |
| 计数操作 | 非原子 `+=1` | 原子 `fetch_add` |
| 性能 | 快 ~3x | 慢（原子操作开销） |
| Send/Sync | !Send + !Sync | Send + Sync (T: Send+Sync) |
| weak | `Weak<T>` | `Weak<T>` |

### Arc::make_mut 的 CoW 语义

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

// 强引用计数 = 1：原地修改
let mut_mut = Arc::make_mut(&mut data.clone());
mut_mut.push(4);

// 强引用计数 > 1：克隆后修改（CoW——写时复制）
let clone = Arc::clone(&data);
// data 的强引用计数 = 2
let mut_mut = Arc::make_mut(&mut data.clone());
// Arc::make_mut 内部：检测到 count > 1 → 克隆 Vec → 修改克隆体
```

---

## 五、RefCell\<T\>：运行时借用检查

> **金句引用**："RefCell 是编译器的禁飞区——借用检查从编译时移至运行时，违规即 panic。"

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);

// 不可变借用
let r1 = data.borrow();
let r2 = data.borrow();  // 多个不可变借用 → OK
println!("{:?}", r1);
drop(r1);
drop(r2);

// 可变借用
{
    let mut r = data.borrow_mut();
    r.push(4);
} // r 离开作用域 → 借用释放

// 违反规则 → panic（而非编译错误）
// let r1 = data.borrow();
// let r_mut = data.borrow_mut();  // panic! 已有不可变借用！

// try_borrow / try_borrow_mut：返回 Result 而非 panic
if let Ok(val) = data.try_borrow() {
    println!("值: {:?}", val);
}
if let Ok(mut val) = data.try_borrow_mut() {
    val.push(5);
}
```

### RefCell 的内存结构

```
RefCell<T>（栈上32字节）
├─ borrow: Cell<isize>    // 借用计数（0=无借用，正=不可变借用数，-1=可变借用）
└─ value: UnsafeCell<T>   // 裸指针包装，绕过编译期借用检查
```

---

## 六、Cell\<T\> vs RefCell\<T\>

| 维度 | Cell\<T\> | RefCell\<T\> |
|------|-----------|-------------|
| 要求 | T: Copy（get/set 整体替换） | 任意类型 |
| 内部可变性机制 | 值整体替换（无借用） | 运行时借用检查 |
| 性能 | 零开销（仅复制 Copy 值） | 有借用计数开销 |
| 典型用途 | 小数值（Cell\<i32\>） | 大结构体或非 Copy 类型 |
| &T 转 &mut T | ❌ | ❌（但有 borrow/bury_mut） |

```rust
use std::cell::Cell;

let counter = Cell::new(0);

// set/get 直接替换值，不涉及借用
counter.set(counter.get() + 1);
counter.set(counter.get() + 1);
assert_eq!(counter.get(), 2);

// 没有 borrow/borrow_mut，只有值级操作
let old = counter.replace(10);  // 原子地：取出旧值，放入新值
assert_eq!(old, 2);
assert_eq!(counter.get(), 10);
```

---

## 七、Cow\<T\>：写时复制

> **金句引用**："Cow 是克隆的延迟术——不写则不复制，写了才克隆。"

```rust
use std::borrow::Cow;

// Cow 的两个变体：
// Cow::Borrowed(&T) — 借用，未修改（零开销）
// Cow::Owned(T)     — 已修改，为克隆体

fn process(input: &str) -> Cow<str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_")) // 有空格 → 需要修改 → 克隆
    } else {
        Cow::Borrowed(input)                // 无空格 → 无需修改 → 借用
    }
}

let s = "hello world";
let cow = process(s);
match cow {
    Cow::Borrowed(b) => println!("借用: {}", b),
    Cow::Owned(o)    => println!("拥有: {}", o),
}

// to_mut() 让 cow 变为可变
let mut cow = Cow::Borrowed("不可变");
// 此时 cow 仍为 Borrowed，未分配内存
let mut_ref = cow.to_mut(); // 触发克隆！转为 Owned(String)
mut_ref.push_str(" → 已修改");

// into_owned() 摘出所有权
let owned_string: String = cow.into_owned();

// 字符串处理经典场景
fn sanitize<'a>(input: &'a str) -> Cow<'a, str> {
    if input.chars().any(|c| c.is_control()) {
        Cow::Owned(input.chars().filter(|c| !c.is_control()).collect())
    } else {
        Cow::Borrowed(input)
    }
}
```

---

## 八、Deref / DerefMut 强制转换

> **金句引用**："Deref 强制转换——编译器为你插入 `*`，智能指针变得透明。"

### 8.1 自动解引用规则

- 从 `&T` 到 `&U`，当 `T: Deref<Target = U>`
- 从 `&mut T` 到 `&mut U`，当 `T: DerefMut<Target = U>`
- 从 `&mut T` 到 `&U`，当 `T: Deref<Target = U>`
- 方法调用时编译器自动尝试解引用以匹配签名
- **仅编译时行为，零运行时开销**

```rust
use std::ops::{Deref, DerefMut};

struct MyBox<T>(T);

impl<T> MyBox<T> {
    fn new(x: T) -> Self { MyBox(x) }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

// 使用
let b = MyBox::new(String::from("hello"));
// 自动解引用：&MyBox<String> → &String → &str
let s: &str = &b;
println!("{}的{}个字符", s, s.len());  // len() 找到 &str 上的方法

// 可变解引用
let mut b = MyBox::new(42);
*b = 100; // 等价于 *b.deref_mut() = 100
assert_eq!(*b, 100);
```

### 8.2 注意事项

1. 解引用不消耗所有权——仅产生引用
2. 编译器最多尝试 N 次解引用（实际无明确上限）
3. 解引用后的 `self` 方法调用会自动适配
4. `DerefMut` 需要 `Deref` 先实现

---

## 九、Drop Trait：析构顺序

> **金句引用**："Drop 是 Rust 的告别礼——逆序析构，先建后拆，次序不乱。"

```rust
struct Custom {
    name: String,
}
impl Drop for Custom {
    fn drop(&mut self) {
        println!("{} 被释放", self.name);
    }
}

let a = Custom { name: String::from("A") };
let b = Custom { name: String::from("B") };
let c = Custom { name: String::from("C") };
// 输出：
// C 被释放
// B 被释放
// A 被释放
// 逆序声明原则：后声明的先释放
```

### std::mem::drop vs std::mem::forget

```rust
// 手动提前释放
let x = Box::new(100);
std::mem::drop(x);  // 立即调用 Box<i32> 的 Drop，释放堆内存
// x 不再可用

// forget 跳过 Drop —— 危险！
let y = String::from("危险");
std::mem::forget(y); // String 的 Drop 不执行 → 堆内存泄漏
// y 不再可用，但其堆内存永不归还

// forget 的合理用途：
//   1. FFI 中将所有权转移给 C 代码
//   2. 避免 Drop 某些 unsafe 构建
```

---

## 十、智能指针总结矩阵

| 指针 | 堆分配 | 计数 | 线程安全 | 内部可变 | 典型使用 |
|------|--------|------|----------|----------|----------|
| `Box<T>` | 是 | 单一 | Send+Sync | 否 | 递归类型、动态分发 |
| `Rc<T>` | 是 | 引用计数 | 仅单线程 | 否 | 共享所有权 |
| `Arc<T>` | 是 | 原子计数 | 多线程 | 否 | 多线程共享 |
| `RefCell<T>` | 否 | 运行时借用 | 仅单线程 | 是 | 绕过编译期借用检查 |
| `Cell<T>` | 否 | 零开销替换 | Send | 是 | Copy小型值 |
| `Cow<T>` | 按需 | 无 | — | 按需 | 延迟克隆 |
| `Weak<T>` | 与Rc/Arc共用 | 弱计数 | 同源 | — | 打破循环引用 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `Rc` 循环引用导致内存泄漏 | A→B→A 的引用链使计数永不归零 | 用 `Weak<T>` 替代其中一个方向 |
| `RefCell` borrow 冲突导致运行时 panic | 编译期无法检查内部可变性 | 控制借用作用域；用 `try_borrow` 提前检查 |
| `Arc::make_mut` 在计数 >1 时意外克隆 | `make_mut` = clone-on-write | 确保预期行为：计数=1 直接修改，>1 克隆 |
| `Deref` 强制转换使类型系统中的信息丢失 | `Box::<T>::foo()` 无法调用 | 需要特定类型方法时手动 `*` 或直接用 owned 类型 |
| `Box::new` 大量分配导致内存碎片 | 堆分配代价高 | 用 `Vec::with_capacity` 批量分配；考虑 `Box<[T]>` 压缩 |
| `mem::forget` 导致的泄漏不易追踪 | 没有 Drop 运行，分析工具无法检测 | 仅在 FFI 等必需场景使用，其余场景用 `ManuallyDrop` |
| `Cow` 的 `to_mut` 总是克隆 Borrowed 变体 | `to_mut()` 无条件将 `Borrowed` 转为 `Owned` | 先检查是否需要修改，确实需要时再 `to_mut()` |
| `Cell<T>` 不支持 PartialEq（取无 Copy 的值） | `Cell` 的 `get` 需要 `T: Copy` | 用 `RefCell<T>` 替代非 Copy 类型 |

> **详见测试**: `tests/rust_features/15_smart_pointers.rs`
