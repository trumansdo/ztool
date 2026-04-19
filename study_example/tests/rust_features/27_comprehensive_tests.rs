use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
/// 测试: 综合实战 - 缓存 + Let Chains + extract_if 组合
fn test_cache_with_let_chains_and_extract_if() {
    // 综合: 泛型结构体 + Arc<Mutex<HashMap>> + Let Chains + extract_if
    // 避坑: MutexGuard 作用域要小, 避免死锁; extract_if 消费后才真正移除元素
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

    // Let Chains (1.88+): 同时解构和条件判断
    if let Some(val) = cache.get(&1)
        && val > 0
    {
        panic!("Should not find value yet");
    }

    cache.insert(1, 42);

    // extract_if: 提取满足条件的元素
    let mut map = cache.data.lock().unwrap();
    let extracted: Vec<_> = map
        .extract_if(|_k, _v| true)
        .collect();
    assert_eq!(extracted.len(), 1);
}

#[test]
/// 测试: 综合实战 - trait 继承 + upcasting + 动态分发
fn test_trait_upcasting_integration() {
    // 综合: trait 继承 + trait upcasting + 动态分发
    // 避坑: upcast 后只能调用超 trait 方法; 向下转型需用 Any::downcast_ref
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
    // 综合: async 运行时 + extract_if 在异步上下文中使用
    // 避坑: extract_if 本身不是 async, 在 async 块中同步执行; 大数据量考虑用 spawn_blocking
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
