// ---------------------------------------------------------------------------
// 2.3 迭代器
// ---------------------------------------------------------------------------

#[test]
/// 测试: 迭代器基础 (iter/iter_mut/into_iter/惰性求值)
fn test_iter_basics() {
    // 语法: 三种获取迭代器的方式:
    //   - .iter()         不可变借用迭代 (&T)
    //   - .iter_mut()     可变借用迭代 (&mut T)
    //   - .into_iter()    消费迭代 (T, 取得所有权)
    //
    // 核心概念:
    //   - 迭代器是惰性的, 不调用消费者方法不会执行任何操作
    //   - 迭代器适配器返回新的迭代器, 不修改原迭代器
    //   - Iterator trait 是核心, 只需实现 next() 方法
    //
    // 避坑:
    //   - for 循环自动调用 into_iter() (Rust 2021+), 遍历后原值被消费
    //   - 需要保留原值时用 .iter() 或 &vec
    //   - 迭代器只能消费一次, 再次调用会返回空

    let vec = vec![1, 2, 3];

    // iter: 借用, 不消费
    let sum: i32 = vec.iter().sum();
    assert_eq!(sum, 6);
    assert_eq!(vec.len(), 3); // vec 仍可用

    // iter_mut: 可变借用
    let mut vec = vec![1, 2, 3];
    for x in vec.iter_mut() {
        *x *= 2;
    }
    assert_eq!(vec, vec![2, 4, 6]);

    // into_iter: 消费
    let vec = vec![1, 2, 3];
    let sum: i32 = vec.into_iter().sum();
    assert_eq!(sum, 6);
    // vec 已被消费, 不能再使用
}

#[test]
/// 测试: 迭代器适配器 (map/filter/flat_map/enumerate/zip)
fn test_iter_adapters() {
    // 语法: 适配器方法返回新迭代器, 不立即执行
    //
    // 常用适配器:
    //   - map(f)              转换每个元素
    //   - filter(pred)        过滤满足条件的元素
    //   - filter_map(f)       过滤 + 转换合为一步
    //   - flat_map(f)         映射后展平一层
    //   - enumerate()         附加索引 (i, item)
    //   - zip(other)          配对两个迭代器
    //   - take(n)             取前 n 个
    //   - skip(n)             跳过前 n 个
    //   - take_while(pred)    取到条件不满足为止
    //   - skip_while(pred)    跳过到条件不满足为止
    //   - chain(other)        连接两个迭代器
    //   - cycle()             无限循环迭代器
    //   - rev()               反向 (需 DoubleEndedIterator)
    //   - cloned()            &T → T (T: Clone)
    //   - copied()            &T → T (T: Copy, 更快)
    //   - peekable()          可 peek 下一个元素
    //   - fuse()              None 后始终返回 None
    //   - step_by(n)          每 n 个取一个
    //
    // 避坑:
    //   - 适配器不调用消费者方法不会执行
    //   - zip 以较短的迭代器为准
    //   - take_while/skip_while 遇到第一个不匹配的即停止, 不会继续检查后续
    //   - cycle 是无限迭代器, 必须配合 take 使用

    // map
    let doubled: Vec<i32> = vec![1, 2, 3]
        .iter()
        .map(|x| x * 2)
        .collect();
    assert_eq!(doubled, vec![2, 4, 6]);

    // filter
    let evens: Vec<i32> = vec![1, 2, 3, 4, 5]
        .into_iter()
        .filter(|x| x % 2 == 0)
        .collect();
    assert_eq!(evens, vec![2, 4]);

    // filter_map (过滤 + 转换)
    let parsed: Vec<i32> = vec!["1", "abc", "3", "xyz"]
        .into_iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    assert_eq!(parsed, vec![1, 3]);

    // flat_map
    let nested = vec![vec![1, 2], vec![3, 4]];
    let flat: Vec<i32> = nested
        .into_iter()
        .flat_map(|v| v)
        .collect();
    assert_eq!(flat, vec![1, 2, 3, 4]);

    // enumerate
    let indexed: Vec<(usize, &str)> = vec!["a", "b", "c"]
        .iter()
        .copied()
        .enumerate()
        .collect();
    assert_eq!(indexed, vec![(0, "a"), (1, "b"), (2, "c")]);

    // zip (以短的为准)
    let a = vec![1, 2, 3];
    let b = vec!["a", "b"];
    let zipped: Vec<_> = a.iter().zip(b.iter()).collect();
    assert_eq!(zipped, vec![(&1, &"a"), (&2, &"b")]);

    // take / skip
    let taken: Vec<i32> = (0..10).take(3).collect();
    assert_eq!(taken, vec![0, 1, 2]);

    let skipped: Vec<i32> = (0..10).skip(7).collect();
    assert_eq!(skipped, vec![7, 8, 9]);

    // take_while / skip_while
    let taken: Vec<i32> = vec![1, 2, 4, 8, 3, 4]
        .into_iter()
        .take_while(|x| *x < 5)
        .collect();
    assert_eq!(taken, vec![1, 2, 4]); // 遇到 8 停止, 后面的 3, 4 也被丢弃

    // chain
    let chained: Vec<i32> = vec![1, 2]
        .into_iter()
        .chain(vec![3, 4])
        .collect();
    assert_eq!(chained, vec![1, 2, 3, 4]);

    // step_by
    let stepped: Vec<i32> = (0..10).step_by(3).collect();
    assert_eq!(stepped, vec![0, 3, 6, 9]);

    // cycle + take (无限迭代器必须限制)
    let cycled: Vec<i32> = vec![1, 2, 3]
        .into_iter()
        .cycle()
        .take(7)
        .collect();
    assert_eq!(cycled, vec![1, 2, 3, 1, 2, 3, 1]);

    // peekable
    let mut iter = vec![1, 2, 3]
        .into_iter()
        .peekable();
    assert_eq!(iter.peek(), Some(&1)); // 不消费
    assert_eq!(iter.next(), Some(1)); // 消费
    assert_eq!(iter.peek(), Some(&2));
}

