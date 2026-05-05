# 结构体与枚举

## 一、结构体的三种形式

Rust 中的结构体不是单一概念，而是三种形态的家族：

```rust
// 1. 具名字段结构体（C-like 结构体）
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

let mut user1 = User {
    email: String::from("alice@example.com"),
    username: String::from("alice"),
    active: true,
    sign_in_count: 1,
};
user1.sign_in_count += 1;  // 整个实例可变才能修改字段

// 2. 元组结构体
struct Color(i32, i32, i32);
struct Point(f64, f64, f64);

let black = Color(0, 0, 0);
let origin = Point(0.0, 0.0, 0.0);
let r = black.0;  // 索引访问
// Color 和 Point 是不同的类型，即使字段相同
// fn process(color: Color) {} 不接受 Point

// 3. 单元结构体（0大小类型）
struct AlwaysEqual;
let subject = AlwaysEqual;
// 用于 trait 实现、标记类型、零大小状态
```

> 结构体不是简单的"组合类型"——三种形式分别对应了三种设计意图：命名字段是数据记录，元组结构体是类型封装，单元结构体是类型标记。

---

## 二、内存布局

Rust 编译器默认使用 `repr(Rust)` 布局——可以任意重排字段以优化对齐和体积：

```rust
struct DefaultLayout {
    a: u8,    // 1 字节
    b: u32,   // 4 字节
    c: u8,    // 1 字节
}
// repr(Rust): 可能重排为 b(4) + a(1) + c(1) = 8 字节（含2字节padding）
// repr(C):    a(1) + 3字节padding + b(4) + c(1) + 3字节padding = 12 字节

#[repr(C)]
struct CCompatible {
    a: u8,
    b: u32,
    c: u8,
}
// 字段按声明顺序排列，对齐为最大字段的对齐要求
// 用于 FFI（外部函数接口），保证与 C 结构体兼容

// repr(align(N)) 提高对齐要求
#[repr(align(16))]
struct Aligned {
    a: u8,
}

// repr(packed) 移除 padding
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,
}

// repr(transparent) 内存布局与字段类型完全相同
#[repr(transparent)]
struct Wrapper(String);  // 与 String 布局一致

// 查看结构体大小
assert_eq!(std::mem::size_of::<Aligned>(), 16);
```

| repr 属性 | 行为 | 适用场景 |
|-----------|------|----------|
| `Rust`（默认） | 编译器自由重排字段优化内存 | 通用代码 |
| `C` | 字段声明顺序，C-兼容对齐 | FFI、二进制兼容 |
| `u8`/`u16`/… | 枚举判别式大小 | 节省内存的枚举 |
| `transparent` | 与单一字段布局相同 | NewType 包装器 |
| `packed` | 1字节对齐，无 padding | 网络协议解析 |
| `align(N)` | 强制 N 字节对齐 | SIMD、原子操作 |

---

## 三、impl 块：方法与关联函数

```rust
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // 关联函数（无 self）：用 :: 调用，相当于"静态方法"
    fn square(size: u32) -> Self {
        Self { width: size, height: size }
    }

    // 方法：self 的各种借用方式
    fn area(&self) -> u32 {           // 不可变借用
        self.width * self.height
    }

    fn can_hold(&self, other: &Self) -> bool {
        self.width > other.width && self.height > other.height
    }

    fn mutate(&mut self, w: u32) {    // 可变借用
        self.width = w;
    }

    fn consume(self) -> (u32, u32) {  // 消耗所有权
        (self.width, self.height)
    }
}

let rect = Rectangle { width: 30, height: 50 };
println!("面积: {}", rect.area());

let sq = Rectangle::square(10);
println!("正方形: {}x{}", sq.width, sq.height);

// 多个 impl 块可以散布在代码中
impl Rectangle {
    fn perimeter(&self) -> u32 {
        2 * (self.width + self.height)
    }
}
```

---

## 四、结构体更新语法的 Move 陷阱

