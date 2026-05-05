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

    // === 高级示例: 斐波那契数列迭代器 ===
    struct Fibonacci {
        a: u64,
        b: u64,
        limit: u64,
    }

    impl Fibonacci {
        fn new(limit: u64) -> Self {
            Self { a: 0, b: 1, limit }
        }
    }

    impl Iterator for Fibonacci {
        type Item = u64;

        fn next(&mut self) -> Option<Self::Item> {
            if self.a > self.limit {
                return None;
            }
            let next = self.a;
            self.a = self.b;
            self.b = next + self.b;
            Some(next)
        }
    }

    // Iterator 自动获得 blanket IntoIterator 实现，
    // 可以直接用于 for 循环、into_iter()、collect() 等方法

    let fib = Fibonacci::new(100);
    let seq: Vec<u64> = fib.into_iter().collect::<Vec<_>>();
    assert_eq!(seq, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);

    // 只收集偶数项
    let fib_even: Vec<u64> = Fibonacci::new(100)
        .filter(|&x| x % 2 == 0)
        .collect();
    assert_eq!(fib_even, vec![0, 2, 8, 34]);

    // === 双向迭代器: 实现 DoubleEndedIterator ===
    struct RangeReversible {
        front: i32,
        back: i32,
    }

    impl RangeReversible {
        fn new(from: i32, to: i32) -> Self {
            Self { front: from, back: to }
        }
    }

    impl Iterator for RangeReversible {
        type Item = i32;

        fn next(&mut self) -> Option<Self::Item> {
            if self.front <= self.back {
                let val = self.front;
                self.front += 1;
                Some(val)
            } else {
                None
            }
        }
    }

    impl DoubleEndedIterator for RangeReversible {
        fn next_back(&mut self) -> Option<Self::Item> {
            if self.front <= self.back {
                let val = self.back;
                self.back -= 1;
                Some(val)
            } else {
                None
            }
        }
    }

    let mut r = RangeReversible::new(1, 5);
    assert_eq!(r.next(), Some(1));
    assert_eq!(r.next_back(), Some(5));
    assert_eq!(r.next(), Some(2));
    assert_eq!(r.next_back(), Some(4));
    assert_eq!(r.next(), Some(3));
    assert_eq!(r.next(), None);

    // rev 方法来自 DoubleEndedIterator
    let reversed: Vec<i32> = RangeReversible::new(1, 5).rev().collect();
    assert_eq!(reversed, vec![5, 4, 3, 2, 1]);
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

#[test]
/// 测试: IntoIterator 三种实现 (for T / for &T / for &mut T)
fn test_into_iterator_impls() {
    // 语法: IntoIterator trait 有三种标准实现, 决定迭代器产出的元素类型和所有权
    //
    // 三种实现:
    //   - impl IntoIterator for Vec<T>      → Item = T     (获取所有权)
    //   - impl IntoIterator for &Vec<T>     → Item = &T     (不可变借用)
    //   - impl IntoIterator for &mut Vec<T> → Item = &mut T (可变借用)
    //
    // 核心理解:
    //   - for x in vec → 消费 vec, x: T
    //   - for x in &vec → 借用, x: &T (等价于 vec.iter())
    //   - for x in &mut vec → 可变借用, x: &mut T (等价于 vec.iter_mut())
    //
    // 避坑:
    //   - &Vec<T> 的 into_iter 不与 Vec<T> 冲突, 因为 self 类型不同
    //   - Rust 2021+ 中 for 循环会自动对数组调用 into_iter()
    //   - 使用 into_iter() 后原集合不可再访问

    // === for T: 获取所有权 ===
    let v = vec![1, 2, 3];
    let mut sum = 0;
    for x in v {
        // x: i32, v 被消费
        sum += x;
    }
    assert_eq!(sum, 6);
    // v 已被 move, 不能再使用

    // 显式调用 into_iter
    let v = vec![String::from("a"), String::from("b")];
    let collected: Vec<String> = v.into_iter().collect();
    assert_eq!(collected, vec!["a", "b"]);

    // === for &T: 不可变借用 ===
    let v = vec![1, 2, 3];
    let mut collected = Vec::new();
    for &x in &v {
        // x: i32 (解引用), v 仍可用
        collected.push(x);
    }
    assert_eq!(collected, vec![1, 2, 3]);
    assert_eq!(v.len(), 3); // v 仍然有效

    // 显式调用: (&v).into_iter()
    let v = vec![1, 2, 3];
    let iter = (&v).into_iter(); // 等价于 v.iter()
    let sum: i32 = iter.sum();
    assert_eq!(sum, 6);
    assert_eq!(v.len(), 3);

    // === for &mut T: 可变借用 ===
    let mut v = vec![1, 2, 3];
    for x in &mut v {
        // x: &mut i32
        *x *= 2;
    }
    assert_eq!(v, vec![2, 4, 6]);

    // 显式调用: (&mut v).into_iter()
    let mut v = vec![10, 20, 30];
    let iter = (&mut v).into_iter(); // 等价于 v.iter_mut()
    for x in iter {
        *x += 1;
    }
    assert_eq!(v, vec![11, 21, 31]);

    // === 数组的 IntoIterator (Rust 2021+) ===
    let arr = [100, 200, 300];
    let sum: i32 = arr.into_iter().sum();
    assert_eq!(sum, 600);

    // === 验证 &Vec<T> 的 IntoIterator 与 vec.iter() 等价 ===
    let v = vec![5, 10, 15];
    let a: Vec<&i32> = (&v).into_iter().collect();
    let b: Vec<&i32> = v.iter().collect();
    assert_eq!(a, b);
}

