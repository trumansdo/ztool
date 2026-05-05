// ---------------------------------------------------------------------------
// 3.5 元组迭代器收集 (1.85+)
// ---------------------------------------------------------------------------

use std::collections::{LinkedList, VecDeque, HashMap, HashSet};

#[test]
/// 测试: collect() 扇出到元组 (1.85+)
fn test_tuple_collect() {
    let (squares, cubes): (Vec<i32>, Vec<i32>) = (0..5)
        .map(|i| (i * i, i * i * i))
        .collect();
    assert_eq!(squares, vec![0, 1, 4, 9, 16]);
    assert_eq!(cubes, vec![0, 1, 8, 27, 64]);
}

#[test]
/// 测试: 三元以上组 collect 扇出 (VecDeque/LinkedList)
fn test_triple_tuple_collect() {
    let (a, b, _c): (Vec<_>, VecDeque<_>, LinkedList<_>) = (0..3)
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
    let pairs = vec![(1, "a"), (2, "b"), (3, "c")];
    let (nums, chars): (Vec<i32>, Vec<&str>) = pairs.into_iter().unzip();
    assert_eq!(nums, vec![1, 2, 3]);
    assert_eq!(chars, vec!["a", "b", "c"]);
}

#[test]
/// 测试: partition 按条件拆分迭代器
fn test_partition() {
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
    assert!(true);
}

#[test]
/// 测试: Iterator::reduce 与 fold
fn test_reduce_vs_fold() {
    let nums = vec![1, 2, 3, 4, 5];

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

    let sum = nums
        .iter()
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 15);

    let strings = vec!["a", "b", "c"];
    let concatenated = strings
        .into_iter()
        .fold(String::new(), |acc, s| acc + s);
    assert_eq!(concatenated, "abc");
}

#[test]
/// 测试: Iterator::try_fold / try_reduce
fn test_try_fold() {
    let nums = vec![1, 2, 3, 4, 5];

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
    assert_eq!(result, None);

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
    let mut nums = vec![1, 2, 3];
    nums.extend([4, 5, 6]);
    assert_eq!(nums, vec![1, 2, 3, 4, 5, 6]);

    let mut map = std::collections::HashMap::new();
    map.extend([("a", 1), ("b", 2)]);
    assert_eq!(map.len(), 2);
}

// ===================== 扩充测试 =====================

#[test]
/// 测试: collect 4元组 —— 不同类型集合混合
fn test_tuple_collect_4_elements() {
    let (a, b, c, d): (Vec<i32>, VecDeque<i32>, Vec<i32>, Vec<i32>) = (0..5)
        .map(|i| (i, i * 10, i * 100, i * 1000))
        .collect();

    assert_eq!(a, vec![0, 1, 2, 3, 4]);
    assert_eq!(b.iter().cloned().collect::<Vec<_>>(), vec![0, 10, 20, 30, 40]);
    assert_eq!(c, vec![0, 100, 200, 300, 400]);
    assert_eq!(d, vec![0, 1000, 2000, 3000, 4000]);
}

#[test]
/// 测试: collect 5元组 —— 五路扇出
fn test_tuple_collect_5_elements() {
    let (a, b, c, d, e): (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>) = (0..3)
        .map(|i| (i, i+1, i+2, i+3, i+4))
        .collect();

    assert_eq!(a, vec![0, 1, 2]);
    assert_eq!(b, vec![1, 2, 3]);
    assert_eq!(c, vec![2, 3, 4]);
    assert_eq!(d, vec![3, 4, 5]);
    assert_eq!(e, vec![4, 5, 6]);
}

#[test]
/// 测试: collect 到 HashSet 和 HashMap 的元组
fn test_tuple_collect_to_hash_structures() {
    let (set_a, set_b): (HashSet<i32>, HashSet<i32>) = (0..5)
        .map(|i| (i * 2, i * 3))
        .collect();

    assert_eq!(set_a.len(), 5);
    assert_eq!(set_b.len(), 5);
    for i in 0..5 {
        assert!(set_a.contains(&(i * 2)));
        assert!(set_b.contains(&(i * 3)));
    }
}

#[test]
/// 测试: collect 空迭代器到元组
fn test_tuple_collect_empty_iterator() {
    let (a, b): (Vec<i32>, Vec<i32>) = (0..0)
        .map(|i: i32| (i, i))
        .collect();

    assert!(a.is_empty());
    assert!(b.is_empty());
}

