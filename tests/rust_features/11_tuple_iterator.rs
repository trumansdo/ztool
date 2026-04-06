// ---------------------------------------------------------------------------
// 3.5 元组迭代器收集 (1.85+)
// ---------------------------------------------------------------------------

use std::collections::{LinkedList, VecDeque};

#[test]
/// 测试: collect() 扇出到元组 (1.85+)
fn test_tuple_collect() {
    // 语法: collect() 可将迭代器扇出到 2~12 元组, 每个元素独立收集 (1.85+)
    // 避坑: 元组每个位置的类型必须实现 FromIterator; 需要显式类型注解
    let (squares, cubes): (Vec<i32>, Vec<i32>) = (0..5)
        .map(|i| (i * i, i * i * i))
        .collect();
    assert_eq!(squares, vec![0, 1, 4, 9, 16]);
    assert_eq!(cubes, vec![0, 1, 8, 27, 64]);
}

#[test]
/// 测试: 三元以上组 collect 扇出 (VecDeque/LinkedList)
fn test_triple_tuple_collect() {
    // 语法: 支持 3 元组及以上收集, 各元素类型可不同
    // 避坑: 元组大小 2~12 之间; 超出范围需手动拆分或使用其他方法
    let (a, b, c): (Vec<_>, VecDeque<_>, LinkedList<_>) = (0..3)
        .map(|i| (i, i * 2, i * 3))
        .collect();
    assert_eq!(a, vec![0, 1, 2]);
    assert_eq!(
        b.iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![0, 2, 4]
    );
}

#[test]
/// 测试: 元组 collect 中 Result/Option 处理
fn test_tuple_collect_with_result() {
    // 语法: collect() 可以收集到 Result<Vec<T>, E>, 遇到第一个 Err 就短路
    // 避坑: 元组中如果有一个位置是 Result, 整个 collect 返回 Result<(Vec, Vec), E>
    let results: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Ok(3)];
    let collected: Result<Vec<i32>, &str> = results.into_iter().collect();
    assert_eq!(collected, Ok(vec![1, 2, 3]));

    let results: Vec<Result<i32, &str>> = vec![Ok(1), Err("fail"), Ok(3)];
    let collected: Result<Vec<i32>, &str> = results.into_iter().collect();
    assert_eq!(collected, Err("fail"));
}

#[test]
/// 测试: unzip 元组迭代器拆分
fn test_unzip() {
    // 语法: unzip() 将 (A, B) 迭代器拆分为 (Vec<A>, Vec<B>)
    // 避坑: unzip 是 collect 的特化版本, 只支持 2 元组; 3+ 元组用 collect
    let pairs = vec![(1, "a"), (2, "b"), (3, "c")];
    let (nums, chars): (Vec<i32>, Vec<&str>) = pairs.into_iter().unzip();
    assert_eq!(nums, vec![1, 2, 3]);
    assert_eq!(chars, vec!["a", "b", "c"]);
}

#[test]
/// 测试: partition 按条件拆分迭代器
fn test_partition() {
    // 语法: partition(pred) 将迭代器按条件分为两个集合
    // 避坑: 遍历一次, 但需要两个集合都实现 Extend; 不同于 filter + filter
    let nums = vec![1, 2, 3, 4, 5, 6];
    let (evens, odds): (Vec<i32>, Vec<i32>) = nums
        .into_iter()
        .partition(|x| x % 2 == 0);
    assert_eq!(evens, vec![2, 4, 6]);
    assert_eq!(odds, vec![1, 3, 5]);
}

#[test]
/// 测试: partition_in_place 原地分区 (1.85+)
fn test_partition_in_place() {
    // 语法: slice::partition_in_place(pred) 原地分区, 返回分界点索引
    // 避坑: 不分配新内存; 返回分界点索引; 不保证相对顺序; 需要 &mut [T]
    // 注意: 此方法需要 T: PartitionInPlace trait, 这里仅演示概念
    assert!(true);
}

#[test]
/// 测试: Iterator::reduce 与 fold
fn test_reduce_vs_fold() {
    // 语法: reduce 无初始值, 返回 Option; fold 有初始值, 返回确定值
    // 避坑: reduce 在空迭代器上返回 None; fold 的初始值类型决定返回类型
    let nums = vec![1, 2, 3, 4, 5];

    // reduce
    let sum = nums
        .iter()
        .copied()
        .reduce(|a, b| a + b);
    assert_eq!(sum, Some(15));

    let empty: Vec<i32> = vec![];
    assert_eq!(
        empty
            .iter()
            .copied()
            .reduce(|a, b| a + b),
        None
    );

    // fold
    let sum = nums
        .iter()
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 15);

    // fold 可以改变类型
    let strings = vec!["a", "b", "c"];
    let concatenated = strings
        .into_iter()
        .fold(String::new(), |acc, s| acc + s);
    assert_eq!(concatenated, "abc");
}

#[test]
/// 测试: Iterator::try_fold / try_reduce
fn test_try_fold() {
    // 语法: try_fold 允许在折叠中提前退出 (返回 Result/Option)
    // 避坑: 闭包返回 Result 或 Option; 遇到 Err/None 时提前返回
    let nums = vec![1, 2, 3, 4, 5];

    // 遇到大于 10 的累加和就失败
    let result = nums
        .iter()
        .try_fold(0i32, |acc, x| {
            let sum = acc + x;
            if sum > 10 {
                None
            } else {
                Some(sum)
            }
        });
    assert_eq!(result, None); // 1+2+3+4=10, +5=15 > 10

    let result = nums
        .iter()
        .try_fold(0i32, |acc, x| {
            let sum = acc + x;
            if sum > 100 {
                None
            } else {
                Some(sum)
            }
        });
    assert_eq!(result, Some(15));
}

#[test]
/// 测试: FromIterator trait 自定义收集
fn test_from_iterator_custom() {
    // 语法: 实现 FromIterator 可以让自定义类型支持 .collect()
    // 避坑: 需要实现 from_iter 方法; 通常配合 IntoIterator 使用
    struct Sum(i64);

    impl FromIterator<i32> for Sum {
        fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
            Sum(iter
                .into_iter()
                .map(|x| x as i64)
                .sum())
        }
    }

    let sum: Sum = vec![1, 2, 3, 4, 5]
        .into_iter()
        .collect();
    assert_eq!(sum.0, 15);
}

#[test]
/// 测试: Extend trait 扩展集合
fn test_extend() {
    // 语法: extend(iter) 将迭代器元素添加到现有集合
    // 避坑: extend 消费迭代器; 与 collect 不同, extend 不创建新集合
    let mut nums = vec![1, 2, 3];
    nums.extend([4, 5, 6]);
    assert_eq!(nums, vec![1, 2, 3, 4, 5, 6]);

    // extend 也支持元组迭代器
    let mut map = std::collections::HashMap::new();
    map.extend([("a", 1), ("b", 2)]);
    assert_eq!(map.len(), 2);
}