#[test]
/// 测试: 迭代器适配器大全 (scan/peekable/flat_map/inspect/fuse/step_by/...)
fn test_iterator_adapters_comprehensive() {
    // 语法: 覆盖标准库中全套适配器方法, 包括不常用的高级适配器
    //
    // 避坑:
    //   - scan 的闭包返回 Option<B>, 返回 None 则停止迭代
    //   - peekable 的 peek 返回 Option<&Item>, 不消费元素
    //   - fuse 保证在首次 None 后永远返回 None
    //   - inspect 不改变迭代器内容, 只执行副作用

    // === scan: 带状态的迭代 ===
    let v = vec![1, 2, 3, 4, 5];
    let running_total: Vec<i32> = v
        .iter()
        .scan(0, |acc, &x| {
            *acc += x;
            Some(*acc)
        })
        .collect();
    assert_eq!(running_total, vec![1, 3, 6, 10, 15]);

    // scan 可提前终止: 返回 None 即停止
    let first_three: Vec<i32> = v
        .iter()
        .scan(0, |count, &x| {
            *count += 1;
            if *count <= 3 { Some(x) } else { None }
        })
        .collect();
    assert_eq!(first_three, vec![1, 2, 3]);

    // === flat_map: 映射并展平 ===
    let pairs = vec!["hello", "world"];
    let chars: Vec<char> = pairs
        .iter()
        .flat_map(|s| s.chars())
        .collect();
    assert_eq!(chars, vec!['h', 'e', 'l', 'l', 'o', 'w', 'o', 'r', 'l', 'd']);

    // === inspect: 调试用 ===
    let mut side_effect = Vec::new();
    let sum: i32 = vec![1, 2, 3]
        .iter()
        .inspect(|&&x| side_effect.push(x * 10))
        .sum();
    assert_eq!(sum, 6);
    assert_eq!(side_effect, vec![10, 20, 30]);

    // === fuse: None 后固化 ===
    let mut iter = (0..3).filter(|&x| x < 2).fuse();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None); // fuse 保证始终为 None
    assert_eq!(iter.next(), None);

    // 非 fuse 的 filter 行为:
    // Filter 迭代器在返回 None 后如果再次调用 next 可能 panic
    // (本例中不会, 因为底层 range 返回 None 后也固化)

    // === step_by: 指定步长 ===
    let stepped: Vec<i32> = (0..=10).step_by(3).collect();
    assert_eq!(stepped, vec![0, 3, 6, 9]);

    let stepped_back: Vec<i32> = (0..=10).step_by(3).collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    assert_eq!(stepped_back, vec![9, 6, 3, 0]);

    // === chain: 连接不同类型迭代器 ===
    let a: Vec<i32> = vec![1, 2];
    let b: Vec<i32> = vec![3, 4, 5];
    let chained: Vec<i32> = a.iter().chain(b.iter()).copied().collect();
    assert_eq!(chained, vec![1, 2, 3, 4, 5]);

    // === 组合: map + filter + take ===
    let result: Vec<i32> = (1..)
        .map(|x| x * x)
        .filter(|x| x % 3 == 0)
        .take(4)
        .collect();
    assert_eq!(result, vec![9, 36, 81, 144]); // 3², 6², 9², 12²

    // === copied / cloned ===
    let v = vec![&1, &2, &3];
    let owned: Vec<i32> = v.into_iter().copied().collect();
    assert_eq!(owned, vec![1, 2, 3]);

    // cloned 用于非 Copy 类型
    let v = vec![String::from("a"), String::from("b")];
    let cloned: Vec<String> = v.iter().cloned().collect();
    assert_eq!(cloned, vec!["a", "b"]);
}

