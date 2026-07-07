// ============================================================================
// 3.2 泛型与静态分发进阶 — 技巧与模式
// ============================================================================
//
// 静态分发(Static Dispatch) vs 动态分发(Dynamic Dispatch)
//
//   静态分发: 编译期确定具体类型, 零运行时开销, 可内联优化
//   动态分发: 通过虚表(vtable)在运行时确定方法实现, 有间接调用开销
//
// 本文件聚焦静态分发的各种技巧模式, 展示如何充分利用编译期类型系统。

use std::fmt::{Debug, Display};

// ============================================================================
// 技巧1: 零大小类型(ZST)状态标记 — 编译期状态机
// ============================================================================

#[test]
fn test_zst_state_machine() {
    // 技巧: 用 ZST 作为 TCP 状态标记, 泛型参数完全在编译期消解
    // 价值: 零运行时开销的状态转移验证

    struct TcpClosed;
    struct TcpListen;
    struct TcpEstablished;

    struct Connection<State> {
        id: u64,
        _state: std::marker::PhantomData<State>,
    }

    impl Connection<TcpClosed> {
        fn new(id: u64) -> Self {
            Self { id, _state: std::marker::PhantomData }
        }

        fn listen(self) -> Connection<TcpListen> {
            Connection { id: self.id, _state: std::marker::PhantomData }
        }
    }

    impl Connection<TcpListen> {
        fn accept(self) -> Connection<TcpEstablished> {
            Connection { id: self.id, _state: std::marker::PhantomData }
        }
    }

    impl Connection<TcpEstablished> {
        fn send(&self, data: &str) -> String {
            format!("conn[{}] sent: {}", self.id, data)
        }

        fn close(self) -> Connection<TcpClosed> {
            Connection { id: self.id, _state: std::marker::PhantomData }
        }
    }

    let conn = Connection::new(42);
    let conn = conn.listen();
    let conn = conn.accept();
    assert_eq!(conn.send("hello"), "conn[42] sent: hello");
    let _conn = conn.close();

    // 内存验证: Connection 只存储了 id(u64) = 8 bytes
    assert_eq!(8, std::mem::size_of::<Connection<TcpEstablished>>());
}

// ============================================================================
// 技巧2: 编译期策略模式 — trait 约束 + 泛型参数零成本切换
// ============================================================================

trait Compress {
    fn compress(data: &[u8]) -> Vec<u8>;
    fn algorithm() -> &'static str;
}

struct GzipCompressor;
impl Compress for GzipCompressor {
    fn compress(data: &[u8]) -> Vec<u8> {
        let mut out = data.to_vec();
        out.push(b'G');
        out
    }
    fn algorithm() -> &'static str { "gzip" }
}

struct SnappyCompressor;
impl Compress for SnappyCompressor {
    fn compress(data: &[u8]) -> Vec<u8> {
        let mut out = data.to_vec();
        out.push(b'S');
        out
    }
    fn algorithm() -> &'static str { "snappy" }
}

struct Compressor<T: Compress> {
    _strategy: std::marker::PhantomData<T>,
}

impl<T: Compress> Compressor<T> {
    fn new() -> Self { Self { _strategy: std::marker::PhantomData } }

    fn compress(&self, data: &[u8]) -> Vec<u8> {
        T::compress(data)
    }

    fn algorithm(&self) -> &'static str {
        T::algorithm()
    }
}

#[test]
fn test_static_strategy() {
    let gz = Compressor::<GzipCompressor>::new();
    assert_eq!(gz.algorithm(), "gzip");
    assert_eq!(gz.compress(b"hi"), b"hiG");

    let sn = Compressor::<SnappyCompressor>::new();
    assert_eq!(sn.algorithm(), "snappy");
    assert_eq!(sn.compress(b"hi"), b"hiS");

    // 两种 Compressor 是不同类型, 不能放在同一个 Vec 中
    // 但每个实例都是 ZST + 调用时完全内联
    assert_eq!(0, std::mem::size_of::<Compressor<GzipCompressor>>());
}

// ============================================================================
// 技巧3: 编译期运算 — 类型级自然数
// ============================================================================

struct Succ<T>(std::marker::PhantomData<T>);
struct Zero;

