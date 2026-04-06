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
