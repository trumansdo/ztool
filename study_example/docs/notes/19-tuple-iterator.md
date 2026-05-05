# 元组与迭代器高级操作

> 迭代器是 Rust 最成功的抽象之一——它将"遍历"从一个具体的动作变成了一组可组合的代数运算，元组收集正是这种组合能力的集中体现。

## 1. unzip：解耦配对为两个集合

`Iterator::unzip` 将一个包含二元组的迭代器拆分为两个独立的集合，只要元素类型实现了 `Extend` trait：

```rust
fn main() {
    // 基本用法：拆成两个 Vec
    let pairs = vec![(1, "一"), (2, "二"), (3, "三")];
    let (numbers, words): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    println!("数字: {:?}, 文字: {:?}", numbers, words);
    // 数字: [1, 2, 3], 文字: ["一", "二", "三"]
}
```

> unzip 的灵魂在于类型标注——你必须显式指定两个目标集合的类型，Rust 的类型推导在这里并非万能。

### 1.1 不同类型的目标集合

```rust
use std::collections::{BTreeSet, LinkedList};

fn main() {
    let data = vec![(3, "c"), (1, "a"), (2, "b")];

    // 左侧收集到 BTreeSet（自动排序去重）
    let (sorted_keys, values): (BTreeSet<_>, Vec<_>) = data.into_iter().unzip();
    println!("排序键: {:?}", sorted_keys); // {1, 2, 3}

    let data2 = vec![(1, "x"), (2, "y")];
    // 左侧收集到 LinkedList
    let (list, vec): (LinkedList<_>, Vec<_>) = data2.into_iter().unzip();
    println!("链表: {:?}, 向量: {:?}", list, vec);
}
```

### 1.2 从范围生成并收集

```rust
fn main() {
    // 生成 (索引, 平方) 配对
    let (indices, squares): (Vec<_>, Vec<_>) = (0..10)
        .map(|x| (x, x * x))
        .unzip();

    println!("索引: {:?}", indices);
    println!("平方: {:?}", squares);
}
```

> 迭代器的生成-转换-收集三部曲，配合 unzip 能让一次性生成两种关联数据集合的代码简洁到只有两行。

## 2. partition：二分集合

`Iterator::partition` 根据一个谓词将迭代器分为两个满足/不满足条件的集合：

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let (evens, odds): (Vec<i32>, Vec<i32>) = numbers
        .into_iter()
        .partition(|&x| x % 2 == 0);

    println!("偶数: {:?}", evens); // [2, 4, 6, 8, 10]
    println!("奇数: {:?}", odds);  // [1, 3, 5, 7, 9]
}
```

> partition 是遍历判断的"一分为二"——对于需要将数据按条件分流到两个集合的场景，它比两次 filter 更简洁也更高效。

### 2.1 partition 的类型灵活性

```rust
use std::collections::{VecDeque, HashSet};

fn main() {
    let words = ["apple", "banana", "apricot", "blueberry", "avocado"];

    // partition 到不同类型的集合
    let (starts_with_a, others): (VecDeque<_>, HashSet<_>) = words
        .iter()
        .partition(|w| w.starts_with('a'));

    println!("以a开头: {:?}", starts_with_a);
    println!("其他: {:?}", others);
}
```

## 3. try_fold：可短路的折叠

`try_fold` 是 `fold` 的可控中断版本，在每次迭代中返回 `Result`/`Option`，遇到 `Err` 或 `None` 时立即停止：

```rust
fn main() {
    let values = [1, 2, 3, 0, 5, 6];

    // 累积求和，遇到 0 时短路
    let result: Result<i32, &str> = values
        .iter()
        .try_fold(0, |acc, &x| {
            if x == 0 {
                Err("遇到了零!")
            } else {
                Ok(acc + x)
            }
        });

    println!("{:?}", result); // Err("遇到了零!")
}
```

> try_fold 是迭代器中的"止损线"——它在 scan、filter_map 均无法表达"遇到异常值即整体失败"语义时横空出世。

### 3.1 try_fold 实现条件累积

```rust
fn main() {
    // 累积到总和超过 100 或迭代结束
    let numbers = vec![20, 30, 40, 50, 60];
    let sum = numbers
        .iter()
        .try_fold(0u32, |acc, &x| {
            let new_sum = acc + x;
            if new_sum > 100 {
                Err(new_sum) // 用 Err 携带最终结果
            } else {
                Ok(new_sum)
            }
        });

    match sum {
        Ok(total) => println!("总和: {}（未超过100）", total),
        Err(total) => println!("总和: {}（已超过100）", total),
    }
}
```

## 4. fold 类型变换

`fold` 允许在迭代过程中改变累积值的类型：

```rust
#[derive(Debug)]
struct Stats {
    sum: i64,
    count: usize,
    max: Option<i64>,
    min: Option<i64>,
}

fn main() {
    let data = [5, 12, -3, 8, 100, -50];

    let stats = data.iter().fold(Stats {
        sum: 0,
        count: 0,
        max: None,
        min: None,
    }, |mut acc, &x| {
        acc.sum += x;
        acc.count += 1;
        acc.max = Some(acc.max.map_or(x, |m| m.max(x)));
        acc.min = Some(acc.min.map_or(x, |m| m.min(x)));
        acc
    });

    println!("统计结果: {:?}", stats);
    // Stats { sum: 72, count: 6, max: Some(100), min: Some(-50) }
}
```

> fold 的类型变换能力使它可以完成"从一个种子值生长出完全不同的数据结构"的任务，这是 map/reduce 范式的最强体现。

## 5. FromIterator trait

`collect` 方法的底层实现依赖于 `FromIterator` trait：

```rust
pub trait FromIterator<A>: Sized {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self;
}
```

```rust
// 手工实现一个可 collect 的类型
#[derive(Debug)]
struct EvenNumbers {
    nums: Vec<i32>,
}