#[test]
/// 测试: 迭代器消费者大全 (try_fold/try_reduce/find_map/partition/unzip/max_by_key/...)
fn test_iterator_consumers_comprehensive() {
    // 语法: 覆盖标准库中不常用的消费者方法
    //
    // 避坑:
    //   - try_fold/try_reduce 遇到 Err 或 None 立即短路
    //   - find_map 等价于 filter_map + next
    //   - partition 需要两个目标类型相同
    //   - unzip 的 Item 必须是 (A, B) 二元组

    // === try_fold: 可短路的折叠 ===
    let result: Result<i32, &str> = vec![1, 2, 3, 4, 5]
        .iter()
        .try_fold(0, |acc, &x| {
            if x > 3 { Err("超过3了!") }
            else { Ok(acc + x) }
        });
    assert_eq!(result, Err("超过3了!"));

    // 截止在 x=4 之前
    let result: Result<i32, &str> = vec![1, 2, 3]
        .iter()
        .try_fold(0, |acc, &x| if x < 5 { Ok(acc + x) } else { Err("太大") });
    assert_eq!(result, Ok(6));

    // === try_reduce 的模拟 (try_reduce 是 nightly 特性, 这里用 try_fold 模拟) ===
    // try_fold 可以在 Err 时短路，效果与 try_reduce 相似
    let result: Result<i32, &str> = vec![1, 2, 3, 100, 5]
        .iter()
        .copied()
        .try_fold(0, |acc, b| if b < 10 { Ok(acc + b) } else { Err("超限") });
    assert!(result.is_err());

    let result: Result<i32, &str> = vec![1, 2, 3]
        .iter()
        .copied()
        .try_fold(0, |acc, b| Ok(acc + b));
    assert_eq!(result, Ok(6));

    // 空迭代器上的 try_fold 返回初始值
    let empty: Vec<i32> = vec![];
    let result: Result<i32, &str> = empty
        .iter()
        .copied()
        .try_fold(0, |acc, b| Ok(acc + b));
    assert_eq!(result, Ok(0));

    // === find_map: 查找并转换 ===
    let v = vec!["1", "abc", "3", "xyz"];
    let first_parsed: Option<i32> = v
        .iter()
        .find_map(|s| s.parse::<i32>().ok());
    assert_eq!(first_parsed, Some(1));

    let no_match: Option<i32> = vec!["abc", "xyz"]
        .iter()
        .find_map(|s| s.parse::<i32>().ok());
    assert_eq!(no_match, None);

    // === partition: 二分收集 ===
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let (evens, odds): (Vec<i32>, Vec<i32>) = numbers
        .into_iter()
        .partition(|n| n % 2 == 0);
    assert_eq!(evens, vec![2, 4, 6, 8]);
    assert_eq!(odds, vec![1, 3, 5, 7]);

    // 空输入
    let empty_input: Vec<i32> = vec![];
    let (e, o): (Vec<i32>, Vec<i32>) = empty_input.into_iter().partition(|n| n % 2 == 0);
    assert!(e.is_empty());
    assert!(o.is_empty());

    // === unzip: 解耦二元组 ===
    let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
    let (letters, numbers): (Vec<&str>, Vec<i32>) = pairs.into_iter().unzip();
    assert_eq!(letters, vec!["a", "b", "c"]);
    assert_eq!(numbers, vec![1, 2, 3]);

    // === max_by_key / min_by_key ===
    #[derive(Debug, PartialEq)]
    struct Item { name: &'static str, value: i32 }
    let items = vec![
        Item { name: "foo", value: 30 },
        Item { name: "bar", value: 10 },
        Item { name: "baz", value: 20 },
    ];
    let max_item = items.iter().max_by_key(|it| it.value);
    assert_eq!(max_item.unwrap().name, "foo");

    let min_item = items.iter().min_by_key(|it| it.value);
    assert_eq!(min_item.unwrap().name, "bar");

    // === all: 空迭代器返回 true ===
    let empty: Vec<i32> = vec![];
    assert!(empty.iter().all(|&x| x > 100)); // 空真

    // === any: 空迭代器返回 false ===
    assert!(!empty.iter().any(|&x| x > 100));

    // === for_each: 不返回值 ===
    let mut count = 0;
    vec![10, 20, 30].iter().for_each(|_| count += 1);
    assert_eq!(count, 3);

    // === try_for_each: 可短路的 for_each ===
    let mut sum = 0;
    let result: Result<(), &str> = vec![1, 2, 3, 100, 4]
        .iter()
        .try_for_each(|&x| {
            if x > 10 { Err("太大") }
            else { sum += x; Ok(()) }
        });
    assert_eq!(result, Err("太大"));
    assert_eq!(sum, 6); // 只累加了 1+2+3

    // 全成功
    let mut sum = 0;
    let result: Result<(), &str> = vec![1, 2, 3]
        .iter()
        .try_for_each(|&x| { sum += x; Ok(()) });
    assert_eq!(result, Ok(()));
    assert_eq!(sum, 6);
}

#[test]
/// 测试: collect 魔法 (收集到 Vec/HashMap/BTreeMap/Result/String/HashSet/...)
fn test_collect_magic() {
    // 语法: collect() 可将迭代器收集为任何实现 FromIterator 的类型
    //
    // 常用收集目标:
    //   - Vec<T>, VecDeque<T>, LinkedList<T>
    //   - HashMap<K,V>, BTreeMap<K,V>  (从 (K,V) 对)
    //   - HashSet<T>, BTreeSet<T>       (自动去重)
    //   - String                         (从 char 迭代器)
    //   - Result<Vec<T>, E>              (短路收集, 遇 Err 停止)
    //   - Option<Vec<T>>                 (短路收集, 遇 None 停止)
    //
    // 避坑:
    //   - 收集到 HashMap 时重复 key 会保留最后一个值
    //   - 收集到 Result<Vec<_>, E> 时迭代器 Item = Result<T, E>
    //   - 类型推导失败时使用 turbofish ::<Vec<_>>() 或显式变量类型

    use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, LinkedList, VecDeque};

    // === Vec / VecDeque / LinkedList ===
    let v: Vec<i32> = (1..=5).collect();
    assert_eq!(v, vec![1, 2, 3, 4, 5]);

    let deque: VecDeque<i32> = (1..=5).collect();
    assert_eq!(deque.len(), 5);

    let list: LinkedList<i32> = (1..=5).collect();
    assert_eq!(list.len(), 5);

    // === HashMap: 从 (K, V) 对收集 ===
    let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
    let map: HashMap<&str, i32> = pairs.into_iter().collect();
    assert_eq!(map.get("b"), Some(&2));
    assert_eq!(map.len(), 3);

    // 重复 key: 最后一次写入生效
    let duplicates = vec![("x", 1), ("x", 2), ("x", 3)];
    let map: HashMap<&str, i32> = duplicates.into_iter().collect();
    assert_eq!(map.get("x"), Some(&3));

    // === BTreeMap: 有序映射 ===
    let pairs = vec![("c", 3), ("a", 1), ("b", 2)];
    let map: BTreeMap<&str, i32> = pairs.into_iter().collect();
    let keys: Vec<&&str> = map.keys().collect();
    assert_eq!(keys, vec![&"a", &"b", &"c"]); // 自动排序

    // === HashSet / BTreeSet: 自动去重 ===
    let nums = vec![3, 1, 2, 1, 3, 2, 4];
    let set: HashSet<i32> = nums.into_iter().collect();
    assert_eq!(set.len(), 4);
    assert!(set.contains(&1));

    let nums = vec![3, 1, 2];
    let bset: BTreeSet<i32> = nums.into_iter().collect();
    assert_eq!(bset.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);

    // === String: 从 char 迭代器收集 ===
    let chars = vec!['h', 'e', 'l', 'l', 'o'];
    let s: String = chars.into_iter().collect();
    assert_eq!(s, "hello");

    // === Result<Vec<T>, E>: 短路收集 ===
    let items: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Err("错误!"), Ok(4)];
    let result: Result<Vec<i32>, &str> = items.into_iter().collect();
    assert_eq!(result, Err("错误!")); // 短路

    let items: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Ok(3)];
    let result: Result<Vec<i32>, &str> = items.into_iter().collect();
    assert_eq!(result, Ok(vec![1, 2, 3]));

    // === Option<Vec<T>>: 短路收集 ===
    let items: Vec<Option<i32>> = vec![Some(1), Some(2), None, Some(4)];
    let result: Option<Vec<i32>> = items.into_iter().collect();
    assert_eq!(result, None);

    let items: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    let result: Option<Vec<i32>> = items.into_iter().collect();
    assert_eq!(result, Some(vec![1, 2, 3]));

    // === turbofish 语法 ===
    let collected = (1..=3).collect::<Vec<i32>>();
    assert_eq!(collected, vec![1, 2, 3]);
}

