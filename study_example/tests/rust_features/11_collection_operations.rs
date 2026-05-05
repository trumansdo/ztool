// ---------------------------------------------------------------------------
// 2.4 集合操作 - extract_if / get_disjoint_mut / HashMap / HashSet / VecDeque / BinaryHeap
// ---------------------------------------------------------------------------

use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};

#[test]
/// 测试: HashMap 基础操作 (插入/查询/删除/迭代)
fn test_hashmap_basics() {
    // 语法: HashMap<K, V> 基于哈希表, O(1) 平均查找/插入/删除
    //
    // 创建:
    //   - HashMap::new()              空 HashMap
    //   - HashMap::with_capacity(n)   预分配容量
    //   - HashMap::from([(k, v), ..]) 字面量初始化
    //   - iter.collect()              从迭代器收集
    //
    // 插入/查询:
    //   - insert(k, v) -> Option<V>   插入, 返回旧值
    //   - get(&k) -> Option<&V>       不可变查询
    //   - get_mut(&k) -> Option<&mut V> 可变查询
    //   - contains_key(&k) -> bool    检查键是否存在
    //   - entry(k) -> Entry           占位符 API (见下)
    //
    // 删除:
    //   - remove(&k) -> Option<V>     删除并返回值
    //   - retain(f)                   保留满足条件的键值对
    //   - clear()                     清空
    //
    // 迭代:
    //   - iter() -> (&K, &V)          不可变迭代
    //   - iter_mut() -> (&K, &mut V)  可变迭代
    //   - into_iter() -> (K, V)       消费迭代
    //   - keys() / values() / values_mut()
    //
    // 避坑:
    //   - key 必须实现 Hash + Eq
    //   - HashMap 不保证迭代顺序
    //   - get 参数是 &Q where K: Borrow<Q>, 所以可以用 &str 查 String 键
    //   - insert 返回的是被替换的旧值 (Option<V>)

    // 创建
    let mut map = HashMap::new();
    map.insert("apple", 3);
    map.insert("banana", 5);
    map.insert("cherry", 7);

    // 查询
    assert_eq!(map.get("apple"), Some(&3));
    assert_eq!(map.get("grape"), None);
    assert!(map.contains_key("banana"));

    // 用 &str 查 String 键
    let mut map2 = HashMap::new();
    map2.insert(String::from("key"), 42);
    assert_eq!(map2.get("key"), Some(&42)); // &str 查 String

    // 修改
    if let Some(v) = map.get_mut("banana") {
        *v = 10;
    }
    assert_eq!(map["banana"], 10);

    // 删除
    assert_eq!(map.remove("apple"), Some(3));
    assert_eq!(map.remove("apple"), None); // 已不存在

    // retain
    map.retain(|_k, v| *v > 5);
    assert_eq!(map.len(), 2); // 只保留 banana(10) 和 cherry(7)

    // 迭代
    let keys: Vec<_> = map.keys().collect();
    assert_eq!(keys.len(), 2);

    let values: Vec<_> = map.values().collect();
    assert_eq!(values.len(), 2);

    // 从迭代器收集
    let map: HashMap<&str, i32> = vec![("a", 1), ("b", 2)]
        .into_iter()
        .collect();
    assert_eq!(map.len(), 2);
}

#[test]
/// 测试: HashMap Entry API (占位符模式)
fn test_hashmap_entry() {
    // 语法: entry(k) 返回 Entry 枚举, 实现 "查找或插入" 原子操作
    //
    // Entry 方法:
    //   - or_insert(v)        不存在则插入 v, 返回 &mut V
    //   - or_insert_with(f)   不存在则用闭包生成值
    //   - or_default()        不存在则插入 Default::default()
    //   - and_modify(f)       存在则执行修改, 返回 Entry
    //   - key() -> &K         获取键引用
    //
    // 避坑:
    //   - Entry API 避免两次查找 (一次 get + 一次 insert)
    //   - and_modify 返回 Entry, 可链式调用 or_insert
    //   - entry 会占用可变借用, 不能再借用 HashMap

    let mut map = HashMap::new();

    // or_insert
    let count = map.entry("apple").or_insert(0);
    *count += 1;
    assert_eq!(map["apple"], 1);

    // 链式: 存在则修改, 不存在则插入默认值
    map.entry("banana")
        .and_modify(|v| *v += 1)
        .or_insert(1);
    map.entry("banana")
        .and_modify(|v| *v += 1)
        .or_insert(1);
    assert_eq!(map["banana"], 2);

    // 词频统计经典模式
    let text = "hello world hello rust world hello";
    let mut freq = HashMap::new();
    for word in text.split_whitespace() {
        *freq.entry(word).or_insert(0) += 1;
    }
    assert_eq!(freq["hello"], 3);
    assert_eq!(freq["world"], 2);
    assert_eq!(freq["rust"], 1);

    // or_insert_with (惰性求值)
    let mut map2: HashMap<&str, Vec<i32>> = HashMap::new();
    map2.entry("nums")
        .or_insert_with(Vec::new)
        .push(42);
    assert_eq!(map2["nums"], vec![42]);
}