impl std::iter::FromIterator<i32> for EvenNumbers {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        let nums = iter.into_iter()
            .filter(|&x| x % 2 == 0)
            .collect();
        EvenNumbers { nums }
    }
}

fn main() {
    let evens: EvenNumbers = (1..=10).collect();
    println!("{:?}", evens);
}
```

> FromIterator 是 collect 背后的隐式契约——任何类型只要实现了这个 trait，就能成为迭代器终点站。

## 6. Extend trait

`Extend` 允许将迭代器的内容批量追加到现有集合中：

```rust
pub trait Extend<A> {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T);
}

fn main() {
    let mut v = vec![1, 2, 3];
    v.extend(4..=6);
    v.extend([7, 8, 9]);

    println!("{:?}", v); // [1, 2, 3, 4, 5, 6, 7, 8, 9]
}
```

> extend 不是"复制数据"，而是"持续生长"——配合迭代器的惰性求值，它构成了 Rust 数据管道模型的基础。

## 7. 组合特性联用

### 7.1 map + filter + partition

```rust
fn main() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let (doubled_evens, others): (Vec<_>, Vec<_>) = data
        .into_iter()
        .map(|x| x * 2)
        .partition(|x| x % 4 == 0);

    println!("偶数加倍后整除4的: {:?}", doubled_evens);
    println!("其余的: {:?}", others);
}
```

### 7.2 filter_map + collect + unzip

```rust
fn main() {
    let mixed = vec!["1", "two", "3", "four", "5"];

    let (numbers, strings): (Vec<i32>, Vec<&str>) = mixed
        .into_iter()
        .filter_map(|s| {
            s.parse::<i32>().ok()
                .map(|n| (n, s))
        })
        .unzip();

    println!("成功解析的: {:?}", numbers);
    println!("所有字符串: {:?}", strings);
}
```

### 7.3 enumerate + partition

```rust
fn main() {
    let chars: Vec<char> = "abcdefghij".chars().collect();

    // 按索引奇偶分流
    let (even_idx, odd_idx): (Vec<_>, Vec<_>) = chars
        .into_iter()
        .enumerate()
        .partition(|(i, _)| i % 2 == 0);

    println!("偶数索引元素: {:?}", even_idx);
    println!("奇数索引元素: {:?}", odd_idx);
}
```

> 迭代器的组合可以看作管道编程——每个适配器是一节管子，数据从中流过，最终在 collect 处凝结成型。

## 8. itertools 的多元组收集

标准库仅支持二元组的 `unzip`。itertools crate 提供了多元组的收集能力：

```rust
use itertools::Itertools;

fn main() {
    let data = vec![
        (1, "a", true),
        (2, "b", false),
        (3, "c", true),
    ];

    let (nums, chars, flags): (Vec<_>, Vec<_>, Vec<_>) = data
        .into_iter()
        .multiunzip();

    println!("数字: {:?}", nums);
    println!("字符: {:?}", chars);
    println!("标志: {:?}", flags);
}
```

> 标准库没有做多元组 collect 不是遗漏，而是权衡——多元组的通用实现需要变长类型参数的 trait，这超出了 Rust 当前trait系统的能力。

## 9. 空迭代器边界

```rust
fn main() {
    // 空迭代器上的操作不会panic
    let empty: Vec<i32> = vec![];

    let result: Option<i32> = empty.iter().max().copied();
    println!("空集合最大值: {:?}", result); // None

    let sum: i32 = empty.iter().sum();
    println!("空集合求和: {}", sum); // 0 (sum 默认值)

    let (v1, v2): (Vec<_>, Vec<_>) = empty.into_iter().unzip();
    println!("空集合 unzip: {:?} {:?}", v1, v2); // [] []

    let (evens, odds): (Vec<i32>, Vec<i32>) = (0..0).partition(|_| true);
    println!("空范围 partition: {:?} {:?}", evens, odds); // [] []
}
```

> 空集合上的迭代器操作必须优雅——返回默认值或空集合，而非崩溃。这是 Rust 迭代器设计的一条无声纪律。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| unzip 忘记类型标注导致编译错误 | 编译器无法推断两个目标集合的类型 | 显式写出 `(Vec\<_\>, Vec\<_\>)` 或等价的类型标注 |
| try_fold 返回值类型不匹配 | try_fold 要求每次迭代返回 `Result<T, E>` 或 `Option<T>` | 统一返回类型，或者在闭包内部做类型转换 |
| 对空迭代器调用 max 返回 None 后 unwrap | `max()` 返回 `Option<T>`，空迭代器返回 `None` | 使用 `unwrap_or(default)` 或显式处理 `None` 情况 |
| partition 和 filter 两次的语义混淆 | partition 一次性分成两份，filter 两次需要迭代两次 | 需要两个集合时用 partition，需要丢弃数据时用 filter |
| fold 初始化值类型决定了最终返回值类型 | fold 的累积值类型在每次迭代中不变 | 如果需要类型变换，确保每次迭代都返回期望的累积类型 |
| 自定义 FromIterator 实现未考虑容量预分配 | 频繁的重新分配降低性能 | 使用 `size_hint()` 预分配容量 |
| itertools multiunzip 依赖外部库 | 标准库只支持二元组 unzip | 评估是否真的需要多元组收集，或用其他方式替代 |

> **详见测试**: `tests/rust_features/19_tuple_iterator.rs`