```rust
#[derive(Debug)]
struct Person {
    name: String,      // 非 Copy 类型
    age: u32,          // Copy 类型
}

let alice = Person {
    name: String::from("Alice"),
    age: 30,
};

let bob = Person {
    name: String::from("Bob"),
    ..alice              // 结构体更新语法
};
// println!("{:?}", alice);  // 错误！alice 的部分字段被 move
// alice.name 被移动了，alice.age（Copy）未受影响
// 整体使用 alice 不再可能，但可以访问 alice.age

// 安全做法：只迁移 Copy 字段
let bob2 = Person {
    name: String::from("Bob"),
    age: alice.age,      // Copy 类型不受影响
};
```

> 结构体更新语法 `..` 看起来像浅拷贝，实际上是选择性移动——每一行代码背后都可能发生所有权转移，这正是 Rust 的诚实所在。

---

## 五、枚举定义与判别式

枚举是 Rust 类型系统中的核心抽象，比 C 的枚举强大得多：

```rust
// 基本枚举
enum IpAddrKind {
    V4,
    V6,
}

// 带数据的枚举
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

let home = IpAddr::V4(127, 0, 0, 1);
let loopback = IpAddr::V6(String::from("::1"));

// 判别式
enum Number {
    Zero = 0,     // 显式赋值
    One,          // 自动递增 = 1
    Two,          // = 2
    Three = 10,   // 显式赋值
    Four,         // = 11
}

// 判别式的默认类型是 isize，可用 repr 改变
#[repr(u8)]
enum Small {
    A,    // 0
    B,    // 1
    C,    // 2
}
assert_eq!(std::mem::size_of::<Small>(), 1);

// 枚举大小 = max(判别式大小) + max(变体数据大小) + 对齐 padding
enum Large {
    A(i32),            // 4 byte 数据
    B([u8; 100]),      // 100 byte 数据
}
// sizeof(Large) >= 104 (100 + 判别式)
```

---

## 六、Option 与空指针优化

```rust
// Option<Rc<T>> 的大小等于 Rc<T> —— 空指针优化
// Rc 内部是指针，None 表示为 null
assert_eq!(
    std::mem::size_of::<Option<Box<i32>>>(),
    std::mem::size_of::<Box<i32>>()
);

// 能够享受 NPO（空指针优化）的类型：
// - &T, &mut T（引用不能为空）
// - Box<T>
// - NonZero* 系列类型
// - Rc<T>, Arc<T>
// - fn 指针
// - std::num::NonZero*

// 不能享受 NPO 的类型：
// - i32, u64 等标量（0 是合法值）
// - 包含这些类型的枚举
assert_eq!(
    std::mem::size_of::<Option<i32>>(),
    std::mem::size_of::<i32>() + 判别式大小
);
```

---

## 七、泛型枚举

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// 类似 Either
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<T, E: std::fmt::Display> Either<T, E> {
    fn expect(self, msg: &str) -> T {
        match self {
            Either::Left(val) => val,
            Either::Right(err) => panic!("{msg}: {err}"),
        }
    }
}
```

---

## 八、impl 枚举与方法

```rust
enum Message {
    Quit,
    ChangeColor(i32, i32, i32),
    Write(String),
}

impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("退出"),
            Message::ChangeColor(r, g, b) => println!("颜色改为 ({r}, {g}, {b})"),
            Message::Write(text) => println!("{text}"),
        }
    }

    fn is_quit(&self) -> bool {
        matches!(self, Message::Quit)
    }
}

// matches! 宏
let msg = Message::Write(String::from("hello"));
assert!(matches!(msg, Message::Write(_)));
assert!(!msg.is_quit());
```

---

## 九、标准库枚举速查

| 枚举 | 用途 | 变体示例 |
|------|------|----------|
| `Option<T>` | 可选值 | `Some(T)`, `None` |
| `Result<T, E>` | 可能出错的操作 | `Ok(T)`, `Err(E)` |
| `Cow<'a, B>` | 写时复制 | `Borrowed(&'a B)`, `Owned(B::Owned)` |
| `Ordering` | 比较结果 | `Less`, `Equal`, `Greater` |
| `IpAddr` | IP 地址 | `V4(Ipv4Addr)`, `V6(Ipv6Addr)` |
| `Bound<T>` | 范围边界 | `Included(T)`, `Excluded(T)`, `Unbounded` |
| `Poll<T>` | 异步就绪状态 | `Ready(T)`, `Pending` |
| `ControlFlow<B, C>` | 控制流抽象 | `Continue(C)`, `Break(B)` |
| `Entry<'a, K, V>` | HashMap 条目 | `Occupied(OccupiedEntry)`, `Vacant(VacantEntry)` |
| `Cow<'a, str>` | 字符串写时复制 | `Borrowed(&'a str)`, `Owned(String)` |