#[test]
/// 测试: HashSet 基础操作
fn test_hashset_basics() {
    // 语法: HashSet<T> 基于 HashMap 实现, 只存储键, 值为 ()
    //
    // 常用方法:
    //   - insert(t) -> bool      插入, 返回是否是新元素
    //   - contains(&t) -> bool   检查是否存在
    //   - remove(&t) -> bool     删除, 返回是否存在
    //   - get(&t) -> Option<&T>  获取引用
    //
    // 集合运算:
    //   - union(&other)          并集
    //   - intersection(&other)   交集
    //   - difference(&other)     差集 (在 self 不在 other)
    //   - symmetric_difference   对称差集
    //   - is_subset(&other)      子集判断
    //   - is_superset(&other)    超集判断
    //   - is_disjoint(&other)    是否无交集
    //
    // 避坑:
    //   - 元素必须实现 Hash + Eq
    //   - 集合运算返回迭代器, 需要 collect
    //   - HashSet 不保证迭代顺序

    let mut set = HashSet::new();
    assert!(set.insert(1)); // true, 新元素
    assert!(set.insert(2));
    assert!(!set.insert(1)); // false, 已存在

    assert!(set.contains(&1));
    assert!(set.remove(&2));
    assert!(!set.remove(&2)); // 已不存在

    // 集合运算
    let a: HashSet<_> = vec![1, 2, 3]
        .into_iter()
        .collect();
    let b: HashSet<_> = vec![3, 4, 5]
        .into_iter()
        .collect();

    let union: HashSet<_> = a.union(&b).cloned().collect();
    assert_eq!(union.len(), 5); // {1,2,3,4,5}

    let inter: HashSet<_> = a
        .intersection(&b)
        .cloned()
        .collect();
    assert_eq!(inter, HashSet::from([3]));

    let diff: HashSet<_> = a
        .difference(&b)
        .cloned()
        .collect();
    assert_eq!(diff, HashSet::from([1, 2]));

    let sym_diff: HashSet<_> = a
        .symmetric_difference(&b)
        .cloned()
        .collect();
    assert_eq!(sym_diff, HashSet::from([1, 2, 4, 5]));

    // 子集/超集
    let small: HashSet<_> = vec![1, 2].into_iter().collect();
    let large: HashSet<_> = vec![1, 2, 3, 4]
        .into_iter()
        .collect();
    assert!(small.is_subset(&large));
    assert!(large.is_superset(&small));
    assert!(small.is_disjoint(&HashSet::from([5, 6])));
}

#[test]
/// 测试: BTreeMap / BTreeSet (有序集合)
fn test_btree_map_set() {
    // 语法: BTreeMap/BTreeSet 基于 B 树, 按键排序, O(log n) 操作
    //
    // 与 HashMap 的区别:
    //   - 有序 (按键的自然顺序)
    //   - 支持范围查询 (range)
    //   - 有 first_key_value / last_key_value
    //   - 有 pop_first / pop_last
    //   - 需要 K: Ord (不需要 Hash)
    //
    // BTreeMap 特有方法:
    //   - range(range)         范围查询迭代器
    //   - first_key_value()    最小键值对
    //   - last_key_value()     最大键值对
    //   - pop_first()          移除并返回最小键值对
    //   - pop_last()           移除并返回最大键值对
    //   - split_off(key)       分割为两个 BTreeMap
    //
    // 避坑:
    //   - 性能比 HashMap 稍慢, 但有序
    //   - range 是半开区间 (start..end), 用 Bound 控制开闭
    //   - pop_first/pop_last 返回 Option

    use std::collections::BTreeMap;
    use std::ops::Bound;

    let mut map = BTreeMap::new();
    map.insert(3, "c");
    map.insert(1, "a");
    map.insert(4, "d");
    map.insert(2, "b");

    // 有序迭代
    let keys: Vec<_> = map.keys().collect();
    assert_eq!(keys, vec![&1, &2, &3, &4]);

    // 首尾
    assert_eq!(map.first_key_value(), Some((&1, &"a")));
    assert_eq!(map.last_key_value(), Some((&4, &"d")));

    // 范围查询
    let range: Vec<_> = map.range(2..=3).collect();
    assert_eq!(range, vec![(&2, &"b"), (&3, &"c")]);

    // 使用 Bound 精确控制开闭
    let range: Vec<_> = map
        .range((Bound::Excluded(&1), Bound::Included(&3)))
        .collect();
    assert_eq!(range, vec![(&2, &"b"), (&3, &"c")]);

    // pop_first / pop_last
    let mut map = BTreeMap::from([(1, "a"), (2, "b"), (3, "c")]);
    assert_eq!(map.pop_first(), Some((1, "a")));
    assert_eq!(map.pop_last(), Some((3, "c")));
    assert_eq!(map.len(), 1);

    // BTreeSet
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    set.insert(5);
    set.insert(1);
    set.insert(3);
    let first = set.first();
    assert_eq!(first, Some(&1));
    let last = set.last();
    assert_eq!(last, Some(&5));
}

#[test]
/// 测试: VecDeque 双端队列
fn test_vec_deque() {
    // 语法: VecDeque 支持 O(1) 头尾插入/删除, 基于环形缓冲区
    //
    // 常用方法:
    //   - push_front(t) / push_back(t)   头尾插入
    //   - pop_front() / pop_back()       头尾删除
    //   - front() / back()               查看头尾
    //   - front_mut() / back_mut()       可变查看头尾
    //   - insert(idx, t) / remove(idx)   中间插入删除 (O(n))
    //   - rotate_left(n) / rotate_right  旋转
    //   - make_contiguous()              保证内存连续 (1.63+)
    //
    // 避坑:
    //   - 中间插入/删除是 O(n), 不如 Vec
    //   - 索引操作也是 O(1), 但比 Vec 慢 (需要取模)
    //   - 适合用作队列 (FIFO) 或双端队列

    let mut deque = VecDeque::new();

    // 尾部插入
    deque.push_back(1);
    deque.push_back(2);
    deque.push_back(3);

    // 头部插入
    deque.push_front(0);
    assert_eq!(deque, VecDeque::from([0, 1, 2, 3]));

    // 查看头尾
    assert_eq!(deque.front(), Some(&0));
    assert_eq!(deque.back(), Some(&3));

    // 头尾弹出
    assert_eq!(deque.pop_front(), Some(0));
    assert_eq!(deque.pop_back(), Some(3));
    assert_eq!(deque, VecDeque::from([1, 2]));

    // 旋转
    let mut deque = VecDeque::from([1, 2, 3, 4, 5]);
    deque.rotate_left(2);
    assert_eq!(deque, VecDeque::from([3, 4, 5, 1, 2]));

    deque.rotate_right(1);
    assert_eq!(deque, VecDeque::from([2, 3, 4, 5, 1]));

    // 用作队列 (FIFO)
    let mut queue = VecDeque::new();
    queue.push_back("first");
    queue.push_back("second");
    queue.push_back("third");
    assert_eq!(queue.pop_front(), Some("first"));
    assert_eq!(queue.pop_front(), Some("second"));
}

