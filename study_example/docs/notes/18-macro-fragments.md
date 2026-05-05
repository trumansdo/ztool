# 宏片段说明符深入 + Edition 2024 变更

> 了解片段说明符的匹配规则，就掌握了声明宏 80% 的调试能力——剩下的 20% 是递归展开的逻辑推理。

## 1. 十四种片段说明符完整表

片段说明符(fragment specifier)决定 `$name:frag` 中 `$name` 可以绑定的 Rust 语法片段类型。以下是完整的十四种说明符及其允许跟随的片段：

| 说明符 | 匹配内容 | 允许跟随的片段 | 说明 |
|--------|----------|----------------|------|
| `ident` | 标识符/关键字 | 任意 | `fn`, `self`, `x`, `crate` |
| `expr` | 表达式 | `=>` `,` `;` | `1+2`, `foo()`, `{ ... }` |
| `ty` | 类型 | `=>` `,` `= ` `;` `:` `>` `as` `where` | `i32`, `&str`, `impl Trait` |
| `stmt` | 语句 | `=>` `,` `;` | `let x = 1` (不含 `;`) |
| `pat` | 模式(2024支持\|) | `=>` `,` `= ` `if` `in` | `Some(x)`, `1..=10` |
| `pat_param` | 模式(不含\|) | `=>` `,` `= ` `if` `in` | 同 pat，但禁止 `\|` |
| `item` | 完整项 | 任意 | `fn f(){}`, `struct S;` |
| `meta` | 属性内容 | 任意 | `derive(Debug)` |
| `tt` | 令牌树 | 任意 | 单令牌或括号组 |
| `block` | 块 | 任意 | `{ stmts }` |
| `vis` | 可见性 | 任意 | `pub`, `pub(crate)` |
| `lifetime` | 生命周期 | 任意 | `'a`, `'static` |
| `literal` | 字面量 | 任意 | `42`, `"hi"` |
| `path` | 路径 | 任意 | `std::collections::HashMap` |

> 片段说明符之间不是"正交"关系——`expr` 包含 `block` 和 `literal`，`pat` 包含 `literal`，这种包含关系会影响匹配优先级。

### 1.1 允许跟随的片段

每个说明符都有限定的"合法后续令牌"集合，编译器会据此判断宏匹配是否成功：

```rust
macro_rules! after_expr {
    ($e:expr => $rest:expr) => {
        format!("{:?} 之后是 {:?}", $e, $rest)
    };
    // 以下不合法：expr 后面不允许直接跟 ident
    // ($e:expr $next:ident) => { ... }
}

fn main() {
    println!("{}", after_expr!(1 + 2 => 3 + 4));
}
```

> 允许跟随的片段是宏解析歧义消除的关键——当编译器遇到 `$e:expr` 后跟 `=>` 时，它知道表达式在此结束，不会贪婪地继续匹配。

## 2. expr 匹配的新变化

### 2.1 Edition 2024：expr 可匹配 const 块

在 Edition 2024 中，`expr` 片段说明符可以匹配 `const { ... }` 块表达式：

```rust
// 仅 Edition 2024 中有效
macro_rules! demo_const_block {
    ($e:expr) => {
        println!("表达式: {}", $e);
    };
}

fn main() {
    // Edition 2021: 编译错误
    // Edition 2024: 正常
    demo_const_block!(const { 1 + 2 });
}
```

> const 块表达式是 Rust 1.79 引入的特性，在 2024 版中正式与 `expr` 统合——这是片段说明符语义演进的一个典型案例。

## 3. pat 与 pat_param 的分化

### 3.1 Edition 2021 引入 pat_param

Edition 2021 引入 `pat_param`，它与旧版 `pat` 的唯一区别是**不允许**在顶层使用 `|` 运算符：

```rust
use std::ops::ControlFlow;

// 仅在旧版 rust (2015/2018) 中可编译
/*
macro_rules! match_arms {
    ($x:expr, $($pat:pat => $body:expr),+) => {
        match $x {
            $($pat => $body),+
        }
    };
}
*/
```

