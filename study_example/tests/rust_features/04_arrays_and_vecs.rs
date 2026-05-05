// ---------------------------------------------------------------------------
// 1.2 数组与 Vec 基础
// ---------------------------------------------------------------------------

#[test]
/// 测试: 数组基础操作 (创建/索引/first/last/get/contains/swap/reverse)
fn test_array_basics() {
    // 语法: [T; N] 固定大小数组, 大小是类型的一部分, 分配在栈上(大数组可能在数据段)
    //
    // 创建方式:
    //   - [1, 2, 3]              字面量初始化
    //   - [0; 100]               重复语法, 创建 100 个 0 (要求 T: Copy)
    //   - [T::default(); N]      默认值填充 (要求 T: Default + Copy)
    //   - [expr; N]              任意表达式重复 (要求 T: Copy)
    //
    // 常用方法:
    //   - arr.len() -> usize     长度(编译期已知)
    //   - arr.first() -> Option<&T>  安全获取首元素
    //   - arr.last() -> Option<&T>   安全获取尾元素
    //   - arr.get(i) -> Option<&T>   安全索引
    //   - arr.contains(&val) -> bool 是否包含某值 (要求 T: PartialEq)
    //   - arr.iter()             不可变迭代器
    //   - arr.iter_mut()         可变迭代器
    //   - arr.into_iter()        消费迭代器 (Rust 2021+)
    //   - arr.swap(i, j)         交换两个元素
    //   - arr.reverse()          原地反转
    //
    // 避坑:
    //   - 数组大小必须是编译期常量, 不能是运行时变量
    //   - 数组越界索引 arr[i] 会 panic, 不会返回 None
    //   - 大数组(>几 MB)可能导致栈溢出, 应用 Box<[T; N]> 或 Vec
    //   - [expr; N] 中 expr 只计算一次然后 Copy, 不是计算 N 次
    //   - 数组不能直接 push/pop, 需转为 Vec 或使用固定大小算法
    //   - Rust 2021 之前 arr.into_iter() 等价于 arr.iter(), 2021 起消费数组
    //
    let arr = [1, 2, 3, 4, 5];
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0], 1);
    assert_eq!(arr.first(), Some(&1));
    assert_eq!(arr.last(), Some(&5));
    assert_eq!(arr.get(2), Some(&3));
    assert_eq!(arr.get(10), None);
    assert!(arr.contains(&3));
    assert!(!arr.contains(&10));

    // 重复语法
    let zeros = [0; 5];
    assert_eq!(zeros, [0, 0, 0, 0, 0]);

    // 交换和反转
    let mut arr = [1, 2, 3, 4, 5];
    arr.swap(0, 4);
    assert_eq!(arr, [5, 2, 3, 4, 1]);

    arr.reverse();
    assert_eq!(arr, [1, 4, 3, 2, 5]);

    // 边界用例: 空数组
    let empty: [i32; 0] = [];
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(empty.first(), None);
    assert_eq!(empty.last(), None);
    assert_eq!(empty.get(0), None);
    assert!(!empty.contains(&1));

    // 边界用例: 单元素数组
    let single = [42i32];
    assert_eq!(single.first(), Some(&42));
    assert_eq!(single.last(), Some(&42));
    assert_eq!(single.first(), single.last()); // 首尾相同

    // 边界用例: is_empty 和 len
    assert!(!arr.is_empty());
    assert!(empty.is_empty());

    // 边界用例: 迭代器收集
    let arr = [1, 2, 3];
    let doubled: Vec<i32> = arr.iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6]);
}

#[test]
/// 测试: Vec 基础操作 (push/pop/insert/remove/append/extend/retain/drain/splice/resize)
fn test_vec_basics() {
    // 语法: Vec<T> 动态数组, 分配在堆上, 容量可增长
    //
    // 创建方式:
    //   - vec![1, 2, 3]          宏创建
    //   - Vec::new()             空 Vec
    //   - Vec::with_capacity(n)  预分配容量
    //   - vec![0; n]             重复语法
    //   - (0..n).collect()       从迭代器收集
    //   - Vec::from(arr)         从数组转换
    //
    // 添加元素:
    //   - push(val)              尾部添加
    //   - insert(idx, val)       指定位置插入 (O(n))
    //   - append(&mut other)     追加另一个 Vec(other 被清空)
    //   - extend(iter)           从迭代器扩展
    //   - resize(len, val)       调整大小, 不足用 val 填充
    //   - resize_with(len, f)    调整大小, 不足用闭包生成
    //
    // 移除元素:
    //   - pop() -> Option<T>     移除尾部元素
    //   - remove(idx) -> T       移除指定位置 (O(n))
    //   - swap_remove(idx) -> T  交换到尾部后移除 (O(1), 不保序)
    //   - truncate(len)          截断到指定长度
    //   - clear()                清空(保留容量)
    //   - retain(f)              保留满足条件的元素
    //   - drain(range)           移除范围元素并返回迭代器
    //   - split_off(at)          分割为两个 Vec
    //   - splice(range, iter)    替换范围内容
    //   - extract_if(f)          移除满足条件的元素并返回迭代器 (Rust 1.86+)
    //
    // 查询:
    //   - len() / is_empty()     长度/是否为空
    //   - capacity()             容量(不重新分配的最大元素数)
    //   - first() / last()       首尾元素
    //   - get(i) / get_mut(i)    安全索引
    //   - contains(&val)         是否包含
    //   - binary_search(&val)    二分查找 (需已排序)
    //   - windows(n)             滑动窗口迭代器
    //   - chunks(n) / chunks_exact(n)  分块迭代器
    //
    // 容量管理:
    //   - reserve(n)             预留至少 n 个额外容量
    //   - reserve_exact(n)       精确预留 n 个额外容量
    //   - shrink_to_fit()        释放多余容量
    //   - shrink_to(min)         缩小到至少 min 容量
    //   - truncate()             截断(不释放容量)
    //
    // 避坑:
    //   - insert/remove 是 O(n), 频繁在中间操作考虑 LinkedList 或 VecDeque
    //   - swap_remove 不保持顺序, 适合不关心顺序的场景
    //   - retain 遍历时不能修改其他元素, 闭包只决定保留/移除
    //   - drain 返回的迭代器必须被消费, 否则元素不移除
    //   - Vec 扩容策略: 旧容量 < 1024 时翻倍, 否则增长 1.5 倍
    //   - 索引越界 panic, 用 get() 安全访问
    //
    let mut v = vec![1, 2, 3];
    v.push(4);
    assert_eq!(v, vec![1, 2, 3, 4]);
    assert_eq!(v.pop(), Some(4));

    // 插入
    v.insert(1, 10);
    assert_eq!(v, vec![1, 10, 2, 3]);

    // 移除
    assert_eq!(v.remove(1), 10);
    assert_eq!(v, vec![1, 2, 3]);

    // swap_remove (O(1), 不保序)
    let mut v = vec![1, 2, 3, 4, 5];
    let removed = v.swap_remove(1); // 用最后一个元素填充位置 1
    assert_eq!(removed, 2);
    assert_eq!(v, vec![1, 5, 3, 4]); // 顺序改变!

    // append
    let mut v1 = vec![1, 2];
    let mut v2 = vec![3, 4];
    v1.append(&mut v2);
    assert_eq!(v1, vec![1, 2, 3, 4]);
    assert!(v2.is_empty());

    // extend
    let mut v = vec![1, 2];
    v.extend([3, 4, 5]);
    assert_eq!(v, vec![1, 2, 3, 4, 5]);

    // retain
    let mut v = vec![1, 2, 3, 4, 5, 6];
    v.retain(|x| x % 2 == 0);
    assert_eq!(v, vec![2, 4, 6]);

    // drain
    let mut v = vec![1, 2, 3, 4, 5];
    let drained: Vec<i32> = v.drain(1..4).collect();
    assert_eq!(drained, vec![2, 3, 4]);
    assert_eq!(v, vec![1, 5]);

    // splice
    let mut v = vec![1, 2, 3, 4, 5];
    v.splice(1..4, vec![20, 30]);
    assert_eq!(v, vec![1, 20, 30, 5]);

    // resize
    let mut v = vec![1, 2, 3];
    v.resize(5, 0);
    assert_eq!(v, vec![1, 2, 3, 0, 0]);

    v.resize_with(7, || 99);
    assert_eq!(v, vec![1, 2, 3, 0, 0, 99, 99]);

    // 边界用例: pop 空 Vec
    let mut v: Vec<i32> = Vec::new();
    assert_eq!(v.pop(), None);

    // 边界用例: remove 第一个和最后一个
    let mut v = vec![1, 2, 3, 4, 5];
    assert_eq!(v.remove(0), 1); // 删除首元素
    assert_eq!(v.remove(3), 5); // 删除尾元素 (索引变化后)
    assert_eq!(v, vec![2, 3, 4]);

    // 边界用例: truncate
    let mut v = vec![1, 2, 3, 4, 5];
    v.truncate(3);
    assert_eq!(v, vec![1, 2, 3]);
    v.truncate(10); // 大于 len 无效果
    assert_eq!(v, vec![1, 2, 3]);
    v.truncate(0);
    assert!(v.is_empty());

    // 边界用例: insert 在末尾
    let mut v = vec![1, 2, 3];
    v.insert(3, 99); // 等价于 push
    assert_eq!(v, vec![1, 2, 3, 99]);
}

