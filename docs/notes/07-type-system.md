# 07 - 类型系统与 Trait

## 概述

Rust 的类型系统是其最重要的特性之一，而 trait 是 Rust 实现多态和接口抽象的核心机制。理解 trait 和类型系统对于编写优雅、安全的 Rust 代码至关重要。

## Trait 定义与实现

### 基本语法

```rust
trait MyTrait {
    fn method(&self) -> i32;  // 方法签名
    
    fn default_method(&self) -> i32 {
        42  // 默认实现
    }
}

struct MyType;
impl MyTrait for MyType {
    fn method(&self) -> i32 { 10 }
}
```

### Trait 对象 dyn Trait

```rust
let obj: &dyn MyTrait = &MyType;
let boxed: Box<dyn MyTrait> = Box::new(MyType);
```

**Fat Pointer**：trait 对象指针包含两个指针：
- 数据指针：指向具体类型的数据
- vtable 指针：指向虚表（方法表）

```rust
// &dyn Trait 是 16 字节 (两个 usize)
struct DynRef {
    data: *const (),
    vtable: *const (),
}
```

## 对象安全规则

Trait 要能作为 dyn Trait 使用，必须满足以下条件：

1. **不能有泛型方法**
2. **不能返回 Self**
3. **不能有 `self: Sized` 约束**

可以用 `where Self: Sized` 标记非对象安全方法：

```rust
trait MyTrait {
    // 对象安全
    fn method(&self) -> i32;
    
    // 非对象安全，需要 Self: Sized
    fn generic_method<T>(&self, _x: T) -> i32
    where
        Self: Sized,
    {
        0
    }
}
```

## 静态 Dispatch vs 动态 Dispatch

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

impl Animal for Cat {
    fn speak(&self) { println!("Meow!"); }
}

// 静态 dispatch - 编译时确定类型
fn static_dispatch<T: Animal>(animal: &T) {
    animal.speak();  // 编译时内联，无运行时开销
}

// 动态 dispatch - 运行时确定
fn dynamic_dispatch(animal: &dyn Animal) {
    animal.speak();  // 通过 vtable 查找，有轻微开销
}
```

**选择建议**：默认使用静态 dispatch（泛型），仅在需要运行时多态时使用 dyn Trait。

## 关联类型

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter {
    current: i32,
}

impl Iterator for Counter {
    type Item = i32;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        Some(self.current)
    }
}
```

关联类型 vs 泛型参数：
- 关联类型：一个 trait 只有一种关联类型
- 泛型参数：一个 trait 可以有多种类型

## 上转型 (Upcasting)

Rust 1.86+ 支持子 trait 对象自动转为超 trait 对象：

```rust
trait Super { fn method(&self) -> i32; }
trait Sub: Super { fn sub_method(&self) -> i32; }

struct MyStruct;
impl Super for MyStruct { fn method(&self) -> 10 }
impl Sub for MyStruct { fn sub_method(&self) -> 20 }

let sub: &dyn Sub = &MyStruct;

// 隐式 upcast
let super_ref: &dyn Super = sub;

// 显式 upcast (Rust 1.86+)
let explicit: &dyn Super = sub as &dyn Super;
```

## Trait 约束进阶

### 高阶 Trait Bounds (HRTB)

```rust
fn call_fn<F>(f: F)
where
    for<'a> F: Fn(&'a str) -> &str,
{
    // 'for<'a>' 表示任意生命周期
}
```

### Trait Objects with Bounds

```rust
fn process<T: Animal + Clone>(animal: &T) { }
// 或者
fn process(animal: &(dyn Animal + Clone)) { }
```

## 常用标准库 Trait

### Clone / Copy

```rust
#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}
```

### Debug / Display

```rust
#[derive(Debug)]
struct Point { x: i32, y: i32 }

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

### Default

```rust
#[derive(Default)]
struct Config {
    host: String,
    port: u16,
}
```

### From / Into

```rust
impl From<String> for MyError {
    fn from(s: String) -> Self {
        MyError(s)
    }
}

fn foo() -> Result<(), MyError> {
    let s = String::from("error");
    // ? 会自动调用 Into
    Err(s.into())?
}
```

## 避坑指南

1. **优先使用静态 dispatch**：dyn Trait 有运行时开销
2. **对象安全规则**：不满足的 trait 方法需标记 `where Self: Sized`
3. **trait 对象大小**：动态分发的对象指针是 16 字节，不是 8 字节
4. **Self 返回类型**：不能用于 dyn Trait 的方法

## 单元测试

详见 `tests/rust_features/07_type_system.rs`

## 参考资料

- [Rust Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Rust 对象安全的 trait](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
- [Understanding Rust Traits](https://medium.com/@ali_alachkar/rust-traits-deep-dive)