trait NatValue {
    const VALUE: usize;
}
impl NatValue for Zero {
    const VALUE: usize = 0;
}
impl<N: NatValue> NatValue for Succ<N> {
    const VALUE: usize = 1 + N::VALUE;
}

#[test]
fn test_type_level_nat() {
    // Zero = 0, Succ<Zero> = 1, Succ<Succ<Zero>> = 2
    assert_eq!(<Zero>::VALUE, 0);
    assert_eq!(<Succ<Zero>>::VALUE, 1);
    assert_eq!(<Succ<Succ<Zero>>>::VALUE, 2);
    assert_eq!(<Succ<Succ<Succ<Zero>>>>::VALUE, 3);
}

// ============================================================================
// 技巧4: 编译期维度/单位检查 — 防止量纲错误
// ============================================================================

#[test]
fn test_unit_dimension_safety() {
    // 技巧: 用泛型标签防止混用不同物理量的值
    // 价值: 编译期防止 米+秒 这种量纲错误

    struct Meters;
    struct Seconds;
    struct MetersPerSecond;

    struct Quantity<T> {
        value: f64,
        _unit: std::marker::PhantomData<T>,
    }

    impl<T> Quantity<T> {
        fn new(value: f64) -> Self {
            Self { value, _unit: std::marker::PhantomData }
        }
    }

    impl Quantity<Meters> {
        fn div_seconds(self, rhs: Quantity<Seconds>) -> Quantity<MetersPerSecond> {
            Quantity::new(self.value / rhs.value)
        }
    }

    let distance = Quantity::<Meters>::new(100.0);
    let time = Quantity::<Seconds>::new(9.58);

    let speed: Quantity<MetersPerSecond> = distance.div_seconds(time);
    assert!((speed.value - 10.438).abs() < 0.5);

    // 类型不同, 即使内部都是 f64 也无法直接比较
    // assert_eq!(Quantity::<Meters>::new(5.0), Quantity::<Seconds>::new(5.0)); // 编译错误
}

// ============================================================================
// 技巧5: 编译期强制数组大小 — trait 约束模拟编译期整数约束
// ============================================================================

trait NonEmpty {}
impl NonEmpty for [u8; 1] {}
impl NonEmpty for [u8; 2] {}
impl NonEmpty for [u8; 4] {}
impl NonEmpty for [u8; 8] {}
impl NonEmpty for [u8; 16] {}
impl NonEmpty for [u8; 32] {}

#[test]
fn test_compile_time_size_constraint() {
    // 技巧: 为特定大小的数组实现 trait, 编译期限制可用的 N 值
    // 价值: 调用方不合法的大小会在编译期被拒绝

    fn process<const N: usize>(arr: [u8; N])
    where
        [u8; N]: NonEmpty,
    {
        // 编译期保证 N 一定是 1/2/4/8/16/32
        let _ = arr[0]; // u8 天然 <= 255, 仅展示编译期约束生效
    }

    process([0u8; 8]);      // OK
    process([0u8; 16]);     // OK
    // process([0u8; 3]);   // 编译错误: the trait `NonEmpty` is not implemented for `[u8; 3]`
}

// ============================================================================
// 技巧6: 条件 trait 实现 — 门控泛型实现
// ============================================================================

struct Container<T> {
    data: Vec<T>,
}

trait MaxValue {
    type Output;
    fn max(&self) -> Option<&Self::Output>;
}

impl<T: Ord> MaxValue for Container<T> {
    type Output = T;

    fn max(&self) -> Option<&T> {
        self.data.iter().max()
    }
}

#[test]
fn test_conditional_trait_impl() {
    // 技巧: 根据类型参数有条件地实现 trait
    // 价值: 只在类型满足特定条件时才暴露接口

    let c = Container { data: vec![3i32, 1, 4, 1, 5] };
    assert_eq!(c.max(), Some(&5));

    // 以下无法编译, 因为 f64 不是 Ord
    // let cf = Container { data: vec![3.0f64, 1.0] };
    // cf.max();
}

// ============================================================================
// 技巧7: 关联类型约束链 — 通过 where 建立类型之间的关系
// ============================================================================