#[test]
/// 测试: 迭代器消费者 (collect/fold/reduce/sum/count/any/all)
fn test_iter_consumers() {
    // 语法: 消费者方法消费整个迭代器, 触发惰性求值
    //
    // 常用消费者:
    //   - collect()            收集到集合 (需类型注解)
    //   - count()              计数
    //   - last()               最后一个元素
    //   - nth(n)               第 n 个元素 (从 0 开始)
    //   - find(pred)           查找第一个满足条件的
    //   - position(pred)       查找第一个满足条件的索引
    //   - rposition(pred)      从右查找 (需 DoubleEndedIterator)
    //   - max() / min()        最大/最小值
    //   - max_by(cmp)          自定义比较的最大值
    //   - sum() / product()    求和/求积
    //   - fold(acc, f)         左折叠 (类似 reduce, 有初始值)
    //   - reduce(f)            折叠 (无初始值, 返回 Option)
    //   - all(pred)            所有元素满足条件
    //   - any(pred)            任一元素满足条件
    //   - for_each(f)          对每个元素执行操作
    //
    // 避坑:
    //   - collect() 需要类型注解, 编译器无法推断时加 turbofish ::<Vec<_>>()
    //   - reduce 在空迭代器上返回 None
    //   - fold 的初始值类型决定返回类型
    //   - for_each 不返回值, 仅副作用; 优先用 for 循环

    // collect (turbofish 语法)
    let nums: Vec<i32> = (1..=5).collect();
    assert_eq!(nums, vec![1, 2, 3, 4, 5]);

    // 或者用变量类型推导
    let nums = (1..=5).collect::<Vec<_>>();
    assert_eq!(nums.len(), 5);

    // count
    assert_eq!(vec![1, 2, 3].iter().count(), 3);

    // last / nth
    let v = vec![10, 20, 30, 40];
    assert_eq!(v.iter().last(), Some(&40));
    assert_eq!(v.iter().nth(1), Some(&20));
    assert_eq!(v.iter().nth(10), None);

    // find
    assert_eq!(
        vec![1, 3, 5, 7]
            .iter()
            .find(|x| **x > 4),
        Some(&5)
    );
    assert_eq!(
        vec![1, 3]
            .iter()
            .find(|x| **x > 4),
        None
    );

    // position
    assert_eq!(
        vec![1, 3, 5, 7]
            .iter()
            .position(|x| *x == 5),
        Some(2)
    );
    assert_eq!(
        vec![1, 3]
            .iter()
            .position(|x| *x == 5),
        None
    );

    // max / min
    assert_eq!(vec![3, 1, 4, 1, 5].iter().max(), Some(&5));
    assert_eq!(vec![3, 1, 4, 1, 5].iter().min(), Some(&1));

    // max_by (自定义比较)
    let words = vec!["apple", "banana", "kiwi"];
    let longest = words
        .iter()
        .max_by(|a, b| a.len().cmp(&b.len()));
    assert_eq!(longest, Some(&"banana"));

    // sum / product
    assert_eq!(
        vec![1, 2, 3, 4]
            .iter()
            .sum::<i32>(),
        10
    );
    assert_eq!(
        vec![1, 2, 3, 4]
            .iter()
            .product::<i32>(),
        24
    );

    // fold
    let sum = vec![1, 2, 3]
        .iter()
        .fold(0, |acc, x| acc + x);
    assert_eq!(sum, 6);

    // fold 也可以做类型转换
    let concatenated = vec!["a", "b", "c"]
        .iter()
        .fold(String::new(), |acc, s| acc + s);
    assert_eq!(concatenated, "abc");

    // reduce (空迭代器返回 None)
    let sum = vec![1, 2, 3]
        .iter()
        .copied()
        .reduce(|a, b| a + b);
    assert_eq!(sum, Some(6));

    let empty: Vec<i32> = vec![];
    assert!(
        empty
            .iter()
            .copied()
            .reduce(|a, b| a + b)
            .is_none()
    );

    // all / any
    assert!(
        vec![2, 4, 6]
            .iter()
            .all(|x| x % 2 == 0)
    );
    assert!(
        !vec![2, 3, 4]
            .iter()
            .all(|x| x % 2 == 0)
    );
    assert!(
        vec![1, 2, 3]
            .iter()
            .any(|x| *x > 2)
    );
    assert!(
        !vec![1, 2, 3]
            .iter()
            .any(|x| *x > 10)
    );

    // for_each
    let mut sum = 0;
    vec![1, 2, 3]
        .iter()
        .for_each(|x| sum += x);
    assert_eq!(sum, 6);
}

