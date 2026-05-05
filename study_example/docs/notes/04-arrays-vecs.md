# 数组与向量

## 一、三种序列类型对比

Rust 中最核心的三种序列类型，各占不同的内存位置和生命周期：

| 类型 | 内存位置 | 大小 | 长度 | 结构 | 典型用途 |
|------|----------|------|------|------|----------|
| `[T; N]` | 栈 | 编译期固定 | 编译期常量 | 连续内存块 | 固定大小缓冲区 |
| `&[T]` | 指向栈/堆的视图 | 16字节（ptr+len） | 运行时已知 | 胖指针 | 函数参数、子序列 |
| `Vec<T>` | 堆 | 24字节（ptr+len+cap） | 运行时动态 | 三段结构 | 动态集合，最常用 |

```rust
let arr: [i32; 4] = [1, 2, 3, 4];       // 栈上分配，16字节
let slice: &[i32] = &arr[1..3];          // 胖指针(ptr+len)，16字节
let vec: Vec<i32> = vec![1, 2, 3, 4];    // 堆上分配数据，栈上24字节
let len = std::mem::size_of::<Vec<i32>>(); // 24字节 (x86_64)
```

> 选择合适的数据结构不是做加法，而是做减法——用最少的开销满足你的需求，多余的容量和复制都是犯罪。

---

## 二、数组 [T; N]

数组所有元素**类型相同、长度固定、栈上分配**。

```rust
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let zeros = [0u8; 1024];               // 1024 个零值字节
let nums = [42; 3];                     // [42, 42, 42]

// 常量泛型：数组长度作为类型参数
fn first_n<const N: usize>(arr: [i32; N]) -> Option<i32> {
    if N == 0 { None } else { Some(arr[0]) }
}

fn sum<const N: usize>(arr: [i32; N]) -> i32 {
    arr.iter().sum()
}

let a = first_n([1, 2, 3]);    // 编译器自动推断 N = 3

// 多维数组
let matrix: [[i32; 3]; 2] = [
    [1, 2, 3],
    [4, 5, 6],
];
assert_eq!(matrix[1][2], 6);
// 内存布局：连续的 [i32; 6]，行优先
```

数组的 `len()` 是编译期常量，可用于 `const` 上下文：

```rust
const ARR: [i32; 10] = [0; 10];
const LEN: usize = ARR.len();   // 编译期求值
```

---

## 三、切片 &[T]

切片是**不拥有数据**的视图，16字节胖指针（8字节指针 + 8字节长度）。

```rust
let arr = [10, 20, 30, 40, 50];

let all: &[i32] = &arr;         // &[10, 20, 30, 40, 50]
let part: &[i32] = &arr[1..4];  // &[20, 30, 40]
let to_end: &[i32] = &arr[2..]; // &[30, 40, 50]
let from_start: &[i32] = &arr[..3]; // &[10, 20, 30]

// 胖指针结构（概念性）
// &[T] = { ptr: *const T, len: usize }
assert_eq!(std::mem::size_of::<&[i32]>(), 16);
```

### 切片模式匹配

```rust
let arr = [1, 2, 3, 4, 5];

match &arr[..] {
    [] => println!("空切片"),
    [first] => println!("单元素: {first}"),
    [first, rest @ ..] => println!("首元素: {first}, 剩余: {rest:?}"),
}

match &arr[..] {
    [first, rest @ .., last] => {
        println!("首: {first}, 尾: {last}, 中间: {rest:?}");
    }
    _ => {}
}
```

> 切片不拥有数据，这是一种哲学——你可以观察世界，但不能占有它。借用检查器保证你观察时世界不会被改动。

---

## 四、Vec 创建与基础操作

```rust
let v1: Vec<i32> = Vec::new();              // 空向量，长度0，容量0
let v2 = vec![1, 2, 3];                      // 宏创建
let v3 = vec![0; 100];                       // 100个零
let v4: Vec<_> = (0..10).collect();          // 从迭代器收集
let v5 = Vec::from([1, 2, 3]);              // 从数组创建
let v6 = Vec::with_capacity(1000);           // 预分配容量

// 基本操作
let mut v = Vec::new();
v.push(1);                                   // O(1) 摊销
v.push(2);
v.push(3);

assert_eq!(v.len(), 3);                      // 元素数量
assert!(v.capacity() >= 3);                  // 已分配容量
assert_eq!(v.pop(), Some(3));                // 从末尾弹出
assert_eq!(v.pop(), Some(2));

// 索引访问
let first = &v[0];                           // 越界会 panic
let safe = v.get(1);                         // 返回 Option<&T>
let last = v.last();                         // 返回 Option<&T>
```

