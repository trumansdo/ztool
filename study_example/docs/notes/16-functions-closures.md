# Rust 函数与闭包：可调用世界的完整拼图

## 一、函数定义与返回值

> **金句引用**："函数是 Rust 的第一公民——它不仅是代码，更是类型，可存储、可传递、可返回。"

### 1.1 基本定义

```rust
fn add(x: i32, y: i32) -> i32 { x + y }

// 最后一个表达式直接作为返回值（无分号）
fn multiply(a: i32, b: i32) -> i32 {
    a * b  // 表达式 → 返回值
}

// 显式 return 提前退出
fn divide_option(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { return None; }
    Some(a / b)
}
```

### 1.2 发散函数（Never Type）

> **金句引用**："`!` 是永不回头类型——`exit`、`loop`、`panic!` 都是 `!` 的化身。"

```rust
// 发散函数：永不返回的函数
fn forever() -> ! {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn exit_with(code: i32) -> ! {
    std::process::exit(code);
}

// ! 类型可以强制转换为任意类型
fn read_or_exit() -> String {
    match std::fs::read_to_string("config.toml") {
        Ok(s) => s,
        Err(_) => exit_with(1),  // ! 自动转换为 String
    }
}

// 用途：用于 match 分支、unwrap/expect 等
let x: u32 = match Some(42) {
    Some(v) => v,
    None    => panic!("不可能"),  // Never 类型
};

// loop 也是 ! 类型
let result: i32 = loop {
    break 42;  // break 带值 → 整个 loop 求值为 42
};
```

### 1.3 函数项类型 vs 函数指针

```rust
fn add_one(x: i32) -> i32 { x + 1 }

// 函数项类型：零大小类型（ZST），每个函数对应唯一类型
let f_item = add_one;  // 类型: fn(i32) -> i32 {add_one}
println!("sizeof fn_item: {}", std::mem::size_of_val(&f_item)); // 0

// 函数指针：8字节胖指针
let f_ptr: fn(i32) -> i32 = add_one;  // 隐式强制转换为指针
println!("sizeof fn_ptr: {}", std::mem::size_of_val(&f_ptr));  // 8

// 函数指针不捕获环境，闭包却可以
let nums = vec![1, 2, 3];
let ptr: fn(i32) -> bool = |x| x > 0;  // ✅ 闭包未捕获环境 → 可转为 fn 指针
let closure = || &nums;                 // ❌ 捕获了 nums → 不能转为 fn 指针
```

---

## 二、const fn：编译期求值

> **金句引用**："const fn 是函数的分身——编译时算出常数，运行时如同常量。"

```rust
// const fn：在编译期为常量上下文求值
const fn square(x: i32) -> i32 {
    x * x
}

const ANSWER: i32 = square(7);     // 编译期计算 49
let runtime = square(10);          // 运行时调用也可

// const fn 限制（Rust 1.85）：
// ✅ 允许：基本运算、循环、条件判断、整数操作
// ❌ 禁止：for 循环（需用 while）、堆分配、I/O、panic

const fn factorial(n: u64) -> u64 {
    let mut result = 1;
    let mut i = 2;
    while i <= n {
        result *= i;
        i += 1;
    }
    result
}

const FACT5: u64 = factorial(5);  // 编译期求值得 120
```

---

## 三、函数作为参数/返回值（5种写法）

```rust
use std::fmt::Display;

// 写法1: 函数指针 fn 类型（不捕获环境）
fn apply_ptr(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

// 写法2: 泛型 Fn trait（零成本静态分发）
fn apply_generic<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

// 写法3: Box<dyn Fn>（动态分发，允许异构存储）
fn apply_boxed(f: Box<dyn Fn(i32) -> i32>, x: i32) -> i32 {
    f(x)
}

// 写法4: where F: Fn 子句
fn apply_where<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32,
{
    f(x)
}

// 写法5: impl Fn（语法糖，等价于写法2）
fn apply_impl(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

// 返回闭包
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y  // move 把 x 的所有权转移到闭包
}

fn make_boxed_fn() -> Box<dyn Fn(i32) -> i32> {
    Box::new(|x| x * 2)
}
```

### 选型对照表

| 写法 | 分发 | 开销 | 捕获环境 | 异构容器 | 适用场景 |
|------|------|------|----------|----------|----------|
| `fn(T) -> U` | 静态 | 8字节指针 | ❌ | ✅ | C FFI、简单函数回调 |
| `impl Fn(T) -> U` | 静态 | 0字节(ZST) | ✅ | ❌ | 一次性传递零开销 |
| `Box<dyn Fn(T) -> U>` | 动态 | 16字节+堆 | ✅ | ✅ | 多个不同闭包存储 |
| `where F: Fn(T) -> U` | 静态 | 0字节(ZST) | ✅ | ❌ | 多约束场景 |
| `F: Fn(T) -> U`(泛型) | 静态 | 0字节(ZST) | ✅ | ❌ | 标准泛型写法 |

---