#[test]
/// 测试: BinaryHeap 二叉堆 (优先队列)
fn test_binary_heap() {
    // 语法: BinaryHeap<T> 最大堆, 顶部始终是最大元素
    //
    // 常用方法:
    //   - push(t)              插入元素
    //   - pop() -> Option<T>   弹出最大元素
    //   - peek() -> Option<&T> 查看最大元素 (不移除)
    //   - peek_mut()           可变查看最大元素
    //   - into_sorted_vec()    转为排序 Vec (比 pop 更高效)
    //   - push_pop(t)          插入并弹出最大 (比分别调用快)
    //
    // 最小堆:
    //   - 用 Reverse(t) 包装元素: BinaryHeap<Reverse<T>>
    //
    // 避坑:
    //   - 是最大堆, 不是最小堆
    //   - 迭代顺序不保证有序, 只有 pop 保证从大到小
    //   - peek_mut 修改后如果元素排序变化, 需要调用 .pop() 重新调整
    //   - T 必须实现 Ord

    use std::cmp::Reverse;

    // 最大堆
    let mut heap = BinaryHeap::new();
    heap.push(1);
    heap.push(5);
    heap.push(2);
    heap.push(10);
    heap.push(3);

    assert_eq!(heap.peek(), Some(&10)); // 最大值
    assert_eq!(heap.pop(), Some(10));
    assert_eq!(heap.pop(), Some(5));
    assert_eq!(heap.pop(), Some(3));

    // push + pop 组合
    let mut heap = BinaryHeap::from([1, 3, 5]);
    heap.push(4);
    let old_max = heap.pop(); // 弹出最大值 5
    assert_eq!(old_max, Some(5));

    // 转为排序 Vec
    let heap = BinaryHeap::from([3, 1, 4, 1, 5]);
    let sorted = heap.into_sorted_vec();
    assert_eq!(sorted, vec![1, 1, 3, 4, 5]);

    // 最小堆 (用 Reverse)
    let mut min_heap = BinaryHeap::new();
    min_heap.push(Reverse(5));
    min_heap.push(Reverse(1));
    min_heap.push(Reverse(3));

    assert_eq!(min_heap.pop(), Some(Reverse(1))); // 最小值
    assert_eq!(min_heap.pop(), Some(Reverse(3)));

    // Dijkstra 风格优先队列: (Reverse<代价>, 节点)
    // 用 Reverse 将最大堆转为最小堆, 每次弹出代价最小的节点
    let mut pq: BinaryHeap<(Reverse<u32>, &str)> = BinaryHeap::new();
    pq.push((Reverse(10), "B"));
    pq.push((Reverse(5), "A"));
    pq.push((Reverse(15), "C"));

    assert_eq!(pq.pop(), Some((Reverse(5), "A")));  // 代价最小先出
    assert_eq!(pq.pop(), Some((Reverse(10), "B")));
    assert_eq!(pq.pop(), Some((Reverse(15), "C")));

    // 多字段排序: 按优先级降序, 相同优先级按插入时间升序
    #[derive(Debug, PartialEq, Eq)]
    struct Task {
        priority: u32,
        created_at: u64,
        name: &'static str,
    }
    impl Ord for Task {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.priority
                .cmp(&other.priority)
                .then_with(|| other.created_at.cmp(&self.created_at)) // 时间早的优先
        }
    }
    impl PartialOrd for Task {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut task_heap = BinaryHeap::new();
    task_heap.push(Task { priority: 1, created_at: 100, name: "low-1" });
    task_heap.push(Task { priority: 3, created_at: 300, name: "high" });
    task_heap.push(Task { priority: 1, created_at: 200, name: "low-2" });

    assert_eq!(task_heap.pop().unwrap().name, "high");   // 优先级最高
    assert_eq!(task_heap.pop().unwrap().name, "low-1");  // 同优先级, 时间早的
    assert_eq!(task_heap.pop().unwrap().name, "low-2");
}

#[test]
/// 测试: Vec extract_if 提取满足条件的元素 (1.87+)
fn test_vec_extract_if() {
    // 语法: extract_if(range, pred) 移除满足条件的元素并返回迭代器 (1.87+)
    // 避坑: 第一个参数是范围(..表示全部), 第二个闭包接收 &mut T; 迭代器消费时才真正移除
    let mut numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let evens: Vec<i32> = numbers
        .extract_if(.., |x| *x % 2 == 0)
        .collect();
    assert_eq!(numbers, vec![1, 3, 5, 7, 9]);
    assert_eq!(evens, vec![2, 4, 6, 8, 10]);
}

#[test]
/// 测试: HashMap extract_if 提取键值对
fn test_hashmap_extract_if() {
    // 语法: HashMap::extract_if(pred) 闭包接收 (&K, &mut V)
    // 避坑: 闭包参数顺序是 (key, value); 返回的迭代器必须被消费否则元素不移除
    let mut map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
    let extracted: Vec<_> = map
        .extract_if(|_k, _v| *_v > 1)
        .collect();
    assert_eq!(extracted.len(), 2);
    assert_eq!(map.len(), 1);
}

#[test]
/// 测试: HashSet extract_if 提取满足条件的元素
fn test_hashset_extract_if() {
    // 语法: HashSet::extract_if(pred) 闭包接收 &T
    // 避坑: HashSet 的闭包只接收一个参数(元素引用), 与 HashMap 不同
    let mut set = HashSet::from([1, 2, 3, 4, 5]);
    let evens: Vec<_> = set
        .extract_if(|x| *x % 2 == 0)
        .collect();
    assert_eq!(set, HashSet::from([1, 3, 5]));
    assert_eq!(evens.len(), 2);
}

