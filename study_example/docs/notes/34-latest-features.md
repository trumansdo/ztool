# 最新 Rust 特性速查

## 目录
1. [const fn 进阶](#const-fn-进阶)
2. [const 泛型进阶：编译期计算](#const-泛型进阶编译期计算)
3. [关联常量与默认值](#关联常量与默认值)
4. [trait upcasting：dyn 子 trait 转父 trait](#trait-upcastingdyn-子-trait-转父-trait)
5. [blanket impl 无冲突规则](#blanket-impl-无冲突规则)
6. [内联属性优化](#内联属性优化)
7. [Cargo workspace 发布与依赖](#cargo-workspace-发布与依赖)
8. [Edition 2024 路线图](#edition-2024-路线图)
9. [避坑指南](#避坑指南)

---

## const fn 进阶

自 Rust 1.46 起，`const fn` 的能力持续扩展：支持分支、循环、复合类型构造、以及有限的 trait 方法调用。Rust 1.83+ 已支持在 const 上下文中创建 `Vec` 和 `String` 等分配类型（需 nightly 或特定 feature）。

> const fn 是在编译时运行的时光机器——结果刻在二进制中，零运行时开销。

递归 const fn：

```rust
const fn factorial(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

const FACT_10: u64 = factorial(10);
// 编译期计算出 3628800，嵌入二进制

fn main() {
    assert_eq!(FACT_10, 3628800);
}
```

const fn 中的循环：

```rust
const fn sum_of_squares(limit: u32) -> u32 {
    let mut total = 0;
    let mut i = 1;
    while i <= limit {
        total += i * i;
        i += 1;
    }
    total
}

const RESULT: u32 = sum_of_squares(5);
// 1^2 + 2^2 + 3^2 + 4^2 + 5^2 = 55
```

const fn 构造复合类型：

```rust
#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

const fn make_point(x: i32, y: i32) -> Point {
    Point { x, y }
}

const ORIGIN: Point = make_point(0, 0);

// const fn 结合泛型
const fn twice<T: ~const std::ops::Mul<Output = T> + Copy>(x: T, factor: T) -> T {
    // 注: ~const 语法是实验性的 (const_trait_impl)
    x // 简化示例
}
```

const fn 中的 Option/Result：

```rust
const fn parse_bool(s: &str) -> Option<bool> {
    if s.len() == 0 {
        return None;
    }
    match s.as_bytes()[0] {
        b't' | b'T' | b'1' => Some(true),
        b'f' | b'F' | b'0' => Some(false),
        _ => None,
    }
}

const IS_TRUE: Option<bool> = parse_bool("true");
```

---

## const 泛型进阶：编译期计算

const 泛型允许使用编译期常量值作为类型参数，实现矩阵维度、缓冲区大小等在类型层面的参数化。

> const 泛型是数字刻在类型上的签名——运行时再也无法越界。

编译期数组大小控制：

```rust
struct Vector<T, const N: usize> {
    data: [T; N],
}

impl<T: Copy + Default, const N: usize> Vector<T, N> {
    const fn new() -> Self {
        Vector { data: [T::default(); N] }
    }
}

impl<T: std::ops::Add<Output = T> + Copy, const N: usize> Vector<T, N> {
    fn sum(&self) -> T {
        let mut acc = self.data[0];
        let mut i = 1;
        while i < N {
            acc = acc + self.data[i];
            i += 1;
        }
        acc
    }
}

fn const_generic_demo() {
    let v: Vector<i32, 4> = Vector { data: [1, 2, 3, 4] };
    assert_eq!(v.sum(), 10);
}
```

const 泛型表达式约束：

```rust
// 确保长度非零
struct NonEmpty<T, const N: usize> where [(); N]: {
    data: [T; N],
}

impl<T, const N: usize> NonEmpty<T, N>
where
    [(); N]:,
{
    fn new(data: [T; N]) -> Self {
        NonEmpty { data }
    }
}

// 编译期算术推导
fn concat_arrays<T: Copy, const N: usize, const M: usize>(
    a: [T; N], b: [T; M]
) -> [T; N + M]
where
    [(); N + M]:,
{
    let mut result = [a[0]; N + M];
    let mut i = 0;
    while i < N {
        result[i] = a[i];
        i += 1;
    }
    while i < N + M {
        result[i] = b[i - N];
        i += 1;
    }
    result
}
```

const 泛型与 trait bound：

```rust
const fn is_power_of_two<const N: usize>() -> bool {
    N > 0 && (N & (N - 1)) == 0
}

struct AlignedBuffer<const N: usize>
where
    [(); N]:,
{
    data: [u8; N],
}

// 调用处：
// let buf: AlignedBuffer<{2_usize.pow(10)}>; // 编译期求值 1024
```

---

## 关联常量与默认值

trait 中的关联常量允许为每个实现定制编译期常量，加强泛型代码的灵活性。

> 关联常量是 trait 赠予每种实现的一份"出厂设置"。

基本关联常量：

```rust
trait Color {
    const RED: Self;
    const GREEN: Self;
    const BLUE: Self;
}

impl Color for u32 {
    const RED: u32 = 0xFF0000;
    const GREEN: u32 = 0x00FF00;
    const BLUE: u32 = 0x0000FF;
}

fn paint<C: Color>(base: C) -> C { base }

// 带默认值的关联常量
trait Buffer {
    const DEFAULT_SIZE: usize = 4096;

    fn new() -> Self;
    fn with_size(size: usize) -> Self;
}

struct FileBuffer {
    size: usize,
}

impl Buffer for FileBuffer {
    // 使用默认值 4096，无需重新声明
    fn new() -> Self {
        FileBuffer { size: Self::DEFAULT_SIZE }
    }

    fn with_size(size: usize) -> Self {
        FileBuffer { size }
    }
}
```

关联常量与 const 泛型联动：

```rust
trait Capacity {
    const CAP: usize;
}

struct Ring<T, const N: usize> {
    buf: [Option<T>; N],
}

impl<T, const N: usize> Ring<T, N>
where
    [Option<T>; N]:,
{
    fn new() -> Self {
        Ring { buf: [const { None }; N] }
    }
}

impl<T> Capacity for Ring<T, 16> {
    const CAP: usize = 16;
}
impl<T> Capacity for Ring<T, 256> {
    const CAP: usize = 256;
}
```

---

## trait upcasting：dyn 子 trait 转父 trait

Rust 1.76 稳定了 trait upcasting 强制转换（trait upcasting coercion），允许 `dyn TraitSub` 自动转换为 `dyn TraitSuper`（当 `TraitSub: TraitSuper` 时）。

> 子 trait 是父 trait 的特化——向上转型就是摘掉细节，只露出契约。

基础 upcasting 示例：

```rust
trait Animal {
    fn name(&self) -> &str;
}

trait Dog: Animal {
    fn bark(&self);
}

struct Labrador;
impl Animal for Labrador {
    fn name(&self) -> "拉布拉多" { "拉布拉多" }
}
impl Dog for Labrador {
    fn bark(&self) { println!("汪汪!"); }
}

fn as_animal(d: &dyn Dog) -> &dyn Animal {
    d // 自动 upcasting: &dyn Dog -> &dyn Animal
}

fn demo_upcast() {
    let lab = Labrador;
    let dog: &dyn Dog = &lab;
    let animal: &dyn Animal = dog; // 向上转型
    println!("{}", animal.name());
}
```

多层 trait 层级：

```rust
trait Drawable {
    fn draw(&self);
}
trait Clickable: Drawable {
    fn click(&self);
}
trait Draggable: Clickable {
    fn drag(&self, dx: f64, dy: f64);
}

// 自动转换链：
// &dyn Draggable -> &dyn Clickable -> &dyn Drawable
fn render(obj: &dyn Drawable) {
    obj.draw();
}
fn interact(obj: &dyn Draggable) {
    render(obj); // 三重 upcast
    obj.click();
    obj.drag(10.0, 20.0);
}
```

---

## blanket impl 无冲突规则

Rust 的 blanket impl（覆盖式实现）即"为所有满足约束的类型实现某 trait"，编译器通过"孤儿规则"（orphan rule）+ 重叠检查确保 impl 无冲突。

> blanket impl 像是给一群人发统一的制服——只要符合条件，人人都有一套。

合法 blanket impl：

```rust
// 为所有实现了 Display 的类型自动实现 AsStr
trait AsStr {
    fn as_string(&self) -> String;
}

impl<T: std::fmt::Display> AsStr for T {
    fn as_string(&self) -> String {
        self.to_string()
    }
}

fn demo() {
    assert_eq!(42.as_string(), "42");
    assert_eq!('A'.as_string(), "A");
}
```

冲突检测：

```rust
trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

// 合法：两个 blanket impl 没有重叠
impl<T: std::fmt::Display> ToBytes for T {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

// impl<T: Copy> ToBytes for T { ... }
// 冲突！存在类型同时满足 Display + Copy (如 i32)
// 编译器会拒绝编译

// 解决办法——使用 marker trait 或 newtype
struct DisplayBytes<T: std::fmt::Display>(T);
impl<T: std::fmt::Display> ToBytes for DisplayBytes<T> {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_string().into_bytes()
    }
}
```

孤儿规则（Orphan Rule）回顾：

```
至少满足以下条件之一，impl 才是合法的：
1. trait 定义在本地 crate
2. 类型定义在本地 crate
（两者不能均为外部 crate）
```

---

## 内联属性优化

`#[inline]` 和 `#[inline(always)]` 控制函数的跨 crate 内联决策，`#[inline(never)]` 阻止内联。

> 内联是性能的微操——inline(always) 是恳求编译器，不是命令。

内联属性层次：

```rust
// 1. 默认——编译器自行决定内联
fn default_behavior(x: i32) -> i32 { x + 1 }

// 2. 建议内联——给编译器一个 hint
#[inline]
fn suggested_inline(x: i32) -> i32 { x * 2 }

// 3. 始终内联——强制请求（编译器可能忽略）
#[inline(always)]
fn always_inline(x: i32) -> i32 { x.wrapping_mul(3) }

// 4. 从不内联——用于调试、递归、冷路径
#[inline(never)]
fn never_inline(x: i32) -> i32 {
    // 冷路径逻辑
    x * x * x
}
```

通用规则表：

| 场景 | 推荐 | 原因 |
|------|------|------|
| 简单 getter/setter | `#[inline]` 或默认 | 小函数内联有利 |
| 跨 crate 泛型函数 | `#[inline]` | 确保跨 crate 可内联 |
| 递归函数 | `#[inline(never)]` | 内联递归通常有害 |
| 冷路径 / 错误处理 | `#[cold]` + `#[inline(never)]` | 减少指令缓存污染 |
| 基准测试关键路径 | `#[inline(always)]` | 消除调用开销 |
| trait 方法默认实现 | `#[inline]` | 避免 vtable 查找开销 |

---

## Cargo workspace 发布与依赖

Cargo workspace 允许多个 crate 在单一仓库中联动开发，共享依赖版本和输出目录。

> workspace 是家族的屋檐——各房独立建，但共享同一片天。

workspace Cargo.toml 示例：

```toml
[workspace]
members = [
    "core",
    "cli",
    "web",
]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

成员 crate 引用 workspace 依赖：

```toml
# cli/Cargo.toml
[package]
name = "my-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
core = { path = "../core" }
serde = { workspace = true }
tokio = { workspace = true, features = ["rt-multi-thread"] }
```

发布策略：

```bash
# 发布单个 crate
cargo publish -p my-core

# 检查所有 crate 是否可发布
cargo check --workspace

# 批量发布（按依赖顺序）
cargo publish -p my-core
cargo publish -p my-cli
cargo publish -p my-web
```

---

## Edition 2024 路线图

Edition 2024 是 Rust 三年 Edition 周期的下一个里程碑，带来多项语法和语义改进。

> 每个 Edition 是 Rust 的一次蜕皮——不是忘记过去，而是以更好的姿态面向未来。

Edition 2024 核心变更清单：

| 特性 | 类别 | 状态 |
|------|------|------|
| unsafe extern 块 | 安全性 | 已稳定 |
| if let 临时作用域调整 | 语义修正 | 已稳定 |
| gen 保留关键字 | 语法预留 | 已稳定 |
| impl Trait 作用域修复 | 语义修正 | 已稳定 |
| `use<>` 精确捕获 | 新特性 | 已稳定 (1.82) |
| RPIT 生命周期改进 | 语义修正 | 已稳定 |
| in 关键字提升 | 语法 | 已稳定 |
| 匹配人体工程学改进 | 人体工程学 | 部分稳定 |

升级前核对：

```bash
# 当前 Rust 版本
rustc --version

# 查看 Edition 目标
rustup doc --edition-guide

# 检查依赖兼容性
cargo tree --invert --package serde

# 执行升级
cargo fix --edition
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| const fn 中使用了非 const trait 方法 | 并非所有 trait 方法都有 `const` 等效版本 | 查阅 `const_fn_trait_bound` feature 或等待稳定 |
| const 泛型表达式过于复杂 | 编译器对 const 泛型求值有复杂度限制 | 将复杂计算拆分为多个 `const fn` |
| 关联常量默认值被误解为虚方法 | 默认值在 trait 层级解析，不存在动态分派 | 理解关联常量是静态的编译期绑定 |
| trait upcasting 导致方法调用歧义 | 多个 trait 中有同名方法 | 使用完全限定语法：`<dyn Trait>::method(&obj)` |
| blanket impl 与下游 crate 冲突 | 孤儿规则阻止跨 crate 的冲突 impl | 使用 newtype 模式隔离冲突 |
| `#[inline(always)]` 滥用导致代码膨胀 | 每处调用点都内联，二进制体积急剧膨胀 | 仅在性能关键的热路径上使用 inline(always) |
| workspace 中版本号不同步 | 多个子 crate 中各自的 dependency 版本漂移 | 使用 `[workspace.dependencies]` 统一管理版本 |

---

> **详见测试**: `tests/rust_features/34_latest_features_quick_ref.rs`
