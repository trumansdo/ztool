# 06 - 集合类型

## 核心概念

Rust 标准库提供多种集合类型：

### 键值对

#### HashMap<K, V>

- 哈希表，实现 O(1) 查找
- 需要 K 实现 Hash + Eq

```rust
use std::collections::HashMap;
let mut map = HashMap::new();
map.insert("key", 42);
```

#### BTreeMap<K, V>

- B 树实现，有序遍历
- 按键顺序迭代

```rust
use std::collections::BTreeMap;
let mut map = BTreeMap::new();
```

### 集合

#### HashSet<T>

- 无序唯一元素集合

```rust
use std::collections::HashSet;
let mut set = HashSet::new();
set.insert(1);
```

#### BTreeSet<T>

- 有序唯一元素集合

### 双端队列

#### VecDeque<T>

- 两端高效插入/删除
- 适合作队列

## 主要操作

### HashMap

- `insert()` / `get()` / `remove()`
- `entry()`: 复杂插入
- `extract_if()`: 条件删除

### HashSet

- `insert()` / `contains()` / `remove()`
- `union()` / `intersection()` / `difference()`

## 避坑指南

1. **性能选择**: HashMap O(1) vs BTreeMap O(log n)
2. **内存**: 哈希表有额外内存开销
3. **Hash 碰撞**: 自定义类型需实现 Hash

## 单元测试

详见 `tests/rust_features/06_collection_operations.rs`

## 参考资料

- [Rust Collections](https://oneuptime.com/blog/post/2026-02-01-rust-collections/view)