### 3.2 Edition 2024：pat 允许 | 但语义受限

Edition 2024 调整了 `pat` 的语义：

```rust
macro_rules! demo_pat {
    ($p:pat) => {
        match 42 {
            $p => "匹配成功",
            _ => "匹配失败",
        }
    };
}

fn main() {
    // Edition 2024: pat 允许 | 但仅在特定上下文
    println!("{}", demo_pat!(1 | 2 | 3));
}
```

**选择指南**：

| 场景 | 使用哪个 |
|------|----------|
| 函数参数、let 绑定 | `pat_param` |
| match 臂模式 | `pat` (2024) |
| 希望通用兼容 | `pat_param` |
| 需要或模式 | `pat` (2024) |

> pat 与 pat_param 的分裂是向后兼容性的生动教材——为了不破坏现存的宏，宁可引入新的说明符，也不能改变旧说明符的语义。

## 4. tt 令牌树深入

`tt` (token tree) 是最灵活的说明符。它要么匹配**单个非定界令牌**，要么匹配**一对配对定界符内的全部内容**：

```rust
macro_rules! tt_demo {
    // 捕获任意输入并逐令牌打印
    ($($tt:tt)*) => {
        tt_inner!($($tt)*)
    };
}

macro_rules! tt_inner {
    () => {};
    ($first:tt $($rest:tt)*) => {
        println!("令牌: {}", stringify!($first));
        tt_inner!($($rest)*);
    };
}

fn main() {
    tt_demo!(a + [b, c] { d } ( e ));
    // 输出四个令牌: a, +, [b, c], { d }, ( e )
}
```

### 4.1 tt 的定界符匹配

`tt` 识别三种配对定界符：`()`、`[]`、`{}`。当遇到左定界符时，tt 会一直匹配到对应的右定界符，包括嵌套结构：

```rust
macro_rules! create_vec {
    ($($item:tt),*) => {
        vec![$($item),*]
    };
}

fn main() {
    let v = create_vec!(1 + 2, { let x = 3; x * 2 }, [4, 5]);
    println!("{:?}", v); // [3, 6, [4, 5]]
}
```

> tt 的"万能"特性是双刃剑——它给了你最大的灵活性，但也让你失去了编译器的语法检查保护。

## 5. 复杂组合示例

### 5.1 ident + ty 生成 getter/setter

```rust
macro_rules! property {
    ($name:ident: $ty:ty) => {
        paste::paste! {
            fn [<get_ $name>](&self) -> &$ty {
                &self.$name
            }
            fn [<set_ $name>](&mut self, value: $ty) {
                self.$name = value;
            }
        }
    };
}
```

> ident 和 ty 的经典组合构成了 Rust 生态中最常见的宏模式之一——从简单属性到完整的 ORM 映射。

### 5.2 vis + ident + ty 生成结构体

```rust
macro_rules! define_struct {
    ($vis:vis struct $name:ident { $( $fname:ident: $fty:ty ),* $(,)? }) => {
        $vis struct $name {
            $( $fname: $fty ),*
        }

        impl $name {
            pub fn new($( $fname: $fty ),*) -> Self {
                Self { $( $fname ),* }
            }
        }
    };
}

define_struct!(pub struct Point { x: f64, y: f64, z: f64 });

fn main() {
    let p = Point::new(1.0, 2.0, 3.0);
    println!("({}, {}, {})", p.x, p.y, p.z);
}
```

### 5.3 pat + expr 生成 match 臂

```rust
macro_rules! match_one {
    ($input:expr, $pat:pat => $arm:expr) => {
        match $input {
            $pat => $arm,
            _ => None,
        }
    };
}

fn main() {
    let result = match_one!(Some(10), Some(x) if x > 5 => Some(x * 2));
    println!("{:?}", result); // Some(20)
}
```

### 5.4 模拟 vec! 宏

