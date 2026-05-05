# Rust 迭代器：惰性求值与零成本抽象

## 一、Iterator Trait 核心

> **金句引用**："迭代器是 Rust 的血管——无开销、可组合、惰性求值。"

```rust
pub trait Iterator {
    type Item;                          // 关联类型：迭代产出的元素类型
    fn next(&mut self) -> Option<Self::Item>;  // 核心方法：每次调用推进迭代器
    // ... 74个自带默认实现的方法
}
```

**惰性求值**：迭代器本身不做事，只有调用消费者方法时才执行计算：

```rust
let v = vec![1, 2, 3, 4, 5];
let iter = v.iter().map(|x| x * 2);  // 什么都没发生，只是构建了管道
let result: Vec<i32> = iter.collect(); // collect 触发全部计算
```

---

## 二、IntoIterator Trait：三种迭代方式

> **金句引用**："`iter()` 借阅不带走，`into_iter()` 搬家不留根，`iter_mut()` 改写当场变。"

```rust
// for 循环语法糖展开：
// for x in collection { ... }
//   ↓ 等价于 ↓
// let mut iter = IntoIterator::into_iter(collection);
// while let Some(x) = iter.next() { ... }

// 三种实现——分别对应消耗/借用/可变借用
impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> IntoIter<T> { ... }  // 消耗 Vec，产出 T
}

impl<'a, T> IntoIterator for &'a Vec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> { ... }  // 借用，产出 &T
}

impl<'a, T> IntoIterator for &'a mut Vec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;
    fn into_iter(self) -> IterMut<'a, T> { ... } // 可变借用，产出 &mut T
}
```

**方法对照**：

| 方法 | 获取方式 | for 循环等效 | 原始集合状态 |
|------|----------|-------------|-------------|
| `into_iter()` | 消耗所有权 | `for x in vec` | 集合被移动，不可再用 |
| `iter()` | 不可变借用 | `for x in &vec` | 集合仍可使用 |
| `iter_mut()` | 可变借用 | `for x in &mut vec` | 集合仍可使用 |

---

## 三、迭代器适配器（不消耗，构建管道）

> **金句引用**："适配器筑管道，消费者开阀门——中间步骤零计算。"

| 适配器 | 功能 | 惰性特性 |
|--------|------|----------|
| `map` | 逐元素变换 | 不改变元素数量 |
| `filter` | 按条件筛选 | 可能减少 |
| `filter_map` | 筛选+变换一步完成 | 丢掉 None 值 |
| `take(n)` | 取前 n 个 | 提前终止 |
| `skip(n)` | 跳过前 n 个 | 延迟开始 |
| `step_by(n)` | 每 n 个取一个 | 跨度步进 |
| `chain` | 拼接两个迭代器 | 先遍历第一个，再第二个 |
| `enumerate` | 附加索引 `(i, val)` | 逐元素 |
| `zip` | 配对两个迭代器 | 最短者决定长度 |
| `peekable` | 预窥下一个元素 | 不消费 |
| `scan` | 带状态变换 | 可早期终止 |
| `flat_map` | 展平嵌套 | 多产一 |
| `inspect` | 旁路观察（调试用） | 不修改流 |
| `copied` | 复制 `&T` → `T`（要求 Copy） | 去引用 |
| `cloned` | 克隆 `&T` → `T`（要求 Clone） | 去引用 |
| `fuse` | 确保 None 后再调用返回 None | 标准化终止 |
| `rev` | 反向迭代（需 DoubleEndedIterator） | 反转流 |
| `cycle` | 无限循环（需 Clone） | 永不终止 |

### 代码示例

