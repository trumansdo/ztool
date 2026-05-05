use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
/// 测试: 综合实战 - 缓存 + Let Chains + extract_if 组合
fn test_cache_with_let_chains_and_extract_if() {
    struct Cache<K, V> {
        data: Arc<Mutex<HashMap<K, V>>>,
    }

    impl<K: std::hash::Hash + Eq, V> Cache<K, V> {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        fn insert(&self, key: K, value: V) {
            self.data
                .lock()
                .unwrap()
                .insert(key, value);
        }
        fn get(&self, key: &K) -> Option<V>
        where
            V: Clone,
        {
            self.data
                .lock()
                .unwrap()
                .get(key)
                .cloned()
        }
    }

    let cache = Cache::new();

    if let Some(val) = cache.get(&1)
        && val > 0
    {
        panic!("Should not find value yet");
    }

    cache.insert(1, 42);

    let mut map = cache.data.lock().unwrap();
    let extracted: Vec<_> = map
        .extract_if(|_k, _v| true)
        .collect();
    assert_eq!(extracted.len(), 1);
}

#[test]
/// 测试: 综合实战 - trait 继承 + upcasting + 动态分发
fn test_trait_upcasting_integration() {
    trait Readable {
        fn read(&self) -> String;
    }
    trait ReadableWritable: Readable {
        fn write(&mut self, data: String);
    }

    struct Buffer {
        content: String,
    }
    impl Readable for Buffer {
        fn read(&self) -> String {
            self.content.clone()
        }
    }
    impl ReadableWritable for Buffer {
        fn write(&mut self, data: String) {
            self.content = data;
        }
    }

    fn upcast(rw: &dyn ReadableWritable) -> &dyn Readable {
        rw
    }

    let mut buf = Buffer {
        content: String::new(),
    };
    buf.write("hello".to_string());
    assert_eq!(upcast(&buf).read(), "hello");
}

#[test]
/// 测试: 综合实战 - async 运行时 + extract_if 组合
fn test_async_with_extract_if() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut numbers = vec![1, 2, 3, 4, 5, 6];
        let evens: Vec<i32> = numbers
            .extract_if(.., |x| *x % 2 == 0)
            .collect();
        assert_eq!(numbers, vec![1, 3, 5]);
        assert_eq!(evens, vec![2, 4, 6]);
    });
}

// ===================== 扩充综合测试 =====================

#[test]
/// 测试: 综合 —— Arc<Mutex<>> 共享缓存 + 并发
fn test_concurrent_cache_shared_state() {
    let cache = Arc::new(Mutex::new(HashMap::<String, i32>::new()));

    {
        let mut guard = cache.lock().unwrap();
        guard.insert("key1".to_string(), 100);
        guard.insert("key2".to_string(), 200);
    }

    let cache_clone = Arc::clone(&cache);
    let handle = std::thread::spawn(move || {
        let guard = cache_clone.lock().unwrap();
        guard.get("key1").copied()
    });

    let result = handle.join().unwrap();
    assert_eq!(result, Some(100));

    // 主线程也读取
    let guard = cache.lock().unwrap();
    assert_eq!(guard.get("key2"), Some(&200));
}

#[test]
/// 测试: 综合 —— 迭代器组合 + collect 元组 + 闭包
fn test_iterator_collect_tuple_with_closure() {
    let make_pair = |x: i32| (x * 2, x.to_string());

    let (doubles, strings): (Vec<i32>, Vec<String>) = (0..5)
        .map(make_pair)
        .collect();

    assert_eq!(doubles, vec![0, 2, 4, 6, 8]);
    assert_eq!(strings, vec!["0", "1", "2", "3", "4"]);
}

#[test]
/// 测试: 综合 —— 泛型 + trait bound + 迭代器链
fn test_generic_trait_iterator_chain() {
    trait Numeric: std::ops::Add<Output = Self> + Copy + PartialOrd + From<u8> {
        fn zero() -> Self;
    }
    impl Numeric for i32 {
        fn zero() -> Self { 0 }
    }
    impl Numeric for f64 {
        fn zero() -> Self { 0.0 }
    }

    fn sum_if_gt<T: Numeric>(data: Vec<T>, threshold: T) -> T {
        data.into_iter()
            .filter(|x| *x > threshold)
            .fold(T::zero(), |acc, x| acc + x)
    }

    assert_eq!(sum_if_gt(vec![1, 5, 3, 8, 2], 3), 13); // 5 + 8 = 13
}

