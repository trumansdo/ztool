# 05 - 迭代器

## 概述

迭代器是 Rust 中最强大的工具之一，它提供了一种惰性处理序列数据的方式。与其他语言中的迭代器不同，Rust 的迭代器不仅是一种设计模式，更是一种零成本抽象——编译器会将其优化为高效的机器码。迭代器的核心思想是将遍历逻辑与数据处理逻辑分离，使得代码更加声明式和组合式。通过链式调用适配器方法，我们可以构建复杂的数据转换管道，而无需显式编写循环逻辑。

## 核心概念

迭代器的核心是一个 trait，它定义了一种按顺序访问元素的方式：

```rust
trait Iterator {
    type Item;  // 关联类型，定义迭代器返回的元素类型
    
    fn next(&mut self) -> Option<Self::Item>;  // 核心方法，返回下一个元素
}
```

每次调用 `next` 方法时，迭代器会返回 `Option` 类型的内容——如果还有元素则返回 `Some(element)`，如果已经遍历完毕则返回 `None`。

**关键特性：惰性求值**

迭代器是**惰性的**，这意味着仅仅创建迭代器或调用适配器方法并不会执行任何操作。只有当我们调用终端操作（terminal operation）时，迭代器才会真正开始工作。

## 适配器方法详解

### 转换类适配器

**map** - 转换元素：
```rust
let numbers = vec![1, 2, 3, 4, 5];
let squares: Vec<i32> = numbers.iter().map(|x| x * x).collect();
// 结果: [1, 4, 9, 16, 25]
```

**filter** - 过滤元素：
```rust
let numbers = vec![1, 2, 3, 4, 5, 6];
let even: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();
// 结果: [&2, &4, &6]
```

**flatten** - 展平嵌套：
```rust
let nested = vec![Some(1), None, Some(3)];
let flattened: Vec<i32> = nested.into_iter().flatten().collect();
// 结果: [1, 3]
```

### 切片类适配器

**take / skip** - 取/跳过元素：
```rust
let numbers: Vec<i32> = (0..100).collect();
let first_five: Vec<i32> = numbers.iter().copied().take(5).collect();
let rest: Vec<i32> = numbers.iter().copied().skip(5).collect();
```

**enumerate** - 附加索引：
```rust
for (index, value) in vec!["a", "b", "c"].iter().enumerate() {
    println!("{}: {}", index, value);
}
```

**zip** - 合并两个迭代器：
```rust
let names = vec!["Alice", "Bob"];
let scores = vec![85, 92];
let combined: Vec<(&str, i32)> = names.iter().zip(scores.iter()).collect();
```

**chain** - 链接迭代器：
```rust
let first = vec![1, 2, 3];
let second = vec![4, 5, 6];
let combined: Vec<i32> = first.iter().chain(second.iter()).copied().collect();
```

## 终端操作

**collect** - 收集到集合：
```rust
use std::collections::{HashMap, HashSet};

let vec: Vec<i32> = numbers.iter().copied().collect();
let set: HashSet<i32> = numbers.iter().copied().collect();
let map: HashMap<&str, i32> = pairs.into_iter().collect();
```

**fold** - 归约：
```rust
let numbers = vec![1, 2, 3, 4, 5];
let sum = numbers.iter().fold(0, |acc, x| acc + x);  // 15
let product = numbers.iter().fold(1, |acc, x| acc * x);  // 120
```

**reduce** - 类似 fold，但返回 Option：
```rust
let max = numbers.iter().copied().reduce(|a, b| if a > b { a } else { b });
```

**find / position** - 查找：
```rust
let first_even = numbers.iter().find(|x| **x % 2 == 0);
let position = numbers.iter().position(|x| *x == 4);
```

**any / all** - 布尔判断：
```rust
let has_even = numbers.iter().any(|x| *x % 2 == 0);
let all_positive = numbers.iter().all(|x| *x > 0);
```

## 创建自定义迭代器

```rust
struct Counter {
    count: u32,
    max: u32,
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
```

## 性能优化

### 零成本抽象

Rust 的迭代器是零成本抽象：
- 编译器进行单态化（monomorphization）
- 迭代器方法通常会被内联
- 不会产生额外的运行时开销

### 优化技巧

```rust
// 使用 by_ref() 借用迭代器而非消费
let mut iter = numbers.iter();
let first_three: Vec<&i32> = iter.by_ref().take(3).collect();
let rest: Vec<&i32> = iter.collect();

// 使用 copied()/cloned() 避免多次 borrow
let numbers = vec![&1, &2, &3];
let owned: Vec<i32> = numbers.iter().copied().collect();
```

## 避坑指南

1. **惰性求值**：迭代器方法不执行，直到调用终端操作
2. **迭代方式选择**：
   - `iter()` - 不可变借用
   - `iter_mut()` - 可变借用
   - `into_iter()` - 获取所有权
3. **collect 类型标注**：使用 `collect()` 时需要指定目标类型
4. **避免不必要克隆**：优先使用引用而非克隆

## 单元测试

详见 `tests/rust_features/05_iterators.rs`

## 参考资料

- [Rust Iterator Documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [Working with Iterator Chains in Rust](https://dev.to/francescoxx/iterators-in-rust-2o0b)
- [Rust Iterators Beyond the Basics](https://codeforgeek.com/map-filter-fold-in-rust/)
- [Effective Rust - Item 9: Prefer Iterators Over Loops](https://www.lurklurk.org/effective-rust/iterators.html)