---

## 五、Vec 内存布局与增长策略

```rust
// Vec 的栈上结构（24字节）
// struct Vec<T> {
//     ptr: *mut T,    // 8 字节 - 堆数据指针
//     len: usize,     // 8 字节 - 当前元素数
//     cap: usize,     // 8 字节 - 已分配容量
// }

let mut v: Vec<i32> = Vec::with_capacity(1);
assert_eq!(v.capacity(), 1);

v.push(1);
// 容量翻倍增长策略：
v.push(2);  // 容量: 1 → 2 → 4 → 8 → 16 ...

v.shrink_to_fit();     // 释放多余容量，使 cap == len
v.reserve(100);        // 预留至少100个额外空间
v.reserve_exact(100);  // 精确预留
```

增长策略：当 `len == cap` 时，Vec 会分配 `max(old_cap * 2, required)` 的新空间，然后移动数据。

---

## 六、Vec 插入与删除

```rust
let mut v = vec![1, 2, 3, 4, 5];

// insert: O(n) — 需移动后续元素
v.insert(2, 9);                  // [1, 2, 9, 3, 4, 5]
v.insert(v.len(), 6);            // 末尾插入 = push

// remove: O(n) — 需移动后续元素
let removed = v.remove(1);      // removed = 2, v = [1, 9, 3, 4, 5, 6]

// swap_remove: O(1) — 交换到末尾再弹出（不保留顺序！）
v.swap_remove(0);                // 把v[0]与v[last]交换后 pop

// retain: O(n) — 保留满足条件的元素
let mut v = vec![1, 2, 3, 4, 5, 6];
v.retain(|&x| x % 2 == 0);      // [2, 4, 6]

// drain: 批量移出元素（返回迭代器）
let mut v = vec![1, 2, 3, 4, 5];
let drained: Vec<_> = v.drain(1..4).collect(); // [2, 3, 4]
assert_eq!(v, vec![1, 5]);

// extract_if (前身是 drain_filter): 条件提取
let mut v = vec![1, 2, 3, 4, 5, 6];
let extracted: Vec<_> = v.extract_if(|x| *x % 2 == 0).collect();
assert_eq!(v, vec![1, 3, 5]);
assert_eq!(extracted, vec![2, 4, 6]);

// truncate: 截断到指定长度
v.truncate(1);                   // [1]

// clear: 清空所有元素
v.clear();                       // len=0, cap不变
```

> 记住操作的复杂度，就像拳击手记住自己的拳头有多重——O(n) 的 `insert` 在循环中会成为 O(n²)，这就是性能灾难的起点。

---

## 七、Vec 排序

```rust
let mut v = vec![5, 3, 1, 4, 2];

v.sort();                                 // 稳定排序，保留相等元素的相对顺序
v.sort_unstable();                        // 不稳定排序（快排变种），更快
v.sort_by(|a, b| b.cmp(a));               // 自定义比较器（降序）
v.sort_by_key(|x| x.abs());               // 按键排序
v.sort_by_cached_key(|x| expensive(x));   // 缓存键值，避免重复计算

// 检查是否已排序
assert!(v.is_sorted());
assert!(v.is_sorted_by(|a, b| a <= b));

// 选择第 k 小的元素（部分排序，O(n)）
let mut v = vec![3, 1, 4, 1, 5, 9, 2, 6];
let len = v.len();
let (left, _mid, right) = v.select_nth_unstable(3);  // 选择第3个元素
// left 全部 <= mid 全部 <= right
```

---

## 八、Vec 搜索

```rust
let v = vec![1, 2, 3, 4, 5];

assert!(v.contains(&3));

// binary_search: 前提是数组已排序！
let sorted = vec![10, 20, 30, 40, 50];
assert_eq!(sorted.binary_search(&30), Ok(2));          // 找到
assert_eq!(sorted.binary_search(&25), Err(3));         // 没找到，应插入到索引 3

// binary_search_by
assert_eq!(
    sorted.binary_search_by(|x| x.cmp(&30)),
    Ok(2)
);

// binary_search_by_key
let items = vec![("a", 10), ("b", 20), ("c", 30)];
assert_eq!(
    items.binary_search_by_key(&20, |&(_, v)| v),
    Ok(1)
);
```

---

## 九、Vec 转换