#[test]
/// 测试: Vec 容量管理 (with_capacity/reserve/shrink_to_fit/增长策略)
fn test_vec_capacity() {
    // 语法: Vec 容量管理, 避免不必要的重新分配
    //
    // 扩容规则:
    //   - 初始容量 0, 第一次 push 分配 4 个元素空间
    //   - 容量 < 1024 时翻倍增长: 4 → 8 → 16 → ...
    //   - 容量 >= 1024 时增长 1.5 倍
    //   - 实际增长因子可能因分配器对齐而略有不同
    //
    // 避坑:
    //   - with_capacity 只分配, 长度仍为 0
    //   - reserve 是"至少"预留, 实际可能更多(分配器对齐)
    //   - shrink_to_fit 不保证完全释放, 取决于分配器
    //   - 频繁 push 不预留容量会导致多次重新分配和内存拷贝
    //   - truncate 不释放容量
    //
    let mut v = Vec::with_capacity(10);
    assert_eq!(v.len(), 0);
    assert_eq!(v.capacity(), 10);

    v.push(1);
    assert_eq!(v.len(), 1);
    assert_eq!(v.capacity(), 10); // 未超过预分配

    // reserve
    v.reserve(20);
    assert!(v.capacity() >= 21);

    // shrink
    v.truncate(1);
    v.shrink_to_fit();
    assert!(v.capacity() >= 1);

    // 边界用例: Vec::new() 初始容量为 0
    let v: Vec<i32> = Vec::new();
    assert_eq!(v.capacity(), 0);

    // 边界用例: 增长策略验证 (翻倍增长)
    let mut v = Vec::new();
    let mut prev_cap = v.capacity();
    for i in 0..100 {
        if v.len() == v.capacity() {
            v.push(i);
            let new_cap = v.capacity();
            // 容量 < 1024 时翻倍 (或初始分配 4)
            if prev_cap > 0 && prev_cap < 1024 {
                assert_eq!(new_cap, prev_cap * 2);
            }
            prev_cap = new_cap;
        } else {
            v.push(i);
        }
    }
    assert_eq!(v.len(), 100);

    // 边界用例: with_capacity(0) 的行为
    let v: Vec<i32> = Vec::with_capacity(0);
    assert_eq!(v.capacity(), 0);
    assert_eq!(v.len(), 0);

    // 边界用例: reserve_exact 精确预留
    let mut v = vec![1, 2, 3];
    let old_cap = v.capacity();
    v.reserve_exact(5);
    assert!(v.capacity() >= old_cap + 5);

    // 边界用例: shrink_to
    let mut v = vec![1, 2, 3];
    v.reserve(100);
    v.shrink_to(3); // 缩小到至少 3
    assert!(v.capacity() >= 3);
}