---

## 十、让非法状态不可表达

"Make illegal states unrepresentable" 是 Rust 类型设计哲学的核心：

```rust
// 糟糕的设计：布尔 + 可选值，存在非法组合
struct BadConnection {
    connected: bool,
    stream: Option<TcpStream>,
}
// 非法状态：connected = true, stream = None

// 优秀的设计：类型系统保证一致性
enum GoodConnection {
    Connected(TcpStream),
    Disconnected,
}
// 不可能出现状态不一致的情况

// 另一个例子：从枚举替代多个可选字段
// 差：三个 Option 字段，7 个无效组合
struct BadConfig {
    port: Option<u16>,
    host: Option<String>,
    timeout: Option<Duration>,
}
// 好：枚举明确表达所有有效配置
enum GoodConfig {
    Default,
    Custom { port: u16, host: String, timeout: Duration },
}
```

> 类型系统是你的第一道防线——好的设计让非法状态连编译都过不了，而不是在运行时崩溃。

---

## 十一、NewType 模式

```rust
// 元组结构体包装，创建语义化的新类型
struct Meters(f64);
struct Feet(f64);

fn height_in_meters(height: Meters) -> f64 {
    height.0
}

fn height_in_feet(height: Feet) -> f64 {
    height.0 * 0.3048
}

let h = Meters(1.8);
// height_in_feet(h);  // 类型错误！Meters 不是 Feet

// NewType 用于绕过孤儿规则
// 孤儿规则：不能对非本地类型实现非本地 trait
// 以下做法不行：
// impl Display for Vec<String> {}  // Vec 和 Display 都不是本地的

// NewType 化解：
struct MyVec(Vec<String>);
impl std::fmt::Display for MyVec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

// NewType 还能封装 unsafe 接口
struct SafeHandle(RawHandle);
impl SafeHandle {
    fn new() -> Self {
        let raw = unsafe { create_raw_handle() };
        SafeHandle(raw)
    }
}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe { destroy_raw_handle(self.0); }
    }
}
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 结构体更新 `..` 后原变量部分字段被 move | 非 Copy 字段被移动 | 仅 Copy 字段能用 `..`，否则手写赋值 |
| `repr(Rust)` 布局不可预测 | 编译器优化字段重排 | 需要确定布局时使用 `#[repr(C)]` |
| 枚举大小可能出乎意料地大 | 判别式 + 最大变体数据 | 大枚举用 `Box` 包装大尺寸变体 |
| `impl` 块无法为外部类型实现外部 trait | 孤儿规则限制 | 使用 NewType 模式包装 |
| 忘记 `#[derive(Debug)]` 导致无法打印 | Rust 不自动派生 trait | 添加 `#[derive(Debug, Clone, ...)]` |
| 枚举变体单独不是类型 | `IpAddr::V4` 是构造器不是类型 | 所有变体共享同一个枚举类型 |
| `repr(packed)` 导致未定义行为 | 引用未对齐字段是 UB | 使用 `unsafe` 时格外小心指针操作 |
| 可变结构体所有字段都可变 | Rust 不支持字段级可变性 | 使用 `Cell`/`RefCell` 实现细粒度可变性 |
| 单元结构体不等于 `()` | 类型名称不同就是不同类型 | 只在需要的时候用单元结构体 |
| `impl` 中类型别名不可用于 self | self 必须是实际类型 | 直接使用 `Self`、`&self`、`&mut self` |

> **详见测试**: `tests/rust_features/07_structs_enums.rs`