```rust
let nums = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let result: Vec<i32> = nums.iter()
    .filter(|&&x| x % 2 == 0)       // 过滤偶数
    .map(|&x| x * x)                 // 平方
    .take(3)                         // 取前3个
    .collect();
assert_eq!(result, vec![4, 16, 36]);

// peekable：预看下一个
let mut iter = vec![1, 2, 3].into_iter().peekable();
assert_eq!(iter.peek(), Some(&1));  // 窥视不消费
assert_eq!(iter.next(), Some(1));   // 消费

// flat_map：展平嵌套
let words = vec!["hello", "world"];
let chars: Vec<char> = words.iter()
    .flat_map(|s| s.chars())
    .collect();
assert_eq!(chars, vec!['h','e','l','l','o','w','o','r','l','d']);

// scan：带累加器的变换
let fib: Vec<i32> = (0..10).scan((0, 1), |state, _| {
    let next = state.0;
    *state = (state.1, state.0 + state.1);
    Some(next)
}).collect();
assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);

// chain + enumerate
let a = vec![1, 2];
let b = vec![3, 4];
let chained: Vec<(usize, &i32)> = a.iter().chain(b.iter()).enumerate().collect();
assert_eq!(chained, vec![(0, &1), (1, &2), (2, &3), (3, &4)]);

// zip：最短者截断
let ids = vec![1, 2, 3];
let vals = vec!["a", "b", "c", "d"];
let pairs: Vec<_> = ids.iter().zip(vals.iter()).collect();
assert_eq!(pairs, vec![(&1, &"a"), (&2, &"b"), (&3, &"c")]);

// step_by
let every_other: Vec<i32> = (1..=10).step_by(3).collect();
assert_eq!(every_other, vec![1, 4, 7, 10]);
```

---

## 四、迭代器消费者（消耗迭代器，释放管道）

> **金句引用**："消费者是末端——一旦调用，管道开始流动，数据应声而出。"

| 消费者 | 返回值 | 行为 |
|--------|--------|------|
| `collect` | 任意集合 | 收集所有元素到目标容器 |
| `fold` | 任意类型 | 有初值的归并 |
| `reduce` | Option | 无初值的归并（首元为初始值） |
| `sum` | 数值 | 求和（需 Sum trait） |
| `count` | usize | 计数（不查看元素） |
| `min` / `max` | Option | 极值 |
| `any` | bool | 存在性（短路） |
| `all` | bool | 全称性（短路） |
| `find` | Option | 查找第一个满足条件的 |
| `position` | Option\<usize\> | 查找第一个满足条件的索引 |
| `for_each` | () | 完全消费 |
| `try_fold` | Result/Option | 可短路的归并 |
| `partition` | (Vec, Vec) | 二分 |
| `unzip` | 两个集合 | 将元组拆解为两个集合 |
| `nth` | Option | 跳过 n-1 个，取第 n 个 |
| `last` | Option | 取最后一个（完全消费） |

```rust
let nums = vec![1, 2, 3, 4, 5];

// fold：自定义归并
let sum = nums.iter().fold(0, |acc, x| acc + x);  // 15
// reduce：无初值归并
let product = nums.iter().copied().reduce(|a, b| a * b); // Some(120)

// try_fold：可短路
let result: Result<i32, &str> = nums.iter().try_fold(0, |acc, &x| {
    if x > 3 { Err("超过3") } else { Ok(acc + x) }
});
assert_eq!(result, Err("超过3"));

// partition：二分
let (evens, odds): (Vec<i32>, Vec<i32>) = nums.iter().copied().partition(|x| x % 2 == 0);
assert_eq!(evens, vec![2, 4]);
assert_eq!(odds, vec![1, 3, 5]);

// unzip：拆解元组
let pairs = vec![(1, "一"), (2, "二"), (3, "三")];
let (ids, names): (Vec<i32>, Vec<&str>) = pairs.into_iter().unzip();

// min/max 典型用法
let min = nums.iter().min();  // Some(&1)
let max = nums.iter().max();  // Some(&5)
// 浮点数特有：partial_cmp 而非 Ord
let floats = vec![1.0, 2.5, -3.0];
let min_f = floats.iter().cloned().reduce(f64::min); // Some(-3.0)

// for_each 完全消费
(0..5).for_each(|i| print!("{} ", i)); // 输出: 0 1 2 3 4
```

---

## 五、collect 的魔法