#[test]
/// 测试: Vec 查找和分块 (binary_search/chunks/chunks_exact/windows)
fn test_vec_search_and_chunks() {
    // 语法: Vec/切片的查找和分块方法
    //
    // 查找:
    //   - binary_search(&val) -> Result<usize, usize>
    //     Ok(idx): 找到, 返回索引; Err(idx): 未找到, idx 为应插入位置
    //   - binary_search_by(cmp)  自定义比较
    //   - binary_search_by_key(key, f)  按 key 比较
    //
    // 分块:
    //   - chunks(n)           每块 n 个元素, 最后一块可能 < n
    //   - chunks_exact(n)     每块严格 n 个, 余数通过 remainder() 获取
    //   - windows(n)          滑动窗口, 相邻窗口重叠 n-1 个元素
    //   - array_chunks::<N>() 返回 [[T; N]] 数组引用 (需足够元素)
    //
    // 避坑:
    //   - binary_search 要求数据已排序, 否则结果未定义
    //   - 有重复元素时 binary_search 不保证返回哪个索引
    //   - windows(n) 当 n > len 时返回空迭代器
    //   - chunks_exact 的余数必须手动处理, 不会自动包含
    //
    // 二分查找
    let v = vec![1, 3, 5, 7, 9];
    assert_eq!(v.binary_search(&5), Ok(2));
    assert_eq!(v.binary_search(&4), Err(2)); // 应插入位置 2

    // 自定义比较
    let names = vec!["alice", "bob", "charlie"];
    assert!(
        names
            .binary_search(&"bob")
            .is_ok()
    );

    // chunks
    let v = vec![1, 2, 3, 4, 5];
    let chunks: Vec<&[i32]> = v.chunks(2).collect();
    assert_eq!(chunks[0], &[1, 2]);
    assert_eq!(chunks[1], &[3, 4]);
    assert_eq!(chunks[2], &[5]);

    // chunks_exact
    let chunks: Vec<&[i32]> = v.chunks_exact(2).collect();
    assert_eq!(chunks[0], &[1, 2]);
    assert_eq!(chunks[1], &[3, 4]);
    // 余数需要手动获取 (chunks_exact 返回的迭代器有 remainder 方法)

    // windows
    let v = vec![1, 2, 3, 4];
    let windows: Vec<_> = v.windows(2).collect();
    assert_eq!(windows, vec![&[1, 2], &[2, 3], &[3, 4]]);

    // windows 步长为 1, 要非重叠分块用 chunks

    // 边界用例: binary_search 完整结果集
    let v = vec![1, 3, 5, 7, 9, 11, 13];
    // 查找小于全部的值
    assert_eq!(v.binary_search(&0), Err(0));
    // 查找大于全部的值
    assert_eq!(v.binary_search(&15), Err(7));
    // 查找中间不存在的值
    assert_eq!(v.binary_search(&6), Err(3)); // 5之后, 7之前 → 索引3
    assert_eq!(v.binary_search(&8), Err(4)); // 7之后, 9之前 → 索引4

    // 边界用例: binary_search_by
    let v = vec![1, 3, 5, 7, 9];
    let idx = v.binary_search_by(|&x| x.cmp(&5));
    assert_eq!(idx, Ok(2));

    // 边界用例: binary_search_by_key
    let v = vec![("a", 1), ("b", 3), ("c", 5)];
    let idx = v.binary_search_by_key(&3, |&(_, k)| k);
    assert_eq!(idx, Ok(1));
    // 找不到的 key
    let idx = v.binary_search_by_key(&4, |&(_, k)| k);
    assert_eq!(idx, Err(2)); // 应插入在索引 2

    // 边界用例: windows 大小 > len
    let v = vec![1, 2, 3];
    let windows: Vec<_> = v.windows(4).collect();
    assert!(windows.is_empty());

    // 边界用例: windows 大小 == len
    let v = vec![1, 2, 3];
    let windows: Vec<_> = v.windows(3).collect();
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0], &[1, 2, 3]);

    // 边界用例: chunks 的 n > len
    let v = vec![1, 2, 3];
    let chunks: Vec<_> = v.chunks(10).collect();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], &[1, 2, 3]);
}

#[test]
/// 测试: Vec 排序方法 (sort/sort_by/sort_by_key/sort_unstable/sort_by_cached_key/dedup)
fn test_vec_sort() {
    // 语法: Vec/切片排序方法
    //
    // 排序方法:
    //   - sort()              原地排序 (要求 T: Ord), 混合排序(快排+归并), 稳定
    //   - sort_unstable()     原地不稳定排序 (更快, 模式破坏快速排序), 不保序
    //   - sort_by(cmp)        自定义比较函数 (稳定)
    //   - sort_by_key(f)      按 key 排序 (稳定)
    //   - sort_by_cached_key(f) 按 key 排序, 缓存 key 结果 (key 计算慢时适用, 稳定)
    //   - reverse()           反转
    //   - dedup()             去重(需已排序, 只去相邻重复)
    //   - dedup_by(same)      自定义去重
    //
    // 避坑:
    //   - sort 是稳定排序, 相等元素保持原顺序
    //   - sort_unstable 更快但不稳定, 适合基本类型
    //   - dedup 只去除相邻重复, 必须先 sort
    //   - sort_by_key 的 key 函数可能被多次调用, 慢时用 sort_by_cached_key
    //   - 对超大 Vec 排序注意内存使用(归并排序需要额外空间)
    //
    let mut v = vec![5, 2, 8, 1, 9, 3];
    v.sort();
    assert_eq!(v, vec![1, 2, 3, 5, 8, 9]);

    // 降序
    let mut v = vec![5, 2, 8, 1];
    v.sort_by(|a, b| b.cmp(a));
    assert_eq!(v, vec![8, 5, 2, 1]);

    // 按 key 排序
    let mut v = vec!["banana", "apple", "cherry"];
    v.sort_by_key(|s| s.len());
    assert_eq!(v, vec!["apple", "banana", "cherry"]);

    // 去重
    let mut v = vec![1, 2, 2, 3, 3, 3, 4];
    v.dedup();
    assert_eq!(v, vec![1, 2, 3, 4]);

    // 未排序时 dedup 只去相邻
    let mut v = vec![1, 2, 1, 2];
    v.dedup();
    assert_eq!(v, vec![1, 2, 1, 2]); // 不变, 没有相邻重复

    // sort_unstable: 不稳定排序, 更快 (基本类型推荐)
    let mut v = vec![5, 3, 1, 4, 2, 6];
    v.sort_unstable();
    assert_eq!(v, vec![1, 2, 3, 4, 5, 6]);

    // sort_unstable_by: 不稳定+自定义比较
    let mut v = vec![5, 3, 1, 4, 2];
    v.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(v, vec![5, 4, 3, 2, 1]);

    // sort_unstable_by_key: 不稳定+按key
    let mut v = vec!["hello", "world", "rust", "foo"];
    v.sort_unstable_by_key(|s| s.len());
    assert_eq!(v, vec!["foo", "rust", "hello", "world"]);

    // sort_by_cached_key: 缓存 key 值 (key 计算开销大时适用)
    let mut v = vec!["ee", "bbbb", "aaa", "c"];
    // 每个元素的 len() 只计算一次并缓存
    v.sort_by_cached_key(|s| s.len());
    // 长度排序: "c"(1), "ee"(2), "aaa"(3), "bbbb"(4)
    // 稳定排序, 等长保持原序
    assert_eq!(v[0].len(), 1);
    assert_eq!(v[3].len(), 4);

    // 边界用例: 空 Vec 排序
    let mut v: Vec<i32> = vec![];
    v.sort();
    assert!(v.is_empty());

    // 边界用例: 单元素 Vec 排序
    let mut v = vec![42];
    v.sort();
    assert_eq!(v, vec![42]);

    // 边界用例: 已排序 Vec 排序 (检查不会 panic)
    let mut v = vec![1, 2, 3, 4, 5];
    v.sort();
    assert_eq!(v, vec![1, 2, 3, 4, 5]);

    // 边界用例: dedup_by 自定义去重
    let mut v = vec![1, 2, 3, 4, 5];
    v.dedup_by(|a, b| (*a as i32 - *b as i32).unsigned_abs() <= 1); // 相邻差值 <= 1 视为重复
    assert!(v.len() <= 5); // 至少不会增长
}