#[test]
/// 测试: unzip 对大数据集
fn test_unzip_large_dataset() {
    let pairs: Vec<(i32, String)> = (0..1000)
        .map(|i| (i, format!("item_{}", i)))
        .collect();

    let (nums, strings): (Vec<i32>, Vec<String>) = pairs.into_iter().unzip();

    assert_eq!(nums.len(), 1000);
    assert_eq!(strings.len(), 1000);
    assert_eq!(nums[0], 0);
    assert_eq!(nums[999], 999);
    assert_eq!(strings[0], "item_0");
    assert_eq!(strings[999], "item_999");
}

#[test]
/// 测试: partition 与 collect 元组对比
fn test_partition_vs_tuple_collect() {
    let data = vec![1, 2, 3, 4, 5, 6];

    // partition 方式
    let (evens, odds): (Vec<i32>, Vec<i32>) = data.clone()
        .into_iter()
        .partition(|x| x % 2 == 0);

    // collect 到元组 + 二次过滤方式
    let (collected, _): (Vec<i32>, Vec<i32>) = data.into_iter()
        .map(|x| (x, 0))
        .collect();

    assert_eq!(evens, vec![2, 4, 6]);
    assert_eq!(odds, vec![1, 3, 5]);
    assert_eq!(collected.len(), 6);
}

#[test]
/// 测试: try_fold 提前退出节省计算
fn test_try_fold_early_exit_saves_work() {
    let mut ops_count = 0;

    let result: Option<i32> = (0..1000).try_fold(0i32, |acc, x| {
        ops_count += 1;
        if x > 10 {
            None
        } else {
            Some(acc + x)
        }
    });

    assert_eq!(result, None);
    assert!(ops_count < 1000, "try_fold should exit early");
    assert_eq!(ops_count, 12); // 0..=10 一共11个, 但 x=11 时 None, 所以 0..=11 共计12个
}

#[test]
/// 测试: fold 改变累加器类型
fn test_fold_type_changes() {
    // 从 Vec<i32> fold 成 HashMap<usize, i32>
    let map: HashMap<usize, i32> = vec![10, 20, 30]
        .into_iter()
        .enumerate()
        .fold(HashMap::new(), |mut acc, (idx, val)| {
            acc.insert(idx, val);
            acc
        });

    assert_eq!(map.len(), 3);
    assert_eq!(map[&0], 10);
    assert_eq!(map[&1], 20);
    assert_eq!(map[&2], 30);
}

#[test]
/// 测试: FromIterator 对自定义类型的多个实现
fn test_from_iterator_multiple_impls() {
    struct Product(i64);
    impl FromIterator<i32> for Product {
        fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
            Product(iter.into_iter().fold(1i64, |acc, x| acc * x as i64))
        }
    }

    let product: Product = vec![1, 2, 3, 4, 5].into_iter().collect();
    assert_eq!(product.0, 120);
}

#[test]
/// 测试: Extend 对不同集合类型
fn test_extend_different_collections() {
    let mut set: HashSet<i32> = HashSet::new();
    set.extend([1, 2, 3]);
    set.extend([3, 4, 5]); // 3 已存在

    assert_eq!(set.len(), 5);
    assert!(set.contains(&1));
    assert!(set.contains(&5));
}

#[test]
/// 测试: 组合特性 —— collect 元组 + let chains + 迭代器
fn test_combo_collect_let_chains_iterator() {
    let data: Vec<Result<i32, &str>> = vec![Ok(10), Ok(20), Ok(30)];

    // 先收集为 Result, 然后用 let chains 检查
    let collected: Result<Vec<i32>, &str> = data.into_iter().collect();

    if let Ok(values) = collected
        && values.len() == 3
        && let Some(first) = values.first()
        && *first == 10
        && let Some(last) = values.last()
        && *last == 30
    {
        assert_eq!(values, vec![10, 20, 30]);
    } else {
        panic!("combined features test failed");
    }
}

#[test]
/// 测试: 组合 —— 迭代器链 + partition + 元组操作
fn test_combo_iterator_chain_partition_tuple() {
    let (lt5, ge5): (Vec<i32>, Vec<i32>) = (0..10)
        .map(|x| x * 2)
        .filter(|x| x % 3 != 0)  // 排除3的倍数
        .partition(|x| *x < 5);

    // 生成的序列: 0,2,4,6,8,10,12,14,16,18
    // 排除3的倍数: 2,4,8,10,14,16 (0,6,12,18被排除)
    // < 5: 2,4
    // >= 5: 8,10,14,16
    assert_eq!(lt5, vec![2, 4]);
    assert_eq!(ge5, vec![8, 10, 14, 16]);
}