#[test]
/// 测试: 切片 get_disjoint_mut 同时获取多个不相交可变引用
fn test_slice_get_disjoint_mut() {
    // 语法: get_disjoint_mut(indices) 同时获取多个不相交位置的可变引用
    // 避坑: 索引必须不相交, 否则返回 Err; 返回 Result<[&mut T; N], GetDisjointMutError>
    let mut v = vec![1, 2, 3, 4, 5];
    if let Ok([a, b]) = v.get_disjoint_mut([0, 4]) {
        *a = 10;
        *b = 50;
    }
    assert_eq!(v, vec![10, 2, 3, 4, 50]);
}

#[test]
/// 测试: HashMap get_disjoint_mut 同时获取多个不相交值引用
fn test_hashmap_get_disjoint_mut() {
    // 语法: HashMap::get_disjoint_mut(keys) 返回 [Option<&mut V>; N]
    // 避坑: 返回的是数组而非 Result, 不存在的 key 对应 None; key 不能重复
    let mut map: HashMap<&str, i32> = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
    let result = map.get_disjoint_mut(["a", "c"]);
    if let [Some(x), Some(y)] = result {
        *x = 10;
        *y = 30;
    }
    assert_eq!(map["a"], 10);
    assert_eq!(map["c"], 30);
}

#[test]
/// 测试: HashMap Entry API 各种方法 (or_insert_with_key / or_default / 完整链式)
fn test_hashmap_entry_api() {
    // 语法: Entry API 是 HashMap 最重要的设计之一, 实现"查找或插入"原子操作
    //
    // Entry 完整方法清单:
    //   - or_insert(v)           不存在则插入 v, 返回 &mut V
    //   - or_insert_with(f)     不存在则用闭包 f() 生成值
    //   - or_insert_with_key(f) 不存在则用闭包 f(&K) 生成值
    //   - and_modify(f)         存在则执行 f(&mut V), 返回 Entry (可链式)
    //   - or_default()          不存在则插入 Default::default()
    //   - key() -> &K           获取键引用
    //   - VacantEntry::insert(v) 在 Vacant 分支直接插入
    //
    // 避坑:
    //   - and_modify 只对存在的键执行, 要配合 or_insert 实现"存在则修改, 不存在则插入"
    //   - or_insert 总是会求值参数 (即使键存在), 用 or_insert_with 实现惰性求值
    //   - Entry 持有 HashMap 的可变借用, 在 Entry 存活期间不能再次借用 HashMap

    let mut map = HashMap::new();

    // ── or_insert ──
    *map.entry("a").or_insert(0) += 1;
    *map.entry("a").or_insert(0) += 1; // 已存在, 不插入, 返回已有值的引用
    assert_eq!(map["a"], 2);

    // ── or_insert_with (惰性求值) ──
    let mut called = false;
    map.entry("b")
        .or_insert_with(|| {
            called = true;
            100
        });
    assert!(called); // 键不存在, 闭包被调用
    assert_eq!(map["b"], 100);

    called = false;
    map.entry("b").or_insert_with(|| {
        called = true;
        999
    });
    assert!(!called); // 键已存在, 闭包不被调用

    // ── or_insert_with_key (值依赖键) ──
    map.entry("hello").or_insert_with_key(|key| key.len());
    assert_eq!(map["hello"], 5);

    // ── or_default ──
    let mut map2: HashMap<&str, Vec<i32>> = HashMap::new();
    let list = map2.entry("c").or_default();
    list.push(1);
    list.push(2);
    assert_eq!(map2["c"], vec![1, 2]);

    // ── and_modify 链式调用 ──
    // "存在则翻倍, 不存在则插入 1"
    map.entry("d")
        .and_modify(|v| *v *= 2)
        .or_insert(1);
    assert_eq!(map["d"], 1); // 不存在, 插入 1

    map.entry("d")
        .and_modify(|v| *v *= 2)
        .or_insert(1);
    assert_eq!(map["d"], 2); // 存在, 翻倍

    // ── 匹配 Entry 枚举 ──
    use std::collections::hash_map::Entry;
    match map.entry("a") {
        Entry::Occupied(mut entry) => {
            *entry.get_mut() += 100;
        }
        Entry::Vacant(entry) => {
            entry.insert(0);
        }
    }
    assert_eq!(map["a"], 102);

    // ── 分组聚合模式 ──
    let pairs = vec![("fruit", "apple"), ("fruit", "banana"), ("color", "red")];
    let mut groups: HashMap<&str, Vec<&str>> = HashMap::new();
    for (category, item) in pairs {
        groups.entry(category).or_default().push(item);
    }
    assert_eq!(groups["fruit"], vec!["apple", "banana"]);
    assert_eq!(groups["color"], vec!["red"]);
}