## 四、闭包：捕获环境的匿名函数

> **金句引用**："闭包是带环境的函数——它偷取作用域、记忆诞生地、携带上下文。"

### 4.1 三种 Fn Trait 自动推导

闭包实现的 trait 由**捕获方式和对其调用的方式**决定：

| Trait | 调用 self | 捕获方式 | 可调用次数 | 自动推导条件 |
|-------|-----------|----------|-----------|-------------|
| `FnOnce` | 消耗 self | 转移所有权 | 一次 | 所有闭包（基础trait） |
| `FnMut` | `&mut self` | 可变引用 | 多次 | 不转移捕获值且不消费self |
| `Fn` | `&self` | 不可变引用 | 多次 | 仅不可变访问捕获值 |

```rust
// FnOnce: 转移所有权（String 被移动）
let owned = String::from("消耗");
let consume = || drop(owned);  // FnOnce：转移 owned 的所有权
consume();
// consume();  // 编译错误！超过 FnOnce 调用次数

// FnMut: 可变引用
let mut count = 0;
let mut increment = || {
    count += 1;        // &mut count
    count
};
assert_eq!(increment(), 1);
assert_eq!(increment(), 2); // 可多次调用（FnMut）
assert_eq!(increment(), 3);

// Fn: 不可变引用
let num = 42;
let read_only = || num;  // &num，只读
assert_eq!(read_only(), 42);
assert_eq!(read_only(), 42); // 可反复调用（Fn）
```

**推导优先级**：编译器先尝试 `Fn`，不成则 `FnMut`，若转移了所有权则退到 `FnOnce`。

### 4.2 逐字段捕获细化

```rust
// Rust 2021+：闭包只捕获实际使用的字段，而非整个结构体
struct Person {
    name: String,
    age: u32,
}

let person = Person { name: String::from("Alice"), age: 30 };

let age_closure = || println!("年龄: {}", person.age); // 仅捕获 person.age
let name_closure = || println!("姓名: {}", person.name); // 仅捕获 person.name
// 两个闭包不会冲突（2018版中会冲突因为都捕获整个 Person）
```

---

## 五、move 关键字：强制所有权转移

```rust
// move 将环境变量的所有权转移到闭包中
let message = String::from("闭包拥有");
let print_msg = move || println!("{}", message);
// message 已移动，外部不可再用

// move + 线程：跨线程捕获所有权的标准模式
use std::thread;

let data = vec![1, 2, 3];
let handle = thread::spawn(move || {
    // data 所有权归此闭包 → 此线程
    println!("线程: {:?}", data);
});
handle.join().unwrap();
// 注意：move 后闭包变为 FnOnce（如果捕获了非 Copy 类型）
// 如果捕获的对象有 Copy 行为，可以是 FnMut 甚至 Fn

let x = 42;          // i32 是 Copy
let only_copy = move || x; // 依然是 Fn，可多次调用
let a = only_copy();
let b = only_copy();
assert_eq!(a, b);
```

---

## 六、非转义闭包优化

> **金句引用**："非转义闭包是编译器眼中的临时工——栈上分配，不堆分配，省 Box 的钱。"

```rust
// 编译器保证闭包不逃离当前函数调用栈时进行优化
fn process_non_escaping(nums: &[i32]) -> i32 {
    let multiplier = 10;
    // 闭包不会超出当前函数作用域 → 栈上按需分配
    nums.iter().map(|&x| x * multiplier).sum()
}

// 闭包转义时（如返回）必须有所有权路径
fn escape_closure() -> Box<dyn Fn(i32) -> i32> {
    let base = 100;
    Box::new(move |x| x + base)  // move + Box 确保数据转义
}
```

---

## 七、高阶函数组合模式

```rust
// 经典 map + and_then + filter 管道
fn process_numbers(nums: &[i32]) -> Option<Vec<i32>> {
    Some(
        nums.iter()
            .map(|x| x * 2)                        // 高阶: map
            .filter(|x| x % 10 == 0)               // 高阶: filter
            .map(|x| x / 10)
            .collect()
    )
}

// fold 构建复杂结构
let result = (1..=5).fold(
    (0, 1),        // 初始累加器：(sum, product)
    |(sum, prod), x| (sum + x, prod * x)
);
assert_eq!(result, (15, 120));

// reduce 无初值
let product: Option<i32> = vec![2, 3, 4].into_iter().reduce(|a, b| a * b);
assert_eq!(product, Some(24));
```

---

## 八、Builder 模式：消费 self 的链式构造

> **金句引用**："Builder 是构造函数的反模式——每步消费旧的，返回新的，链式成最终对象。"

