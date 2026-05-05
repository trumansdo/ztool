# Rust 泛型编程：零成本抽象的引擎

## 一、泛型函数与泛型结构体

> **金句引用**："泛型是 Rust 的诗——写一次，编译为千千万万个具体类型，运行时不留痕迹。"

### 1.1 泛型函数

```rust
// 泛型函数：最大值的泛型版本
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut max = list[0];
    for &item in list.iter() {
        if item > max {
            max = item;
        }
    }
    max
}

let nums = vec![10, 42, 5, 18];
let chars = vec!['y', 'm', 'a', 'q'];
assert_eq!(largest(&nums), 42);
assert_eq!(largest(&chars), 'y');
```

### 1.2 泛型结构体与枚举

```rust
// 结构体：坐标点（支持任意类型）
#[derive(Debug)]
struct Point<T, U> {
    x: T,
    y: U,
}
let int_float = Point { x: 5, y: 4.0 };
let char_char = Point { x: 'a', y: 'b' };

// 枚举：Option<T> / Result<T, E>
enum Option<T> { Some(T), None }
enum Result<T, E> { Ok(T), Err(E) }

// 泛型在方法上
impl<T, U> Point<T, U> {
    fn mixup<V, W>(self, other: Point<V, W>) -> Point<T, W> {
        Point { x: self.x, y: other.y }
    }
}
```

---

## 二、单态化原理

> **金句引用**："单态化是泛型的编译期魔法——每个具体类型一份独立机器码，运行时不查虚表。"

```rust
// 源代码：
fn identity<T>(x: T) -> T { x }

fn main() {
    let a = identity(42);      // T = i32
    let b = identity("hi");    // T = &str
}

// 编译器单态化后（概念等价）：
fn identity_i32(x: i32) -> i32 { x }
fn identity_ref_str(x: &str) -> &str { x }
// 每个具体组合生成一份独立副本 → 零运行时开销
```

**影响**：
- 运行速度快（无运行时分发开销）
- 代码体积增大（每个具体类型一份拷贝）
- 编译时间稍长（类型检查与代码生成）

---

## 三、Turbofish 语法

```rust
let nums = vec![1, 2, 3];

// 解析结果为 Result，编译器无法推断 E 的类型
// let parsed = "42".parse().unwrap();  // 编译错误：类型不明确

// Turbofish 显式指定类型参数
let parsed = "42".parse::<i32>().unwrap();         // Ok(42)
let sum: f64 = nums.iter().map(|x| *x as f64).sum::<f64>(); // turbofish on method
let collected = (0..5).collect::<Vec<i32>>();
```

---

## 四、Trait Bound 约束

> **金句引用**："约束不是枷锁，是承诺——告诉调用者'我能做什么'，告诉编译器'你该检查什么'。"

### 4.1 八种组合模式

```rust
// 1. 单一约束
fn print<T: std::fmt::Display>(item: T) { println!("{}", item); }

// 2. 多约束
fn clone_and_print<T: Clone + std::fmt::Display>(item: T) {
    let cloned = item.clone();
    println!("{}", cloned);
}

// 3. 泛型约束 + 关联类型
fn print_debug<T: std::fmt::Debug + ?Sized>(item: &T) {
    println!("{:?}", item);
}

// 4. where 子句多约束
fn complex<T, U>(t: T, u: U) -> i32
where
    T: std::fmt::Display + Clone,
    U: Clone + std::fmt::Debug,
{ 0 }

// 5. 多约束 + 超trait
trait Printable: std::fmt::Display + Clone {}
fn print_clone<T: Printable>(item: T) { println!("{}", item); }

// 6. 相等约束（关联类型绑定）
fn add_twice<T: std::ops::Add<Output = T> + Copy>(x: T) -> T {
    x + x
}

// 7. 生命周期 + 泛型
fn longest_with_display<'a, T>(x: &'a str, y: &'a str, ann: T) -> &'a str
where T: std::fmt::Display
{ if x.len() > y.len() { x } else { y } }

// 8. 多个 trait 间 AND 关系
fn save<T: serde::Serialize + std::fmt::Debug>(item: &T) -> String {
    serde_json::to_string(item).unwrap()
}
```

