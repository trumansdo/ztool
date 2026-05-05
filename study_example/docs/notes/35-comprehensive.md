# 综合实战测试知识点

## 目录
1. [并发缓存：Arc<Mutex<HashMap>> + 多线程读写](#并发缓存arcmutexhashmap--多线程读写)
2. [迭代器与闭包组合：fold/scan/partition](#迭代器与闭包组合foldscanpartition)
3. [泛型链路：显式类型参数链式调用](#泛型链路显式类型参数链式调用)
4. [Option<Result<...>> 双? 组合](#optionresult-双-组合)
5. [自定义统计收集器：impl Extend + FromIterator](#自定义统计收集器impl-extend--fromiterator)
6. [生命周期与闭包交互：借用-捕获-返回](#生命周期与闭包交互借用-捕获-返回)
7. [RAII 守卫模式：Drop 析构资源管理](#raii-守卫模式drop-析构资源管理)
8. [unsafe 安全包装：内部 unsafe 暴露安全 API](#unsafe-安全包装内部-unsafe-暴露安全-api)
9. [类型状态模式：编译期状态转换约束](#类型状态模式编译期状态转换约束)
10. [const 泛型 compile-time 检查](#const-泛型-compile-time-检查)
11. [Edition 2024 多项特性联用](#edition-2024-多项特性联用)
12. [错误处理 Result 链：and_then/or_else 组合](#错误处理-result-链and_thenor_else-组合)
13. [智能指针+动态分发：Rc<RefCell<dyn Trait>>](#智能指针动态分发rcrefcelldyn-trait)
14. [避坑指南](#避坑指南)

---

## 并发缓存：Arc<Mutex<HashMap>> + 多线程读写

`Arc<Mutex<HashMap<K, V>>>` 是 Rust 并发场景中最常见的共享可变状态模式。Arc 提供多所有权，Mutex 提供内部可变性与同步，HashMap 提供键值存储。

> 三把锁扣成一个环：Arc 管活多久，Mutex 管谁先用，HashMap 管记什么。

基本并发缓存实现：

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

struct Cache {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn insert(&self, key: String, value: String) {
        let mut map = self.store.lock().unwrap();
        map.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<String> {
        let map = self.store.lock().unwrap();
        map.get(key).cloned()
    }

    fn clone_ref(&self) -> Self {
        Cache {
            store: Arc::clone(&self.store),
        }
    }
}

fn concurrent_cache_demo() {
    let cache = Cache::new();

    let mut handles = vec![];
    for i in 0..5 {
        let c = cache.clone_ref();
        let handle = thread::spawn(move || {
            let key = format!("键_{i}");
            let val = format!("值_{i}");
            c.insert(key.clone(), val);
            println!("线程 {i}: 插入 {key}");
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    assert!(cache.get("键_2").is_some());
}
```

使用 `parking_lot::Mutex` 优化性能：

```rust
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::Mutex;

struct FastCache {
    store: Arc<Mutex<HashMap<u64, Vec<u8>>>>,
}

impl FastCache {
    fn new() -> Self {
        FastCache { store: Arc::new(Mutex::new(HashMap::new())) }
    }

    fn get_or_insert<F>(&self, key: u64, factory: F) -> Vec<u8>
    where
        F: FnOnce() -> Vec<u8>,
    {
        let mut map = self.store.lock();
        map.entry(key).or_insert_with(factory).clone()
    }
}
// 注意：parking_lot 需要添加依赖
```

避免死锁——保持锁粒度最小：

```rust
fn safe_double_access(cache: &Cache, k1: &str, k2: &str) -> Option<(String, String)> {
    // 在同一作用域内获取一次锁并完成操作
    let map = cache.store.lock().unwrap();
    let v1 = map.get(k1).cloned()?;
    let v2 = map.get(k2).cloned()?;
    // MutexGuard 在此离开作用域，锁释放
    Some((v1, v2))
}
```

---

## 迭代器与闭包组合：fold/scan/partition

Rust 的迭代器适配器是可组合的函数式编程利器，`fold`、`scan`、`partition` 覆盖了聚合、带状态映射、以及二分归类。

> 迭代器是流水线，闭包是工人——fold 是最终的质检员，scan 是每道工序的记录员，partition 是分拣机。

fold —— 归约聚合：

```rust
fn fold_examples() {
    let nums = [1, 2, 3, 4, 5];

    // 求和
    let sum: i32 = nums.iter().fold(0, |acc, &x| acc + x);

    // 字符串拼接
    let s: String = nums.iter()
        .fold(String::new(), |acc, &x| acc + &x.to_string() + ",");

    // fold 构建 HashMap
    let map: std::collections::HashMap<i32, i32> = nums.iter()
        .fold(std::collections::HashMap::new(), |mut acc, &x| {
            acc.insert(x, x * x);
            acc
        });

    assert_eq!(sum, 15);
    assert_eq!(map.get(&3), Some(&9));
}
```

scan —— 带状态的迭代映射：

```rust
fn scan_example() {
    let values = [1, 2, 3, 4, 5];

    // scan 维护一个累加状态
    let cumulative: Vec<i32> = values.iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect();

    assert_eq!(cumulative, vec![1, 3, 6, 10, 15]);

    // scan 返回 None 提前终止迭代
    let until_exceed: Vec<i32> = values.iter()
        .scan(0, |state, &x| {
            *state += x;
            if *state > 8 { None } else { Some(*state) }
        })
        .collect();

    assert_eq!(until_exceed, vec![1, 3, 6]);
}
```

partition —— 二分归类：

```rust
fn partition_example() {
    let numbers = [1, -2, 3, -4, 5, -6];

    // partition 将满足条件的放在前面，不满足的放在后面
    let mut data = numbers.to_vec();
    let pivot = data.iter_mut().partition_in_place(|&n| n > 0);

    // pivot 之后全是负数
    assert!(&data[..pivot].iter().all(|&x| x > 0));
    assert!(&data[pivot..].iter().all(|&x| x <= 0));
}

fn partition_collect() {
    let items = ["苹果", "香蕉", "鳄梨", "草莓", "蓝莓"];

    let (fruits_a, others): (Vec<_>, Vec<_>) = items
        .into_iter()
        .partition(|&item| item.starts_with('苹') || item.starts_with('香') || item.starts_with('草'));

    // fruits_a 不以 'A' 开头的一类
}
```

链式组合——filter+map+fold：

```rust
fn chained_processing() {
    let raw: Vec<Option<i32>> = vec![Some(1), None, Some(2), Some(3), None];

    let result: i32 = raw.iter()
        .filter_map(|opt| *opt)          // 过滤 None，解包 Some
        .filter(|&x| x % 2 == 1)        // 只保留奇数
        .map(|x| x * 10)               // 值转换
        .fold(0, |acc, x| acc + x);    // 求和

    assert_eq!(result, 40); // (1+3)*10 = 40
}
```

---

## 泛型链路：显式类型参数链式调用

Rust 的 turbofish 语法 `::<T>` 允许在链式调用中显式指定泛型参数，解决类型推导歧义。

> 当编译器猜不出你的意图时，turbofish 就是你投下的锚。

基本 turbofish：

```rust
fn turbofish_basics() {
    // collect 需要类型提示
    let numbers: Vec<i32> = (0..5).collect();
    // 等价写法
    let numbers = (0..5).collect::<Vec<i32>>();

    // parse 的 turbofish
    let val = "42".parse::<i32>().unwrap();

    // 多层 turbofish 链
    let result: Vec<String> = (0..3)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| format!("值_{s}"))
        .collect();
}
```

链式调用中的泛型约束：

```rust
struct Processor<T> {
    value: T,
}

impl<T> Processor<T> {
    fn new(value: T) -> Self { Processor { value } }

    fn map<U, F>(self, f: F) -> Processor<U>
    where
        F: FnOnce(T) -> U,
    {
        Processor { value: f(self.value) }
    }

    fn unwrap(self) -> T { self.value }
}

fn chained_generics() {
    let result = Processor::new(10i32)
        .map::<String, _>(|x| format!("数字: {x}"))
        .map::<Vec<u8>, _>(|s| s.into_bytes())
        .unwrap();

    assert_eq!(result, vec![
        b'\xe6', b'\x95', b'\xb0', b'\xe5', b'\xad', b'\x97',
        b':', b' ', b'1', b'0'
    ]);
}
```

复杂泛型链路——从创建到收集：

```rust
use std::collections::{HashMap, HashSet};

fn complex_chain() {
    let data = vec![("a", 1), ("b", 2), ("a", 3), ("c", 4), ("b", 5)];

    // 按 key 聚合到 HashMap<String, Vec<i32>>
    let grouped: HashMap<String, Vec<i32>> = data.into_iter()
        .fold(HashMap::new(), |mut acc, (k, v)| {
            acc.entry(k.to_string()).or_default().push(v);
            acc
        });

    // 对每个 group 去重求和
    let sums: HashMap<String, i32> = grouped.into_iter()
        .map(|(k, vals)| {
            let unique: HashSet<_> = vals.into_iter().collect();
            let sum: i32 = unique.into_iter().sum();
            (k, sum)
        })
        .collect();

    assert_eq!(sums.get("a"), Some(&4)); // 1 + 3
}
```

---

## Option<Result<...>> 双? 组合

当函数调用返回 `Option<Result<T, E>>` 时，可以使用 `?` 先解包 `Option`，再解包 `Result`，或使用 `.transpose()` 互换层级。

> 双层包装就像套娃——`?` 一个一个开，`transpose()` 换个顺序开。

Option 与 Result 嵌套及互通：

```rust
fn nested_handling() -> Option<Result<i32, String>> {
    let raw = Some(Ok(42i32));
    raw
}

fn double_question() -> Result<i32, String> {
    let val = nested_handling()? ?;
    // 第一个 ? 解包 Option (如果 None 则返回 Err)
    // 第二个 ? 解包 Result (如果 Err 则返回 Err)
    Ok(val)
}

// transpose：互换 Option 和 Result 的层级
fn transpose_demo() {
    let x: Option<Result<i32, &str>> = Some(Ok(5));
    let y: Result<Option<i32>, &str> = x.transpose();
    assert_eq!(y, Ok(Some(5)));

    let a: Option<Result<i32, &str>> = Some(Err("错误"));
    let b: Result<Option<i32>, &str> = a.transpose();
    assert_eq!(b, Err("错误"));

    let c: Option<Result<i32, &str>> = None;
    let d: Result<Option<i32>, &str> = c.transpose();
    assert_eq!(d, Ok(None));
}
```

真实场景——数据库查询：

```rust
use std::collections::HashMap;

fn query_user(db: &HashMap<u32, String>, id: u32) -> Option<Result<String, String>> {
    db.get(&id).map(|name| {
        if name.is_empty() {
            Err("名称为空".to_string())
        } else {
            Ok(name.clone())
        }
    })
}

fn process_user(db: &HashMap<u32, String>, id: u32) -> Result<String, String> {
    let name: String = query_user(db, id).ok_or("用户不存在".to_string())? ?;
    Ok(format!("处理用户: {name}"))
}
```

? 在 `impl From<E>` 链上的自动转换：

```rust
fn mixed_errors() -> Result<i32, Box<dyn std::error::Error>> {
    let s: Option<&str> = Some("42");

    let val_str: &str = s.ok_or("缺失值")?;       // Option -> Result
    let val_int: i32 = val_str.parse::<i32>()?;    // Result -> Result (自动转换)

    Ok(val_int * 2)
}
```

---

## 自定义统计收集器：impl Extend + FromIterator

通过实现 `Extend` 和 `FromIterator` trait，可以让自定义类型直接参与迭代器生态，使用 `.collect()` 等方法。

> 让你的类型学会 `.collect()`，就像教会它说一口流利的迭代器方言。

实现 Extend：

```rust
#[derive(Debug, Default)]
struct Stats {
    count: u32,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
}

impl Extend<f64> for Stats {
    fn extend<T: IntoIterator<Item = f64>>(&mut self, iter: T) {
        for val in iter {
            self.count += 1;
            self.sum += val;
            self.min = Some(self.min.map_or(val, |m| m.min(val)));
            self.max = Some(self.max.map_or(val, |m| m.max(val)));
        }
    }
}

fn stats_demo() {
    let mut stats = Stats::default();
    stats.extend([1.5, 2.3, 3.7, 0.8, 5.0]);

    assert_eq!(stats.count, 5);
    assert_eq!(stats.sum, 13.3);
    assert_eq!(stats.min, Some(0.8));
    assert_eq!(stats.max, Some(5.0));
}
```

实现 FromIterator：

```rust
impl std::iter::FromIterator<f64> for Stats {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        let mut stats = Stats::default();
        stats.extend(iter);
        stats
    }
}

fn from_iterator_demo() {
    let stats: Stats = vec![10.0, 20.0, 30.0].into_iter().collect();
    assert_eq!(stats.sum, 60.0);
    assert_eq!(stats.count, 3);
}
```

---

## 生命周期与闭包交互：借用-捕获-返回

闭包捕获环境变量涉及复杂的生命周期关系：捕获可以是借用、可变借用、或移动（move），每种方式对应不同的生命周期约束。

> 闭包不是在借一个值，是在借一个时间片段——它捕获的，必须在它存活的全程不灭。

闭包捕获模式：

```rust
fn closure_capture_demo() {
    let mut s = String::from("hello");

    // 1. 不可变借用捕获
    let print = || println!("{}", s);
    print();  // &s
    println!("{}", s); // 可以：print 的借用已结束

    // 2. 可变借用捕获
    let mut append = || s.push_str(" world");
    append();
    // println!("{}", s); // 不能：s 仍在 append 的可变借用下
    drop(append); // 显式放弃可变借用
    println!("{}", s); // 现在可以

    // 3. move 捕获
    let s = String::from("owned");
    let owned_closure = move || s;
    let s_back = owned_closure();
    // println!("{}", s); // 错误：s 已被移动
}
```

返回闭包时的生命周期问题：

```rust
fn returns_closure<'a>(prefix: &'a str) -> impl Fn(&str) -> String + 'a {
    move |suffix| format!("{prefix}{suffix}")
}

fn lifetime_in_return() {
    let prefix = String::from("【");
    let closure = returns_closure(&prefix);
    let result = closure("内容");
    assert_eq!(result, "【内容");
    // drop(prefix); // 不能：closure 仍借用 prefix
}
```

闭包捕获 + 泛型参数的生命周期缠绕：

```rust
fn compose<'a, F, G, T, U, V>(f: F, g: G) -> impl Fn(T) -> V + 'a
where
    F: Fn(T) -> U + 'a,
    G: Fn(U) -> V + 'a,
{
    move |x| g(f(x))
}

fn compose_demo() {
    let double = |x: i32| x * 2;
    let to_string = |x: i32| x.to_string();
    let composed = compose(double, to_string);
    assert_eq!(composed(21), "42");
}
```

---

## RAII 守卫模式：Drop 析构资源管理

RAII（Resource Acquisition Is Initialization）是 Rust 资源管理的基石——通过 Drop trait 在变量离开作用域时自动释放资源。

> 构造时获取，析构时释放——RAII 不仅是一种模式，是 Rust 对"确定性"的承诺。

自定义文件守卫：

```rust
use std::fs::File;
use std::io::{self, Write};

struct TempFile {
    path: std::path::PathBuf,
    file: File,
}

impl TempFile {
    fn create(prefix: &str) -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!("{}_{}.tmp", prefix, std::process::id()));
        let file = File::create(&path)?;
        Ok(TempFile { path, file })
    }

    fn write_all(&mut self, data: &[u8]) -> io::Result<()> {
        self.file.write_all(data)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        println!("已清理临时文件: {}", self.path.display());
    }
}

fn temp_file_demo() -> io::Result<()> {
    let mut tmp = TempFile::create("rust_demo")?;
    tmp.write_all(b"临时数据")?;
    // tmp 在此离开作用域 -> Drop::drop 自动删除文件
    Ok(())
}
```

锁守卫模式：

```rust
use std::sync::Mutex;

struct ResourcePool {
    items: Mutex<Vec<String>>,
}

impl ResourcePool {
    fn acquire(&self) -> impl std::ops::DerefMut<Target = Vec<String>> + '_ {
        self.items.lock().unwrap()
    }

    fn with_item<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<String>) -> R,
    {
        let mut guard = self.items.lock().unwrap();
        f(&mut *guard)
        // guard 在此析构 -> 锁释放
    }
}
```

嵌套 RAII 资源的析构顺序——LIFO（后进先出）：

```rust
struct NamedDrop(&'static str);
impl Drop for NamedDrop {
    fn drop(&mut self) {
        println!("释放: {}", self.0);
    }
}

fn drop_order() {
    let _a = NamedDrop("A");
    let _b = NamedDrop("B");
    let _c = NamedDrop("C");
    // 输出：
    //   释放: C
    //   释放: B
    //   释放: A
}
```

---

## unsafe 安全包装：内部 unsafe 暴露安全 API

Rust 安全抽象的核心范式：用 unsafe 实现底层操作，但对外暴露安全接口。

> unsafe 是地下室——你在里面做电力维护，但给楼上提供的永远是安全的插座。

安全包装原始指针：

```rust
struct SafeSlice<'a> {
    ptr: *const u8,
    len: usize,
    _phantom: std::marker::PhantomData<&'a [u8]>,
}

impl<'a> SafeSlice<'a> {
    /// 从普通切片创建——安全接口
    pub fn from_slice(slice: &'a [u8]) -> Self {
        SafeSlice {
            ptr: slice.as_ptr(),
            len: slice.len(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 安全地访问指定索引的元素
    pub fn get(&self, index: usize) -> Option<u8> {
        if index < self.len {
            // unsafe 封装在安全接口内部
            Some(unsafe { *self.ptr.add(index) })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}
```

安全包装 FFI 调用：

```rust
mod ffi {
    // 假设这是 C 库函数
    pub unsafe extern "C" {
        pub fn compress_bound(input_len: usize) -> usize;
        pub fn compress(
            dest: *mut u8,
            dest_len: *mut usize,
            src: *const u8,
            src_len: usize,
        ) -> i32;
    }
}

pub struct Compressor;

impl Compressor {
    /// 安全压缩接口——调用者无需接触 unsafe
    pub fn compress(input: &[u8]) -> Vec<u8> {
        if input.is_empty() {
            return Vec::new();
        }

        // unsafe 调用封装在此
        let max_len = unsafe { ffi::compress_bound(input.len()) };
        let mut output = vec![0u8; max_len];
        let mut output_len: usize = output.len();

        let rc = unsafe {
            ffi::compress(
                output.as_mut_ptr(),
                &mut output_len,
                input.as_ptr(),
                input.len(),
            )
        };

        if rc != 0 {
            panic!("压缩失败，错误码: {rc}");
        }

        unsafe { output.set_len(output_len) };
        output
    }
}
```

unsafe 块的"安全证明"注释规范：

```rust
impl<T> SafeOpt<T> {
    /// 拿走 Option 中的值
    /// 
    /// # 安全
    /// 
    /// 仅在 `self` 满足 `Some` 条件时可调用。
    /// 调用前必须通过 `is_some()` 检查。
    pub unsafe fn take_unchecked(&mut self) -> T {
        // SAFETY: 调用者承诺 self 是 Some 变体
        match self.inner.take() {
            Some(val) => val,
            None => std::hint::unreachable_unchecked(),
        }
    }
}

struct SafeOpt<T> {
    inner: Option<T>,
}
```

---

## 类型状态模式：编译期状态转换约束

类型状态模式利用 Rust 的类型系统在编译期强制执行状态转换规则，将运行时错误转化为编译错误。

> 给状态穿上类型的外衣——错误的状态转换在编译期就被挡在门外。

TCP 连接状态机：

```rust
use std::marker::PhantomData;

struct Closed;
struct Connected;
struct Authenticated;

struct Connection<State = Closed> {
    addr: String,
    _state: PhantomData<State>,
}

impl Connection<Closed> {
    pub fn new(addr: impl Into<String>) -> Self {
        Connection {
            addr: addr.into(),
            _state: PhantomData,
        }
    }

    pub fn connect(self) -> Result<Connection<Connected>, String> {
        println!("连接 {}", self.addr);
        Ok(Connection {
            addr: self.addr,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    pub fn authenticate(self, token: &str) -> Result<Connection<Authenticated>, String> {
        if token == "secret" {
            Ok(Connection {
                addr: self.addr,
                _state: PhantomData,
            })
        } else {
            Err("认证失败".to_string())
        }
    }
}

impl Connection<Authenticated> {
    pub fn send(&self, msg: &str) {
        println!("[{}] 发送: {msg}", self.addr);
    }

    pub fn close(self) -> Connection<Closed> {
        Connection { addr: self.addr, _state: PhantomData }
    }
}

fn type_state_demo() -> Result<(), String> {
    let conn = Connection::new("127.0.0.1:8080");
    // conn.send("hi"); // 编译错误: Closed 状态没有 send 方法

    let conn = conn.connect()?;
    // conn.connect(); // 编译错误: Connected 状态没有 connect 方法

    let conn = conn.authenticate("secret")?;
    conn.send("hello world");
    let _closed = conn.close();

    Ok(())
}
```

状态模式的编译期保证：

| 状态转换 | 编译期检查 | 原运行时可能错误 |
|----------|------------|-----------------|
| Closed -> Connected | 必须调用 connect() | 忘记连接 |
| Connected -> Authenticated | 必须调用 authenticate() | 忘记认证 |
| Connected -> Closed (直接) | 不允许 | 无意义的生命周期 |
| Authenticated -> Connected (回退) | 不允许 | 状态混乱 |

---

## const 泛型 compile-time 检查

const 泛型 + const fn 可将多项运行时不变量前置到编译期验证。

> 在编译期就排除的不可能事件，永远没有机会变成运行时的惊喜。

编译期除法零检查：

```rust
struct NonZero<const N: usize> where [(); N]: {
    value: usize,
}

impl<const N: usize> NonZero<N>
where
    [(); N]:,
{
    fn new() -> Self {
        // 编译期保证 N > 0
        NonZero { value: N }
    }

    fn safe_divide(&self, dividend: usize) -> usize {
        dividend / self.value // 安全：永不为 0
    }
}

// NonZero::<0>::new(); // 编译错误：const 表达式不满足 where 条件
```

编译期类型大小断言：

```rust
struct FixedSize<const EXPECTED: usize> {
    data: [u8; EXPECTED],
}

impl<const EXPECTED: usize> FixedSize<EXPECTED> {
    const fn assert_size<T>() {
        // 编译期断言：T 的大小必须等于 EXPECTED
        if std::mem::size_of::<T>() != EXPECTED {
            panic!("类型大小不匹配"); // const panic
        }
    }
}

fn demo_size_assert() {
    // FixedSize::<4>::assert_size::<i32>();  // 通过
    // FixedSize::<8>::assert_size::<i32>();  // 编译期 panic
}
```

编译期非空数组：

```rust
fn only_non_empty<const N: usize>(arr: [i32; N]) -> [i32; N]
where
    [(); N]:, // const 泛型表达式
{
    // N > 0 由 where 子句保证
    println!("首元素: {}", arr[0]);
    arr
}
```

---

## Edition 2024 多项特性联用

将 Edition 2024 的多项新特性组合使用，能显著提升代码的安全性和表达能力。

> 新特性的价值不在于单个闪光，而在于互相辉映的组合之美。

联用示例——安全 FFI + 精确捕获 + let 链：

```rust
// Edition 2024: unsafe extern + use<> + let chains 组合
unsafe extern "C" {
    fn get_version() -> u32;
    fn get_name(buf: *mut u8, len: usize) -> i32;
}

fn safe_get_version() -> u32 {
    unsafe { get_version() }
}

fn safe_get_name() -> Option<String> {
    let mut buf = vec![0u8; 256];
    let len = unsafe { get_name(buf.as_mut_ptr(), buf.len()) };

    if let Ok(len_usize) = usize::try_from(len)
        && len_usize > 0
        && len_usize <= buf.len()
    {
        buf.truncate(len_usize);
        String::from_utf8(buf).ok()
    } else {
        None
    }
}

// 精确捕获 + impl Trait
fn build_formatter() -> impl Fn(&str) -> String + use<> {
    |input| format!("[{}]", input.to_uppercase())
}
```

Edition 2024 if let 作用域 + Mutex 安全：

```rust
use std::sync::Mutex;

fn edition_2024_lock_pattern() {
    let shared = Mutex::new(vec![1, 2, 3]);

    // Edition 2024: 锁在 if 块结束时自动释放
    if let Some(mut data) = shared.lock().ok() {
        data.push(4);
    } // 锁在此释放

    // 无需担心死锁，可立即再次获取
    let len = shared.lock().unwrap().len();
    assert_eq!(len, 4);
}
```

---

## 错误处理 Result 链：and_then/or_else 组合

Rust 提供丰富的方法组合 Result 操作：`and_then`、`or_else`、`map_err`、`unwrap_or` 等，形成函数式的错误处理流水线。

> 错误处理不是分岔路口，而是一条有向图的流水线——and_then 搭桥，or_else 建备路。

and_then 链：成功时才继续：

```rust
fn parse_and_process(input: &str) -> Result<String, String> {
    input
        .parse::<i32>()
        .map_err(|e| format!("解析失败: {e}"))
        .and_then(|n| {
            if n < 0 {
                Err("负数不允许".to_string())
            } else {
                Ok(n * 2)
            }
        })
        .and_then(|n| Ok(format!("结果: {n}")))
}

fn and_then_demo() {
    assert_eq!(parse_and_process("21"), Ok("结果: 42".to_string()));
    assert!(parse_and_process("-5").is_err());
    assert!(parse_and_process("abc").is_err());
}
```

or_else 链：失败时提供备选方案：

```rust
fn multi_fallback(key: &str) -> Result<String, String> {
    try_primary(key)
        .or_else(|e| {
            println!("主源失败: {e}，尝试备源");
            try_secondary(key)
        })
        .or_else(|e| {
            println!("备源失败: {e}，使用默认值");
            Ok("默认值".to_string())
        })
}

fn try_primary(key: &str) -> Result<String, String> {
    if key == "admin" { Ok("root".to_string()) }
    else { Err("未找到".to_string()) }
}

fn try_secondary(key: &str) -> Result<String, String> {
    if key == "user" { Ok("guest".to_string()) }
    else { Err("也未找到".to_string()) }
}
```

map / map_err / unwrap_or 组合：

```rust
fn pipeline_demo(input: Option<&str>) -> Option<String> {
    input
        .map(|s| s.trim())                         // Option -> Option
        .filter(|s| !s.is_empty())                 // 过滤空字符串
        .map(|s| s.parse::<u32>().ok())            // Option<Result> -> Option<Option>
        .flatten()                                 // Option<Option> -> Option
        .map(|n: u32| format!("结果是 {}", n * n)) // 平方
}
```

---

## 智能指针+动态分发：Rc<RefCell<dyn Trait>>

`Rc<RefCell<dyn Trait>>` 结合了多所有权共享、运行时内部可变性、以及动态分发，常用于 GUI、事件系统、插件架构等场景。

> Rc 是共享的生命线，RefCell 是内部的可变门，dyn Trait 是统一的接口面具——三者合一，构成了 Rust 最灵活的运行时结构。

基本组件组装：

```rust
use std::rc::Rc;
use std::cell::RefCell;

trait Handler {
    fn on_event(&self, event: &str);
    fn name(&self) -> &'static str;
}

struct LogHandler { id: u32 }
impl Handler for LogHandler {
    fn on_event(&self, event: &str) {
        println!("[日志{}] {}", self.id, event);
    }
    fn name(&self) -> &'static str { "日志处理器" }
}

struct AlertHandler { threshold: u32 }
impl Handler for AlertHandler {
    fn on_event(&self, event: &str) {
        if event.len() as u32 > self.threshold {
            println!("告警: {event}");
        }
    }
    fn name(&self) -> &'static str { "告警处理器" }
}

fn dynamic_dispatch_demo() {
    // 类型擦除容器
    let handlers: Vec<Rc<RefCell<dyn Handler>>> = vec![
        Rc::new(RefCell::new(LogHandler { id: 1 })),
        Rc::new(RefCell::new(AlertHandler { threshold: 10 })),
    ];

    for handler in &handlers {
        println!("处理器: {}", handler.borrow().name());
        handler.borrow_mut().on_event("测试事件");
    }
}
```

组件间共享引用 + 运行时注册：

```rust
use std::rc::Rc;
use std::cell::RefCell;

trait Observer {
    fn update(&mut self, message: &str);
}

struct Subject {
    observers: Vec<Rc<RefCell<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self { Subject { observers: vec![] } }

    fn attach(&mut self, observer: Rc<RefCell<dyn Observer>>) {
        self.observers.push(observer);
    }

    fn notify(&self, message: &str) {
        for obs in &self.observers {
            obs.borrow_mut().update(message);
        }
    }
}

struct ConsoleObserver { name: String }
impl Observer for ConsoleObserver {
    fn update(&mut self, msg: &str) {
        println!("[{}] 收到: {msg}", self.name);
    }
}

fn observer_pattern() {
    let obs1 = Rc::new(RefCell::new(ConsoleObserver { name: "A".into() }));
    let obs2 = Rc::new(RefCell::new(ConsoleObserver { name: "B".into() }));

    let mut subject = Subject::new();
    subject.attach(obs1.clone());
    subject.attach(obs2.clone());

    subject.notify("事件发生");
    // obs1 和 obs2 仍可独立使用（共享所有权）
    obs1.borrow_mut().update("直接消息");
}

impl ConsoleObserver {
    fn new(name: impl Into<String>) -> Self {
        ConsoleObserver { name: name.into() }
    }
}
```

RefCell 的运行时借用规则——违反时 panic：

```rust
fn refcell_panic_demo() {
    let cell: Rc<RefCell<dyn Handler>> = Rc::new(RefCell::new(LogHandler { id: 1 }));

    let borrow1 = cell.borrow(); // 不可变借用
    // let borrow2 = cell.borrow_mut(); // panic: 已有不可变借用
    drop(borrow1); // 释放后可变借用才可用

    let mut borrow2 = cell.borrow_mut(); // 现在安全
    borrow2.on_event("测试");
}
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| Arc<Mutex<HashMap>> 锁粒度过大 | 所有访问都争用同一把锁，高并发时成为瓶颈 | 使用分片锁 `HashMap<K, Arc<Mutex<V>>>` 或 `dashmap` |
| 在持有 MutexGuard 时调用闭包 | 闭包可能尝试再次获取同一把锁导致死锁 | 先复制数据，释放锁后再调用闭包 |
| scan 返回 None 提前终止被忽略 | 一旦 scan 返回 None，后续元素全部丢弃 | 如需标记而非删除，仍返回 Some，用特殊值表达 |
| 闭包 move 后原变量不可用 | move 闭包获取所有权 | 仅在需要所有权时使用 move；否则让闭包借用 |
| Drop 析构中 panic | 析构中 panic 导致程序 abort（双 panic） | Drop 实现中绝不 panic，使用 `let _ = fallible_op()` |
| unsafe 块中遗漏安全证明注释 | 代码审查者无法判断 unsafe 的合法性 | 每个 unsafe 块上方添加 `// SAFETY:` 注释 |
| 类型状态模式转换方法消耗 self | 每次转换销毁旧状态，不适用于多次操作同一对象 | 使用 `&self` 方法 + `Cell` 或 `RefCell` 内部状态 |
| const 泛型表达式超过编译器限制 | 复杂编译期计算可能触发 "unconstrained generic constant" | 拆分为多个 const fn，逐步求值 |
| `or_else` 中不返回 `Result` | `or_else` 要求闭包返回 `Result<T, E>` | 检查返回值类型，确保一致 |
| Rc<RefCell<dyn Trait>> 循环引用 | Rc 不能自动处理循环，导致内存泄漏 | 对可能形成循环的结构使用 `Weak<RefCell<dyn Trait>>` |

---

> **详见测试**: `tests/rust_features/35_comprehensive_tests.rs`
