# Rust 类型系统与 Trait

## 一、Trait：行为契约

> **金句引用**："Trait 是 Rust 的灵魂——它用组合替代继承，用契约约束泛型。"

### 1.1 定义与实现

```rust
// 定义 Trait
trait Summary {
    fn summarize(&self) -> String;

    // 默认方法：实现者可以不重写
    fn default_summary(&self) -> String {
        String::from("(更多详情...)")
    }
}

// 为类型实现 Trait
struct Article {
    title: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} - {}...", self.title, &self.content[..20.min(self.content.len())])
    }
}

// Self 指代实现 Trait 的具体类型
trait Clone {
    fn clone(&self) -> Self;  // Self = Article 当为 Article 实现时
}
```

### 1.2 孤儿规则 (Orphan Rule)

> **金句引用**："孤儿规则是 Rust 的边界墙——trait 或类型，总有一方是你的子民。"

**规则**：为类型 `T` 实现 trait `Tr` 时，`T` 或 `Tr` 至少有一方在当前 crate 中定义。

```rust
// 正确：Summary 是本 crate 定义的，类型 Article 也是本 crate 的
impl Summary for Article { }

// 正确：Display 是标准库的，但 Article 是本 crate 的
impl std::fmt::Display for Article { }

// 错误！Vec 和 Display 都来自标准库
// impl std::fmt::Display for Vec<i32> { }  // 编译错误！

// 绕过：Newtype 模式
struct MyVec(Vec<i32>);
impl std::fmt::Display for MyVec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
    }
}
```

---

## 二、超 Trait (Supertrait)

```rust
trait Draw {
    fn draw(&self);
}

// 凡实现 Figure 者必须先实现 Draw
trait Figure: Draw {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

// 实现时需同时满足超trait要求
struct Circle { radius: f64 }

impl Draw for Circle {
    fn draw(&self) { println!("画一个圆"); }
}

impl Figure for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
    fn perimeter(&self) -> f64 { 2.0 * std::f64::consts::PI * self.radius }
}
```

---

## 三、完全限定语法：消歧义

> **金句引用**："同名方法不迷路——完全限定语法是类型系统的导航仪。"

```rust
trait Pilot {
    fn fly(&self);
}
trait Wizard {
    fn fly(&self);
}
struct Human;

impl Pilot for Human { fn fly(&self) { println!("机长发言"); } }
impl Wizard for Human { fn fly(&self) { println!("上升！"); } }
impl Human { fn fly(&self) { println!("*挥舞手臂*"); } }

let person = Human;

person.fly();                           // *挥舞手臂*（固有方法优先）
Pilot::fly(&person);                    // 机长发言
Wizard::fly(&person);                   // 上升！
// 完全限定语法（最通用形式）：
<Human as Pilot>::fly(&person);        // 机长发言
<Human as Wizard>::fly(&person);       // 上升！

// 关联函数（无 self）的同名冲突
trait Animal {
    fn baby_name() -> String;
}
struct Dog;
impl Animal for Dog {
    fn baby_name() -> String { String::from("小狗") }
}
impl Dog {
    fn baby_name() -> String { String::from("斑点") }
}

// 无 self 的关联函数只能用完全限定语法区分
println!("{}", Dog::baby_name());              // 斑点（固有方法）
println!("{}", <Dog as Animal>::baby_name());  // 小狗（trait方法）
```

---

## 四、Trait 对象 (dyn Trait)：动态分发

> **金句引用**："Trait 对象是一个胖指针——8 字节数据，8 字节虚表，运行时动态寻找真身。"

### 4.1 虚表原理

```rust
trait Draw { fn draw(&self); }

struct Circle;
impl Draw for Circle { fn draw(&self) { println!("○"); } }

struct Square;
impl Draw for Square { fn draw(&self) { println!("□"); } }

// 胖指针结构（16字节）：
// ┌──────────────────┬──────────────────┐
// │  数据指针(8字节)  │  虚表指针(8字节)  │
// └──────────────────┴──────────────────┘

let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle),
    Box::new(Square),
];
for shape in &shapes {
    shape.draw(); // 动态分发：运行时查虚表
}

// &dyn Draw: 16字节胖指针
// Box<dyn Draw>: 堆上数据 + 16字节栈上胖指针
// 标准引用 &T: 8字节瘦指针
```

### 4.2 对象安全性 (Object Safety)

只有满足**对象安全**的 trait 才能生成 trait 对象：

