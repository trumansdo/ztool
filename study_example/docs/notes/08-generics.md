# 08 - 泛型编程

## 概述

泛型允许编写类型无关的代码，这是 Rust 实现代码复用和抽象的核心机制。Rust 的泛型是零成本抽象——在编译时通过单态化（monomorphization）生成具体类型，不会有运行时开销。

## 泛型函数

```rust
fn generic<T>(x: T) -> T {
    x
}

fn largest<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut largest = None;
    for item in list {
        match largest {
            None => largest = Some(item),
            Some(l) if item > l => largest = Some(item),
            _ => {}
        }
    }
    largest
}
```

## 泛型结构体

```rust
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// 具体类型方法实现
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

## 泛型枚举

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Either<L, R> {
    Left(L),
    Right(R),
}
```

## Trait 约束

### Trait Bound 语法

```rust
fn print<T: Debug>(x: T) {
    println!("{:?}", x);
}

fn notify(item: &impl Summary) {
    println!("Breaking: {}", item.summarize());
}
```

### where 子句

当约束较多时，where 子句更清晰：

```rust
fn process<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Debug,
{
    format!("{:?} {}", t, u)
}
```

### 多重约束

```rust
fn process<T: Clone + Debug>(x: T) { }

// 或者
fn process<T>(x: T)
where
    T: Clone + Debug,
{ }
```

### 返回类型约束

```rust
fn returns_summarizable() -> impl Summary {
    Tweet { ... }
}
```

## 关联类型

```rust
trait Container {
    type Item;
    fn get(&self) -> Self::Item;
    fn add(&mut self, item: Self::Item);
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T> Container for Stack<T> {
    type Item = T;
    
    fn get(&self) -> Self::Item {
        self.items.last().cloned().unwrap()
    }
    
    fn add(&mut self, item: Self::Item) {
        self.items.push(item);
    }
}
```

关联类型 vs 泛型参数：
- 关联类型：一个 trait 只有一种关联类型
- 泛型参数：一个 trait 可以有多种类型

## 泛型常量 (const generics)

```rust
fn create_array<T, const N: usize>() -> [T; N] {
    [T::default(); N]
}

struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

impl<T: Default + Copy, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    fn zero() -> Self {
        Matrix { data: [[T::default(); COLS]; ROWS] }
    }
}
```

### 泛型数组方法

```rust
impl<T, const N: usize> [T; N] {
    fn as_slice(&self) -> &[T] {
        self
    }
}
```

## PhantomData 幽灵类型

用于标记泛型但不直接存储的类型：

```rust
use std::marker::PhantomData;

struct Wrapper<T, A> {
    data: T,
    _marker: PhantomData<A>,
}

fn main() {
    let w: Wrapper<i32, &'static str> = Wrapper {
        data: 42,
        _marker: PhantomData,
    };
}
```

## 高阶 Trait Bounds (HRTB)

```rust
fn apply<F, T>(func: F, value: T) -> T
where
    F: Fn(T) -> T,
{
    func(value)
}

// for<'a> 表示任意生命周期
fn with_lifetime<F>(f: F)
where
    for<'a> F: Fn(&'a str) -> &'a str,
{
    println!("{}", f("hello"));
}
```

## 单态化 (Monomorphization)

Rust 编译器会将泛型代码在编译时生成为具体类型：

```rust
fn main() {
    let integers = vec![1, 2, 3];
    let floats = vec![1.0, 2.0, 3.0];
    
    let i = integers.first();  // 生成: Option<&i32>
    let f = floats.first();   // 生成: Option<&f64>
}
```

**零成本**：单态化没有运行时开销，与手写具体类型代码性能相同。

## 避坑指南

1. **泛型约束**：确保泛型参数有足够约束完成所需操作
2. **关联类型 vs 泛型**：关联类型适用于固定关联，泛型适用于多种关联
3. **const generics**：编译时计算，可用于数组大小
4. **单态化**：编译器会为每种具体类型生成代码，可能增加编译时间

## 单元测试

详见 `tests/rust_features/08_generics.rs`

## 参考资料

- [Rust Generics](https://doc.rust-lang.org/book/ch10-01-generics.html)
- [Const Generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics)
- [Rust Monomorphization](https://duart.io/rust-monomorphization-explained/)