---

## 五、Where 子句高级用法

> **金句引用**："where 子句是泛型签名的分行符——把拥挤的尖括号摊成清晰的清单。"

```rust
use std::fmt::Debug;
use std::hash::Hash;

// 关联类型约束
fn hash_and_debug<K, V>(map: &std::collections::HashMap<K, V>)
where
    K: Hash + Eq + Debug,
    V: Debug,
    <K as Debug>::Error: Send,  // 关联类型的关联约束
{ }

// 相等约束
trait Graph {
    type Node;
    type Edge;
}
fn nodes_equal<G1, G2>(g1: &G1, g2: &G2) -> bool
where
    G1: Graph,
    G2: Graph<Node = G1::Node>,  // 相等约束：两个 Graph 的 Node 相同
{ true }

// 生命周期混合
fn process<'a, T>(data: &'a [T]) -> &'a T
where
    T: 'a + Debug,  // T 必须存活至少 'a
{ &data[0] }

// ?Sized 放宽
fn print_slice<T: ?Sized + Debug>(value: &T) {
    println!("{:?}", value);
}
```

---

## 六、泛型参数默认类型

```rust
// trait 级别默认类型参数
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
// impl Add for i32 → 等价于 impl Add<i32> for i32

// 结构体级别默认类型参数
struct Wrapper<T = i32> {
    value: T,
}
let w1 = Wrapper { value: 42 };    // T = i32（默认）
let w2 = Wrapper { value: "str" }; // T = &str（编译器推断）
```

---

## 七、Const 泛型：编译期整数参数

> **金句引用**："Const 泛型把编译期整数锻造成了类型的一部分。"

```rust
// 编译期固定长度数组的泛型抽象
struct Vector3<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> Vector3<T, N> {
    fn new() -> Self {
        Vector3 { data: [T::default(); N] }
    }
    fn len(&self) -> usize { N }
}

let v: Vector3<f64, 3> = Vector3::new();
assert_eq!(v.len(), 3);

// const fn + const 泛型 结合
const fn factorial<const N: usize>() -> usize {
    if N == 0 { 1 } else { N * factorial::<{ N - 1 }>() }
}
let f5 = factorial::<5>(); // 编译期求值得 120

// const 泛型限制（Rust 1.85）：
// 支持的类型：整数、bool、char
// 不支持：浮点数、&str、自定义类型
```

---

## 八、毯式实现（Blanket Implementation）

```rust
// 标准库的毯式实现示例
impl<T: Display> ToString for T {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}
// 一行为所有 Display 类型添加 to_string()

// 自定义毯式实现
trait IntoJson {
    fn into_json(&self) -> String;
}
impl<T: serde::Serialize> IntoJson for T {
    fn into_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// 覆盖规则：具体实现优先毯式实现
struct MyStruct;
impl Display for MyStruct { fn fmt(&self, f: &mut Formatter) -> fmt::Result { ... } }
// MyStruct 自动获得 ToString
// 如果为 MyStruct 手动实现 ToString，则手动实现优先
```

---

## 九、条件 Trait 实现

```rust
// 条件实现：只有当内部类型满足条件时才实现 trait
struct Pair<T> {
    first: T,
    second: T,
}

// 仅当 T 可比较时，Pair<T> 才能比较
impl<T: PartialOrd> Pair<T> {
    fn max(&self) -> &T {
        if self.first >= self.second { &self.first } else { &self.second }
    }
}

// 仅当 T 可哈希时，Pair<T> 才能被哈希
impl<T: Hash> Hash for Pair<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.first.hash(state);
        self.second.hash(state);
    }
}

// 经典案例：Vec<T> 仅在 T 可打印时可调试
// impl<T: Debug> Debug for Vec<T> { ... }
```

---

## 十、类型状态模式

