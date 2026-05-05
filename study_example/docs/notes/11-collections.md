# Rust 集合类型：选对容器，性能翻倍

## 一、集合总览对比表

> **金句引用**："数据结构选型是架构的第一刀——切对了省一年调优，切错了补三年坑。"

| 集合 | 底层结构 | 有序? | 重复? | 查找 | 插入 | 删除 | 适用场景 |
|------|----------|-------|-------|------|------|------|----------|
| `Vec<T>` | 连续数组 | 插入序 | 是 | O(n) | O(1)* | O(n) | 通用顺序容器 |
| `VecDeque<T>` | 环形缓冲区 | 插入序 | 是 | O(n) | O(1)* | O(1)* | 双端队列/滑动窗口 |
| `LinkedList<T>` | 双向链表 | 插入序 | 是 | O(n) | O(1) | O(1) | 频繁头尾拆分合并 |
| `HashMap<K,V>` | 瑞士表 | 无 | 键唯一 | O(1) | O(1)* | O(1)* | 通用键值映射 |
| `BTreeMap<K,V>` | B树 | 键序 | 键唯一 | O(log n) | O(log n) | O(log n) | 范围查询/有序遍历 |
| `HashSet<T>` | 哈希表 | 无 | 唯一 | O(1) | O(1)* | O(1)* | 去重/集合运算 |
| `BTreeSet<T>` | B树 | 键序 | 唯一 | O(log n) | O(log n) | O(log n) | 有序去重/范围 |
| `BinaryHeap<T>` | 二叉堆 | 堆序 | 是 | O(1)max | O(log n) | O(log n) | 优先级队列/Dijkstra |

*\* 均摊复杂度*

---

## 二、HashMap：哈希表之王

> **金句引用**："HashMap 是 Rust 集合库的皇冠——瑞士表、SipHash、SIMD 三剑合璧。"

### 2.1 内部结构：瑞士表（Swiss Table）

Rust 的 `HashMap` 自 1.36 起由 Swiss Table 实现，核心优势：
- **元数据数组**：每16个桶共用一个控制字节，利用 SIMD 指令一次比较16个
- **SipHash 算法**：防 HashDoS 攻击，提供密钥随机化的安全散列
- **局部性**：数据紧密排列，缓存友好

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();

// 插入（移动所有权）
scores.insert("Alice".to_string(), 95);
scores.insert("Bob".to_string(), 80);

// get 返回 Option<&V>（引用，不移动）
let alice_score = scores.get("Alice");     // Some(&95)
let charlie_score = scores.get("Charlie"); // None

// 索引语法（无值时 panic）
let bob_score = scores["Bob"];             // 80

// remove 返回 Option<V>
let removed = scores.remove("Bob");        // Some(80)
let removed_again = scores.remove("Bob");  // None

// 预分配容量避免重哈希
let mut map = HashMap::with_capacity(1000);
for i in 0..1000 {
    map.insert(i, i * 2);
}
```

### 2.2 Entry API：最精巧的插入控制

> **金句引用**："Entry API 是 Rust 的插入艺术——一次查找，三种命运，零冗余。"

```rust
let mut map = HashMap::new();

// or_insert: 无键时插入默认值，返回 &mut V
let val = map.entry("counter").or_insert(0);
*val += 1;
// "counter" → 1

// or_insert_with: 惰性计算默认值
let val = map.entry("expensive").or_insert_with(|| {
    println!("计算默认值...");
    100
});
assert_eq!(*val, 100);

// or_default: 使用 Default trait
let val = map.entry("default").or_default();
assert_eq!(*val, 0_i32);

// and_modify: 有键时修改（链式组合）
map.insert("key".to_string(), vec![1, 2]);
let entry = map.entry("key".to_string())
    .and_modify(|v| v.push(3))
    .or_insert(vec![4]);  // 只有 key 不存在时才执行
assert_eq!(map["key"], vec![1, 2, 3]);

// 链式组合模式：存在则改，不存在则插入
let stats = map.entry("user_id")
    .and_modify(|count| *count += 1)
    .or_insert(1);
```

**Entry API 完整表**：

| 方法 | 键存在时 | 键不存在时 | 返回值 |
|------|----------|-----------|--------|
| `entry(k)` | — | — | `Entry<&K, V>` |
| `or_insert(v)` | 不执行 | 插入 v | 两种情况均返回 `&mut V` |
| `or_insert_with(f)` | 不执行 | 调用 f() 后插入 | 同上，惰性求值 |
| `or_default()` | 不执行 | 插入 `V::default()` | 同上 |
| `and_modify(f)` | 调用 f(&mut V) | 不执行 | 返回 `Entry<&K, V>` 以继续链式调用 |
| `or_insert_with_key(f)` | 不执行 | 调用 f(&K) 后插入 | 返回 `&mut V`，可以访问键 |
| `remove_entry(k)` | — | — | `Option<(K, V)>` 返回移除的键-值对 |

### 2.3 所有权交互

```rust
let mut map = HashMap::new();