#[test]
/// 测试: 迭代器惰性求值行为
fn test_iterator_lazy_evaluation() {
    // 语法: 迭代器适配器是惰性的, 只有消费者方法才驱动 next() 调用
    //
    // 核心概念:
    //   - 创建迭代器/链式调用适配器: 不执行任何计算
    //   - 消费者方法 (collect/sum/count/find/...) 触发执行
    //   - 每个 next() 调用按需拉取一个元素
    //   - 提前终止的消费者 (find/take) 不会处理剩余元素
    //
    // 避坑:
    //   - 忘记调用消费者, 迭代器管道不会有任何效果
    //   - 无限迭代器 (0..) + 惰性求值不会 OOM

    use std::cell::Cell;

    // === 证明适配器链不会立即执行 ===
    let call_count = Cell::new(0);
    let _pipeline = (0..5).map(|x| {
        call_count.set(call_count.get() + 1);
        x * 2
    });
    // 此时 call_count 应该为 0 —— 没有任何元素被处理
    assert_eq!(call_count.get(), 0);

    // === 消费者触发执行 ===
    let call_count = Cell::new(0);
    let result: Vec<i32> = (0..5)
        .map(|x| {
            call_count.set(call_count.get() + 1);
            x * 2
        })
        .collect();
    assert_eq!(call_count.get(), 5);
    assert_eq!(result, vec![0, 2, 4, 6, 8]);

    // === find 提前终止 ===
    let call_count = Cell::new(0);
    let found = (0..100)
        .map(|x| {
            call_count.set(call_count.get() + 1);
            x * 2
        })
        .find(|&x| x > 10);
    assert_eq!(found, Some(12));
    assert!(call_count.get() < 100); // 不会处理所有 100 个元素

    // === take 提前终止 ===
    let call_count = Cell::new(0);
    let taken: Vec<i32> = (0..100)
        .map(|x| {
            call_count.set(call_count.get() + 1);
            x * x
        })
        .take(3)
        .collect();
    assert_eq!(taken, vec![0, 1, 4]);
    assert_eq!(call_count.get(), 3); // 只处理了 3 个

    // === 无限迭代器 + 惰性求值 ===
    let first_5: Vec<i32> = (0..).take(5).collect();
    assert_eq!(first_5, vec![0, 1, 2, 3, 4]);

    // === 组合: 无限迭代器 + filter + take ===
    let even_squares: Vec<i32> = (1..)
        .map(|x| x * x)
        .filter(|x| x % 2 == 0)
        .take(3)
        .collect();
    assert_eq!(even_squares, vec![4, 16, 36]); // 2², 4², 6²

    // === by_ref: 借用迭代器, 分段消费 ===
    let mut iter = vec![1, 2, 3, 4, 5].into_iter();
    let first_two: Vec<i32> = iter.by_ref().take(2).collect();
    assert_eq!(first_two, vec![1, 2]);
    let rest: Vec<i32> = iter.collect();
    assert_eq!(rest, vec![3, 4, 5]);
}