```rust
#[derive(Debug)]
struct Server {
    host: String,
    port: u16,
    timeout: u64,
    tls: bool,
}

#[derive(Default)]
struct ServerBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u64>,
    tls: bool,
}

impl ServerBuilder {
    fn new() -> Self { ServerBuilder::default() }

    fn host(mut self, host: &str) -> Self {
        self.host = Some(host.into());
        self           // 返回 self，链式调用
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = Some(seconds);
        self
    }

    fn enable_tls(mut self) -> Self {
        self.tls = true;
        self
    }

    fn build(self) -> Result<Server, &'static str> {
        Ok(Server {
            host: self.host.ok_or("host 未设置")?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
            tls: self.tls,
        })
    }
}

let server = ServerBuilder::new()
    .host("localhost")
    .port(9090)
    .enable_tls()
    .build()
    .unwrap();
```

---

## 九、RAII 守卫模式：闭包配合 Drop

> **金句引用**："守卫是自动化的清理——创建时获取资源，Drop 时轻拭而去，闭包注入行为。"

```rust
// 自定义守卫：进入时执行 A，离开时执行 B
struct ScopeGuard<F: FnOnce()> {
    callback: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    fn new(callback: F) -> Self {
        ScopeGuard { callback: Some(callback) }
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cb) = self.callback.take() {
            cb();  // 析构时执行回调
        }
    }
}

fn guarded_operation() {
    let _guard = ScopeGuard::new(|| {
        println!("操作已清理");
    });
    println!("执行关键操作...");
    // panic 或 return 都会调用 _guard.drop()
}
```

---

## 十、策略模式 via 闭包

> **金句引用**："策略即闭包——静态泛型零开销重走编译期，动态调度按需换实现。"

### 10.1 静态策略（泛型闭包，零开销）

```rust
struct Processor<F> {
    strategy: F,
}

impl<F> Processor<F>
where
    F: Fn(i32) -> i32,
{
    fn new(strategy: F) -> Self { Processor { strategy } }

    fn process(&self, inputs: &[i32]) -> Vec<i32> {
        inputs.iter().map(|&x| (self.strategy)(x)).collect()
    }
}

let doubler = Processor::new(|x| x * 2);
let squarer = Processor::new(|x| x * x);

assert_eq!(doubler.process(&[1, 2, 3]), vec![2, 4, 6]);
assert_eq!(squarer.process(&[1, 2, 3]), vec![1, 4, 9]);
```

### 10.2 动态策略（Box<dyn Fn>，运行时切换）

```rust
struct FlexibleProcessor {
    strategy: Box<dyn Fn(i32) -> i32>,
}

impl FlexibleProcessor {
    fn new(strategy: Box<dyn Fn(i32) -> i32>) -> Self {
        FlexibleProcessor { strategy }
    }

    fn set_strategy(&mut self, strategy: Box<dyn Fn(i32) -> i32>) {
        self.strategy = strategy;
    }

    fn process(&self, inputs: &[i32]) -> Vec<i32> {
        inputs.iter().map(|&x| (self.strategy)(x)).collect()
    }
}

let mut fp = FlexibleProcessor::new(Box::new(|x| x + 1));
assert_eq!(fp.process(&[1, 2, 3]), vec![2, 3, 4]);
fp.set_strategy(Box::new(|x| x * 10));
assert_eq!(fp.process(&[1, 2, 3]), vec![10, 20, 30]);
```

---

## 十一、闭包与所有权全景总结

| 闭包类型 | Trait | 捕获方式 | 调用次数 | move 后 | 典型场景 |
|----------|-------|----------|----------|-----------|----------|
| 只读闭包 | `Fn` | `&T` | 无限制 | 可保持 Fn | 查询/读取 |
| 可变闭包 | `FnMut` | `&mut T` | 无限制 | 通常 FnOnce | 累加/计数 |
| 转移闭包 | `FnOnce` | `T`（所有权） | 一次 | FnOnce | drop/线程移动 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 闭包捕获了可变引用后外部也修改了同一值 | `FnMut` 持有 `&mut` 引用，阻止外部同时借用 | 确保闭包使用完后再进行外部修改 |
| 在多线程中使用非 move 的闭包 | 编译器无法推断引用的线程安全性 | 使用 `move` 关键字转移所有权 |
| 返回 `impl Fn` 但闭包捕获了临时变量的引用 | 临时变量在函数结束时释放 | 使用 `move` 将捕获值所有权转入闭包 |
| 闭包自动推导为 FnOnce 导致只能调用一次 | 闭包转移了非 Copy 捕获值 | 需要多次调用时改为 `&` 或 `&mut` 引用 |
| `const fn` 中使用不支持的语句 | `const fn` 不支持 `for`、堆操作等 | 用 `while` 替代 `for`；拆分 `const` 和运行时逻辑 |
| 函数指针与闭包混淆 | fn 指针不能捕获环境 | 需要捕获环境时用泛型 `Fn` trait 而非 `fn` 指针 |
| Builder 中链式调用后使用旧实例 | `self` 被消费后原值不可用 | 仅在最终 `build()` 之后使用构造出的对象 |
| 守卫类型未实现 `forget` 防护 | 守卫被 `forget` 后 Drop 不执行 | 守卫设计应放在 `ManuallyDrop` 可控区域 |

> **详见测试**: `tests/rust_features/16_functions_closures.rs`