// insert 移动键和值的所有权
let key = String::from("name");
let value = String::from("Alice");
map.insert(key, value);  // key 和 value 被移动，不能再使用
// println!("{}", key);  // 编译错误！所有权已转移

// get 返回引用，不移动所有权
let name_ref: Option<&String> = map.get("name");

// remove 返回 Option<V>，所有权还给调用者
let name_owned = map.remove("name");  // Option<String>
```

---

## 三、BTreeMap：有序键值映射

> **金句引用**："BTree 有序则有界——区间查询、二分定位，举手投足皆天然。"

```rust
use std::collections::BTreeMap;

let mut btree = BTreeMap::new();
btree.insert(3, "三");
btree.insert(1, "一");
btree.insert(2, "二");
btree.insert(5, "五");
btree.insert(4, "四");

// 始终按键排序迭代
let keys: Vec<_> = btree.keys().collect();
assert_eq!(keys, vec![&1, &2, &3, &4, &5]);

// 范围查询 range()：BTreeMap 的杀手锏
let range: Vec<_> = btree.range(2..=4).collect();
assert_eq!(range, vec![(&2, &"二"), (&3, &"三"), (&4, &"四")]);

// 前后无边界的范围
let from_3: Vec<_> = btree.range(3..).collect();
let to_3: Vec<_> = btree.range(..3).collect();

// 二分查找边界
if let Some((k, v)) = btree.range(..).next() {
    println!("最小键: {} -> {}", k, v);
}
```

---

## 四、HashSet / BTreeSet：集合运算

> **金句引用**："集合就是降维的映射——值即键，键即值，运算即查询。"

```rust
use std::collections::{HashSet, BTreeSet};

let a: HashSet<_> = [1, 2, 3, 4].iter().cloned().collect();
let b: HashSet<_> = [3, 4, 5, 6].iter().cloned().collect();

// 并集
let union: HashSet<_> = a.union(&b).cloned().collect();  // {1,2,3,4,5,6}

// 交集
let intersection: HashSet<_> = a.intersection(&b).cloned().collect(); // {3,4}

// 差集：a有b无
let diff: HashSet<_> = a.difference(&b).cloned().collect(); // {1,2}

// 对称差：只在一边的元素
let sym_diff: HashSet<_> = a.symmetric_difference(&b).cloned().collect(); // {1,2,5,6}

// 子集/超集判断
assert!(!a.is_subset(&b));
assert!(!a.is_superset(&b));
assert!(HashSet::from([3, 4]).is_subset(&a));

// 不相交判断
assert!(HashSet::from([1, 2]).is_disjoint(&HashSet::from([3, 4])));
```

---

## 五、BinaryHeap：优先级队列

> **金句引用**："最大堆如君王——最大值永远立于堆顶。"

### 5.1 基础使用

```rust
use std::collections::BinaryHeap;

let mut heap = BinaryHeap::new();
heap.push(3);
heap.push(5);
heap.push(1);
heap.push(4);

assert_eq!(heap.peek(), Some(&5));  // 最大元素在堆顶
assert_eq!(heap.pop(), Some(5));
assert_eq!(heap.pop(), Some(4));
assert_eq!(heap.pop(), Some(3));
assert_eq!(heap.pop(), Some(1));
assert_eq!(heap.pop(), None);
```

### 5.2 自定义排序：Reverse 包装器

```rust
use std::cmp::Reverse;

// 最小堆：用 Reverse 翻转排序
let mut min_heap = BinaryHeap::new();
min_heap.push(Reverse(3));
min_heap.push(Reverse(5));
min_heap.push(Reverse(1));
assert_eq!(min_heap.pop(), Some(Reverse(1))); // 最小值出堆
```

### 5.3 多字段优先级队列（Dijkstra 经典应用）

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Task {
    priority: i32,  // 数字越小优先级越高
    id: u32,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // 先按优先级反向（堆默认最大，这里要最小堆）
        other.priority.cmp(&self.priority)
            .then_with(|| self.id.cmp(&other.id))  // 优先级相同按 id
    }
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

let mut task_heap = BinaryHeap::new();
task_heap.push(Task { priority: 2, id: 100 });
task_heap.push(Task { priority: 1, id: 200 });
task_heap.push(Task { priority: 2, id: 50 });
// 出堆顺序：id=200(优先级1) → id=50(优先级2,id小) → id=100
```

---

## 六、VecDeque：双向队列

> **金句引用**："Vec 是单向传送带，VecDeque 是双向手扶梯——头尾皆可高效出入。"