```rust
// ✅ 对象安全的 trait（方法满足以下全部条件）
trait ObjectSafe {
    fn method1(&self);          // 1. Self: Sized 不是要求
    fn method2(self: &Rc<Self>);// 2. self 在接收者位置
    fn method3(self: Box<Self>);// 3. 无泛型参数
}
// → 可以创建 Box<dyn ObjectSafe>

// ❌ 非对象安全的 trait
trait NotObjectSafe {
    fn new() -> Self;          // 1. 返回 Self（无 self 参数且返回 Self）
    fn generic<T>(&self, x: T);// 2. 有泛型参数
    fn clone(&self) -> Self;   // 3. 返回 Self
}

// 绕过技巧：where Self: Sized
trait Drawable {
    fn draw(&self);
    fn create() -> Self where Self: Sized;  // 此方法在 trait 对象上不可调用
}
let obj: Box<dyn Drawable> = Box::new(Circle);
// obj.create(); // 编译错误：trait 对象不支持 Sized 方法
```

---

## 五、impl Trait：静态分发

> **金句引用**："impl Trait 是泛型参数的语法糖——编译期展开，运行时零开销。"

```rust
// 参数位置：语法糖，等价于泛型约束
fn draw_it_impl(item: &impl Draw) {
    item.draw();
}
// 等价于：
fn draw_it_generic<T: Draw>(item: &T) {
    item.draw();
}

// 返回位置：不透明类型（RPIT）
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y  // 返回具体闭包类型，但对外不可见
}
let add5 = make_adder(5);
println!("{}", add5(10)); // 15

// RPITIT: Trait 中的 RPIT（Rust 1.75+）
trait Factory {
    fn create(&self) -> impl std::fmt::Display; // 关联返回位置 impl Trait
}
struct IntFactory;
impl Factory for IntFactory {
    fn create(&self) -> i32 { 42 } // 实现者可指定具体类型
}
```

**impl Trait vs dyn Trait vs 泛型**：

| 特性 | `impl Trait` | `dyn Trait` | 泛型 `<T: Trait>` |
|------|-------------|-------------|-------------------|
| 分发 | 静态(单态化) | 动态(虚表) | 静态(单态化) |
| 运行时开销 | 无 | 有(间接调用) | 无 |
| 代码体积 | 每个类型一份 | 一份 | 每个类型一份 |
| 异构集合 | 不支持 | 支持 | 不支持 |
| 返回值 | 单一具体类型 | 任意实现类型 | — |

---

## 六、关联类型 vs 泛型参数

> **金句引用**："一个概念一个实现用关联类型；一个概念多个实现用泛型参数。"

```rust
// 关联类型：每个实现只有一个 Item 类型
trait Graph {
    type Node;        // 关联类型
    type Edge;
    fn nodes(&self) -> &[Self::Node];
}

// 泛型参数：同一个类型可实现多次，每次不同的 Rhs
pub trait Add<Rhs = Self> {  // 泛型参数，有默认值
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
// 一个 i32 可以实现 Add<i32> 和 Add<&i32> 多次
```

**选型指南**：

| 场景 | 推荐方案 |
|------|----------|
| 一个类型只需实现 trait 一次 | 关联类型 |
| 同一类型需要以多种方式实现 trait | 泛型参数 |
| 输出类型由输入唯一确定 | 关联类型 |
| 调用者想灵活组合类型 | 泛型参数 |

---

## 七、Derive 宏完整清单

```rust
// 常用派生宏一览
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Point {
    x: i32,
    y: i32,
}

// Debug:  调试打印，{:#?} 试格式
// Clone:  显式克隆 .clone()
// Copy:   隐式复制（赋值即复制），需所有字段实现 Copy
// PartialEq: 等于/不等于
// Eq:     无额外逻辑，仅标记"可以用作 HashMap 键"
// PartialOrd: 部分排序（浮点数用）
// Ord:    全序，用于 BTreeMap 键 / sort
// Hash:   哈希，用于 HashMap/HashSet 键
// Default: 默认值，对应 Default::default()
```

---

## 八、Sized / ?Sized 深度解析

```rust
// 所有类型默认 Sized（编译期大小已知）
fn sized_fn<T>(x: T) { }  // T: Sized（隐式）

// ?Sized 放宽约束：允许非定长类型（如 [u8]、dyn Trait、str）
fn unsized_fn<T: ?Sized>(x: &T) { }  // 允许传入切片引用

// 典型应用：Box<dyn Trait>、Rc<str>、&[T]

// Trait 默认 Sized，若想作为 trait 对象需放宽
trait MyTrait: ?Sized {
    fn method(&self);
}
// 现在可以 Box<dyn MyTrait>

// 部分方法要求 Sized（where Self: Sized 技巧）
trait Container {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn into_boxed(self) -> Box<Self> where Self: Sized;  // trait 对象不可用
}
```

---

## 九、标记 Trait 与自动 Trait

### 9.1 标记 Trait