#[test]
/// 测试: 迭代器操作 (chain/flatten/inspect/sum)
fn test_iter_methods() {
    // 语法: chain 连接迭代器, flatten 展平嵌套, inspect 副作用调试, sum 求和
    // 避坑: 迭代器是惰性的, 不调用 collect/sum 等消费方法不会执行; inspect 不消费元素
    let nums = vec![1, 2, 3, 4, 5];

    let chained: Vec<i32> = nums
        .iter()
        .chain(&vec![6, 7])
        .cloned()
        .collect();
    assert_eq!(chained, vec![1, 2, 3, 4, 5, 6, 7]);

    let nested = vec![vec![1, 2], vec![3, 4]];
    let flat: Vec<i32> = nested
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(flat, vec![1, 2, 3, 4]);

    let mut inspected = Vec::new();
    let sum: i32 = (1..=3)
        .inspect(|x| inspected.push(*x))
        .sum();
    assert_eq!(sum, 6);
    assert_eq!(inspected, vec![1, 2, 3]);
}

#[test]
/// 测试: 专用迭代器 (once/repeat/from_fn)
fn test_specialized_iter() {
    // 语法: once 单个元素, repeat 无限重复(需配合 take), from_fn 闭包生成
    // 避坑: repeat 是无限迭代器, 必须用 take/for 等限制; from_fn 闭包返回 Option
    let once: Vec<i32> = std::iter::once(42).collect();
    assert_eq!(once, vec![42]);

    let repeat: Vec<i32> = std::iter::repeat(7)
        .take(3)
        .collect();
    assert_eq!(repeat, vec![7, 7, 7]);

    let mut count = 0;
    let from_fn: Vec<i32> = std::iter::from_fn(|| {
        count += 1;
        if count <= 3 { Some(count) } else { None }
    })
    .collect();
    assert_eq!(from_fn, vec![1, 2, 3]);
}

#[test]
/// 测试: 自定义迭代器 (实现 Iterator trait)
fn test_custom_iterator() {
    // 语法: 实现 Iterator trait 只需实现 next() 方法
    // 避坑: next 返回 Option<Item>; 迭代器结束后应持续返回 None

    struct Counter {
        current: u32,
        max: u32,
    }

    impl Counter {
        fn new(max: u32) -> Self {
            Self { current: 1, max }
        }
    }

    impl Iterator for Counter {
        type Item = u32;

        fn next(&mut self) -> Option<Self::Item> {
            if self.current <= self.max {
                let val = self.current;
                self.current += 1;
                Some(val)
            } else {
                None
            }
        }
    }

    let counter = Counter::new(5);
    let nums: Vec<u32> = counter.collect();
    assert_eq!(nums, vec![1, 2, 3, 4, 5]);

    // 自定义迭代器可以使用所有 Iterator 适配器
    let sum: u32 = Counter::new(5)
        .filter(|x| x % 2 == 0)
        .sum();
    assert_eq!(sum, 2 + 4);
}