#[test]
/// 测试: HashMap 所有权转移行为
fn test_hashmap_ownership() {
    // 语法: HashMap 插入非 Copy 类型时会转移所有权到 map 内部
    //
    // 所有权规则:
    //   - insert(k, v) 转移 k 和 v 的所有权
    //   - get(&k) 返回 Option<&V>, 借用 map
    //   - remove(&k) 返回 Option<V>, 所有权移出
    //   - into_iter() 消费 map, 返回 (K, V) 所有权
    //   - drain() 清空 map 并返回所有 (K, V) 的迭代器
    //
    // 避坑:
    //   - 插入后原变量不可再使用 (非 Copy 类型)
    //   - get 期间不能对 map 进行可变操作 (借用规则)
    //   - 需要同时持有多个值的所有权: 先 remove 再操作, 或克隆

    let mut map = HashMap::new();

    // ── 插入转移所有权 ──
    let key = String::from("key1");
    let value = vec![10, 20, 30];
    map.insert(key, value);
    // key 和 value 所有权已移入 map, 无法再使用
    assert_eq!(map.len(), 1);

    // ── 覆盖旧值取回所有权 (新 map, String 值) ──
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert(String::from("key2"), String::from("old"));
    let old = map.insert(String::from("key2"), String::from("new"));
    assert_eq!(old, Some(String::from("old"))); // 旧值所有权取回

    // ── remove 取回所有权 ──
    let removed_value = map.remove("key2");
    assert_eq!(removed_value, Some(String::from("new")));

    // ── drain 清空并获取所有权 ──
    let mut map = HashMap::from([("a".to_string(), 1), ("b".to_string(), 2)]);
    let drained: Vec<_> = map.drain().collect();
    assert_eq!(map.len(), 0);
    assert_eq!(drained.len(), 2);
    // drained 中的 (String, i32) 可以自由使用

    // ── into_iter 消费 map ──
    let map = HashMap::from([("x".to_string(), 10), ("y".to_string(), 20)]);
    let pairs: Vec<_> = map.into_iter().collect();
    assert_eq!(pairs.len(), 2);
    // map 已被消费, 不可再使用

    // ── 克隆保留所有权 ──
    let mut map = HashMap::new();
    let key = String::from("shared");
    let value = vec![42];
    map.insert(key.clone(), value.clone()); // 克隆后插入, 原变量仍可用
    assert_eq!(key, "shared");
    assert_eq!(value, vec![42]);
    assert_eq!(map.get(&key), Some(&vec![42]));
}

#[test]
/// 测试: BTreeMap 的有序遍历及范围查询
fn test_btree_map_ordering() {
    // 语法: BTreeMap 始终按键的 Ord 顺序组织, 迭代严格有序
    //
    // 有序特性:
    //   - iter() / keys() / values() 始终按键升序
    //   - range(range) 高效范围查询 O(log n + k)
    //   - first_key_value() / last_key_value() 获取极值
    //   - pop_first() / pop_last() 弹出极值
    //
    // 避坑:
    //   - 不需要 Hash trait, 但需要 Ord (不同于 HashMap)
    //   - range 使用 Bound 控制开闭区间
    //   - BTreeMap 的 B 因子为 6, 查询最多约 log_6(n) 层

    let mut map = BTreeMap::new();

    // 乱序插入, 自动排序
    map.insert(50, "fifty");
    map.insert(10, "ten");
    map.insert(30, "thirty");
    map.insert(40, "forty");
    map.insert(20, "twenty");

    // 迭代严格升序
    let keys: Vec<_> = map.keys().copied().collect();
    assert_eq!(keys, vec![10, 20, 30, 40, 50]);

    let values: Vec<_> = map.values().copied().collect();
    assert_eq!(values, vec!["ten", "twenty", "thirty", "forty", "fifty"]);

    // ── 极值操作 ──
    assert_eq!(map.first_key_value(), Some((&10, &"ten")));
    assert_eq!(map.last_key_value(), Some((&50, &"fifty")));

    // ── 范围查询 ──
    // 闭区间 20..=40
    let range: Vec<_> = map.range(20..=40).map(|(k, v)| (*k, *v)).collect();
    assert_eq!(range, vec![(20, "twenty"), (30, "thirty"), (40, "forty")]);

    // 半开区间 20..40 (不含 40)
    let range: Vec<_> = map.range(20..40).map(|(k, _)| *k).collect();
    assert_eq!(range, vec![20, 30]);

    // 使用 Bound 精确控制
    use std::ops::Bound;
    let range: Vec<_> = map
        .range((Bound::Excluded(&10), Bound::Included(&40)))
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(range, vec![20, 30, 40]);

    // ── pop 操作 ──
    let mut map = BTreeMap::from([(1, "a"), (3, "c"), (5, "e")]);
    assert_eq!(map.pop_first(), Some((1, "a")));
    assert_eq!(map.pop_last(), Some((5, "e")));
    assert_eq!(map.len(), 1);
    assert_eq!(map.first_key_value(), Some((&3, &"c")));

    // ── split_off 分割 ──
    let mut a: BTreeMap<i32, char> = BTreeMap::from([(1, 'a'), (2, 'b'), (3, 'c')]);
    let b = a.split_off(&2); // 键 >= 2 的元素移入 b
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 2);
    assert!(a.contains_key(&1));
    assert!(b.contains_key(&2));
    assert!(b.contains_key(&3));
}