#[test]
/// 测试: Vec 类型转换 (数组转Vec/切片转Vec/into_boxed_slice/as_slice/as_mut_slice/as_ptr)
fn test_vec_conversion() {
    // 语法: Vec 与其他类型的转换
    //
    // 转换方法:
    //   - Vec::from(arr)       数组 → Vec
    //   - arr.to_vec()         切片 → Vec (复制)
    //   - v.into_boxed_slice() Vec → Box<[T]> (释放多余容量)
    //   - v.leak() -> &'static mut [T]  Vec → 静态切片(永不释放)
    //   - v.as_slice() -> &[T]         Vec → 切片引用
    //   - v.as_mut_slice() -> &mut [T] Vec → 可变切片引用
    //   - v.as_ptr() -> *const T       Vec → 裸指针
    //   - v.as_mut_ptr() -> *mut T     Vec → 可变裸指针
    //   - v.into_iter()                Vec → 消费迭代器
    //   - String::from_utf8(v)         Vec<u8> → String
    //   - v.into_raw_parts() -> (*mut T, usize, usize)  解构为 (ptr, len, cap)
    //   - Vec::from_raw_parts(ptr, len, cap)  从原始指针重建 (unsafe)
    //
    // 避坑:
    //   - leak() 永久内存泄漏, 仅在需要 'static 生命周期时使用
    //   - from_raw_parts 必须确保 ptr/len/cap 合法, 否则 UB
    //   - into_boxed_slice 释放多余容量但数据仍在堆上
    //   - to_vec() 是深拷贝, 不是移动
    //
    // 数组 → Vec
    let arr = [1, 2, 3];
    let v = Vec::from(arr);
    assert_eq!(v, vec![1, 2, 3]);

    // 切片 → Vec
    let slice: &[i32] = &[1, 2, 3];
    let v = slice.to_vec();
    assert_eq!(v, vec![1, 2, 3]);

    // Vec → Box<[T]>
    let mut v = vec![1, 2, 3, 4, 5];
    v.reserve(100); // 浪费容量
    let boxed: Box<[i32]> = v.into_boxed_slice();
    assert_eq!(boxed.len(), 5);
    // 容量已释放, 只保留 5 个元素的空间

    // Vec → 切片引用
    let v = vec![1, 2, 3];
    let slice: &[i32] = v.as_slice();
    assert_eq!(slice, &[1, 2, 3]);

    // 边界用例: as_mut_slice 可变借用
    let mut v = vec![1, 2, 3, 4, 5];
    let s: &mut [i32] = v.as_mut_slice();
    s[0] = 10;
    s[4] = 50;
    assert_eq!(v, vec![10, 2, 3, 4, 50]);

    // 边界用例: as_ptr 获取裸指针
    let v = vec![1, 2, 3];
    let ptr = v.as_ptr();
    unsafe {
        assert_eq!(*ptr, 1);
        assert_eq!(*ptr.add(1), 2);
        assert_eq!(*ptr.add(2), 3);
    }

    // 边界用例: into_raw_parts 解构
    let mut v = vec![1, 2, 3];
    v.reserve(10);
    let cap = v.capacity();
    let (ptr, len, cap2) = v.into_raw_parts();
    assert_eq!(len, 3);
    assert_eq!(cap2, cap);
    // 手动释放内存 (必须用 from_raw_parts)
    let _v = unsafe { Vec::from_raw_parts(ptr, 0, cap2) }; // len=0, 不 drop 元素
}

#[test]
/// 测试: Vec split_off 分割方法
fn test_vec_split_off() {
    // 语法: split_off(at) 从 at 处分割, 返回右半部分, 原 Vec 保留左半部分
    // 避坑: at > len 会 panic; 分割后两个 Vec 各自独立分配内存; at == len 返回空 Vec
    let mut v = vec![1, 2, 3, 4, 5];
    let second_half = v.split_off(3);
    assert_eq!(v, vec![1, 2, 3]);
    assert_eq!(second_half, vec![4, 5]);

    // 边界情况
    let mut v = vec![1, 2, 3];
    let empty = v.split_off(3);
    assert!(empty.is_empty());
    assert_eq!(v, vec![1, 2, 3]);

    // 边界用例: split_off(0) — 全部移到新 Vec
    let mut v = vec![1, 2, 3];
    let all = v.split_off(0);
    assert!(v.is_empty());
    assert_eq!(all, vec![1, 2, 3]);
}

#[test]
/// 测试: 切片 split_at 分割方法 (不可变/可变版本)
fn test_slice_split_at() {
    // 语法: split_at(mid) 返回两个不可变切片引用 (&[T], &[T])
    // 避坑: 不复制数据, 只返回引用; mid > len 会 panic; 可变版本 split_at_mut
    let v = vec![1, 2, 3, 4, 5];
    let (first, rest) = v.split_at(2);
    assert_eq!(first, &[1, 2]);
    assert_eq!(rest, &[3, 4, 5]);

    // 可变版本
    let mut v = vec![1, 2, 3, 4, 5];
    let (left, right) = v.split_at_mut(3);
    left[0] = 10;
    right[0] = 40;
    assert_eq!(v, vec![10, 2, 3, 40, 5]);

    // 边界用例: split_at(0) — 前半为空
    let v = vec![1, 2, 3];
    let (empty, rest) = v.split_at(0);
    assert!(empty.is_empty());
    assert_eq!(rest, &[1, 2, 3]);

    // 边界用例: split_at(len) — 后半为空
    let v = vec![1, 2, 3];
    let (all, empty) = v.split_at(3);
    assert_eq!(all, &[1, 2, 3]);
    assert!(empty.is_empty());
}

#[test]
/// 测试: 切片 as_chunks 固定大小分块方法
fn test_as_chunks() {
    // 语法: as_chunks::<N>() 将切片分为 N 大小的块 + 余数, 返回 (&[[T;N]], &[T])
    // 避坑: N=0 会 panic; 返回的是引用不是 owned 数据; 可变版本 as_chunks_mut
    let v = vec![1, 2, 3, 4, 5, 6, 7];
    let (chunks, remainder) = v.as_chunks::<3>();
    assert_eq!(chunks, &[[1, 2, 3], [4, 5, 6]]);
    assert_eq!(remainder, &[7]);

    // 边界用例: 刚好整除无余数
    let v = vec![1, 2, 3, 4, 5, 6];
    let (chunks, remainder) = v.as_chunks::<2>();
    assert_eq!(chunks, &[[1, 2], [3, 4], [5, 6]]);
    assert!(remainder.is_empty());
}

#[test]
/// 测试: 切片 as_rchunks 从右侧分块方法
fn test_as_rchunks() {
    // 语法: as_rchunks::<N>() 从右侧开始分块, 余数在左边
    // 避坑: 返回值顺序是 (remainder, chunks), 与 as_chunks 相反!
    let v = vec![1, 2, 3, 4, 5];
    let (remainder, chunks) = v.as_rchunks::<2>();
    assert_eq!(remainder, &[1]);
    assert_eq!(chunks, &[[2, 3], [4, 5]]);

    // 边界用例: 刚好整除无余数
    let v = vec![1, 2, 3, 4];
    let (remainder, chunks) = v.as_rchunks::<2>();
    assert!(remainder.is_empty());
    assert_eq!(chunks, &[[1, 2], [3, 4]]);
}

// ===========================================================================
// 新增测试函数
// ===========================================================================

