# 14 - 生命周期与 RPIT

## 概述

生命周期是 Rust 防止悬垂引用的核心机制。通过生命周期标注，编译器可以确保引用在使用的期间始终有效。

## 生命周期标注

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

`'a` 表示参数和返回值引用相同的生命周期。

## 生命周期消除规则

1. 每个引用参数有自己的生命周期
2. 只有一个输入生命周期时，它赋给所有输出生命周期
3. 方法的 `&self` 输入生命周期赋给所有输出生命周期

## 结构体中的生命周期

```rust
struct Excerpt<'a> {
    part: &'a str,
}
```

## 'static

静态生命周期存活整个程序：

```rust
static CONST: &str = "hello";

fn with_static() -> &'static str {
    CONST
}
```

## HRTB (Higher-Ranked Trait Bounds)

```rust
fn call_fn<F>(f: F)
where
    for<'a> F: Fn(&'a str) -> &str,
{
    // 对任意生命周期都有效
}
```

## RPIT (impl Trait in return position)

```rust
fn foo() -> impl Trait {
    // 返回具体类型隐藏
}

fn bar() -> impl Trait + '_ {
    // 使用 '_
}
```

## 单元测试

详见 `tests/rust_features/14_lifetimes_and_rpit.rs`

## 参考资料

- [Rust Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [HRTB RFC](https://rust-lang.github.io/rfcs/0387-higher-ranked-trait-bounds.html)