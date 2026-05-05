# 精确捕获 use<> 语法

## 目录
1. [impl Trait 捕获问题背景](#impl-trait-捕获问题背景)
2. [use<> 精确捕获语法](#use-精确捕获语法)
3. [use<> 为空：不捕获任何泛型](#use-为空不捕获任何泛型)
4. [多生命周期使用场景](#多生命周期使用场景)
5. [异步场景中的精确捕获](#异步场景中的精确捕获)
6. [RPIT 与 RPITIT 对比](#rpit-与-rpitit-对比)
7. [与 Edition 的兼容性](#与-edition-的兼容性)
8. [避坑指南](#避坑指南)

---

## impl Trait 捕获问题背景

在返回位置使用 `impl Trait`（RPIT, Return Position Impl Trait）时，Rust 编译器默认会**自动捕获所有出现在函数签名中的泛型参数和生命周期**。这种"全部捕获"策略虽然安全，但经常导致过于保守的 trait 约束，限制了 API 的灵活度。

> 自动捕获就像把所有钥匙都串在一起——安全，但丧失了灵活性。

问题演示——自动捕获导致不必要的约束：

```rust
// 仅凭签名看，Collector 似乎不需要知道 T 的具体类型
// 但实际上 impl Display 自动捕获了 T
fn make_printer<T: std::fmt::Display>(val: T) -> impl Fn() {
    // 返回的闭包捕获了 T 的信息
    move || println!("{}", val)
}

fn use_printers() {
    let p1 = make_printer(42i32);
    let p2 = make_printer("hello");
    // p1 和 p2 是不同的类型，无法放入同一个 Vec
}
```

更隐蔽的问题——生命周期被意外捕获：

```rust
fn returns_closure<'a>(s: &'a str) -> impl Fn() -> &str {
    // impl Fn() -> &str 自动捕获了 'a 生命周期
    // 导致返回的闭包只能活到 'a 结束
    move || s
}
```

---

## use<> 精确捕获语法

`use<>` 语法允许开发者**显式声明** `impl Trait` 类型需要捕获哪些泛型参数和生命周期，未被列出的参数不会被捕获。

> 只拿你需要的，不背负你不该背负的——use<> 给了你挑选的权利。

基本语法：

```rust
// 语法: impl Trait + use<TypeParam1, Lifetime1, ...>
fn example<'a, T, U>(x: &'a T, y: U) -> impl std::fmt::Display + use<'a, T>
where
    T: std::fmt::Display,
{
    format!("{x}")
}
// 仅捕获 'a 和 T，U 不被捕获
```

对比四种捕获模式：

```rust
// 1. 默认行为——自动捕获所有泛型参数和生命周期
fn default_capture<T: Display>(t: T) -> impl Display {
    t
}

// 2. use<T> —— 精确捕获指定泛型 T
fn precise_capture<T: Display>(t: T) -> impl Display + use<T> {
    t
}

// 3. use<> —— 不捕获任何泛型（罕见，含义为返回类型与泛型无关）
fn empty_capture<T>(_t: T) -> impl Display + use<> {
    "固定字符串"
}

// 4. use<'a, T> —— 捕获生命周期和泛型类型
fn lifetime_capture<'a, T: Display>(t: &'a T) -> impl Display + use<'a, T> {
    t
}
use std::fmt::Display;
```

精确捕获的实际收益——类型统一：

```rust
use std::fmt::Display;

fn before<T: Display>(t: T) -> impl Display {
    t
}
// before::<i32> 和 before::<String> 返回不同类型

fn after<T: Display>(t: T) -> impl Display + use<> {
    let _ = t;
    "固定值".to_string()
}
// after::<i32> 和 after::<String> 现在返回相同类型！
```

---

## use<> 为空：不捕获任何泛型

`impl Trait + use<>` 表示返回类型完全独立于任何泛型参数，所有调用返回同一具体类型。

> 空 use<> 就像一纸独立宣言——"我的类型与你传入的所有参数无关。"

不捕获泛型的典型场景——类型擦除的接口层：

```rust
trait Handler {
    fn handle(&self, input: &str) -> String;
}

// 所有对 T 的调用返回同一类型
fn build_handler<T: Handler>(h: T) -> impl Fn(&str) -> String + use<> {
    // 将 h 的具体类型擦除，只在内部使用
    // 返回类型与 T 无关
    let owned_h: Box<dyn Handler> = Box::new(h);
    panic!("not implemented")
}

// 不同的 T 产生相同的返回类型
```

空 use<> 的约束——必须保证返回类型确实与泛型无关：

```rust
// 以下代码无法通过编译，因为返回类型使用了泛型参数
// fn invalid<T>(t: T) -> impl Display + use<> {
//     t  // 编译错误：返回类型依赖 T，但 use<> 声明不捕获 T
// }
```

实际应用——零成本类型擦除 + 同构容器：

```rust
fn make_constant<T>(_t: T) -> impl Fn() -> i32 + use<> {
    || 42
}

fn collect_fns() {
    let f1 = make_constant(1u8);
    let f2 = make_constant("hello");
    let f3 = make_constant(vec![1, 2, 3]);

    // 三个函数类型完全相同，可以放入同一 Vec
    let fns: Vec<Box<dyn Fn() -> i32>> = vec![
        Box::new(f1),
        Box::new(f2),
        Box::new(f3),
    ];
}
```

---

## 多生命周期使用场景

当函数涉及多个生命周期时，`use<>` 可以精确地表达哪些生命周期被捕获、哪些被舍弃，避免不必要的生命周期耦合。

> 生命周期的网越密越紧，use<> 让你只保留必要的那几根线。

多生命周期精简：

```rust
fn multi_lifetime<'a, 'b>(
    a: &'a str,
    _b: &'b str,
) -> impl std::fmt::Display + use<'a> {
    // 只用到了 'a，不捕获 'b
    a
}

// 调用时 'b 的生命周期更灵活
fn demo_multi() {
    let long = String::from("长字符串");
    let result;
    {
        let short = String::from("短");
        result = multi_lifetime(&long, &short);
        // short 在此离开作用域，但 result 仍然有效
        // 因为 use<'a> 未捕获 'b
    }
    println!("{result}");
}
```

混合捕获的语义细节：

```rust
// 同时捕获具体生命周期和泛型
fn capture_some<'a, 'b, T: Display, U>(
    a: &'a T,
    b: &'b U,
) -> impl Display + use<'a, T> {
    format!("{a}") // 仅需要 T 的 Display + 'a 生命周期
}

// CAPTURED:   'a, T
// FREE:       'b, U  —— 调用处更灵活
```

---

## 异步场景中的精确捕获

异步函数返回的 `Future` 类型自动捕获所有泛型参数，`use<>` 可以让异步 Future 不被无关的类型参数撑大。

> 一个大胖 Future 是内存的噩梦——use<> 帮异步函数保持苗条。

异步函数中精确捕获的区别：

```rust
use std::future::Future;

// 问题：返回的 Future 捕获了 T（即使 T 在 .await 前已被消耗）
async fn async_example<T: Display>(t: T) -> String {
    let s = t.to_string();
    // t 在此已被消耗
    some_async_work().await;
    s
    // 返回的 Future 仍"记住"了 T 的类型信息
}

// 解决：使用 use<> 配合类型擦除
fn async_lean<T: Display>(t: T) -> impl Future<Output = String> + use<> {
    let s = t.to_string(); // T 在 Future 创建前被消耗
    async move {
        some_async_work().await;
        s
    }
}

async fn some_async_work() {}
use std::fmt::Display;
```

精简异步 Future 尺寸的实用技巧：

```rust
use std::future::Future;
use std::pin::Pin;

// 将泛型相关的工作与纯异步工作分离
fn build_future<T: Display>(val: T) -> impl Future<Output = String> + use<> {
    let processed = format!("预处理: {val}");
    async move {
        // 此闭包不依赖 T，仅依赖 String
        network_call(&processed).await
    }
}

async fn network_call(s: &str) -> String {
    format!("响应: {s}")
}
```

---

## RPIT 与 RPITIT 对比

RPIT（Return Position Impl Trait）和 RPITIT（Return Position Impl Trait In Trait）在捕获行为上有细微差别。

> 返回位置的抽象是一场戏，RPIT 是独白，RPITIT 是对白——use<> 在两者中的台词不同。

RPIT 中的 use<>（稳定特性）：

```rust
// 自由函数中的 RPIT——use<> 语法已稳定
fn rpit_demo<'a, T: Display>(t: &'a T) -> impl Display + use<'a, T> {
    format!("{t}")
}
```

RPITIT 中的自动捕获（trait 方法中的 impl Trait 返回）：

```rust
trait Factory {
    // trait 方法中的 RPIT——行为与自由函数不同
    fn create<T: Display>(&self, val: T) -> impl Display;
    // 自动捕获 Self 和 T
}

struct MyFactory;
impl Factory for MyFactory {
    fn create<T: Display>(&self, val: T) -> impl Display {
        val
    }
}
```

捕获行为差异速查表：

| 场景 | 默认捕获 | use<> 支持 |
|------|----------|------------|
| 自由函数 RPIT | 所有泛型+生命周期 | 是 |
| 固有方法 RPIT | Self + 所有泛型+生命周期 | 是 |
| Trait 方法 RPIT (RPITIT) | Self + 所有泛型+生命周期 | 部分支持 |
| 闭包返回 impl Trait | 所有捕获的变量 | 不适用 |

---

## 与 Edition 的兼容性

`use<>` 语法随 Rust 1.82 稳定，与 Edition 2021 和 2024 均兼容。

> 好的新特性不绑架 Edition——use<> 可以在任何现有的 Rust 版本中使用。

跨 Edition 使用：

```rust
// 这段代码在 Edition 2021 和 2024 下均可编译
fn cross_edition<T: std::fmt::Display>(t: T) -> impl std::fmt::Display + use<> {
    let _ = t;
    "跨版本兼容"
}
```

注意事项：

1. `use<>` 中的类型/生命周期必须在函数的泛型参数列表中声明
2. `use<>` 不能列出不在签名中的外部类型
3. 放在 `+` 链的末尾：`impl Trait1 + Trait2 + use<...>`

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| use<> 中列出了多余的类型 | 多余的捕获可能不报错但降低了 API 灵活性 | 只列出返回类型实际需要的最小泛型集合 |
| 以为 use<> 能隐藏生命周期约束 | use<> 声明捕获的生命周期仍然参与借用检查 | use<> 控制的是隐藏类型对泛型的依赖，不改变借用规则 |
| 异步块内部使用 move 但与 use<> 冲突 | async move 块本身就有捕获语义，两者可能矛盾 | 确保 use<> 声明的捕获集与实际异步块的需求一致 |
| use<> 为空但返回类型使用了 T | 编译器报错 "type parameter T is part of concrete type" | 要么在 use<> 中加入 T，要么重构代码消除对 T 的依赖 |
| 在 trait 方法中滥用 use<> | RPITIT 的捕获规则更复杂，use<> 支持受限 | 仔细阅读目标 Rust 版本的 RPITIT 支持矩阵 |
| 混淆 use<> 和 where 子句 | use<> 控制隐藏类型的捕获，where 控制类型约束 | 二者独立使用：where 约束可见类型，use<> 声明捕获集 |

---

> **详见测试**: `tests/rust_features/31_precise_capturing.rs`
