# 所有权与借用

## 一、所有权三大规则

Rust 没有垃圾回收器，取而代之的是编译期所有权系统——这是 Rust 最独特也最重要的机制。

```rust
{
    let s = String::from("hello");  // s 获得字符串所有权
    // s 在这里有效...
}   // 作用域结束，s 被 drop，内存自动释放
```

> 所有权不是语言设计者的任性，而是无 GC 时代系统编程的唯一出路——要么自己管内存，要么让编译器替你管。

### 规则一：每个值有且仅有一个所有者

```rust
let s1 = String::from("hello");  // 字符串数据在堆上
let s2 = s1;                      // 所有权从 s1 转移到 s2
// println!("{s1}");              // 编译错误：s1 已失效 (E0382)
println!("{s2}");                 // 正常
```

### 规则二：离开作用域时自动 drop

```rust
fn make_string() -> String {
    let s = String::from("hello");  // 分配堆内存
    s                                // 所有权转移给调用者，不会被 drop
}
```

### 规则三：赋值 = 移动（Move）语义

---

## 二、Move 语义 vs Copy 语义

Move 语义适用于**堆数据**和所有非 Copy 类型；Copy 语义适用于**纯栈数据**。

```rust
// Copy 类型：简单位复制，原变量仍可用
let x = 5;
let y = x;       // x 被复制，x 仍有效
println!("{x}"); // 正常

// Move 类型：所有权转移，原变量失效
let s1 = String::from("hello");
let s2 = s1;
// println!("{s1}"); // 错误！s1 值已被移动
```

### Copy 类型完整清单

| 类别 | 具体类型 |
|------|----------|
| 所有整数类型 | `u8`、`i32`、`usize` 等 |
| 浮点类型 | `f32`、`f64` |
| 布尔类型 | `bool` |
| 字符类型 | `char` |
| 元组 | **仅当所有元素都是 Copy 时**，如 `(i32, bool)` |
| 固定长度数组 | **仅当元素是 Copy 时**，如 `[i32; 5]` |
| 不可变引用 | `&T`（引用本身是 Copy） |
| 裸指针 | `*const T`、`*mut T` |
| 函数指针 | `fn(T) -> U` |

> Copy 和 Move 的分界线就是数据是否在堆上——栈上的东西可以随意复制，堆上的东西必须追踪所有权。

---

## 三、借用

借用让数据可以被**临时使用**而不转移所有权：

```rust
fn calculate_length(s: &String) -> usize {  // 不可变借用
    s.len()
}

let s = String::from("hello");
let len = calculate_length(&s);   // & 创建引用
println!("'{s}' 的长度是 {len}");
// s 依然是所有者，未被移动
```

### 不可变借用 `&T`

- 共享读访问，`&T` 是 Copy 类型
- 可以同时存在**任意多个**不可变借用
- 借用期间不可修改原值

```rust
let s = String::from("hello");
let r1 = &s;
let r2 = &s;          // 多个不可变借用共存
println!("{r1} {r2}");
```

### 可变借用 `&mut T`

- 排他读写访问
- **同一时刻只能存在一个**可变借用
- 可变借用期间，不能有不可变借用

```rust
let mut s = String::from("hello");

let r1 = &mut s;
// let r2 = &mut s;   // 错误：E0499 不能同时有两个可变借用
r1.push_str(", world");

println!("{s}");      // r1 最后一次使用后，可变借用失效
let r2 = &mut s;      // 因此 r2 可以借用
```

> 借用检查器不是编译器在刁难你，而是它在证明你的程序不可能出现数据竞争——这是静态分析的最高境界。

---

## 四、NLL — 非词法生命周期

Rust 2018 引入 NLL，借用检查基于**实际使用**而非词法作用域：

```rust
let mut s = String::from("hello");

let r1 = &s;                    // r1 不可变借用
let r2 = &s;                    // r2 不可变借用
println!("{r1} and {r2}");      // r1、r2 最后一次使用
// r1 和 r2 的借用在 println! 后立即失效

let r3 = &mut s;                // 此时没有活跃借用，r3 可以创建
r3.push_str(" world");

// 传统词法生命周期：整个块内 r1/r2 有效 → r3 冲突
// NLL：r1/r2 在 println! 后已失效 → r3 无冲突
```

NLL 基于 MIR（Mid-level IR）数据流分析，精确追踪每个引用的活跃范围。

---

## 五、引用再借用

```rust
let mut x = 42;
let r = &mut x;           // 可变借用
let r2 = &mut *r;         // 再借用：从 &mut T 创建新的 &mut T
*r2 += 1;                 // 通过再借用修改
// r2 用完，r 恢复活跃
*r += 1;
```

---

## 六、部分移动

```rust
struct Person {
    name: String,
    age: u32,
}

let p = Person {
    name: String::from("Alice"),
    age: 30,
};

let name = p.name;            // name 字段被移动
// println!("{}", p.name);    // 错误：name 字段已失效
println!("{}", p.age);        // age 字段（Copy 类型）仍可用
// let whole = p;              // 错误：无法整体使用 p（缺失 name 字段）
```

解构也是部分移动：

```rust
let (name, age) = (p.name, p.age);  // 元组解构
// 或使用 let Person { name, age } = p;
```

---

## 七、所有权与闭包

```rust
let mut s = String::from("hello");

// FnMut：可变借用捕获
let mut append = || s.push('!');
append();

// Fn：不可变借用捕获
let len = || s.len();
println!("{}", len());

// FnOnce：消耗捕获（所有权转移）
let consume = || {
    let owned = s;          // 所有权移入闭包
    println!("{owned}");
};
consume();                   // s 已失效
// println!("{s}");          // 错误

// move 关键字强制所有权转移
let x = 42;
let closure = move || x + 1;  // x 被移入闭包（尽管是 Copy）
```