#[test]
/// 测试: const 泛型数组 — 数组长度作为编译期类型参数
fn test_array_const_generics() {
    // 语法: const 泛型允许 [T; N] 中的 N 作为泛型参数参与类型计算
    //
    // 核心概念:
    //   - N 是编译期常量, 可以在泛型约束中使用
    //   - 函数可为任意长度的数组提供通用实现
    //   - 编译器可基于 N 的值进行分支消除和优化
    //
    // 避坑:
    //   - N 必须是编译期已知常量, 不能是运行时变量
    //   - 不同 N 会生成不同的单态化版本 (代码膨胀)
    //   - [T; 0] 零长数组是合法的特殊类型
    //

    // 泛型函数: 获取任意长度数组的第一个元素
    fn first<T: Copy, const N: usize>(arr: [T; N]) -> Option<T> {
        if N > 0 {
            Some(arr[0])
        } else {
            None
        }
    }

    assert_eq!(first([1, 2, 3]), Some(1));
    assert_eq!(first([5, 6, 7, 8]), Some(5));
    assert_eq!(first([]), None::<i32>);

    // 泛型函数: 数组求和
    fn sum<const N: usize>(arr: [i32; N]) -> i32 {
        arr.iter().sum()
    }
    assert_eq!(sum([1, 2, 3, 4]), 10);
    assert_eq!(sum([10, 20]), 30);
    assert_eq!(sum([]), 0);

    // trait 实现: 为任意长度数组提供功能
    trait ArraySize {
        fn size_bytes(&self) -> usize;
    }
    impl<T, const N: usize> ArraySize for [T; N] {
        fn size_bytes(&self) -> usize {
            N * std::mem::size_of::<T>()
        }
    }
    assert_eq!([1i32, 2, 3].size_bytes(), 12); // 3 * 4
    assert_eq!([0u8; 100].size_bytes(), 100); // 100 * 1

    // 编译期条件消除: 零长数组的 is_empty 在编译期可知
    let empty: [i32; 0] = [];
    assert!(empty.is_empty());

    // 使用 const 泛型联合默认值
    fn zero_array<T: Default + Copy, const N: usize>() -> [T; N] {
        [T::default(); N]
    }
    let zeros: [i32; 5] = zero_array();
    assert_eq!(zeros, [0, 0, 0, 0, 0]);
}

#[test]
/// 测试: 切片胖指针大小 — &[T] 在 64 位系统上占 16 字节 (数据指针8 + 长度8)
fn test_slice_fat_pointer() {
    // 语法: &[T] 是胖指针 (fat pointer), 包含两个 usize: 数据指针 + 长度
    //
    // 关键差异:
    //   - 普通引用 &T:         8 字节 (只是一个指针)
    //   - 切片引用 &[T]:      16 字节 (指针 + 长度)
    //   - 数组引用 &[T; N]:    8 字节 (长度在类型中, 不需要运行时存储)
    //   - trait object &dyn T: 16 字节 (数据指针 + vtable 指针)
    //
    // 避坑:
    //   - FFI 中 &[T] 不能直接传为指针, 需用 as_ptr() 获取数据指针
    //   - &[T] 和 &[T; N] 是不同的类型, 大小不同
    //

    use std::mem::size_of;

    // 普通引用: 8 字节 (64位系统)
    let n: i32 = 42;
    let ref_n: &i32 = &n;
    assert_eq!(ref_n as *const i32 as usize, &n as *const i32 as usize);

    // 切片引用 (胖指针): 16 字节
    assert_eq!(size_of::<&[i32]>(), 16);
    assert_eq!(size_of::<&[u8]>(), 16);
    assert_eq!(size_of::<&str>(), 16); // &str 也是胖指针

    // 数组引用: 8 字节 (长度在类型中)
    assert_eq!(size_of::<&[i32; 3]>(), 8);
    assert_eq!(size_of::<&[i32; 100]>(), 8);

    // trait object: 也是胖指针 (16字节)
    assert_eq!(size_of::<&dyn std::fmt::Display>(), 16);

    // 验证切片胖指针的正确性
    let arr = [1, 2, 3, 4, 5];
    let slice: &[i32] = &arr;
    assert_eq!(slice.len(), 5);
    // 数据指针指向 arr 的首元素
    assert_eq!(slice.as_ptr(), arr.as_ptr());

    // 切片可以缩小范围, 但仍然是胖指针 (指向子范围)
    let sub_slice: &[i32] = &arr[1..4];
    assert_eq!(sub_slice.len(), 3);
    assert_eq!(sub_slice[0], 2);
    // 长度变短但仍然是 16 字节
    assert_eq!(size_of::<&[i32]>(), 16);
}

#[test]
/// 测试: Vec 容量增长行为 — 翻倍策略和摊销 O(1)
fn test_vec_memory_layout() {
    // 语法: Vec 内部由 (ptr, len, cap) 三个字段组成, 大小 24 字节
    //
    // 增长策略:
    //   - Vec::new() → cap=0
    //   - 第一次 push: cap 变为 4 (初始最小分配)
    //   - cap < 1024: 容量翻倍 ×2
    //   - cap >= 1024: 容量增长 ×1.5
    //
    // 避坑:
    //   - 扩容时所有元素被拷贝到新内存, 大型 Vec 扩容代价高
    //   - 扩容后旧指针失效, 之前获取的引用必须已释放
    //

    // 初始状态
    let v: Vec<i32> = Vec::new();
    assert_eq!(v.capacity(), 0);
    assert_eq!(v.len(), 0);

    // 第一次 push 触发初始分配
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    assert!(v.capacity() >= 1);
    assert_eq!(v.len(), 1);

    // 验证翻倍增长 (从初始容量开始)
    let mut v = Vec::new();
    let mut caps = Vec::new();
    let mut prev_cap = v.capacity();
    caps.push(prev_cap);
    for _ in 0..1000 {
        if v.len() == v.capacity() {
            v.push(0);
            let new_cap = v.capacity();
            if prev_cap > 0 && prev_cap < 1024 && prev_cap != new_cap {
                // 容量 < 1024 时应当翻倍
                assert_eq!(new_cap, prev_cap * 2);
            }
            prev_cap = new_cap;
            caps.push(prev_cap);
        } else {
            v.push(0);
        }
    }
    assert_eq!(v.len(), 1000);
    // 验证至少有多次扩容
    assert!(caps.len() > 1, "应该发生了扩容");

    // with_capacity 精确预分配
    let mut v = Vec::with_capacity(10);
    assert_eq!(v.capacity(), 10);
    assert_eq!(v.len(), 0);
    for i in 0..10 {
        v.push(i);
        assert_eq!(v.capacity(), 10); // 未触发扩容
    }
    // 第 11 个元素触发扩容
    v.push(10);
    assert!(v.capacity() > 10);

    // into_raw_parts: 验证 Vec 三字段
    let v = vec![1, 2, 3, 4, 5];
    let (ptr, len, cap) = v.into_raw_parts();
    assert_eq!(len, 5);
    assert!(cap >= 5);
    // 重建并清理 (必须手动管理)
    let mut v = unsafe { Vec::from_raw_parts(ptr, len, cap) };
    assert_eq!(v, vec![1, 2, 3, 4, 5]);
    v.clear(); // 清理元素但不释放
    assert!(v.is_empty());
    // v 在此 drop 时安全释放内存
}

