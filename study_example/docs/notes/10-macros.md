# 10 - 宏与元编程

## 概述

宏是 Rust 中强大的元编程工具，它允许在编译时生成代码。与函数不同，宏可以在编译时接受任意形式的代码片段，并生成相应的代码。Rust 有两种主要宏：声明宏（Declarative Macros）和过程宏（Procedural Macros）。

## 声明宏 macro_rules!

```rust
macro_rules! my_macro {
    ($name:expr) => {
        println!("{}", $name);
    };
    ($name:expr, $($rest:expr),*) => {
        println!("{}", $name);
        $(my_macro!($rest);)*
    };
}

my_macro!("hello");
my_macro!("a", "b", "c");
```

### 片段说明符

| 说明符 | 匹配 |
|--------|------|
| `expr` | 表达式 |
| `stmt` | 语句 |
| `pat` / `pat_param` | 模式 |
| `ty` | 类型 |
| `ident` | 标识符 |
| `path` | 路径 |
| `block` | 代码块 |
| `tt` | Token Tree |
| `meta` | 属性内容 |

### 重复修饰符

- `$($item:expr),*` - 逗号分隔，0或多次
- `$($item:expr);*` - 分号分隔，0或多次
- `$($item:expr)+*` - 至少一次

```rust
macro_rules! sum {
    ($($x:expr),*) => {{
        0 $( + $x)*
    }};
}
```

## 内置宏

```rust
// 格式化
println!("{}", value);
format!("{:.2}", 3.14);

// 容器创建
vec![1, 2, 3];
vec![0; 10];

// 调试
panic!("error");
assert!(condition);
assert_eq!(a, b);

// 元数据
stringify!(expr);
concat!("a", "b");
env!("PATH");
option_env!("DEBUG");

// 文件包含
include!("file.rs");
include_str!("file.txt");
```

## derive 宏

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
```

## 卫生性 (Hygiene)

宏展开时，宏内部的标识符不会与外部冲突：

```rust
macro_rules! foo {
    () => {
        let x = 42;
    };
}

fn main() {
    foo!();
    // x 在这里不可用，不会与外部 x 冲突
}
```

## 过程宏简介

### 函数式宏

```rust
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    // 处理输入，生成输出
}
```

### 属性式宏

```rust
#[proc_macro_attribute]
pub fn my_attr(args: TokenStream, item: TokenStream) -> TokenStream {
    // 处理属性和条目
}
```

### derive 宏

```rust
#[proc_macro_derive(MyTrait)]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    // 为类型派生 trait 实现
}
```

## cargo-expand 调试工具

```bash
cargo install cargo-expand
cargo expand > expanded.rs
```

可以看到宏展开后的实际代码。

## 避坑指南

1. **优先使用函数**：简单操作优先用函数而非宏
2. **谨慎使用宏**：宏复杂且难以调试
3. **卫生性**：了解宏的卫生性规则
4. **类型安全**：宏不提供类型检查

## 单元测试

详见 `tests/rust_features/10_macros.rs`

## 参考资料

- [Rust Macros](https://doc.rust-lang.org/book/ch19-06-macros.html)
- [Rust By Example - Macros](https://doc.rust-lang.org/rust-by-example/macros.html)
- [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/)