#[test]
/// 测试: IntoIterator trait (for 循环背后的机制)
fn test_into_iterator() {
    // 语法: for x in iter 自动调用 IntoIterator::into_iter()
    // 避坑: Vec 的 into_iter 消费值; &Vec 的 into_iter 等价于 iter()

    // 在 for 循环中使用
    let mut sum = 0;
    for x in vec![1, 2, 3] {
        sum += x;
    }
    assert_eq!(sum, 6);

    // 显式使用 IntoIterator
    let vec = vec![1, 2, 3];
    let iter: std::vec::IntoIter<i32> = vec.into_iter();
    let collected: Vec<i32> = iter.collect();
    assert_eq!(collected, vec![1, 2, 3]);

    // 数组的 into_iter (Rust 2021+)
    let arr = [10, 20, 30];
    let sum: i32 = arr.into_iter().sum();
    assert_eq!(sum, 60);
}

#[test]
/// 测试: 迭代器与所有权 (by-value vs by-ref vs by-mut-ref)
fn test_iterator_ownership() {
    // 语法: 迭代器的所有权行为决定你能对元素做什么
    //
    // 三种模式:
    //   - .into_iter()  → T       取得所有权, 可移动
    //   - .iter()       → &T      只读借用
    //   - .iter_mut()   → &mut T  可变借用
    //
    // 避坑:
    //   - 对 &Vec 调用 into_iter() 得到 &T 迭代器 (Deref 自动转换)
    //   - 对 Vec 调用 into_iter() 后原 Vec 不可再用
    //   - 迭代器中的引用生命周期受原数据约束

    let strings = vec![String::from("hello"), String::from("world")];

    // by-value: 消费, 可移动
    let upper: Vec<String> = strings
        .clone()
        .into_iter()
        .map(|s| s.to_uppercase())
        .collect();
    assert_eq!(upper, vec!["HELLO", "WORLD"]);

    // by-ref: 借用, 不可修改
    let lengths: Vec<usize> = strings
        .iter()
        .map(|s| s.len())
        .collect();
    assert_eq!(lengths, vec![5, 5]);

    // by-mut-ref: 可变借用, 可修改
    let mut strings = strings;
    for s in strings.iter_mut() {
        s.push('!');
    }
    assert_eq!(strings, vec!["hello!", "world!"]);
}

#[test]
/// 测试: 范围迭代器 (Range/RangeInclusive/StepBy/Rev)
fn test_range_iterators() {
    // 语法: 范围本身就是迭代器, 无需转换
    //
    // 范围类型:
    //   - a..b       Range       [a, b)
    //   - a..=b      RangeIncl   [a, b]
    //   - ..b        RangeTo     (-∞, b)
    //   - a..        RangeFrom   [a, +∞)
    //   - ..         RangeFull   (-∞, +∞) 用于切片 s[..]
    //
    // 避坑:
    //   - a > b 时 a..b 为空迭代, 不 panic
    //   - 浮点数不支持范围迭代
    //   - 大范围 (0..u64::MAX) 不会内存溢出, 惰性生成

    // 基本范围
    let nums: Vec<i32> = (1..5).collect();
    assert_eq!(nums, vec![1, 2, 3, 4]);

    // 包含范围
    let nums: Vec<i32> = (1..=5).collect();
    assert_eq!(nums, vec![1, 2, 3, 4, 5]);

    // 反向
    let nums: Vec<i32> = (1..=5).rev().collect();
    assert_eq!(nums, vec![5, 4, 3, 2, 1]);

    // 步长
    let nums: Vec<i32> = (0..10).step_by(3).collect();
    assert_eq!(nums, vec![0, 3, 6, 9]);

    // 空范围
    let nums: Vec<i32> = (5..3).collect();
    assert!(nums.is_empty());

    // 范围用于索引
    let chars = ['a', 'b', 'c', 'd', 'e'];
    let slice: Vec<char> = chars
        .iter()
        .take(3)
        .cloned()
        .collect();
    assert_eq!(slice, vec!['a', 'b', 'c']);
}