```rust
use std::collections::VecDeque;

let mut deque = VecDeque::new();
deque.push_back(1);    // 队尾入
deque.push_front(0);   // 队头入
deque.push_back(2);

assert_eq!(deque, [0, 1, 2]);
assert_eq!(deque.pop_front(), Some(0)); // 队头出
assert_eq!(deque.pop_back(), Some(2));  // 队尾出

// 滑动窗口经典用法：固定容量 deque
let data = vec![1, 3, -1, -3, 5, 3, 6, 7];
let k = 3; // 窗口大小
let mut window = VecDeque::new();
for (i, &val) in data.iter().enumerate() {
    // 移除超出窗口的索引
    if window.front().map_or(false, |&idx| idx + k <= i) {
        window.pop_front();
    }
    // 移除所有小于当前值的索引
    while window.back().map_or(false, |&idx| data[idx] <= val) {
        window.pop_back();
    }
    window.push_back(i);
    // window.front() 即为当前窗口最大值索引
}
```

### Vec vs VecDeque 对比

| 操作 | Vec | VecDeque |
|------|-----|----------|
| push/pop 尾 | O(1)* | O(1)* |
| push/pop 头 | O(n) | O(1)* |
| 随机访问 | O(1) 连续 | O(1) 但不连续（可能需要两次访问） |
| 内存布局 | 连续 | 环形，虚拟连续 |

---

## 七、LinkedList：双向链表

```rust
use std::collections::LinkedList;

let mut list = LinkedList::new();
list.push_back(1);
list.push_front(0);
list.push_back(2);

// 在任意位置分割链表 O(1)
let mut tail = list.split_off(2);
assert_eq!(list, [0, 1]);
assert_eq!(tail, [2]);

// 拼接
list.append(&mut tail);
assert_eq!(list, [0, 1, 2]);
```

> ⚠️ **性能提醒**：多数场景下 `Vec` 优于 `LinkedList`（缓存局部性差）；仅在频繁**头尾拆分合并**时使用。

---

## 八、自定义键类型

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
struct ComplexKey {
    namespace: String,
    id: u64,
}

// Eq：需要 PartialEq + Eq
impl PartialEq for ComplexKey {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.id == other.id
    }
}
impl Eq for ComplexKey {}

// Hash：必须实现
impl Hash for ComplexKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.id.hash(state);
    }
}

let mut map: HashMap<ComplexKey, String> = HashMap::new();
map.insert(
    ComplexKey { namespace: "user".into(), id: 42 },
    "value".into()
);
```

---

## 九、集合与迭代器集成

```rust
// collect / extend / from_iter 三种构造方式

let squares: Vec<_> = (0..5).map(|x| x * x).collect();

let mut set = HashSet::new();
set.extend([1, 2, 3, 4]);

use std::iter::FromIterator;
let map: HashMap<_, _> = HashMap::from_iter([
    ("a", 1),
    ("b", 2),
    ("c", 3),
]);
```

---

## 十、性能对照与操作选择速查

> **金句引用**："选集合如选兵器——没有最好的，只有最合手的。"

```
需要按键查找？
├─ 需要 → 需要有序？
│  ├─ 是 → BTreeMap / BTreeSet
│  └─ 否 → HashMap / HashSet
└─ 不需要 → 需要双端操作？
   ├─ 是 → VecDeque
   └─ 否 → 需要频繁拆分合并？
      ├─ 是 → LinkedList
      └─ 否 → Vec
```

**具体性能建议**：
- 键查找优先 `HashMap`，除非必须有序
- `BTreeMap` 的缓存局部性在某些场景下优于链表实现的传统 B 树
- `Vec` > `VecDeque` 除非确需头端 O(1) 操作
- 小数据量（< 100 元素）`Vec` 线性扫描常比哈希表快

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `HashMap::get` 返回锁定的不可变引用 | `get` 返回 `Option<&V>`，借用了整个 HashMap | 在引用释放前不能插入/删除，可用 `remove` + 后插替代 |
| `BinaryHeap` 遍历顺序非排序 | `iter()` 按内部数组顺序，非堆序 | 用 `into_sorted_vec()` 获取排序结果 |
| `HashSet` 元素类型未实现 `Hash + Eq` | 自定义类型必须手动实现这两个 trait | 同时派生或手动实现 `Hash` 和 `Eq` |
| `Entry API` 中 `or_insert` 参数在调用时立即求值 | 即使键已存在，参数表达式也已计算完毕 | 用 `or_insert_with(|| expensive_fn())` 延迟计算 |
| 数组越界后 `Vec::remove` 移动剩余元素 | 移除第 i 个元素需要 O(n) 移动全部后续元素 | 不关心顺序用 `swap_remove`，O(1) |
| `BTreeMap::range` 返回的迭代器借用原集合 | 在迭代期间不能修改 BTreeMap | 先 `collect` 到 Vec，再遍历修改 |
| HashMap 的迭代顺序不保证稳定 | Swiss Table 内部顺序取决于哈希值和插入历史 | 不依赖迭代顺序，需要排序时用 BTreeMap |
| `LinkedList::split_off` 后原链表仍存在 | split_off 产生两个独立链表，不是移动 | 仔细管理 split_off 产生的两端，避免内存泄漏 |

> **详见测试**: `tests/rust_features/11_collections.rs`