#[test]
/// 测试: swap_remove vs remove 性能差异和语义对比
fn test_vec_remove_methods() {
    // 语法: remove O(n) 保序, swap_remove O(1) 弃序
    //
    // remove:
    //   - 移除指定索引, 后续元素前移, O(n)
    //   - 保持剩余元素的相对顺序
    //
    // swap_remove:
    //   - 用最后一个元素替换被移除位置, O(1)
    //   - 不保持顺序! 适合顺序无关场景 (如集合、栈)
    //
    // 避坑:
    //   - 频繁调用 remove(i) 会累积 O(n²)
    //   - swap_remove 后不要依赖索引
    //   - 索引越界两者都会 panic
    //

    // remove: 保序但 O(n)
    let mut v = vec![10, 20, 30, 40, 50];
    let removed = v.remove(2);
    assert_eq!(removed, 30);
    assert_eq!(v, vec![10, 20, 40, 50]); // 顺序保持

    // swap_remove: 弃序但 O(1)
    let mut v = vec![10, 20, 30, 40, 50];
    let removed = v.swap_remove(2);
    assert_eq!(removed, 30);
    assert_eq!(v, vec![10, 20, 50, 40]); // 顺序改变! 50 移到索引 2

    // 对比: swap_remove 删除首元素
    let mut v1 = vec![1, 2, 3, 4, 5];
    assert_eq!(v1.remove(0), 1);
    assert_eq!(v1, vec![2, 3, 4, 5]);

    let mut v2 = vec![1, 2, 3, 4, 5];
    assert_eq!(v2.swap_remove(0), 1);
    assert_eq!(v2, vec![5, 2, 3, 4]); // 5 移到索引 0

    // 边界用例: 删除最后元素 (两者行为一致, 都是 O(1))
    let mut v1 = vec![1, 2, 3];
    v1.remove(2);
    assert_eq!(v1, vec![1, 2]);

    let mut v2 = vec![1, 2, 3];
    v2.swap_remove(2);
    assert_eq!(v2, vec![1, 2]);

    // 边界用例: 删除倒数第二个
    let mut v = vec![1, 2, 3, 4];
    assert_eq!(v.swap_remove(2), 3); // 3 被移除, 4 替换到位置 2
    assert_eq!(v, vec![1, 2, 4]);

    // 边界用例: 单元素 Vec
    let mut v = vec![42];
    assert_eq!(v.remove(0), 42);
    assert!(v.is_empty());

    let mut v = vec![42];
    assert_eq!(v.swap_remove(0), 42);
    assert!(v.is_empty());
}

#[test]
/// 测试: retain / drain / extract_if 过滤和移除方法对比
fn test_vec_retain_drain() {
    // 语法: 三种不同的元素移除策略
    //
    // retain:     原地保留满足条件的元素, 丢弃不满足的
    // drain:      按范围移除元素, 返回迭代器可收集
    // extract_if: 按条件移除元素, 返回被移除元素的迭代器 (Rust 1.86+)
    //
    // 避坑:
    //   - retain 的闭包不能修改其他元素
    //   - drain 迭代器必须被消费
    //   - extract_if 移除后原 Vec 保留不满足条件的元素
    //

    // retain: 保留偶数
    let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8];
    v.retain(|&x| x % 2 == 0);
    assert_eq!(v, vec![2, 4, 6, 8]);

    // retain_mut: 可变版本, 可以在闭包内修改元素
    let mut v = vec![1, 2, 3, 4, 5];
    v.retain_mut(|x| {
        if *x < 3 {
            *x *= 10; // 修改保留的元素
            true
        } else {
            false
        }
    });
    assert_eq!(v, vec![10, 20]);

    // drain: 范围移除
    let mut v = vec![1, 2, 3, 4, 5, 6];
    let drained: Vec<i32> = v.drain(2..5).collect(); // 移除索引 2,3,4
    assert_eq!(drained, vec![3, 4, 5]);
    assert_eq!(v, vec![1, 2, 6]);

    // drain 范围语法: ..n, n.., ..
    let mut v = vec![1, 2, 3, 4, 5];
    let _: Vec<_> = v.drain(..2).collect(); // 移除前 2 个
    assert_eq!(v, vec![3, 4, 5]);

    let _: Vec<_> = v.drain(1..).collect(); // 从索引 1 开始移除
    assert_eq!(v, vec![3]);

    let _: Vec<_> = v.drain(..).collect(); // 全部移除
    assert!(v.is_empty());

    // extract_if: 条件移除并收集被移除的元素 (Rust 1.86+, 需两个参数: 范围+过滤器)
    let mut v = vec![1, 2, 3, 4, 5, 6];
    let removed: Vec<i32> = v.extract_if(.., |x: &mut i32| *x % 2 == 0).collect();
    assert_eq!(removed, vec![2, 4, 6]);
    assert_eq!(v, vec![1, 3, 5]);

    // 边界用例: retain 全部保留/全部移除
    let mut v = vec![1, 2, 3];
    v.retain(|_| true);
    assert_eq!(v, vec![1, 2, 3]);

    v.retain(|_| false);
    assert!(v.is_empty());

    // 边界用例: drain 空范围
    let mut v = vec![1, 2, 3];
    let drained: Vec<i32> = v.drain(1..1).collect();
    assert!(drained.is_empty());
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
/// 测试: VecDeque 环形缓冲区基本用法
fn test_vecdeque() {
    // 语法: VecDeque<T> 双端队列, 内部用环形缓冲区实现
    //
    // 核心操作 (全 O(1)):
    //   - push_front / push_back    两端插入
    //   - pop_front / pop_back      两端删除
    //   - front / back              查看两端元素
    //
    // 特点:
    //   - 支持索引访问 O(1): deque[i]
    //   - 两端操作均摊 O(1)
    //   - as_slices 返回 (front_slice, back_slice) 可能不连续的两个切片
    //
    // 适用场景:
    //   - 队列/双端队列
    //   - 滑动窗口
    //   - 需要在两端高效增删的场景
    //
    // 避坑:
    //   - 内部环形缓冲区可能产生不连续的两个切片
    //   - make_contiguous 会重新排列元素使内存连续
    //   - 索引访问虽然是 O(1), 但有取模运算的额外开销
    //

    use std::collections::VecDeque;

    // 创建和基本操作
    let mut deque = VecDeque::new();
    assert!(deque.is_empty());

    deque.push_back(1);
    deque.push_back(2);
    deque.push_front(0);
    assert_eq!(deque, vec![0, 1, 2]);

    // 两端查看
    assert_eq!(deque.front(), Some(&0));
    assert_eq!(deque.back(), Some(&2));

    // 两端删除
    assert_eq!(deque.pop_front(), Some(0));
    assert_eq!(deque.pop_back(), Some(2));
    assert_eq!(deque, vec![1]);

    // 索引访问 O(1)
    let mut deque = VecDeque::from(vec![10, 20, 30, 40, 50]);
    assert_eq!(deque[0], 10);
    assert_eq!(deque[2], 30);
    assert_eq!(deque[4], 50);

    // 可变索引
    deque[1] = 25;
    assert_eq!(deque[1], 25);

    // as_slices: 获取环形缓冲区的两个切片
    let mut deque = VecDeque::from(vec![1, 2, 3, 4, 5]);
    // 弹出后再压入, 可能导致数据不连续
    deque.pop_front();
    deque.pop_front();
    deque.push_back(6);
    deque.push_back(7);
    // 此时逻辑顺序是 [3, 4, 5, 6, 7], 但物理可能不连续
    let (front, back) = deque.as_slices();
    // front 和 back 合在一起构成完整序列
    let combined: Vec<&i32> = front.iter().chain(back.iter()).collect();
    assert_eq!(combined, vec![&3, &4, &5, &6, &7]);

    // make_contiguous: 确保内存连续
    deque.make_contiguous();
    let (front, back) = deque.as_slices();
    assert_eq!(front, &[3, 4, 5, 6, 7]);
    assert!(back.is_empty());

    // with_capacity 预分配
    let deque: VecDeque<i32> = VecDeque::with_capacity(100);
    assert!(deque.capacity() >= 100);
    assert!(deque.is_empty());

    // 滑动窗口场景
    let mut window = VecDeque::with_capacity(3);
    for i in 0..5 {
        window.push_back(i);
        if window.len() > 3 {
            window.pop_front();
        }
    }
    assert_eq!(window, vec![2, 3, 4]);

    // 边界用例: 从空 deque 操作
    let mut deque: VecDeque<i32> = VecDeque::new();
    assert_eq!(deque.pop_front(), None);
    assert_eq!(deque.pop_back(), None);
    assert_eq!(deque.front(), None);
    assert_eq!(deque.back(), None);

    // 边界用例: truncate
    let mut deque = VecDeque::from(vec![1, 2, 3, 4, 5]);
    deque.truncate(3);
    assert_eq!(deque, vec![1, 2, 3]);
}