```rust
macro_rules! my_vec {
    () => {
        Vec::new()
    };
    ($elem:expr; $n:expr) => {
        std::vec::from_elem($elem, $n)
    };
    ($($x:expr),+ $(,)?) => {
        <[_]>::into_vec(Box::new([$($x),+]))
    };
}

fn main() {
    let v1 = my_vec![1, 2, 3];
    let v2 = my_vec![0; 10];
    let v3 = my_vec![];
    println!("{:?} {:?} {:?}", v1, v2, v3);
}
```

> 模拟标准宏是最好的学习方法——读懂 `vec!` 的源码，就等于掌握了声明宏 90% 的实际运用场景。

## 6. 宏调试技巧

### 6.1 stringify! 查看展开结果

```rust
macro_rules! debug_expand {
    ($($t:tt)*) => {
        println!("展开结果: {}", stringify!($($t)*));
    };
}
```

### 6.2 log_syntax! (nightly)

仅在 Nightly Rust 中可用，用于在编译时打印匹配到的语法：

```rust
#![feature(log_syntax)]

macro_rules! trace_match {
    ($x:expr) => {
        log_syntax!("匹配到表达式: {}", $x);
    };
}
```

### 6.3 trace_macros!

```rust
// 在模块或函数作用域内启用宏跟踪
fn main() {
    trace_macros!(true);
    // 此后每个宏展开都会打印到 stderr
    let v = vec![1, 2, 3];
    trace_macros!(false);
}
```

### 6.4 cargo expand

使用 `cargo-expand` 工具可以看到宏展开后的完整代码：

```bash
cargo install cargo-expand
cargo expand
cargo expand path::to::function  # 只看特定函数
```

> 调试宏的黄金法则：先 `cargo expand`，再 `compile_error!` 定位，最后用 `stringify!` 确认单个展开——这是从猜谜到实证的必经之路。

## 7. Edition 迁移注意事项

### 7.1 检测当前版本

```rust
// Cargo.toml
[package]
edition = "2021"  // 或 "2024"
```

### 7.2 pat → pat_param 迁移

Edition 2021+ 中，若宏在 `match` 臂以外的上下文中使用模式参数，应迁移到 `pat_param`：

```rust
// 旧版（仅 2015/2018）
macro_rules! fn_param {
    ($p:pat => $body:expr) => {
        |$p| $body
    };
}

// 新版（2021+）
macro_rules! fn_param {
    ($p:pat_param => $body:expr) => {
        |$p| $body
    };
}
```

### 7.3 版本门控

```rust
// 为不同 edition 提供不同的宏实现
#[cfg(edition_2021)]
macro_rules! pattern {
    ($p:pat_param) => { ... };
}

#[cfg(not(edition_2021))]
macro_rules! pattern {
    ($p:pat) => { ... };
}
```

> edition 迁移不是一次性的工作，而是持续的演进——用 cfg 门控可以让一个宏同时兼容多个版本。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 误在 pat 中使用 `\|`，但宏用于函数参数 | pat 在 2021+ 的函数参数位置被限制 | 函数参数用 `pat_param` |
| tt 匹配了超出预期的括号内容 | tt 将整个括号组当作一个令牌 | 对括号内容需要逐级展开，用 tt muncher |
| expr 后跟 `=>` 以外符号时匹配失败 | 每个说明符有合法跟随令牌白名单 | 查阅"允许跟随的片段"，使用正确的分隔符 |
| 在 2021 项目中含 `const {}` 的宏匹配失败 | Edition 2024 前 expr 不匹配 const 块 | 升级 edition 或避免在宏匹配中使用 const 块 |
| 宏展开后的错误信息指向调用处而非定义处 | 编译器在展开后阶段报告错误 | 用 `cargo expand` 查看展开结果，用 `compile_error!` 提前拦截 |
| 递归宏导致编译时间爆炸 | 每次递归都产生新的令牌流被再次解析 | 限制递归深度，或改用过程宏 |
| trace_macros! 输出过多无法阅读 | 每个宏展开都打印 | 仅在怀疑的代码区域启用，配合 `cargo expand` 使用 |

> **详见测试**: `tests/rust_features/18_macro_fragment_specifiers.rs`