```rust
use std::collections::{HashMap, BTreeMap, HashSet, VecDeque, LinkedList};

let pairs = vec![
    ("a".to_string(), 1),
    ("b".to_string(), 2),
    ("c".to_string(), 3),
];

// 收集到各种容器
let vec: Vec<(String, i32)> = pairs.clone().into_iter().collect();
let map: HashMap<String, i32> = pairs.clone().into_iter().collect();
let btree: BTreeMap<String, i32> = pairs.clone().into_iter().collect();
let set: HashSet<String> = vec!["x", "y", "z"].into_iter().map(String::from).collect();

// Result 短路收集（遇错即停）
let results = vec![Ok(1), Ok(2), Err("失败"), Ok(4)];
let collected: Result<Vec<i32>, &str> = results.into_iter().collect();
assert_eq!(collected, Err("失败"));

// Option 收集（遇 None 即停）
let opts = vec![Some(1), Some(2), None, Some(4)];
let collected: Option<Vec<i32>> = opts.into_iter().collect();
assert_eq!(collected, None);

// 收集到 String
let chars = vec!['H', 'e', 'l', 'l', 'o'];
let s: String = chars.into_iter().collect();
assert_eq!(s, "Hello");
```

---

## 六、自定义迭代器

> **金句引用**："实现 `Iterator` 就是为你的数据结构赋予一套遍历的语言。"

### 6.1 计数器

```rust
struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Self { Counter { count: 0, max } }
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
// 使用
let c = Counter::new(5);
let nums: Vec<u32> = c.collect();
assert_eq!(nums, vec![1, 2, 3, 4, 5]);
```

### 6.2 斐波那契迭代器 + DoubleEndedIterator

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self { Fibonacci { current: 0, next: 1 } }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let ret = self.current;
        self.current = self.next;
        self.next = ret + self.next;
        Some(ret)
    }
}

// 限定取10个
let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);

// DoubleEndedIterator 示例：可双向遍历的结构
struct Range {
    start: i32,
    end: i32,
}

impl Iterator for Range {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.start < self.end {
            let val = self.start;
            self.start += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for Range {
    fn next_back(&mut self) -> Option<i32> {
        if self.start < self.end {
            self.end -= 1;
            Some(self.end)
        } else {
            None
        }
    }
}

let mut r = Range { start: 0, end: 5 };
assert_eq!(r.next(), Some(0));        // 从头取
assert_eq!(r.next_back(), Some(4));   // 从尾取
assert_eq!(r.next(), Some(1));
assert_eq!(r.next_back(), Some(3));
assert_eq!(r.next(), Some(2));
assert_eq!(r.next(), None);
```

---

## 七、itertools 扩展库

```rust
// Cargo.toml: itertools = "0.13"
use itertools::Itertools;

let nums = vec![1, 5, 3, 2, 4];

// sorted / unique
let sorted: Vec<_> = nums.iter().sorted().collect();
let unique: Vec<_> = vec![1, 2, 1, 3, 2].into_iter().unique().collect();

// chunk_by：按 key 分组
let data = vec![("a", 1), ("a", 2), ("b", 3), ("b", 4)];
let groups: Vec<_> = data.iter()
    .chunk_by(|(key, _)| *key)
    .into_iter()
    .map(|(key, group)| (key, group.count()))
    .collect();

// cartesian_product：笛卡尔积
let cards: Vec<_> = (1..=3)
    .cartesian_product("ABC".chars())
    .collect();

// permutations / combinations
let perm: Vec<Vec<i32>> = vec![1, 2, 3].into_iter()
    .permutations(2)
    .collect();

let comb: Vec<Vec<&i32>> = vec![1, 2, 3].iter()
    .combinations(2)
    .collect();

// intersperse：穿插分隔符
let joined: String = ["foo", "bar", "baz"].iter()
    .intersperse(&", ")
    .collect();
assert_eq!(joined, "foo, bar, baz");

// merge / kmerge：归并有序迭代器
let a = vec![1, 3, 5];
let b = vec![2, 4, 6];
let merged: Vec<_> = a.iter().merge(b.iter()).collect();

// tuple_windows：滑动窗口
let vals = vec![1, 2, 3, 4, 5];
let diffs: Vec<i32> = vals.iter()
    .tuple_windows()
    .map(|(a, b)| b - a)
    .collect();
assert_eq!(diffs, vec![1, 1, 1, 1]);
```

---

## 八、返回迭代器的三种写法

```rust
// 方式1: impl Trait（推荐，零成本静态分发）
fn even_squares_impl(input: &[i32]) -> impl Iterator<Item = i32> + '_ {
    input.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
}

// 方式2: Box<dyn Iterator>（动态分发，有堆开销）
fn even_squares_box(input: &[i32]) -> Box<dyn Iterator<Item = i32> + '_> {
    Box::new(input.iter().filter(|&&x| x % 2 == 0).map(|&x| x * x))
}

