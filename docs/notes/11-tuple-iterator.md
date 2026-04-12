# 11 - 元组与迭代器高级操作

## 概述

本笔记介绍 Rust 中迭代器的高级操作，特别是元组相关的收集操作。这些功能在 Rust 1.85+ 版本中得到显著增强。

## FromIterator

```rust
impl FromIterator<(A, B)> for (Vec<A>, Vec<B>) {
    fn from_iter<T: IntoIterator<Item = (A, B)>>(iter: T) -> Self {
        let mut a = Vec::new();
        let mut b = Vec::new();
        for (av, bv) in iter {
            a.push(av);
            b.push(bv);
        }
        (a, b)
    }
}
```

## unzip

将元组迭代器拆分为两个集合：

```rust
let pairs = vec![(1, "a"), (2, "b"), (3, "c")];
let (nums, chars): (Vec<i32>, Vec<&str>) = pairs.into_iter().unzip();
```

## partition

按条件分为两个集合：

```rust
let nums = vec![1, 2, 3, 4, 5, 6];
let (evens, odds): (Vec<i32>, Vec<i32>) = nums
    .into_iter()
    .partition(|x| x % 2 == 0);
// evens: [2, 4, 6], odds: [1, 3, 5]
```

## partition_in_place (Rust 1.85+)

原地分区：

```rust
let mut data = [3, 1, 4, 1, 5, 9, 2, 6];
let pivot = data.partition_in_place(|&x| x < 5);
```

## reduce vs fold

| 特性 | reduce | fold |
|------|--------|------|
| 初始值 | 无 | 需要 |
| 空迭代器 | 返回 None | 返回初始值 |

```rust
let nums = vec![1, 2, 3, 4, 5];
let sum = nums.iter().fold(0, |acc, x| acc + x);  // 15
let sum = nums.iter().copied().reduce(|a, b| a + b);  // Some(15)
```

## try_fold / try_reduce

处理可能失败的归约：

```rust
let result = (1..5).try_fold(0i32, |acc, x| {
    if x == 3 { Err("stop") } else { Ok(acc + x) }
});
```

## 元组收集 (Rust 1.85+)

```rust
let (a, b): (Vec<i32>, Vec<i32>) = (0..5)
    .map(|i| (i, i * 2))
    .collect();
```

## 单元测试

详见 `tests/rust_features/11_tuple_iterator.rs`

## 参考资料

- [Rust Iterator docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [Rust 1.85 Release Notes](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)