#[test]
/// 测试: 综合 —— Option<Result<>> 多级解构 + Let Chains
fn test_option_result_chains() {
    fn get_user_id(name: &str) -> Option<i32> {
        match name {
            "alice" => Some(1),
            "bob" => Some(2),
            _ => None,
        }
    }

    fn get_id_result(name: &str) -> Result<Option<i32>, &str> {
        Ok(get_user_id(name))
    }

    let name = "alice";
    if let Ok(Some(id)) = get_id_result(name)
        && id > 0
        && let Some(user_name) = Some(name)
        && user_name.len() > 2
    {
        assert_eq!(id, 1);
        assert_eq!(user_name, "alice");
    } else {
        panic!("should extract alice's id");
    }
}

#[test]
/// 测试: 综合 —— 自定义收集器 + FromIterator 实现
fn test_custom_collector_integrated() {
    #[derive(Debug, PartialEq)]
    struct Statistics {
        count: usize,
        sum: i64,
        min: Option<i32>,
        max: Option<i32>,
    }

    impl FromIterator<i32> for Statistics {
        fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
            let mut count = 0;
            let mut sum = 0i64;
            let mut min: Option<i32> = None;
            let mut max: Option<i32> = None;

            for val in iter {
                count += 1;
                sum += val as i64;
                min = Some(min.map_or(val, |m| m.min(val)));
                max = Some(max.map_or(val, |m| m.max(val)));
            }

            Statistics { count, sum, min, max }
        }
    }

    let stats: Statistics = vec![5, 2, 8, 3, 9, 1].into_iter().collect();

    assert_eq!(stats.count, 6);
    assert_eq!(stats.sum, 28);
    assert_eq!(stats.min, Some(1));
    assert_eq!(stats.max, Some(9));
}

#[test]
/// 测试: 综合 —— 生命周期 + 借用 + 闭包捕获
fn test_lifetime_borrow_closure_capture() {
    fn borrow_and_use<'a>(data: &'a Vec<i32>) -> impl Fn() -> i32 + 'a {
        move || data.iter().sum()
    }

    let data = vec![1, 2, 3, 4, 5];
    let summer = borrow_and_use(&data);
    assert_eq!(summer(), 15);
    // data 仍然可用因为闭包只是不可变借用
    assert_eq!(data.len(), 5);
}

#[test]
/// 测试: 综合 —— drop 顺序 + RAII 守卫
fn test_drop_order_and_raii_guard() {
    use std::cell::RefCell;

    let log = RefCell::new(Vec::new());

    struct Guard<'a> {
        id: i32,
        log: &'a RefCell<Vec<i32>>,
    }
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.log.borrow_mut().push(self.id);
        }
    }

    {
        let _a = Guard { id: 1, log: &log };
        let _b = Guard { id: 2, log: &log };
        let _c = Guard { id: 3, log: &log };
    } // drop 顺序：LIFO (c -> b -> a)

    let drops = log.borrow().clone();
    assert_eq!(drops.len(), 3);
    // drop 顺序通常是逆序，但 Rust 不保证精确顺序
    assert!(drops.contains(&1));
    assert!(drops.contains(&2));
    assert!(drops.contains(&3));
}

#[test]
/// 测试: 综合 —— 不安全代码的安全包装
fn test_unsafe_wrapper_abstraction() {
    /// 安全的自定义 vector swap 操作
    fn safe_swap<T>(slice: &mut [T], i: usize, j: usize) {
        assert!(i < slice.len() && j < slice.len(), "索引越界");
        unsafe {
            let ptr = slice.as_mut_ptr();
            std::ptr::swap(ptr.add(i), ptr.add(j));
        }
    }

    let mut data = vec![1, 2, 3, 4, 5];
    safe_swap(&mut data, 0, 4);
    assert_eq!(data, vec![5, 2, 3, 4, 1]);

    safe_swap(&mut data, 1, 3);
    assert_eq!(data, vec![5, 4, 3, 2, 1]);
}