#[test]
/// 测试: 返回 impl Iterator 的函数
fn test_returning_iterator() {
    // 语法: fn foo() -> impl Iterator<Item = T> 是返回迭代器的惯用方式
    //
    // 核心概念:
    //   - 具体迭代器类型通常很长 (如 Map<Filter<Take<...>, ...>, ...>)
    //   - impl Trait 在返回位置让编译器推断具体类型
    //   - 不能在同一函数中返回两种不同类型的迭代器 (需用 Box<dyn Iterator>)
    //
    // 避坑:
    //   - impl Iterator 是静态分发, 零成本
    //   - 装箱迭代器 (Box<dyn Iterator>) 是动态分发, 有少量运行时开销
    //   - impl Iterator 返回类型不能用于 trait 方法 (除非使用 RPITIT)

    // 返回 impl Iterator — 简洁签名
    fn even_numbers(v: &[i32]) -> impl Iterator<Item = i32> + '_ {
        v.iter().copied().filter(|x| x % 2 == 0)
    }

    // 返回带 map 的迭代器
    fn squares(v: &[i32]) -> impl Iterator<Item = i32> + '_ {
        v.iter().map(|&x| x * x)
    }

    // 返回复杂链式管道
    fn even_squares(v: &[i32]) -> impl Iterator<Item = i32> + '_ {
        v.iter()
            .filter(|&&x| x % 2 == 0)
            .map(|&x| x * x)
            .filter(|&x| x < 50)
    }

    let input = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let evens: Vec<i32> = even_numbers(&input).collect();
    assert_eq!(evens, vec![2, 4, 6, 8]);

    let sq: Vec<i32> = squares(&input).collect();
    assert_eq!(sq, vec![1, 4, 9, 16, 25, 36, 49, 64]);

    let es: Vec<i32> = even_squares(&input).collect();
    assert_eq!(es, vec![4, 16, 36]);

    // === Box<dyn Iterator> — 用于需要运行时多态的场景 ===
    fn maybe_rev(rev: bool, v: &[i32]) -> Box<dyn Iterator<Item = i32> + '_> {
        if rev {
            Box::new(v.iter().rev().copied())
        } else {
            Box::new(v.iter().copied())
        }
    }

    let v = vec![1, 2, 3];
    let fwd: Vec<i32> = maybe_rev(false, &v).collect();
    let rev: Vec<i32> = maybe_rev(true, &v).collect();
    assert_eq!(fwd, vec![1, 2, 3]);
    assert_eq!(rev, vec![3, 2, 1]);
}

