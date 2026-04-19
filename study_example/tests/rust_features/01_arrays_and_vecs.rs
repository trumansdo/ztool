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
}

#[test]
/// 测试: Vec 容量管理 (with_capacity/reserve/shrink_to_fit)
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
}

#[test]
/// 测试: Vec 排序方法 (sort/sort_by/sort_by_key/dedup)
fn test_vec_sort() {
    // 语法: Vec/切片排序方法
    //
    // 排序方法:
    //   - sort()              原地排序 (要求 T: Ord), 混合排序(快排+归并)
    //   - sort_unstable()     原地不稳定排序 (更快, 不保序)
    //   - sort_by(cmp)        自定义比较函数
    //   - sort_by_key(f)      按 key 排序 (稳定)
    //   - sort_by_cached_key(f) 按 key 排序, 缓存 key 结果 (key 计算慢时适用)
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
}

#[test]
/// 测试: Vec 类型转换 (数组转Vec/切片转Vec/into_boxed_slice/as_slice)
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
}