#[test]
/// 测试: 切片模式匹配 — slice patterns [first, rest @ .., last]
fn test_slice_patterns() {
    // 语法: Rust 支持在 match 中对切片进行结构化模式匹配
    //
    // 常用模式:
    //   - []                    空切片
    //   - [x]                   单元素
    //   - [first, rest @ ..]    首元素 + 剩余
    //   - [first, rest @ .., last]  首+剩余+尾
    //   - [a, b, ..]            至少两个元素
    //   - [a, .., z]            至少两个元素, 绑定首尾
    //   - [1, 2, ..]           以特定值开头
    //   - [.., 5, 6]           以特定值结尾
    //

    // 基础: 按长度匹配
    fn describe_slice(s: &[i32]) -> String {
        match s {
            [] => "空切片".to_string(),
            [x] => format!("单元素: {}", x),
            [first, rest @ ..] => format!("首={}, 剩余{}个", first, rest.len()),
        }
    }

    assert_eq!(describe_slice(&[]), "空切片");
    assert_eq!(describe_slice(&[42]), "单元素: 42");
    assert_eq!(describe_slice(&[1, 2, 3, 4]), "首=1, 剩余3个");

    // 首尾模式: [first, .., last]
    fn first_and_last(s: &[i32]) -> Option<(i32, i32)> {
        match s {
            [first, .., last] => Some((*first, *last)),
            [single] => Some((*single, *single)), // 单元素, 首尾相同
            [] => None,
        }
    }

    assert_eq!(first_and_last(&[1, 2, 3, 4, 5]), Some((1, 5)));
    assert_eq!(first_and_last(&[42]), Some((42, 42)));
    assert_eq!(first_and_last(&[]), None);

    // 前缀/后缀模式
    fn check_prefix(s: &[i32]) -> &str {
        match s {
            [1, 2, ..] => "以 1,2 开头",
            [1, ..] => "以 1 开头",
            _ => "其他",
        }
    }

    assert_eq!(check_prefix(&[1, 2, 3, 4]), "以 1,2 开头");
    assert_eq!(check_prefix(&[1, 3, 4]), "以 1 开头");
    assert_eq!(check_prefix(&[2, 3, 4]), "其他");

    fn check_suffix(s: &[i32]) -> &str {
        match s {
            [.., 99] => "以 99 结尾",
            [.., a, b] if a == b => "最后两个相同",
            _ => "其他",
        }
    }

    assert_eq!(check_suffix(&[1, 2, 99]), "以 99 结尾");
    assert_eq!(check_suffix(&[1, 5, 5]), "最后两个相同");
    assert_eq!(check_suffix(&[1, 2, 3]), "其他");

    // 绑定中间部分
    let arr = [10, 20, 30, 40, 50, 60, 70, 80];
    match arr.as_slice() {
        [first, middle @ .., last] => {
            assert_eq!(first, &10);
            assert_eq!(last, &80);
            assert_eq!(middle, &[20, 30, 40, 50, 60, 70]);
        }
        _ => unreachable!(),
    }

    // 带守卫的模式
    fn sum_if_starts_with_one(s: &[i32]) -> i32 {
        match s {
            [1, rest @ ..] if rest.len() >= 2 => rest.iter().sum(),
            _ => 0,
        }
    }
    assert_eq!(sum_if_starts_with_one(&[1, 2, 3, 4]), 9); // 2+3+4
    assert_eq!(sum_if_starts_with_one(&[1, 2]), 0); // rest只有1个元素, 不满足>=2
    assert_eq!(sum_if_starts_with_one(&[1]), 0); // rest < 2
    assert_eq!(sum_if_starts_with_one(&[2, 3, 4]), 0); // 不是 1 开头

    // 边界用例: 用 let 解构
    let arr = [1, 2, 3, 4, 5];
    let [first, .., last] = &arr;
    assert_eq!(first, &1);
    assert_eq!(last, &5);

    // 边界用例: 在 for 循环中使用切片模式
    let nested: [&[i32]; 3] = [&[1, 2], &[3, 4, 5], &[6]];
    for slice in nested {
        match slice {
            [a, _b] => assert_eq!(*a, slice[0]),
            [x] => assert_eq!(*x, slice[0]),
            _ => {}
        }
    }
}