#[test]
/// 测试: peekable 和 scan 高级用法
fn test_peekable_and_scan() {
    // 语法: peekable 允许预览下一个元素, scan 允许带状态的迭代
    //
    // 避坑:
    //   - peek 返回 Option<&Item>, 不修改迭代器状态
    //   - peeked_mut 仅在 nightly 或特殊场景可用; 标准库中 peek 不可变
    //   - scan 闭包返回 None 时迭代终止

    // === peekable: 合并相邻相同元素 ===
    let v = vec![1, 1, 2, 2, 2, 3, 3, 1];
    let mut iter = v.into_iter().peekable();
    let mut result = Vec::new();
    loop {
        match iter.next() {
            Some(current) => {
                let mut count = 1;
                while iter.peek() == Some(&current) {
                    iter.next();
                    count += 1;
                }
                result.push((current, count));
            }
            None => break,
        }
    }
    assert_eq!(result, vec![(1, 2), (2, 3), (3, 2), (1, 1)]);

    // === peekable: 解析器 lookahead ===
    let v = vec![1, 2, 3, 4, 5];
    let mut iter = v.into_iter().peekable();
    let mut sum = 0;
    while let Some(&_next) = iter.peek() {
        sum += iter.next().unwrap();
    }
    assert_eq!(sum, 15);

    // === peekable: 下一个元素满足某条件时执行额外操作 ===
    let v = vec![1, 2, 10, 3, 4];
    let mut iter = v.into_iter().peekable();
    let mut result = Vec::new();
    while let Some(val) = iter.next() {
        if iter.peek() == Some(&&10) {
            result.push(val * 100); // 下一个是 10 时特殊处理
        } else {
            result.push(val);
        }
    }
    assert_eq!(result, vec![1, 200, 10, 3, 4]);

    // === scan: 斐波那契数列 ===
    let fib: Vec<u64> = (0..10)
        .scan((0u64, 1u64), |state, _| {
            let next = state.0;
            *state = (state.1, state.0 + state.1);
            Some(next)
        })
        .collect();
    assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);

    // === scan: 限制累积值 ===
    let v = vec![10, 20, 30, 40, 50];
    let capped: Vec<i32> = v
        .iter()
        .scan(0, |acc, &x| {
            *acc += x;
            if *acc > 70 { None }
            else { Some(*acc) }
        })
        .collect();
    assert_eq!(capped, vec![10, 30, 60]); // 10+20+30=60, 下一个40会导致100>70

    // === peekable + fuse 组合 ===
    let mut iter = vec![1].into_iter().peekable();
    assert_eq!(iter.peek(), Some(&1));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.peek(), None); // 耗尽后 peek 也返回 None
}