> **金句引用**："类型即状态——编译期的状态机让非法操作连编译都过不了。"

```rust
// 编译期状态机：每个状态一个类型，0 运行时开销
mod machine {
    pub struct Initialized;
    pub struct Running;
    pub struct Stopped;

    pub struct StateMachine<State> {
        id: u64,
        _state: std::marker::PhantomData<State>,
    }

    impl StateMachine<Initialized> {
        pub fn new() -> Self {
            StateMachine { id: 0, _state: std::marker::PhantomData }
        }
        pub fn start(self) -> StateMachine<Running> {
            StateMachine { id: self.id, _state: std::marker::PhantomData }
        }
    }

    impl StateMachine<Running> {
        pub fn stop(self) -> StateMachine<Stopped> {
            StateMachine { id: self.id, _state: std::marker::PhantomData }
        }
    }
}

use machine::*;
let machine = StateMachine::new();       // Initialized
let machine = machine.start();            // Initialized → Running（消耗旧状态）
// machine.start();  // 编译错误！Running 没有 start()
let machine = machine.stop();             // Running → Stopped
```

**优势**：非法状态转换在编译期被拒绝，零运行时开销。

---

## 十一、泛型与生命周期结合

```rust
use std::fmt::Display;

// T: 'a —— T 类型不包含任何生命周期短于 'a 的引用
fn longest_with_announcement<'a, T>(
    x: &'a str,
    y: &'a str,
    ann: T,
) -> &'a str
where
    T: Display,
{
    println!("公告: {}", ann);
    if x.len() > y.len() { x } else { y }
}

// 结构体级生命周期 + 泛型
struct Context<'a, T> {
    data: &'a [T],
    name: &'a str,
}

// HRTB: 高阶 trait 限定
fn apply_to_list<F>(list: &[i32], f: F) -> Vec<i32>
where
    F: for<'a> Fn(&'a i32) -> i32,  // F 对任意生命周期 'a 都适用
{
    list.iter().map(f).collect()
}
```

---

## 十二、关联类型 vs 泛型参数 选型表

> **金句引用**："同一概念一个实现用关联类型，同一概念多种可能性用泛型参数。"

| 维度 | 关联类型 | 泛型参数 |
|------|----------|----------|
| 实现次数 | 每种类型只能实现一次 | 可为不同参数多次实现 |
| API 简洁度 | 调用者不指定类型 | 调用者常需指定类型 |
| 示例 | `Iterator<Item = T>` | `Add<Rhs = Self>` |
| 约束表达 | `where T: Iterator<Item = i32>` | `where T: Add<i32>` |
| 歧义性 | 无（一对一） | 需 turbofish 消歧 |
| 何时使用 | 输出类型由实现唯一确定 | 同一类型支持多种组合方式 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 泛型约束过度具体 | 用 `T: Clone + Debug + ...` 限制了调用方 | 仅添加方法体实际使用的 trait 约束 |
| 忘记约束 trait 导致 "method not found" | 方法来自 trait 但泛型参数未约束 | 在 where 子句或尖括号中添加对应 trait 约束 |
| const 泛型尝试用浮点数 | const 泛型只支持整数/bool/char | 用宏或类型级编程变通 |
| 毯式实现导致冲突 | 两个毯式实现覆盖了同一组类型 | 遵守孤儿规则，通过 newtype 区分 |
| turbofish 遗漏 | 编译器无法推断类型参数 | 用 `::<T>` 或提供足够的上下文类型信息 |
| 类型状态模式误用 | 状态转换后试图重用已消费的状态 | 每个转换方法 consume self，编译器自动拒绝 |
| `where T: 'a` 缺失 | 结构体包含引用但泛型参数无生命周期约束 | 添加 `T: 'a` 确保 T 内引用寿命够长 |
| 毯式实现覆盖了后续的手动优化 | 毯式实现是一刀切，后期无法为特定类型优化 | 优先写毯式实现，为关键路径类型覆盖具体实现 |

> **详见测试**: `tests/rust_features/13_generics.rs`