#[test]
/// 测试: HashSet 交/并/差/子集/超集/对称差集运算
fn test_hashset_operations() {
    // 语法: HashSet 提供全套集合论运算, 均返回惰性迭代器
    //
    // 集合运算一览:
    //   - union(&other)             并集 A ∪ B
    //   - intersection(&other)     交集 A ∩ B
    //   - difference(&other)       差集 A - B
    //   - symmetric_difference     对称差集 (A-B) ∪ (B-A)
    //   - is_subset(&other)        self ⊆ other
    //   - is_superset(&other)      self ⊇ other
    //   - is_disjoint(&other)      self ∩ other == ∅
    //
    // 避坑:
    //   - 集合运算返回迭代器 (非具体集合), 需要 collect
    //   - 运算传入的是 &other 引用, 不消费原集合
    //   - HashSet 的 get 可以取回已存储的等效元素引用

    let a: HashSet<_> = [1, 2, 3, 4, 5].into_iter().collect();
    let b: HashSet<_> = [3, 4, 5, 6, 7].into_iter().collect();

    // ── 并集 A ∪ B ──
    let union: Vec<_> = {
        let mut v: Vec<_> = a.union(&b).copied().collect();
        v.sort();
        v
    };
    assert_eq!(union, vec![1, 2, 3, 4, 5, 6, 7]);

    // ── 交集 A ∩ B ──
    let inter: Vec<_> = {
        let mut v: Vec<_> = a.intersection(&b).copied().collect();
        v.sort();
        v
    };
    assert_eq!(inter, vec![3, 4, 5]);

    // ── 差集 A - B ──
    let diff: Vec<_> = {
        let mut v: Vec<_> = a.difference(&b).copied().collect();
        v.sort();
        v
    };
    assert_eq!(diff, vec![1, 2]);

    // ── 对称差集 (A-B) ∪ (B-A) ──
    let sym_diff: Vec<_> = {
        let mut v: Vec<_> = a.symmetric_difference(&b).copied().collect();
        v.sort();
        v
    };
    assert_eq!(sym_diff, vec![1, 2, 6, 7]);

    // ── 子集/超集判断 ──
    let small: HashSet<_> = [2, 4].into_iter().collect();
    let medium: HashSet<_> = [1, 2, 3, 4, 5].into_iter().collect();

    assert!(small.is_subset(&medium));
    assert!(!medium.is_subset(&small));
    assert!(medium.is_superset(&small));
    assert!(!small.is_superset(&medium));
    // 相等集合既子集又超集
    assert!(small.is_subset(&small));
    assert!(small.is_superset(&small));

    // ── 不相交 ──
    let x: HashSet<_> = [1, 2, 3].into_iter().collect();
    let y: HashSet<_> = [4, 5, 6].into_iter().collect();
    let z: HashSet<_> = [3, 4, 5].into_iter().collect();
    assert!(x.is_disjoint(&y));
    assert!(!x.is_disjoint(&z));

    // ── get 取回已有元素引用 ──
    let set: HashSet<String> = ["hello".into(), "world".into()].into_iter().collect();
    let stored = set.get("hello");
    assert!(stored.is_some());
    assert_eq!(stored.unwrap(), "hello");

    // ── take / replace ──
    let mut set: HashSet<i32> = [1, 2, 3].into_iter().collect();
    let taken = set.take(&2);
    assert_eq!(taken, Some(2));
    assert!(!set.contains(&2));

    let replaced = set.replace(4);
    assert_eq!(replaced, None); // 4 不存在, 插入
    let replaced = set.replace(1);
    assert_eq!(replaced, Some(1)); // 1 存在, 被替换 (值相同)
}

#[test]
/// 测试: BTreeSet 有序集合操作
fn test_btreeset_operations() {
    // 语法: BTreeSet 保证元素有序, 集合运算与 HashSet 相同但输出有序
    //
    // BTreeSet 特有方法:
    //   - first() / last()        极值
    //   - pop_first() / pop_last() 弹出极值
    //   - range(range)            范围查询
    //   - split_off(key)          分割
    //
    // 避坑:
    //   - 需要 T: Ord (不是 Hash + Eq)
    //   - 集合运算结果也是 BTreeSet 时有序列

    let a: BTreeSet<_> = [5, 1, 3, 7, 9].into_iter().collect();
    let b: BTreeSet<_> = [3, 7, 8, 10].into_iter().collect();

    // 自动排序
    let all: Vec<_> = a.iter().copied().collect();
    assert_eq!(all, vec![1, 3, 5, 7, 9]);

    // ── 并集 (有序) ──
    let union: Vec<_> = a.union(&b).copied().collect();
    assert_eq!(union, vec![1, 3, 5, 7, 8, 9, 10]);

    // ── 交集 (有序) ──
    let inter: Vec<_> = a.intersection(&b).copied().collect();
    assert_eq!(inter, vec![3, 7]);

    // ── 差集 (有序) ──
    let diff: Vec<_> = a.difference(&b).copied().collect();
    assert_eq!(diff, vec![1, 5, 9]);

    // ── 对称差集 (有序) ──
    let sym: Vec<_> = a.symmetric_difference(&b).copied().collect();
    assert_eq!(sym, vec![1, 5, 8, 9, 10]);

    // ── 极值 ──
    assert_eq!(a.first(), Some(&1));
    assert_eq!(a.last(), Some(&9));

    // ── 范围查询 ──
    let range: Vec<_> = a.range(3..=7).copied().collect();
    assert_eq!(range, vec![3, 5, 7]);

    // ── 弹出操作 ──
    let mut set: BTreeSet<_> = [100, 200, 300].into_iter().collect();
    assert_eq!(set.pop_first(), Some(100));
    assert_eq!(set.pop_last(), Some(300));
    assert_eq!(set.len(), 1);

    // ── HashSet 与 BTreeSet 互转 ──
    let hash_set: HashSet<_> = [3, 2, 1].into_iter().collect();
    let btree_set: BTreeSet<_> = hash_set.into_iter().collect();
    let sorted: Vec<_> = btree_set.into_iter().collect();
    assert_eq!(sorted, vec![1, 2, 3]);
}