#[test]
fn test_associated_type_chain() {
    // 技巧: 利用关联类型 + where 子句在编译期建立复杂的类型关系
    // 价值: 描述"迭代器的元素类型必须实现 Display"这类跨类型约束

    trait DataSource {
        type Item;
        type Iter: Iterator<Item = Self::Item>;
        fn iter(&self) -> Self::Iter;
    }

    struct VecSource<T>(Vec<T>);

    impl<T: Clone> DataSource for VecSource<T> {
        type Item = T;
        type Iter = std::vec::IntoIter<T>;
        fn iter(&self) -> Self::Iter {
            self.0.clone().into_iter()
        }
    }

    // 使用 DataSource 的泛型函数, 额外要求 Item 可 Debug
    fn debug_all<S: DataSource>(source: &S)
    where
        S::Item: Debug,
    {
        for item in source.iter() {
            let _ = format!("{:?}", item);
        }
    }

    let vs = VecSource(vec![1, 2, 3]);
    debug_all(&vs);
}

// ============================================================================
// 技巧8: 孤儿规则绕过 — newtype + Deref 委托
// ============================================================================

mod orphan_trick {
    use std::fmt::Display;
    use std::ops::Deref;

    // 模拟外部 crate 的类型
    #[derive(Debug, PartialEq)]
    pub struct ExternalVec<T>(pub Vec<T>);

    pub trait ToJson {
        fn to_json(&self) -> String;
    }

    // newtype 包装外部类型, 实现自定义 trait
    pub struct JsonVec<T: Display>(pub ExternalVec<T>);

    impl<T: Display> ToJson for JsonVec<T> {
        fn to_json(&self) -> String {
            let items: Vec<String> = self.0.0.iter().map(|v| format!("\"{}\"", v)).collect();
            format!("[{}]", items.join(", "))
        }
    }