```rust
let v = vec![1, 2, 3];

let s: &[i32] = v.as_slice();             // 获取切片引用
let ms: &mut [i32] = &mut v.as_mut_slice(); // 获取可变切片引用
let ptr: *const i32 = v.as_ptr();          // 获取裸指针

// Vec 与数组互转（需要 TryInto，因为长度可能不匹配）
let arr: [i32; 3] = v.try_into().unwrap();  // Vec<T> → [T; N]

// Box<[T]>：不可调整大小的堆数组（比 Vec 少一个 usize）
let heap_arr: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
assert_eq!(std::mem::size_of::<Box<[i32]>>(), 16);  // 比 Vec 少 8 字节(cap)
```

---

## 十、VecDeque — 双端队列

环形缓冲区实现，前后均可 O(1) 操作：

```rust
use std::collections::VecDeque;

let mut deque: VecDeque<i32> = VecDeque::new();
let mut deque = VecDeque::with_capacity(10);

deque.push_back(1);              // 尾部压入
deque.push_front(0);             // 头部压入
deque.push_back(2);

assert_eq!(deque.pop_front(), Some(0));  // 头部弹出
assert_eq!(deque.pop_back(), Some(2));   // 尾部弹出

// make_contiguous：把环形缓冲区整理为连续切片
let mut deque = VecDeque::from([1, 2, 3]);
deque.rotate_left(1);            // 左移 [2, 3, 1]
let slice = deque.make_contiguous(); // 获得 &mut [T]

// 作为切片访问（可能分两段）
let (front, back) = deque.as_slices();
```

| 操作 | Vec | VecDeque |
|------|-----|----------|
| `push/pop 末尾` | O(1)* | O(1)* |
| `push/pop 首端` | O(n) | O(1)* |
| 索引访问 | O(1) | O(1) |
| 插入/删除中间 | O(n) | O(n) |

---

## 十一、二维数组三种方案

```rust
// 方案一：固定大小二维数组 [[T; M]; N]
let matrix: [[i32; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
// 内存：连续 24 字节，缓存友好
// 限制：每行长度必须相同，编译期确定

// 方案二：锯齿数组 Vec<Vec<T>>
let jagged: Vec<Vec<i32>> = vec![
    vec![1, 2],
    vec![3, 4, 5],
    vec![6],
];
// 内存：每行独立分配，缓存不友好
// 优势：每行长度可以不同，运行时动态

// 方案三：扁平化 Vec（推荐用于性能敏感场景）
let cols = 3;
let flat: Vec<i32> = vec![1, 2, 3, 4, 5, 6];
let value = flat[row * cols + col];  // 手动索引计算
// 内存：连续数组，CPU 缓存最优
```

| 维度 | 固定 `[[T; M]; N]` | 锯齿 `Vec<Vec<T>>` | 扁平 `Vec<T>` |
|------|---------------------|---------------------|---------------|
| 内存连续性 | 完全连续 | 每行独立 | 完全连续 |
| 缓存效率 | 高 | 低 | 最高 |
| 灵活性 | 低（固定大小） | 高（各行可不同长度） | 中（需手动计算索引） |
| 分配开销 | 0 | N次分配 | 1次分配 |
| 适用场景 | 小矩阵、编译期已知 | 动态稀疏数据 | 大矩阵、性能敏感 |

> 数据结构的选择是运行时性能的静态决策——扁平化一个二维数组可能让你的程序快十倍，而改动只需一行代码。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `Vec::with_capacity(0)` 实际容量可能 >0 | 分配器最小对齐导致 | 不要假设 `capacity() == 0` |
| `binary_search` 用于未排序数组 | 结果无意义 | 先 `sort()` 再搜索 |
| `insert(0, ...)` 频繁调用的 O(n²) 性能 | 每次需移动所有元素 | 使用 `VecDeque` 替代 |
| `push` 导致迭代器失效 | 触发了重新分配 | 收集完再 push，或提前 `reserve()` |
| `remove()` 返回的元素如果未使用会被 drop | 元素被移出所有权 | 用 `mem::forget` 延迟 drop（通常不需要） |
| `swap_remove` 改变了元素顺序 | 设计如此 | 如果顺序重要，使用 `remove()` |
| `.get()` 返回 `Option<&T>`，`.as_ptr()` 不检查边界 | 裸指针无安全检查 | 优先使用 `get()`，裸指针仅用于 FFI 或性能热点 |
| 多维数组 `[i32; 3]` 是 Copy 类型 | 整个数组在栈上 | 大数组考虑使用 `Box<[i32; N]>` 或 `Vec` |
| `Vec::drain()` 后迭代器未消费完，drop 时会 panic | drain 迭代器会执行清理 | 确保 drain 迭代器被完全消费 |
| `vec![]` 是宏，不是函数 | 需要注意宏的展开规则 | 避免在宏参数中使用复杂表达式 |

> **详见测试**: `tests/rust_features/04_arrays_and_vecs.rs`