#[test]
/// 测试: VecDeque 双端队列操作 (队列/栈/滑动窗口)
fn test_vecdeque_operations() {
    // 语法: VecDeque 基于环形缓冲区, 两端 O(1) 插入/删除
    //
    // 双端操作:
    //   - push_front / push_back     O(1) 摊还
    //   - pop_front / pop_back       O(1)
    //   - front / back               查看 (不移除)
    //   - front_mut / back_mut       可变查看
    //
    // 实用模式:
    //   - FIFO 队列: push_back + pop_front
    //   - LIFO 栈: push_back + pop_back (或用 Vec)
    //   - 滑动窗口: push_back + 条件 pop_front
    //   - 循环缓冲区: rotate_left/right
    //
    // 避坑:
    //   - 中间索引访问 O(1) 但比 Vec 慢 (需要取模)
    //   - 中间插入/删除 O(n)
    //   - make_contiguous() 确保内存连续, 会重排元素

    // ── FIFO 队列模式 ──
    let mut queue = VecDeque::new();
    queue.push_back(10);
    queue.push_back(20);
    queue.push_back(30);

    assert_eq!(queue.pop_front(), Some(10));
    assert_eq!(queue.pop_front(), Some(20));
    assert_eq!(queue.pop_front(), Some(30));
    assert_eq!(queue.pop_front(), None); // 空队列

    // ── LIFO 栈模式 ──
    let mut stack = VecDeque::new();
    stack.push_back("bottom");
    stack.push_back("middle");
    stack.push_back("top");

    assert_eq!(stack.pop_back(), Some("top"));
    assert_eq!(stack.pop_back(), Some("middle"));
    assert_eq!(stack.pop_back(), Some("bottom"));

    // ── 双端混合 ──
    let mut deque = VecDeque::new();
    deque.push_front(2);
    deque.push_back(3);
    deque.push_front(1);

    assert_eq!(deque, VecDeque::from([1, 2, 3]));
    assert_eq!(deque.pop_front(), Some(1));
    assert_eq!(deque.pop_back(), Some(3));
    assert_eq!(deque, VecDeque::from([2]));

    // ── 旋转 ──
    let mut d = VecDeque::from([1, 2, 3, 4, 5]);

    d.rotate_left(2);
    assert_eq!(d, VecDeque::from([3, 4, 5, 1, 2]));

    d.rotate_right(1);
    assert_eq!(d, VecDeque::from([2, 3, 4, 5, 1]));

    // rotate_left(n) 等价于 rotate_right(len - n)
    let mut d = VecDeque::from([1, 2, 3, 4, 5]);
    d.rotate_left(3);
    let mut e = VecDeque::from([1, 2, 3, 4, 5]);
    e.rotate_right(2);
    assert_eq!(d, e); // [4, 5, 1, 2, 3]

    // ── make_contiguous 保证内存连续 ──
    let mut deque: VecDeque<i32> = VecDeque::new();
    deque.push_back(1);
    deque.push_back(2);
    deque.pop_front(); // 留有空洞
    deque.push_back(3);
    deque.push_back(4);

    let slice = deque.make_contiguous();
    assert!(slice.len() >= 3); // 连续切片可传给 C FFI 或切片 API

    // ── 滑动窗口 ──
    let data = vec![1, 3, -1, -3, 5, 3, 6, 7];
    let window_size = 3;
    let mut window = VecDeque::new();
    let mut maxima = Vec::new();

    for &item in &data {
        window.push_back(item);
        if window.len() > window_size {
            window.pop_front();
        }
        if window.len() == window_size {
            maxima.push(*window.iter().max().unwrap());
        }
    }
    assert_eq!(maxima, vec![3, 3, 5, 5, 6, 7]);
}

#[test]
/// 测试: 自定义键类型的 Hash + Eq 实现
fn test_custom_hash_key() {
    // 语法: 自定义类型作为 HashMap/HashSet 的键时, 必须实现 Hash + Eq
    //
    // 实现方式:
    //   1. derive(Hash, PartialEq, Eq)  —— 全自动
    //   2. 手动 impl Hash + PartialEq + Eq —— 部分字段参与
    //   3. 使用第三方库 (如 derivative) 的 attribute 宏
    //
    // 避坑:
    //   - Hash 和 Eq 必须一致: 如果 a == b 则 hash(a) == hash(b)
    //   - Eq 要求 PartialEq, 不要漏掉
    //   - 浮点数做键要小心 NaN (FloatEq 问题), 考虑用 ordered_float crate

    use std::hash::{Hash, Hasher};

    // ── 方式一: 全自动派生 ──
    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    struct FullKey {
        name: String,
        id: u64,
    }

    let mut map = HashMap::new();
    map.insert(
        FullKey {
            name: "alice".into(),
            id: 1,
        },
        "value1",
    );
    map.insert(
        FullKey {
            name: "bob".into(),
            id: 2,
        },
        "value2",
    );

    assert_eq!(
        map.get(&FullKey {
            name: "alice".into(),
            id: 1,
        }),
        Some(&"value1")
    );

    // ── 方式二: 手动实现 —— 部分字段参与 ──
    #[derive(Debug, Clone)]
    struct PartialKey {
        id: u64,       // 参与 Hash/Eq
        cache: String, // 不参与
    }

    impl Hash for PartialKey {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state); // 仅 id 参与哈希
        }
    }
    impl PartialEq for PartialKey {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id // 仅 id 参与判等
        }
    }
    impl Eq for PartialKey {}

    let mut map = HashMap::new();
    map.insert(PartialKey { id: 1, cache: "abc".into() }, "first");
    map.insert(PartialKey { id: 2, cache: "xyz".into() }, "second");

    // 相同 id 被视为同一个键
    assert_eq!(
        map.get(&PartialKey {
            id: 1,
            cache: "different".into()
        }),
        Some(&"first")
    );

    // 插入相同 id 会覆盖
    let old = map.insert(PartialKey { id: 1, cache: "new".into() }, "replaced");
    assert_eq!(old, Some("first"));
    assert_eq!(map.len(), 2);

    // ── HashSet 自定义键 ──
    let mut set: HashSet<PartialKey> = HashSet::new();
    set.insert(PartialKey { id: 1, cache: "a".into() });
    set.insert(PartialKey { id: 2, cache: "b".into() });
    assert!(!set.insert(PartialKey { id: 1, cache: "c".into() })); // id=1 已存在
    assert_eq!(set.len(), 2);
}

