# 08 - 泛型编程

## 核心概念

泛型允许编写类型无关的代码：

### 泛型函数

```rust
fn generic<T>(x: T) -> T {
    x
}
```

### 泛型结构体

```rust
struct Point<T> {
    x: T,
    y: T,
}
```

### 泛型枚举

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

## Trait 约束

### Trait Bound

```rust
fn print<T: Debug>(x: T) {
    println!("{:?}", x);
}
```

### where 子句

```rust
fn process<T>(x: T)
where
    T: Debug + Clone,
{
    // ...
}
```

## 关联类型

```rust
trait Container {
    type Item;
    fn get(&self) -> Self::Item;
}
```

## 泛型常量

```rust
fn create_array<T, const N: usize>() -> [T; N] {
    [T::default(); N]
}
```

## 单元测试

详见 `tests/rust_features/08_generics.rs`