#[test]
/// 测试: partition 和 unzip 详细用法
fn test_partition_and_unzip() {
    // 语法: partition 二分收集, unzip 解耦二元组
    //
    // 避坑:
    //   - partition 需要两个目标集合类型相同 (都是 Vec<_>)
    //   - unzip 的 Item 必须是 (A, B) 元组
    //   - 两者都是消费者, 会消费迭代器

    // === partition: 按条件二分 ===
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let (evens, odds): (Vec<i32>, Vec<i32>) = numbers
        .into_iter()
        .partition(|n| n % 2 == 0);
    assert_eq!(evens, vec![2, 4, 6, 8, 10]);
    assert_eq!(odds, vec![1, 3, 5, 7, 9]);

    // === partition: 字符串分类 ===
    let words = vec!["apple", "ant", "banana", "bat", "cat", "car"];
    let (short, long): (Vec<&str>, Vec<&str>) = words
        .into_iter()
        .partition(|w| w.len() <= 3);
    assert_eq!(short, vec!["ant", "bat", "cat", "car"]);
    assert_eq!(long, vec!["apple", "banana"]);

    // === unzip: 从 (K, V) 对拆分为两个 Vec ===
    let pairs = vec![
        ("Alice", 85),
        ("Bob", 92),
        ("Charlie", 78),
    ];
    let (names, scores): (Vec<&str>, Vec<i32>) = pairs.into_iter().unzip();
    assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
    assert_eq!(scores, vec![85, 92, 78]);

    // === unzip: 从枚举解耦 ===
    #[derive(Debug, PartialEq)]
    enum Either { Left(i32), Right(String) }

    // unzip 用于 (A, B, C) 三元组需要中间步骤:
    // 先将 Item 映射为 (A, (B, C))，再分步 unzip
    let data = vec![(1, "a", true), (2, "b", false), (3, "c", true)];
    let (nums, rest): (Vec<i32>, Vec<(&str, bool)>) = data
        .into_iter()
        .map(|(n, s, b)| (n, (s, b)))
        .unzip();
    assert_eq!(nums, vec![1, 2, 3]);
    let (letters, flags): (Vec<&str>, Vec<bool>) = rest.into_iter().unzip();
    assert_eq!(letters, vec!["a", "b", "c"]);
    assert_eq!(flags, vec![true, false, true]);

    // === partition + 空集合 ===
    let empty: Vec<i32> = vec![];
    let (e, o): (Vec<i32>, Vec<i32>) = empty.into_iter().partition(|x| x % 2 == 0);
    assert!(e.is_empty());
    assert!(o.is_empty());

    // === unzip + 空集合 ===
    let empty: Vec<(i32, &str)> = vec![];
    let (nums, strs): (Vec<i32>, Vec<&str>) = empty.into_iter().unzip();
    assert!(nums.is_empty());
    assert!(strs.is_empty());
}