#[test]
/// 测试: 迭代器收集到各种集合 (collect / from_iter / extend)
fn test_collect_into_collections() {
    // 语法: 集合类型实现 FromIterator, 可通过 .collect() 从迭代器构建
    //
    // 通用构建:
    //   - iter.collect::<TargetCollection>()  推断目标类型
    //   - HashMap::from_iter(iter)            显式构建
    //   - collection.extend(iter)             追加到已有集合
    //
    // 避坑:
    //   - collect 的类型必须明确, 必要时使用 ::< > 涡轮鱼语法
    //   - extend 不会清空已有元素
    //   - 从 Iterator<Item=(K,V)> 可 collect 为 HashMap/BTreeMap

    // ── collect 到 Vec ──
    let squares: Vec<i32> = (1..=5).map(|x| x * x).collect();
    assert_eq!(squares, vec![1, 4, 9, 16, 25]);

    // ── collect 到 HashMap ──
    let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
    let map: HashMap<&str, i32> = pairs.into_iter().collect();
    assert_eq!(map.len(), 3);
    assert_eq!(map["b"], 2);

    // ── collect 到 BTreeMap ──
    let map: BTreeMap<i32, char> = [(3, 'c'), (1, 'a'), (2, 'b')].into_iter().collect();
    let keys: Vec<_> = map.keys().copied().collect();
    assert_eq!(keys, vec![1, 2, 3]); // 按键排序

    // ── collect 到 HashSet (自动去重) ──
    let nums = vec![3, 1, 4, 1, 5, 9, 2, 6, 5];
    let set: HashSet<i32> = nums.into_iter().collect();
    assert_eq!(set.len(), 7); // 去除了重复的 1 和 5

    // ── collect 到 VecDeque ──
    let deque: VecDeque<i32> = (0..5).collect();
    assert_eq!(deque, VecDeque::from([0, 1, 2, 3, 4]));

    // ── collect 到 BinaryHeap ──
    let heap: BinaryHeap<i32> = [3, 1, 4, 1, 5].into_iter().collect();
    assert_eq!(heap.peek(), Some(&5)); // 最大堆

    // ── collect 到 LinkedList ──
    use std::collections::LinkedList;
    let list: LinkedList<i32> = [1, 2, 3].into_iter().collect();
    assert_eq!(list.len(), 3);

    // ── from_iter 显式构建 ──
    let map: HashMap<&str, i32> = HashMap::from_iter([("x", 10), ("y", 20)]);
    assert_eq!(map.get("x"), Some(&10));

    // ── extend 追加 ──
    let mut set = HashSet::from([1, 2, 3]);
    set.extend([3, 4, 5]); // 3 已存在不会重复添加
    assert_eq!(set.len(), 5); // {1, 2, 3, 4, 5}

    // 追加到 HashMap
    let mut map = HashMap::new();
    map.extend([("a", 1), ("b", 2)]);
    map.extend([("b", 20), ("c", 3)]); // "b" 会被覆盖
    assert_eq!(map["b"], 20);

    // ── enumerate + collect ──
    let map: HashMap<usize, &str> = ["zero", "one", "two"]
        .into_iter()
        .enumerate()
        .collect();
    assert_eq!(map[&0], "zero");
}

#[test]
/// 测试: 集合操作的实际性能特征 (操作计数验证)
fn test_collection_complexity() {
    // 语法: 通过操作计数验证各集合的复杂度特性 (非精确 benchmark)
    //
    // 验证目标:
    //   - HashMap: O(1) 插入/查询
    //   - BTreeMap: O(log n) 查找
    //   - HashSet: O(1) 成员检测
    //   - BinaryHeap: O(log n) push / O(1) peek
    //   - VecDeque: O(1) 两端操作
    //
    // 避坑:
    //   - 这是复杂度特征测试, 不是精确性能 benchmark
    //   - 真实性能受缓存、分配器等影响
    //   - 需要精确 benchmark 请使用 criterion 库

    const N: usize = 1000;

    // ── HashMap: O(1) 插入, O(1) 查询 ──
    let mut map = HashMap::with_capacity(N);
    for i in 0..N {
        map.insert(i, i * 2);
    }
    // 插入后能查询到
    for i in 0..N {
        assert_eq!(map.get(&i), Some(&(i * 2)));
    }
    // 不存在的键
    assert_eq!(map.get(&N), None);

    // ── BTreeMap: 有序性验证 ──
    let mut btree = BTreeMap::new();
    for i in (0..N).rev() {
        btree.insert(i, format!("val_{i}"));
    }
    // 验证有序: 键必须严格递增
    let keys: Vec<_> = btree.keys().copied().collect();
    for w in keys.windows(2) {
        assert!(w[0] < w[1]);
    }

    // ── HashSet: O(1) 去重 ──
    let mut set = HashSet::with_capacity(N);
    for i in 0..N {
        set.insert(i);
    }
    // 重复元素无法插入
    assert!(!set.insert(0));
    assert_eq!(set.len(), N);
    assert!(set.contains(&(N - 1)));
    assert!(!set.contains(&N));

    // ── BinaryHeap: 堆性质 ──
    use std::cmp::Reverse;
    let mut heap = BinaryHeap::with_capacity(N);
    for i in 0..N {
        heap.push(Reverse(i)); // 最小堆
    }
    // 按升序弹出
    for i in 0..N {
        assert_eq!(heap.pop(), Some(Reverse(i)));
    }
    assert!(heap.is_empty());

    // push_pop 批量操作
    for i in 0..N {
        heap.push(Reverse(i));
    }
    // peek 是 O(1)
    assert_eq!(heap.peek(), Some(&Reverse(0)));

    // ── VecDeque: O(1) 两端 ──
    let mut deque = VecDeque::with_capacity(N);
    for i in 0..N {
        deque.push_back(i);
    }
    assert_eq!(deque.len(), N);
    assert_eq!(deque.front(), Some(&0));
    assert_eq!(deque.back(), Some(&(N - 1)));

    // 两端弹出各一半
    for _ in 0..N / 2 {
        deque.pop_front();
    }
    for _ in 0..N / 2 {
        deque.pop_back();
    }
    assert!(deque.is_empty());

    // ── HashMap 重复键覆盖 ──
    let mut map = HashMap::new();
    map.insert("key", 1);
    let old = map.insert("key", 2);
    assert_eq!(old, Some(1));
    assert_eq!(map["key"], 2);
    assert_eq!(map.len(), 1);

    // ── 大容量验证 ──
    let large = 10000;
    let mut map = HashMap::with_capacity(large);
    for i in 0..large {
        map.insert(i, i);
    }
    assert_eq!(map.len(), large);
    for i in 0..large {
        assert_eq!(map[&i], i);
    }
}