| Trait | 含义 | 用途 |
|-------|------|------|
| `Send` | 可以安全转移到另一线程 | 跨线程所有权传递 |
| `Sync` | 可以安全地多线程共享引用 | `Arc<T>` 的共享条件 |
| `Copy` | 位级复制，赋值后原值可用 | 简单数值/无堆指针类型 |
| `Sized` | 编译期大小确定 | 绝大多数类型（默认） |
| `Unpin` | 可以在内存中安全移动 | 异步 future 语义 |

### 9.2 自动 Trait：Send/Sync 推导规则

```rust
// Send/Sync 是自动 trait：字段全满足则结构体自动满足
struct MyStruct {
    data: i32,          // i32: Send + Sync ✓
    ptr: *const i32,    // 裸指针: !Send + !Sync
}
// MyStruct: !Send + !Sync（因为裸指针不满足）

// 手动标注 Send/Sync（需 unsafe 块，需确保线程安全）
struct Wrapper { ptr: *const i32 }
unsafe impl Send for Wrapper {}
unsafe impl Sync for Wrapper {}

// 负号实现（取消自动推导）
#![feature(negative_impls)]
impl !Send for MyStruct {}  // 显式声明不满足 Send
```

---

## 十、ZST 零大小类型

```rust
// 零大小类型运行时占 0 字节
struct Unit;              // 单元结构体：0 字节
let _ = ();               // 单元类型：0 字节
let _ = [(); 100];        // 100 个零大小值：0 字节！

// PhantomData<T>：类型级占位，0 字节运行时成本
use std::marker::PhantomData;

// 场景1：标记所有权
struct SliceWrapper<'a, T> {
    ptr: *const T,
    _marker: PhantomData<&'a T>,  // 告诉编译器：此结构体借用 T
}

// 场景2：标记类型参数
struct Handle<T> {
    id: u32,
    _type: PhantomData<T>,  // 未使用 T，但需要编译器知道 T 存在
}

// 场景3：控制变型（不变性）
struct Invariant<T> {
    _marker: PhantomData<fn(T) -> T>,  // fn(T) 使 T 变为不变
}

// 场景4：Send/Sync 标注
struct NonSend {
    _marker: PhantomData<*const ()>,  // 裸指针使结构体 !Send + !Sync
}
```

---

## 十一、TypeId 与 Any Trait

```rust
use std::any::{Any, TypeId};

fn type_id_of<T: 'static>(_: &T) -> TypeId {
    TypeId::of::<T>()
}

let a = 42i32;
let b = "hello";
assert_ne!(type_id_of(&a), type_id_of(&b));

// Any trait：运行时类型转换
fn print_if_string(value: Box<dyn Any + 'static>) {
    if let Some(s) = value.downcast_ref::<String>() {
        println!("字符串: {}", s);
    } else if let Some(n) = value.downcast_ref::<i32>() {
        println!("整数: {}", n);
    }
}
```

---

## 十二、PhantomData 五种场景详解

| 场景 | 写法 | 效果 |
|------|------|------|
| 所有权 | `PhantomData<&'a T>` | 告知编译器此结构体"借用" T |
| 生命周期 | `PhantomData<&'a ()>` | 仅绑定生命周期，不涉及具体类型 |
| 类型参数 | `PhantomData<T>` | 即使未使用 T，编译器也认为它存在 |
| 不变性 | `PhantomData<fn(T) -> T>` | 使 T 在结构体中为不变(invariant) |
| Send/Sync | `PhantomData<*const T>` | 阻止 Send/Sync 的自动推导 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| Trait 对象方法缺失 | trait 非对象安全（含泛型/返回 Self） | 找出不安全的签名，使用 `where Self: Sized` 隔离 |
| `impl Trait` 返回不同类型 | RPIT 要求函数所有返回路径为**同一具体类型** | 用 `Box<dyn Trait>` 或 `enum` 统合不同返回类型 |
| 违反孤儿规则 | 想为外部类型实现外部 trait | 使用 newtype 模式包装外部类型 |
| `dyn Trait` 自动推导的 Send/Sync 丢失 | `dyn Trait` 不自动实现 Send/Sync 除非 trait 声明 | 显式使用 `dyn Trait + Send + Sync` |
| `PhantomData<T>` 导致 T 被 dropped 时恐慌 | PhantomData 确实拥有类型参数的生命周期关系 | 用 `PhantomData<fn() -> T>` 消除所有权语义 |
| derive 宏被忽视 | 忘记 derive 关键 trait 导致功能不可用 | 在 `Cargo.toml` 添加对应的 derive 特征门或引入宏包 |
| 混淆关联类型与泛型 | 用关联类型但需要多种实现方式 | 根据"一对一"还是"一对多"决定使用关联类型或泛型参数 |
| `Any` downcast 失败未处理 | `downcast_ref` 返回 `None`，可能静默丢失信息 | 始终检查 downcast 结果并处理失败路径 |

> **详见测试**: `tests/rust_features/12_type_system.rs`
