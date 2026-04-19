# 06 - 集合类型

## 概述

Rust 标准库提供了丰富的集合类型，涵盖了大多数日常编程场景。不同于其他语言需要引入第三方库，Rust 的内置集合设计精良、性能优异，并且与所有权系统完美集成。选择合适的集合类型是编写高效 Rust 代码的关键。

## 键值对集合

### HashMap<K, V>

哈希表实现，提供 O(1) 平均查找复杂度。需要 K 实现 Hash + Eq。

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, i32> = HashMap::new();
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Red"), 25);

let blue_score = scores.get("Blue");  // Some(&10)
```

**Entry API** - 处理"插入如果不存在"的模式：

```rust
let mut word_counts: HashMap<String, u32> = HashMap::new();
for word in text.split_whitespace() {
    *word_counts.entry(word.to_string()).or_insert(0) += 1;
}
```

### BTreeMap<K, V>

B 树实现，保持键的有序性，提供 O(log n) 查找。适合需要有序遍历或范围查询的场景。

```rust
use std::collections::BTreeMap;

let mut events: BTreeMap<u64, String> = BTreeMap::new();
events.insert(1706745600, String::from("Server started"));
// 迭代总是按键顺序
for (ts, event) in &events { }
```

## 集合类型

### HashSet<T>

无序唯一元素集合，底层是 HashMap<T, ()>。

```rust
use std::collections::HashSet;

let mut ids: HashSet<u64> = HashSet::new();
ids.insert(42);
let contains = ids.contains(&42);
```

**集合操作**：

```rust
let a: HashSet<i32> = [1, 2, 3, 4, 5].iter().cloned().collect();
let b: HashSet<i32> = [3, 4, 5, 6, 7].iter().cloned().collect();

let union: HashSet<_> = a.union(&b).cloned().collect();      // 并集
let intersection: HashSet<_> = a.intersection(&b).cloned().collect();  // 交集
let difference: HashSet<_> = a.difference(&b).cloned().collect();  // 差集
```

### BTreeSet<T>

有序唯一元素集合。

## 双端队列

### VecDeque<T>

两端高效插入/删除，适合作队列和循环缓冲区。

```rust
use std::collections::VecDeque;

let mut queue = VecDeque::new();
queue.push_back(1);
queue.push_back(2);
if let Some(front) = queue.pop_front() { }
```

## 性能对比与选择

| 操作 | HashMap | BTreeMap |
|------|---------|----------|
| 查找 | O(1) | O(log n) |
| 插入 | O(1) amortized | O(log n) |
| 迭代 | 无序 | 按键排序 |
| 范围查询 | 不支持 | 高效 |

**选择建议**：
- 大多数场景使用 HashMap
- 需要有序迭代或范围查询使用 BTreeMap
- 内存敏感场景考虑 BTree 系列

### FxHasher - 快速哈希

对于内部数据结构（不考虑 HashDoS），可使用更快的哈希函数：

```rust
use rustc_hash::FxHasher;
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
```

FxHasher 比默认的 SipHash 快约 10 倍，适合游戏开发、内部缓存等场景。

## 常见模式

### 频率统计

```rust
fn count_frequencies<T: std::hash::Hash + Eq>(items: &[T]) -> HashMap<&T, usize> {
    let mut counts = HashMap::new();
    for item in items {
        *counts.entry(item).or_insert(0) += 1;
    }
    counts
}
```

### 按字段分组

```rust
fn group_by_department(users: Vec<User>) -> HashMap<String, Vec<User>> {
    let mut groups: HashMap<String, Vec<User>> = HashMap::new();
    for user in users {
        groups.entry(user.department.clone()).or_insert_with(Vec::new).push(user);
    }
    groups
}
```

## 避坑指南

1. **Hash 碰撞**：自定义类型需实现 Hash 和 Eq
2. **内存开销**：哈希表有额外的内存开销
3. **迭代方式**：`iter()` 借用、`iter_mut()` 可变、`into_iter()` 获取所有权

## 单元测试

详见 `tests/rust_features/06_collection_operations.rs`

## 参考资料

- [Rust Collections Guide](https://oneuptime.com/blog/post/2026-02-01-rust-collections/view)
- [Choosing the Right Rust Collection](https://medium.com/@ali-alachkar/choosing-the-right-rust-collection-a-performance-deep-dive-7fc66f3fbdd9)
- [Rust Data Structures](https://infobytes.guru/articles/rust-data-structures-collections.html)