#[test]
/// 测试: itertools 扩展方法 (sorted/unique/group_by/cartesian_product/permutations/...)
fn test_itertools_extensions() {
    // 语法: itertools::Itertools trait 为所有迭代器提供额外方法
    //
    // 常用方法:
    //   - sorted()             排序后返回 (非惰性)
    //   - unique()             去重 (内部维护 HashSet)
    //   - group_by(f)          按相邻等值分组 (惰性, 需预排序)
    //   - cartesian_product()  笛卡尔积 (惰性)
    //   - permutations(n)      排列 (非惰性)
    //   - combinations(n)      组合 (惰性)
    //   - intersperse(sep)     元素间插入分隔符 (惰性)
    //   - join(sep)            连接为 String (非惰性)
    //   - kmerge()             合并多个有序迭代器 (惰性)
    //
    // 避坑:
    //   - 需要 `use itertools::Itertools;` 导入 trait
    //   - group_by 按相邻比较, 需预先 sorted 才能全局分组
    //   - permutations/combinations 对大 n 结果数量爆炸

    use itertools::Itertools;

    // === sorted: 排序 ===
    let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
    let sorted: Vec<i32> = v.iter().copied().sorted().collect();
    assert_eq!(sorted, vec![1, 1, 2, 3, 4, 5, 6, 9]);

    // sorted_by: 自定义排序
    let sorted_desc: Vec<i32> = v.iter().copied().sorted_by(|a, b| b.cmp(a)).collect();
    assert_eq!(sorted_desc, vec![9, 6, 5, 4, 3, 2, 1, 1]);

    // === unique: 去重 ===
    let v = vec![1, 2, 2, 3, 1, 3, 4];
    let unique: Vec<i32> = v.iter().copied().unique().collect();
    assert_eq!(unique.len(), 4);
    assert!(unique.contains(&1));

    // === group_by: 相邻等值分组 ===
    let words = vec!["apple", "apricot", "banana", "berry", "cherry"];
    // 使用 chunk_by 按相邻等值分组 (需预先排序, 这里已排好)
    let groups: Vec<(char, Vec<&str>)> = words
        .iter()
        .chunk_by(|w| w.chars().next().unwrap())
        .into_iter()
        .map(|(key, group)| (key, group.copied().collect()))
        .collect();
    assert_eq!(groups.len(), 3); // a, b, c 三组
    assert_eq!(groups[0].0, 'a');
    assert_eq!(groups[1].0, 'b');
    assert_eq!(groups[2].0, 'c');

    // === cartesian_product: 笛卡尔积 ===
    let suits = vec!["红桃", "黑桃", "方块", "梅花"];
    let ranks = vec!["A", "2", "3"];
    let deck: Vec<(&str, &str)> = suits
        .iter()
        .cartesian_product(ranks.iter())
        .map(|(s, r)| (*s, *r))
        .collect();
    assert_eq!(deck.len(), 12); // 4 × 3

    // === permutations: 排列 ===
    let items = vec![1, 2, 3];
    let perms: Vec<Vec<i32>> = items
        .into_iter()
        .permutations(2)
        .collect();
    assert_eq!(perms.len(), 6); // P(3,2) = 6
    assert!(perms.contains(&vec![1, 2]));
    assert!(perms.contains(&vec![2, 1]));

    // === combinations: 组合 ===
    let items = vec![1, 2, 3, 4];
    let combos: Vec<Vec<i32>> = items
        .into_iter()
        .combinations(2)
        .collect();
    assert_eq!(combos.len(), 6); // C(4,2) = 6

    // === intersperse: 插入分隔符 ===
    let words = vec!["a", "b", "c"];
    let joined: String = itertools::Itertools::intersperse(words.iter(), &",")
        .copied()
        .collect();
    assert_eq!(joined, "a,b,c");

    // === join: 直接连接为 String ===
    let words = vec!["hello", "world"];
    let sentence: String = words.iter().join(" ");
    assert_eq!(sentence, "hello world");

    // === kmerge: 合并有序迭代器 ===
    let a = vec![1, 3, 5, 7];
    let b = vec![2, 4, 6];
    let c = vec![0, 8, 9];
    let merged: Vec<i32> = vec![a.into_iter(), b.into_iter(), c.into_iter()]
        .into_iter()
        .kmerge()
        .collect();
    assert_eq!(merged, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

    // === tuple_windows: 滑动窗口 ===
    let v = vec![1, 2, 3, 4, 5];
    let diffs: Vec<i32> = v
        .iter()
        .tuple_windows()
        .map(|(&a, &b)| b - a)
        .collect();
    assert_eq!(diffs, vec![1, 1, 1, 1]);

    // === merge: 合并两个有序迭代器 ===
    let a = vec![1, 3, 5];
    let b = vec![2, 4, 6];
    let merged: Vec<i32> = a.iter().merge(b.iter()).copied().collect();
    assert_eq!(merged, vec![1, 2, 3, 4, 5, 6]);
}