    impl<T: Display> Deref for JsonVec<T> {
        type Target = ExternalVec<T>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[test]
fn test_newtype_orphan_bypass() {
    // 技巧: newtype + Deref: 既能加新 trait, 又能无缝使用原类型方法
    // 价值: 绕过孤儿规则为外部类型添加行为

    use orphan_trick::{ExternalVec, JsonVec, ToJson};

    let jv = JsonVec(ExternalVec(vec!["a".to_string(), "b".to_string()]));
    assert_eq!(jv.to_json(), r#"["a", "b"]"#);

    // 通过 Deref 仍能访问 ExternalVec 的方法
    assert_eq!(jv.0, ExternalVec(vec!["a".to_string(), "b".to_string()]));
}

// ============================================================================
// 技巧9: 编译期 Tag 类型 — 消除布尔参数和魔法数字
// ============================================================================

#[test]
fn test_tag_types() {
    // 技巧: 用 ZST tag 替代 bool/枚举 控制行为, 在编译期消解分支
    // 价值: 消除 "fn foo(flag: bool)" 的运行时分支, 且自文档化

    struct Asc;
    struct Desc;

    trait SortOrder {
        fn should_swap<T: PartialOrd>(a: &T, b: &T) -> bool;
    }
    impl SortOrder for Asc {
        fn should_swap<T: PartialOrd>(a: &T, b: &T) -> bool { a > b }
    }
    impl SortOrder for Desc {
        fn should_swap<T: PartialOrd>(a: &T, b: &T) -> bool { a < b }
    }

    struct Sorter<Order>(std::marker::PhantomData<Order>);

    impl<Order: SortOrder> Sorter<Order> {
        fn sort<T: PartialOrd>(&self, data: &mut [T]) {
            for i in 0..data.len() {
                for j in i + 1..data.len() {
                    if Order::should_swap(&data[i], &data[j]) {
                        data.swap(i, j);
                    }
                }
            }
        }
    }

    let mut data = vec![3, 1, 4, 1, 5];
    Sorter::<Asc>(std::marker::PhantomData).sort(&mut data);
    assert_eq!(data, vec![1, 1, 3, 4, 5]);

    let mut data = vec![3, 1, 4, 1, 5];
    Sorter::<Desc>(std::marker::PhantomData).sort(&mut data);
    assert_eq!(data, vec![5, 4, 3, 1, 1]);
}

// ============================================================================
// 技巧10: Auto-trait 利用 — Send/Sync 作为静态能力约束
// ============================================================================

#[test]
fn test_auto_trait_as_capability() {
    // 技巧: 利用 Send/Sync/Unpin 等自动 trait 做编译期能力检查
    // 价值: 多线程安全由编译器保证, 而非文档约定

    fn spawn_task<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        std::thread::spawn(f).join().unwrap();
    }

    spawn_task(|| {
        let _ = 42;
    });

    // 以下会在编译期被拒绝:
    // let local = vec![1, 2, 3];
    // spawn_task(move || { let _ = &local; }); // &Vec<i32> 不是 Send
}

// ============================================================================
// 技巧11: 无分配链式组合 — 迭代器适配器链的静态组合
// ============================================================================

#[test]
fn test_iterator_static_composition() {
    // 技巧: Rust 迭代器适配器链完全静态分发, 零堆分配
    // 价值: map().filter().take().collect() 整个计算图在编译期融合成一段线性代码

    let result: Vec<i32> = (0..100)
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .take(5)
        .collect();

    assert_eq!(result, vec![0, 4, 16, 36, 64]);

    // 类型巨大, 但运行时完全内联 — 这是静态分发的威力
    let _chain = (0..100).filter(|x: &i32| *x % 2 == 0).map(|x| x * x);
    // _chain 的类型: Map<Filter<Range<i32>, closure>, closure>
    // 编译器知道所有具体类型, 可以激进内联

    // 内存: 迭代器栈上分配, 无堆开销
    let chain = (0..5).map(|x| x * 2).filter(|x| x % 3 == 0);
    assert!(std::mem::size_of_val(&chain) < 100);
}

// ============================================================================
// 技巧12: blanket impl + 默认方法 — 开箱即用的扩展能力
// ============================================================================

trait Describe {
    fn describe(&self) -> String;
    fn loud_describe(&self) -> String {
        self.describe().to_uppercase() + "!"
    }
}

// blanket impl: 所有实现 Display 的类型自动获得 Describe
impl<T: Display> Describe for T {
    fn describe(&self) -> String {
        format!("{}", self)
    }
}

#[test]
fn test_blanket_with_default_methods() {
    // 技巧: blanket impl + 默认方法体的组合, 提供开箱即用的扩展能力
    // 价值: 任意 Display 类型自动获得 describe() 和 loud_describe()

    assert_eq!(42.describe(), "42");
    assert_eq!(42.loud_describe(), "42!");

    assert_eq!("hello".describe(), "hello");
    assert_eq!("hello".loud_describe(), "HELLO!");

    assert_eq!(3.14.describe(), "3.14");
}

// ============================================================================
// 技巧13: 编译期 builder 验证 — 通过类型状态防止遗忘调用
// ============================================================================

#[test]
fn test_type_state_builder() {
    // 技巧: 将 builder 的每个必需步骤编码进类型参数
    // 价值: 编译期保证 build() 之前所有字段都已设置

    struct Ready;
    struct WithUrl;

    struct ClientBuilder<State> {
        url: String,
        timeout_secs: u64,
        _state: std::marker::PhantomData<State>,
    }

    impl ClientBuilder<Ready> {
        fn new() -> Self {
            Self { url: String::new(), timeout_secs: 30, _state: std::marker::PhantomData }
        }
    }

    impl ClientBuilder<Ready> {
        fn url(mut self, url: impl Into<String>) -> ClientBuilder<WithUrl> {
            self.url = url.into();
            ClientBuilder { url: self.url, timeout_secs: self.timeout_secs, _state: std::marker::PhantomData }
        }
    }

    impl<State> ClientBuilder<State> {
        fn timeout(mut self, secs: u64) -> Self {
            self.timeout_secs = secs;
            self
        }
    }

    impl ClientBuilder<WithUrl> {
        fn build(self) -> String {
            format!("client -> {} (timeout: {}s)", self.url, self.timeout_secs)
        }
    }

    let client = ClientBuilder::<Ready>::new()
        .timeout(10)
        .url("https://example.com")
        .build();

    assert_eq!(client, "client -> https://example.com (timeout: 10s)");

    // ClientBuilder::<Ready>::new().build(); // 编译错误: build 只在 WithUrl 状态可用
}
