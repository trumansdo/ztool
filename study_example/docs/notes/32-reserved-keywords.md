# 保留关键字

## 目录
1. [Rust 关键字四分类表](#rust-关键字四分类表)
2. [raw identifier (r#keyword) 语法](#raw-identifier-rkeyword-语法)
3. [gen 关键字：保留用于未来生成器](#gen-关键字保留用于未来生成器)
4. [dyn 关键字：Trait 对象的动态分发](#dyn-关键字trait-对象的动态分发)
5. [async/await 关键字](#asyncawait-关键字)
6. [Self/self 别名含义](#selfself-别名含义)
7. [in 关键字 Edition 2024 用途](#in-关键字-edition-2024-用途)
8. [static vs const](#static-vs-const)
9. [Edition 迁移建议](#edition-迁移建议)
10. [避坑指南](#避坑指南)

---

## Rust 关键字四分类表

Rust 的关键字分为四个类别：严格关键字、保留关键字、弱关键字、以及 Edition 新增关键字。每类关键字的限制级别和使用场景不同。

> 名字里藏着权力——关键字就是 Rust 编译器手中国王的印章。

### 类别一：严格关键字 (Strict)

严格关键字在任何上下文中都不能作为标识符使用。

```text
as      break   const   continue  crate   else    enum
extern  false   fn      for       if      impl    in
let     loop    match   mod       move    mut     pub
ref     return  self    Self      static  struct  super
trait   true    type    unsafe    use     where   while
async   await   dyn     union
```

### 类别二：保留关键字 (Reserved)

预留但尚未使用的关键字，不能作为标识符。

```text
abstract  become   box       do        final     macro
override  priv     try       typeof    unsized   virtual
yield     gen
```

### 类别三：弱关键字 (Weak)

在特定上下文中有特殊含义，但大多数情况下可作为普通标识符使用。

```text
union   'static    macro_rules
```

`union` 仅在声明联合体时作为关键字；`'static` 是生命周期名称；`macro_rules!` 是宏调用。其余位置它们可以自由用作标识符。

```rust
fn demo_weak_keywords() {
    let union = 42;      // 合法：union 作为变量名
    let r#static = "ok"; // 也可以但在模块作用域可能引起混淆
    println!("{union}");
}
```

### 类别四：Edition 新增关键字

在特定 Edition 中作为新关键字引入，旧版中可用 raw identifier 过渡。

| 关键字 | 引入的 Edition | 用途 |
|--------|---------------|------|
| `async` | 2018 | 异步函数与异步块 |
| `await` | 2018 | 等待异步操作 |
| `dyn` | 2018 | Trait 对象 |
| `try` | 2018 (保留) | 预留 |
| `gen` | 2024 (保留) | 预留生成器 |

---

## raw identifier (r#keyword) 语法

`r#` 前缀允许将保留关键字作为标识符使用，用于版本迁移和 FFI 互操作。

> `r#` 是你给编译器的特殊通行证——"我知道这是关键字，但我现在需要它当名字。"

基本用法：

```rust
// 定义一个与关键字同名的函数（不推荐，但有时必要）
fn r#as() -> &'static str {
    "这是一个名为 'as' 的函数"
}

fn call_raw() {
    let r#let = 3;          // 变量名叫 let
    let r#match = "value";  // 变量名叫 match
    let r#fn = || 42;      // 闭包名叫 fn

    println!("{} {} {}", r#let, r#match, r#fn());
}
```

Edition 迁移场景——因参数名冲突：

```rust
// Edition 2021: in 不是关键字，可作为参数名
// Edition 2024: in 变成关键字，需用 r#in

struct Range {
    start: i32,
    end: i32,
}

impl Range {
    // Edition 2024 兼容写法
    fn contains(&self, r#in: i32) -> bool {
        r#in >= self.start && r#in <= self.end
    }
}

// 老代码自动迁移：cargo fix --edition
```

FFI 调用包含 Rust 关键字的 C 函数：

```rust
extern "C" {
    fn r#match(path: *const u8) -> i32; // C 函数名是 match
}

fn call_c_match() {
    let result = unsafe { r#match(b"/some/path\0".as_ptr()) };
}
```

---

## gen 关键字：保留用于未来生成器

`gen` 自 Rust 2024 Edition 起被列为保留关键字，计划用于原生生成器语法（类似 Python 的 `yield` 但编译期为迭代器）。

> gen 是一颗播在土里的种子——今天只是保留，明天将开花结果。

当前 gen 块实验性用法（需 nightly）：

```rust
// 实验性生成器（nightly only）
// #![feature(gen_blocks)]
//
// fn numbers() -> impl Iterator<Item = u32> {
//     gen {
//         yield 1;
//         yield 2;
//         yield 3;
//     }
// }
```

当前代码中 `gen` 是保留字，不能作为标识符：

```rust
// 旧代码中有名为 gen 的变量需改名
// let gen = 5; // 编译错误 (Edition 2024)
let r#gen = 5; // 使用 raw identifier 过渡
```

---

## dyn 关键字：Trait 对象的动态分发

`dyn` 关键字用于声明 trait 对象，表示通过虚函数表（vtable）进行动态分发。

> dyn 是王国里的弄臣——它在运行时才决定将消息送往哪个宫殿。

Trait 对象基础：

```rust
trait Animal {
    fn speak(&self) -> &'static str;
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) -> "汪汪" { "汪汪" }
}

struct Cat;
impl Animal for Cat {
    fn speak(&self) -> "喵喵" { "喵喵" }
}

fn zoo() {
    // 静态分发：编译期确定具体类型
    let dog = Dog;
    dog.speak();

    // 动态分发：运行时通过 vtable 调用
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog),
        Box::new(Cat),
    ];
    for a in &animals {
        println!("{}", a.speak());
    }
}
```

`dyn` + 自动解引用：

```rust
fn process(animal: &dyn Animal) {
    animal.speak();
}

fn demo_dispatch() {
    let dog = Dog;
    process(&dog); // &Dog 自动转为 &dyn Animal
}
```

对象安全规则——不是所有 trait 都可以用 `dyn`：

```rust
// 对象安全的 trait（可用 dyn）
trait Drawable {
    fn draw(&self);     // 方法接收 &self
    fn name() -> &'static str where Self: Sized; // Sized 限定 => OK
}

// 非对象安全的 trait（不可用 dyn）
// trait Cloneable: Clone {} // Clone 需要 Sized -> 不行
// fn bad() { let _: Box<dyn Clone>; } // 编译错误
```

---

## async/await 关键字

`async` 定义一个异步块或异步函数，返回实现了 `Future` trait 的类型。`.await` 在当前上下文等待一个 `Future` 完成。

> async 画出异步的蓝图，.await 则按下执行键——一动一静，合作无间。

异步函数与块：

```rust
async fn fetch_data(id: u32) -> String {
    format!("数据_{id}")
}

async fn orchestrate() {
    // async 块——创建包含异步操作的闭包
    let future = async {
        let a = fetch_data(1).await;
        let b = fetch_data(2).await;
        format!("{a} + {b}")
    };

    let result = future.await;
    println!("{result}");
}
```

`.await` 的语法演变：

```rust
// Edition 2018: 后缀 .await（当前标准语法）
async fn edition_2018_style() {
    let x = fetch_data(1).await;
}

// 早期 RFC 中的前缀 await! 宏（历史遗迹）
// let x = await!(fetch_data(1));
```

async 生命周期：

```rust
async fn borrowed_input(s: &str) -> &str {
    s
}

fn hold_future() {
    let owned = String::from("hello");
    // let fut = borrowed_input(&owned);
    // drop(owned);
    // fut.await; // 错误: owned 已被释放
}
```

---

## Self/self 别名含义

`Self` 和 `self` 是 Rust 中两个易混淆但含义完全不同的标识符。

> Self 是你是谁，self 是你本身——类型与值之间的一个字母之差。

| 标识符 | 含义 | 使用位置 |
|--------|------|----------|
| `Self` | 当前实现块的**类型** | impl 块内、trait 定义内 |
| `self` | 方法的**接收者**，等价于 `self: Self` | 方法参数 |
| `&self` | 不可变借用的接收者，等价于 `self: &Self` | 方法参数 |
| `&mut self` | 可变借用的接收者，等价于 `self: &mut Self` | 方法参数 |

```rust
struct Counter {
    value: i32,
}

impl Counter {
    // Self = Counter (类型)
    fn new() -> Self {
        Self { value: 0 }
    }

    // self = Counter 的值 (self: Self)
    fn increment(mut self) -> Self {
        self.value += 1;
        self
    }

    // &self = &Counter
    fn value(&self) -> i32 {
        self.value
    }

    // &mut self = &mut Counter
    fn reset(&mut self) {
        self.value = 0;
    }
}
```

trait 内部的 Self：

```rust
trait Builder {
    type Output;

    fn build(self) -> Self::Output;
}

impl Builder for Counter {
    type Output = i32;

    fn build(self) -> Self::Output { // Self = Counter
        self.value
    }
}
```

---

## in 关键字 Edition 2024 用途

`in` 在 Edition 2024 中被提升为严格关键字，不再能作为变量或参数名。

> 一个 `in` 爬上了关键字的王座——你的旧代码变量名该改名了。

Edition 2024 中 `in` 的新身份：

```rust
// Edition 2024: in 是关键字
// for x in iter { } —— 合法语法位置

// 旧代码中的冲突
// fn test(in: i32) { } // Edition 2024 编译错误

// 迁移方案
fn test(r#in: i32) { // raw identifier 过渡
    println!("{r#in}");
}

fn test_fixed(input: i32) { // 重命名为更清晰的名称
    println!("{input}");
}
```

---

## static vs const

`static` 和 `const` 都定义编译期常量，但内存模型和使用场景不同。

> const 是模板，每次使用时刻印一次；static 是圣地，只有一座永久的雕像。

对比表：

| 特性 | `const` | `static` |
|------|---------|----------|
| 内存地址 | 无固定地址（内联） | 固定地址 |
| 可变性 | 不可变 | 可用 `static mut`（需 unsafe） |
| 类型 | 任意常量表达式 | 任意常量表达式 |
| 内部可变性 | 不允许 | 可用 `Mutex/RwLock` 包裹 |
| Drop | 无（Copy/内联） | 程序退出时析构 |

```rust
const MAX_SIZE: usize = 1024;       // 编译期内联
static APP_NAME: &str = "MyApp";    // 静态内存地址

// static mut——需要 unsafe 访问
static mut COUNTER: u32 = 0;

fn increment() {
    unsafe {
        COUNTER += 1;
    }
}

// 懒初始化 static——使用 LazyLock (Rust 1.80+)
use std::sync::LazyLock;
static CONFIG: LazyLock<String> = LazyLock::new(|| {
    std::env::var("APP_CONFIG").unwrap_or_default()
});
```

`const` 与泛型参数交互：

```rust
const fn const_add<const N: usize>(x: usize) -> usize {
    x + N
}

struct Buffer<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> Buffer<N> {
    const fn size() -> usize { N }
}
```

---

## Edition 迁移建议

从旧 Edition 升级时，按照以下步骤处理关键字冲突。

> 迁移不是革命，是一步一个脚印的演变。

迁移四步法：

```bash
# 第一步：尝试自动迁移
cargo fix --edition

# 第二步：检查未被自动修复的关键字冲突
cargo clippy --all-targets

# 第三步：手动处理遗留的 raw identifier
# 将 r#gen, r#in 等重命名为更具描述性的名称

# 第四步：更新 Cargo.toml 中的 edition 字段
# edition = "2024"
```

常见冲突及处理方案：

| 旧标识符 | 冲突原因 | 推荐方案 |
|----------|----------|----------|
| `gen` | Edition 2024 保留关键字 | 重命名为 `generator` / `r#gen` |
| `in` (参数名) | Edition 2024 严格关键字 | 重命名为 `input` / `r#in` |
| `yield` | 保留关键字 | 重命名为 `produce` / `r#yield` |
| `macro` | 保留关键字 | 重命名为 `mac` / `r#macro` |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 误将 `Self` 当 `self` 使用 | `Self` 是类型，`self` 是值——语法上不兼容 | 仔细区分：类型位置用 `Self`，值/参数位置用 `self` |
| `dyn Trait` 用于非对象安全 trait | Trait 包含泛型方法或不兼容 Sized 的方法签名 | 检查 trait 是否对象安全，必要时重构 trait 设计 |
| `static mut` 不加 unsafe 访问 | Rust 保证 `static mut` 存取必须 unsafe | 改用 `Atomic*` 类型或 `Mutex<static>` 代替 `static mut` |
| raw identifier 滥用 | `r#` 是**过渡手段**而非长久之计 | 迁移完成后将 r#name 重命名为有意义的标识符 |
| 混淆 `const` 和 `static` 的内存语义 | `const` 无固定地址，作为指针时每次使用指向不同位置 | 需要固定地址的场景（如 FFI）务必使用 `static` |
| 异步函数未经 `.await` 就丢弃 | 返回的 Future 被 drop 时内部工作不会执行 | 确保所有 async 调用都被 `.await` 或被显式推进 |
| Edition 升级后漏掉第三方依赖的兼容性检查 | 依赖可能使用了被新 Edition 禁用的旧语法 | 升级前检查所有依赖是否兼容目标 Edition |

---

> **详见测试**: `tests/rust_features/32_reserved_keywords.rs`