| Trait | 捕获方式 | 调用次数 | 典型场景 |
|-------|----------|----------|----------|
| `Fn` | `&self` 不可变借用 | 多次 | 纯计算 |
| `FnMut` | `&mut self` 可变借用 | 多次 | 修改捕获变量 |
| `FnOnce` | `self` 所有权消耗 | 一次 | 转移所有权 |

---

## 八、迭代中的所有权

```rust
let v = vec![1, 2, 3];

// into_iter：消费集合，转移所有权
for x in v.into_iter() {
    // v 已失效
    println!("{x}");
}

let v = vec![1, 2, 3];
// iter：不可变借用，返回 &T
for x in v.iter() {
    println!("{x}");   // x 是 &i32
}
println!("{v:?}");     // v 仍可用

let mut v = vec![1, 2, 3];
// iter_mut：可变借用，返回 &mut T
for x in v.iter_mut() {
    *x += 1;
}
println!("{v:?}");     // [2, 3, 4]
```

> 迭代器是零成本抽象的典范——`for` 循环展开后和手写的 `while` 循环一样快，却拥有声明式的表现力。

---

## 九、Box 的 Move 语义

```rust
let b1 = Box::new(42);     // 堆上分配
let b2 = b1;               // 所有权转移，b1 失效
// 堆数据通过指针移动，不复制数据本身
```

---

## 十、Rc 与 Arc — 多所有权

```rust
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);   // 引用计数 +1，不复制数据
let c = Rc::clone(&a);

println!("引用计数: {}", Rc::strong_count(&a));  // 3
println!("{a} {b} {c}");

// Arc 用于多线程（原子引用计数）
use std::sync::Arc;
use std::thread;
let a = Arc::new(42);
let b = Arc::clone(&a);
thread::spawn(move || {
    println!("{b}");
}).join().unwrap();
```

---

## 十一、内部可变性

在不可变借用下修改内部数据：

```rust
use std::cell::Cell;

// Cell：运行时零开销，仅用于 Copy 类型
let x = Cell::new(42);
let y = &x;                // 不可变借用
x.set(100);                // 但仍能修改！（内部可变性）
println!("{}", x.get());   // 100

use std::cell::RefCell;

// RefCell：运行时借用检查（运行时 panic 而非编译期错误）
let data = RefCell::new(vec![1, 2, 3]);
{
    let mut v = data.borrow_mut();  // 可变借用（运行时计数）
    v.push(4);
}
println!("{:?}", data.borrow());   // 不可变借用读取
```

| 类型 | 零开销 | 适用数据 | 检查时机 |
|------|--------|----------|----------|
| `Cell<T>` | 是 | `T: Copy` | 编译期 |
| `RefCell<T>` | 否（运行时计数） | 任意 `T` | 运行时 |

---

## 十二、切片 — 胖指针

```rust
let s = String::from("hello world");
let slice: &str = &s[0..5];  // "hello"
// &str 是胖指针：(指针, 长度) = 16字节

let arr = [1, 2, 3, 4, 5];
let slice: &[i32] = &arr[1..4];  // [2, 3, 4]
// &[i32] 是胖指针：(指针, 长度) = 16字节

fn split_at<T>(slice: &[T], mid: usize) -> (&[T], &[T]) {
    (&slice[..mid], &slice[mid..])
}

let (left, right) = split_at(&arr, 2);
```

> 切片的胖指针（ptr + len）是 Rust 安全设计的一个缩影——每一个引用都带着足够的元信息来保证越界不可能发生。

---

## 十三、借用检查器常见错误码

| 错误码 | 含义 | 典型场景 |
|--------|------|----------|
| `E0382` | 使用了已移动的值 | `let y = x; println!("{x}");` |
| `E0502` | 同时存在不可变和可变借用 | 持有 `&T` 时尝试 `&mut T` |
| `E0506` | 尝试在借用期间修改 | 有不可变借用，尝试 `x.push(...)` |
| `E0515` | 返回引用但生命周期不足 | 返回局部变量的引用 |
| `E0597` | 借用存活不够长 | 引用的生命周期短于所有者 |
| `E0499` | 同时有多个可变借用 | 两个 `&mut` 同时存在 |
| `E0507` | 尝试从借用中移出 | `let x = *ref;` 而 `T` 非 Copy |
| `E0716` | 临时值在被借用时被销毁 | 链式临时值的生命周期问题 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 字符串切片 `&s[0..1]` 会在非 UTF-8 边界 panic | 索引必须落在字符边界 | 使用 `chars()` 迭代、`find()`、`char_indices()` |
| `&String` 可以自动转为 `&str`，反过来不行 | Deref 强制只能单向 | 用 `to_owned()` 或 `to_string()` 反向转换 |
| 闭包同时使用 `&self` 和 `&mut self` 编译失败 | 同一时刻不可变和可变借用冲突 | 缩小借用范围或使用 Cell/RefCell |
| 返回局部变量的引用 | 引用指向已释放的栈内存 | 使用 `Box`、`Rc`、克隆或移动所有权 |
| `Rc` 不是线程安全的 | 非原子引用计数 | 使用 `Arc` 替代 |
| `RefCell` 运行时双重可变借用将 panic | 运行时借用计数器爆了 | 使用 `try_borrow_mut()` 或重新设计 |
| 部分移动后无法整体使用结构体 | 字段所有权被移走 | 先借用字段，或同时解构所有字段 |
| `into_iter()` 之后原集合不可用 | 所有权被迭代器消耗 | 用 `iter()` 或 `iter_mut()` 代替 |

> **详见测试**: `tests/rust_features/03_ownership_borrowing.rs`