#[test]
/// 测试: 数组/切片高级迭代方法 (chunks/windows/iter_mut/step_by/zip)
fn test_array_iteration_advanced() {
    // 语法: 切片提供了多种零拷贝的迭代视图
    //
    // iter / iter_mut:
    //   - iter()      不可变引用迭代器
    //   - iter_mut()  可变引用迭代器
    //   - into_iter() 消费迭代器 (owned)
    //
    // chunks / windows:
    //   - chunks(n)         每块最多 n 个元素, 不重叠
    //   - chunks_exact(n)   每块严格 n 个, 余数单独处理
    //   - windows(n)        滑动窗口, 相邻重叠 n-1 个元素
    //   - chunks_mut()      可变版本
    //
    // 其它:
    //   - rchunks(n)        从右侧分块
    //   - step_by(n)        步进迭代
    //   - enumerate()       带索引迭代
    //

    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

    // chunks: 不重叠分块
    let chunk_sums: Vec<i32> = arr.chunks(4).map(|chunk| chunk.iter().sum()).collect();
    assert_eq!(chunk_sums, vec![10, 26, 42]); // [1..4]=10, [5..8]=26, [9..12]=42

    // chunks_exact: 严格分块 + 余数
    let mut chunks = arr.chunks_exact(4);
    assert_eq!(chunks.next(), Some(&[1, 2, 3, 4][..]));
    assert_eq!(chunks.next(), Some(&[5, 6, 7, 8][..]));
    assert_eq!(chunks.next(), Some(&[9, 10, 11, 12][..]));
    assert_eq!(chunks.next(), None);

    // windows: 滑动窗口
    let window_sums: Vec<i32> = arr.windows(3).map(|w| w.iter().sum()).collect();
    assert_eq!(window_sums[0], 6);   // 1+2+3
    assert_eq!(window_sums[1], 9);   // 2+3+4
    assert_eq!(window_sums[9], 33);  // 10+11+12
    assert_eq!(window_sums.len(), 10); // 12 - 3 + 1 = 10

    // iter_mut: 可变迭代 (原地修改)
    let mut arr = [1, 2, 3, 4, 5];
    arr.iter_mut().for_each(|x| *x *= 2);
    assert_eq!(arr, [2, 4, 6, 8, 10]);

    // step_by: 步进
    let step: Vec<i32> = arr.iter().step_by(2).copied().collect();
    assert_eq!(step, vec![2, 6, 10]);

    // enumerate: 带索引
    let indexed: Vec<(usize, i32)> = arr.iter().enumerate().map(|(i, &x)| (i, x)).collect();
    assert_eq!(indexed[0], (0, 2));
    assert_eq!(indexed[2], (2, 6));

    // zip: 两个切片并行迭代
    let a = [1, 2, 3, 4];
    let b = [10, 20, 30, 40];
    let summed: Vec<i32> = a.iter().zip(b.iter()).map(|(x, y)| x + y).collect();
    assert_eq!(summed, vec![11, 22, 33, 44]);

    // rchunks: 从右侧分块
    let arr = [1, 2, 3, 4, 5, 6, 7];
    let rchunks: Vec<Vec<i32>> = arr.rchunks(3).map(|c| c.to_vec()).collect();
    // 从右: [5,6,7], [2,3,4], [1]
    assert_eq!(rchunks[0], vec![5, 6, 7]);
    assert_eq!(rchunks[1], vec![2, 3, 4]);
    assert_eq!(rchunks[2], vec![1]);

    // 边界用例: windows 大小 > len
    let arr = [1, 2, 3];
    let windows: Vec<_> = arr.windows(5).collect();
    assert!(windows.is_empty());

    // 边界用例: chunks 正好整除
    let arr = [1, 2, 3, 4, 5, 6];
    let chunks: Vec<_> = arr.chunks(2).collect();
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], &[1, 2]);
    assert_eq!(chunks[2], &[5, 6]);

    // 边界用例: as_chunks 返回固定大小数组引用 (const 泛型, Rust 1.79+)
    let arr = [1i32, 2, 3, 4, 5, 6];
    let (chunks, remainder) = arr.as_slice().as_chunks::<3>();
    assert_eq!(chunks, &[[1, 2, 3], [4, 5, 6]]);
    assert!(remainder.is_empty());
}

#[test]
/// 测试: 二维数据结构 — 数组的数组 vs Vec<Vec<T>> vs 扁平化 Vec
fn test_two_dimensional_arrays() {
    // 语法: Rust 中有三种主要方式表示二维数据
    //
    // 1. [[T; COLS]; ROWS]  —  栈上, 行和列都是编译期常量
    // 2. Vec<Vec<T>>         —  堆上, 每行独立分配
    // 3. Vec<T> 扁平化       —  堆上一次分配, 手动计算索引 row*cols+col
    //
    // 内存布局:
    //   数组的数组: 一行接一行, 缓存最友好
    //   Vec<Vec<T>>: 不连续, 每行可能散落堆各处, 缓存最差
    //   扁平 Vec: 连续, 仅次于栈数组
    //

    // 方案1: 数组的数组 — 固定大小, 栈上
    const ROWS: usize = 3;
    const COLS: usize = 4;
    let matrix: [[i32; COLS]; ROWS] = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
    ];
    assert_eq!(matrix[0][0], 1);
    assert_eq!(matrix[1][2], 7);
    assert_eq!(matrix[2][3], 12);
    assert_eq!(matrix.len(), 3); // 行数
    assert_eq!(matrix[0].len(), 4); // 列数

    // 遍历
    let sum: i32 = matrix.iter().flatten().sum();
    assert_eq!(sum, 78); // 1+2+...+12 = 78

    // 方案2: Vec<Vec<T>> — 动态大小, 每行独立
    let mut matrix: Vec<Vec<i32>> = vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]];
    // 不规则的"锯齿"结构
    assert_eq!(matrix[0].len(), 3);
    assert_eq!(matrix[1].len(), 2);
    assert_eq!(matrix[2].len(), 4);
    // 可以单独增长某行
    matrix[1].push(99);
    assert_eq!(matrix[1], vec![4, 5, 99]);

    // 方案3: 扁平化 Vec — 手动索引, 最高效
    let rows = 3;
    let cols = 4;
    let mut matrix: Vec<i32> = vec![0; rows * cols];
    // 填充: matrix[row][col] → matrix[row * cols + col]
    for r in 0..rows {
        for c in 0..cols {
            matrix[r * cols + c] = (r * cols + c + 1) as i32;
        }
    }
    assert_eq!(matrix[0 * cols + 0], 1);  // [0][0]
    assert_eq!(matrix[1 * cols + 2], 7);  // [1][2]
    assert_eq!(matrix[2 * cols + 3], 12); // [2][3]

    // 边界用例: 空二维数组
    let empty: [[i32; 0]; 0] = [];
    assert_eq!(empty.len(), 0);

    // 边界用例: 转置 (仅在方阵或矩形中可行)
    let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    let mut transposed = [[0i32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            transposed[j][i] = matrix[i][j];
        }
    }
    assert_eq!(transposed[0], [1, 4, 7]);
    assert_eq!(transposed[1], [2, 5, 8]);
    assert_eq!(transposed[2], [3, 6, 9]);

    // 边界用例: 创建全零矩阵
    let zeros = [[0i32; 5]; 3];
    assert_eq!(zeros[0], [0, 0, 0, 0, 0]);
    assert_eq!(zeros.len(), 3);
}