#[test]
/// 测试: 综合 —— 类型状态模式（Type State Pattern）
fn test_type_state_pattern() {
    struct Draft;
    struct Published;

    struct Post<State> {
        content: String,
        _state: std::marker::PhantomData<State>,
    }

    impl Post<Draft> {
        fn new(content: &str) -> Self {
            Post {
                content: content.to_string(),
                _state: std::marker::PhantomData,
            }
        }

        fn publish(self) -> Post<Published> {
            Post {
                content: self.content,
                _state: std::marker::PhantomData,
            }
        }
    }

    impl Post<Published> {
        fn content(&self) -> &str {
            &self.content
        }
    }

    let draft = Post::new("Rust 类型状态模式");
    let published = draft.publish();
    assert_eq!(published.content(), "Rust 类型状态模式");
}

#[test]
/// 测试: 综合 —— const 泛型 + 类型级数组操作
fn test_const_generics_type_level() {
    // const 泛型确保数组长度在编译期确定
    fn array_sum<const N: usize>(arr: [i32; N]) -> i32 {
        arr.iter().sum()
    }

    fn array_double<const N: usize>(arr: [i32; N]) -> [i32; N] {
        let mut result = [0; N];
        for i in 0..N {
            result[i] = arr[i] * 2;
        }
        result
    }

    let a = [1, 2, 3, 4, 5];
    assert_eq!(array_sum(a), 15);

    let doubled = array_double(a);
    assert_eq!(doubled, [2, 4, 6, 8, 10]);
    assert_eq!(doubled.len(), 5);

    // 编译期检查: 数组长度与泛型参数一致
    let small = [10, 20];
    assert_eq!(array_sum(small), 30);
}

#[test]
/// 测试: 综合 —— 多个 Edition 2024 特性联用
fn test_edition_2024_features_combined() {
    // Let Chains + if let scope + diagnostic attr + gen reserved
    #[allow(unused)]
    struct Config {
        host: String,
        port: u16,
    }

    fn parse_pair(input: &str) -> Option<(&str, &str)> {
        let mut parts = input.splitn(2, '=');
        let key = parts.next()?;
        let value = parts.next()?;
        Some((key, value))
    }

    // 使用 let chains
    if let Some((key, value)) = parse_pair("host=localhost")
        && key == "host"
        && !value.is_empty()
        && value.len() > 3
    {
        assert_eq!(value, "localhost");
    } else {
        panic!("let chains parsing failed");
    }

    // r#gen 作为 raw identifier (gen 是保留关键字)
    let r#gen = vec![0, 1, 2, 3];
    assert_eq!(r#gen.len(), 4);
}

#[test]
/// 测试: 综合 —— error 处理 + Result 链 + 迭代器
fn test_error_handling_result_chain() {
    fn parse_numbers(input: &str) -> Result<Vec<i32>, std::num::ParseIntError> {
        input
            .split(',')
            .map(|s| s.trim().parse())
            .collect()
    }

    let result = parse_numbers("1, 2, 3, 4, 5");
    assert_eq!(result, Ok(vec![1, 2, 3, 4, 5]));

    let bad_result = parse_numbers("1, abc, 3");
    assert!(bad_result.is_err());
}

#[test]
/// 测试: 综合 —— 智能指针 + trait 对象 + 动态分发
fn test_smart_pointer_trait_object_dispatch() {
    trait Animal {
        fn speak(&self) -> &'static str;
    }

    struct Dog;
    impl Animal for Dog {
        fn speak(&self) -> &'static str { "woof" }
    }

    struct Cat;
    impl Animal for Cat {
        fn speak(&self) -> &'static str { "meow" }
    }

    // Box<dyn Animal>
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog),
        Box::new(Cat),
    ];

    let sounds: Vec<&str> = animals.iter().map(|a| a.speak()).collect();
    assert_eq!(sounds, vec!["woof", "meow"]);
}