// 方式3: 具体类型（编译期可知，但签名不透明）
fn even_squares_concrete(input: &[i32])
    -> std::iter::Map<
        std::iter::Filter<std::slice::Iter<i32>, fn(&&i32) -> bool>,
        fn(&i32) -> i32,
    >
{
    input.iter().filter(|&&x| x % 2 == 0).map(|&x| x * x)
}
```

**选型建议**：常规场景用 `impl Iterator`，需类型擦除时用 `Box<dyn Iterator>`，避免写具体类型签名。

---

## 九、零成本抽象原理

> **金句引用**："迭代器链 = 手写循环——编译器为你展开得一模一样。"

```rust
// 这三段代码编译为相同的机器码：

// (a) 迭代器风格
let sum1: i32 = (0..1000).filter(|x| x % 2 == 0).map(|x| x * x).sum();

// (b) 等价手写循环
let mut sum2 = 0;
for x in 0..1000 {
    if x % 2 == 0 {
        sum2 += x * x;
    }
}
assert_eq!(sum1, sum2);
// 编译器内联展开后，两者产生的汇编指令完全一致。
```

---

## 十、实战模式

```rust
// 管道式转换：从原始数据到最终结果的流水线
let input = "1,2,3,4,5,invalid,6";
let valid_sum: i32 = input.split(',')
    .filter_map(|s| s.trim().parse::<i32>().ok())
    .filter(|&n| n % 2 != 0)
    .sum();
// valid_sum = 9 (1 + 3 + 5)

// 分段消费：先 peek 判断，再决定消费策略
let mut chars = "hello world".chars().peekable();
let mut uppercase = false;
let result: String = std::iter::from_fn(|| {
    if chars.peek().is_none() { return None; }
    let ch = chars.next().unwrap();
    if ch == ' ' { uppercase = true; Some(ch) }
    else if uppercase { uppercase = false; Some(ch.to_ascii_uppercase()) }
    else { Some(ch) }
}).collect();
assert_eq!(result, "hello World");
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 迭代器未消费，管道不执行 | 迭代器是惰性的，适配器仅构建计划 | 在最终位置调用 `collect`/`fold`/`for_each` 等消费者 |
| `iter()` 和 `into_iter()` 混乱 | `for x in vec` 消费所有权，`for x in &vec` 只借用 | 明确所有权需求：消耗用 `into_iter`，借阅用 `iter` |
| `filter_map` 中因副作用丢失 `None` | `filter_map` 丢弃 `None`，副作用代码不会执行 | 调试用 `inspect`，副作用避免放在 `filter_map` 内 |
| zip 两个不等长迭代器静默截断 | zip 以最短迭代器为准终止 | 用 `itertools::zip_longest` 或用断言检查等长 |
| 迭代器含 `&mut` 引用时 borrow 冲突 | 同一作用域内多个迭代器可变借用同一容器 | 使用完一个迭代器后才创建下一个，或缩小作用域 |
| `lazy_static` 常量中写 `iter().map(...).collect()` | 宏展开时不会自动触发 collect | 在外部预先计算，或使用 `once_cell::Lazy` 延迟初始化 |
| `rev()` 用于单向迭代器 | `rev()` 需要 `DoubleEndedIterator` 实现 | 先 `collect` 到 `Vec` 再 `rev`，或实现逆向遍历 |
| 返回 `Box<dyn Iterator>` 导致额外分配 | 动态分发有虚表 + 堆分配开销 | 优先使用 `impl Iterator`，静态分发零开销 |

> **详见测试**: `tests/rust_features/10_iterators.rs`
