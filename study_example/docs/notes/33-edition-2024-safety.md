# Edition 2024 安全特性

## 目录
1. [unsafe extern 块必须显式 unsafe](#unsafe-extern-块必须显式-unsafe)
2. [隐式 ABI 警告](#隐式-abi-警告)
3. [Never type ! 类型改进](#never-type--类型改进)
4. [MaybeUninit 使用规范](#maybeuninit-使用规范)
5. [repr 属性组合与限制](#repr-属性组合与限制)
6. [ZST (零大小类型)](#zst-零大小类型)
7. [安全层次全景表](#安全层次全景表)
8. [Edition 2024 迁移建议](#edition-2024-迁移建议)
9. [避坑指南](#避坑指南)

---

## unsafe extern 块必须显式 unsafe

Edition 2024 要求所有包含 `unsafe` 函数的 `extern` 块必须在 `extern` 前标注 `unsafe` 关键字。这是 Edition 2024 最重要的安全性改进之一。

> 你声明的每一条 extern 协议都是一个承诺——unsafe 标签就是你签上去的名字。

Edition 2021 vs 2024 对比：

```rust
// Edition 2021：允许不带 unsafe 的 extern 块包含 unsafe 声明
extern "C" {
    fn strlen(s: *const u8) -> usize; // 隐式 unsafe——遗漏的安全隐患
}

// Edition 2024：必须显式标注
unsafe extern "C" {
    fn strlen(s: *const u8) -> usize;
    fn memcpy(dest: *mut u8, src: *const u8, n: usize);
}

fn safe_wrapper(s: &str) -> usize {
    unsafe { strlen(s.as_ptr()) }
}
```

完整 FFI 模块的 Edition 2024 风格：

```rust
unsafe extern "C" {
    pub fn abs(input: i32) -> i32;
    pub fn atan2(y: f64, x: f64) -> f64;
}

// 建议：每个 unsafe extern 块提供对应的安全封装
pub fn safe_abs(input: i32) -> i32 {
    unsafe { abs(input) }
}

pub fn safe_atan2(y: f64, x: f64) -> f64 {
    unsafe { atan2(y, x) }
}
```

Rust ABI (无 FFI 风险) 的 extern 块同样需要标注：

```rust
unsafe extern "Rust" {
    fn internal_syscall(n: u32) -> i32;
}

// 唯一例外：不包含任何 unsafe 函数或 static 的 extern 块
// 可以省略 unsafe 前缀（罕见）
extern "Rust" {
    fn safe_extern() -> i32; // 假设有安全实现
}
```

---

## 隐式 ABI 警告

Edition 2024 对 `extern` 块缺少显式 ABI 字符串的情况发出警告，要求开发者明确 FFI 调用约定。

> ABI 是协议，不是猜谜游戏——编译器不再帮你猜测你想用什么约定。

显式 vs 隐式 ABI：

```rust
// Edition 2021: 可以省略 "C"，默认 "C" ABI
extern {
    fn legacy_call();
}

// Edition 2024: 必须显式指定 ABI
extern "C" {
    fn clear_call();
    fn another_call(x: i32) -> bool;
}
```

多 ABI 场景：

```rust
// Windows 上的不同调用约定
extern "C" {
    fn c_style();
}

extern "stdcall" {
    fn winapi_style(param: u32);
}

extern "system" {
    fn os_default(x: *mut u8) -> i32;
}

// Rust 的默认 extern 块映射：
// extern "C" === extern "C" { ... }
// 不可再使用裸 extern { ... } (Edition 2024 警告)
```

---

## Never type ! 类型改进

`!`（never type）表示永远不会返回的类型，在 Edition 2024 中有更精确的类型推导能力，尤其是 `match` 分支中混合 `!` 时。

> `!` 是反证法的化身——因为不可能发生，所以任何结论都成立。

! 在 match 中的作用：

```rust
fn only_some(x: Option<i32>) -> i32 {
    match x {
        Some(v) => v,
        None => panic!("不可能为 None"), // panic! 返回 ! 类型
    }
    // 因为 ! 可以自动转为任何类型 (i32)，
    // 编译器接受这个 match 是完整的（无需 else）
}

// 高级用法：循环中匹配
fn process_until_done(inputs: &[Option<i32>]) -> i32 {
    for opt in inputs {
        let val = match opt {
            Some(&v) => v,
            None => continue, // continue 返回 ! 
        };
        // 此分支 val 必然是 i32
        if val > 100 { return val; }
    }
    0
}
```

! 与 Result 组合：

```rust
fn infallible_parse() -> Result<i32, !> {
    // 因为 ! 永不构造，所以此 Result 永远为 Ok
    Ok(42)
}

fn unwrap_infallible() {
    let x = infallible_parse().unwrap(); // 安全：Err(!) 不存在
    // 编译器优化：unwrap() 可以被优化掉
}
```

---

## MaybeUninit 使用规范

`MaybeUninit<T>` 是 Rust 未初始化内存的安全抽象，必须通过 `write` 初始化后再 `assume_init` 读取。

> MaybeUninit 是一块未开垦的土地——你不播种就收获，编译器会对你怒吼。

标准使用流程：

```rust
use std::mem::MaybeUninit;

fn create_array() -> [String; 3] {
    // 步骤 1：创建未初始化内存
    let mut arr: [MaybeUninit<String>; 3] = unsafe {
        MaybeUninit::uninit().assume_init()
    };

    // 步骤 2：逐个初始化
    arr[0].write("一".to_string());
    arr[1].write("二".to_string());
    arr[2].write("三".to_string());

    // 步骤 3：转换整个数组为已初始化类型
    unsafe {
        std::mem::transmute::<[MaybeUninit<String>; 3], [String; 3]>(arr)
    }
}
```

零初始化：

```rust
use std::mem::MaybeUninit;

fn zeroed_buffer(size: usize) -> Vec<u8> {
    // MaybeUninit::zeroed() —— 内存全部置零
    let mut buf: Vec<MaybeUninit<u8>> = (0..size)
        .map(|_| MaybeUninit::zeroed())
        .collect();

    // 因为 u8 的所有位模式都有效，直接 assume_init 安全
    unsafe {
        std::mem::transmute::<Vec<MaybeUninit<u8>>, Vec<u8>>(buf)
    }
}
```

常见错误——未初始化就读：

```rust
use std::mem::MaybeUninit;

fn bad_example() {
    let uninit: MaybeUninit<i32> = MaybeUninit::uninit();
    // let x = unsafe { uninit.assume_init() }; // 未定义行为！
    // 必须先用 write 初始化
}

fn good_example() {
    let mut uninit = MaybeUninit::<i32>::uninit();
    uninit.write(42);
    let x = unsafe { uninit.assume_init() }; // 安全
    assert_eq!(x, 42);
}
```

部分初始化数组：

```rust
use std::mem::MaybeUninit;

fn partial_init() -> [i32; 4] {
    let mut arr: [MaybeUninit<i32>; 4] = unsafe { MaybeUninit::uninit().assume_init() };

    for i in 0..4 {
        arr[i].write(i as i32 * 10);
    }

    unsafe { std::mem::transmute(arr) }
}
```

---

## repr 属性组合与限制

`repr` 属性控制类型的内部内存布局和 ABI 表现。多个 repr 属性可以组合，但某些组合互斥。

> 内存布局是编程里的地缘政治——repr 就是划分国界的条约。

基本 repr 清单：

```rust
// 1. C 布局——与 C 语言兼容，字段有序排列
#[repr(C)]
struct CPoint {
    x: f64,
    y: f64,
}

// 2. aligned(N)——指定对齐要求
#[repr(align(16))]
struct AlignedData {
    buffer: [u8; 64],
}

// 3. packed——取消字段间 padding
#[repr(packed)]
struct PackedHeader {
    version: u8,   // 1 byte
    flags: u16,    // 2 bytes（无 padding）
    length: u32,   // 4 bytes
}
// sizeof(PackedHeader) == 7 (不是 8)

// 4. transparent——单字段结构体外层透明
#[repr(transparent)]
struct Wrapper(u32);

// 5. Rust 默认布局——编译器优化，不保证顺序
struct DefaultLayout {
    a: u8,
    b: u32,  // 可能有 3-byte padding
}
```

组合规则：

```rust
// 合法组合
#[repr(C, align(8))]
struct AlignedC { x: i32 }

// 合法组合
#[repr(C, packed)]
struct PackedC { a: u8, b: u32 }

// 非法组合（编译错误）
// #[repr(Rust, C)] // 不能同时指定默认布局和 C 布局
// struct Bad { x: i32 }

// #[repr(packed, align(4))] // packed 和 align 一般不共存
```

repr 选择决策树：

| 需求 | 推荐 repr |
|------|-----------|
| 与 C 库交互 | `#[repr(C)]` |
| FFI 零开销包装 | `#[repr(transparent)]` |
| 网络协议/二进制格式 | `#[repr(C, packed)]` |
| 性能敏感+紧凑存储 | 默认 (Rust) 或 `#[repr(packed)]` |
| SIMD / 缓存行对齐 | `#[repr(align(64))]` |

---

## ZST (零大小类型)

零大小类型 (Zero-Sized Type) 在运行时占用零字节内存，编译期保证无开销。常用于状态标记、类型级编程。

> ZST 是形而上的化身——它存在，但不占空间。

三种 ZST 类型：

```rust
// 1. 单元类型
let unit: () = ();

// 2. 无字段结构体
struct PhantomMarker;
let marker = PhantomMarker;

// 3. 无变体枚举（永不构造）
enum Void {}
// let v: Void; // 无法创建实例
```

ZST 的实际用途：

```rust
use std::marker::PhantomData;

// 类型级状态标记
struct Initialized;
struct Uninitialized;

struct Connection<State = Uninitialized> {
    id: u32,
    _state: PhantomData<State>, // ZST——不占内存
}

impl Connection<Uninitialized> {
    fn new(id: u32) -> Self {
        Connection { id, _state: PhantomData }
    }
}

impl Connection<Uninitialized> {
    fn connect(self) -> Connection<Initialized> {
        println!("连接 {} 已建立", self.id);
        Connection { id: self.id, _state: PhantomData }
    }
}

impl Connection<Initialized> {
    fn send(&self, msg: &str) {
        println!("[{}] 发送: {msg}", self.id);
    }
}

fn demo() {
    let conn = Connection::new(1);
    let active = conn.connect(); // Uninitialized -> Initialized
    active.send("hello");
    // active.connect(); // 编译错误：Initialized 没有 connect
}

use std::marker::PhantomData;
```

ZST 内存保证：

```rust
use std::mem;

fn zst_size() {
    assert_eq!(mem::size_of::<()>(), 0);
    assert_eq!(mem::size_of::<PhantomData<String>>(), 0);
    assert_eq!(mem::size_of::<[u8; 0]>(), 0);

    // ZST 数组同样零大小
    assert_eq!(mem::size_of::<[(); 100]>(), 0);

    // ZST 的指针是悬垂的（非零地址但零大小）
    // let dangling: *const () = std::ptr::NonNull::dangling().as_ptr();
}
```

---

## 安全层次全景表

Rust 的安全机制呈多层防御体系，从外到内分别为 Edition 级、编译期静态检查、运行时检查、以及显式 unsafe 豁免区。

> 安全带不是一道，是五道——Rust 把防御编织成了一张天网。

```
层次1: Edition 安全策略 (<-- 最外层)
   |     Edition 2024 强制 unsafe extern
   |
层次2: 类型系统 (<-- 编译期)
   |     所有权、借用检查、生命周期、trait bounds
   |
层次3: Lint 检查 (<-- 编译期 + Clippy)
   |     deny(unsafe_code), clippy::pedantic
   |
层次4: 运行时检查 (<-- debug / release)
   |     数组边界检查、整数溢出检查 (debug)、Mutex poisoning
   |
层次5: unsafe 豁免区 (<-- 最内层)
        开发者承担安全证明责任（安全封装以供上层安全调用）
```

每一层的职责边界：

| 层次 | 典型机制 | 代价 | 违反后果 |
|------|----------|------|----------|
| Edition | `unsafe extern`, never type | 零 | 编译错误 |
| 类型系统 | 所有权/借用/生命周期 | 零 | 编译错误 |
| Lint | `#[deny(unsafe_code)]` | 零 | 编译错误/警告 |
| 运行时 | 边界检查/溢出检查 | 微小性能 | panic |
| unsafe | 原始指针/手动 Drop | 零但风险大 | **未定义行为** |

---

## Edition 2024 迁移建议

迁移到 Edition 2024 需关注安全语义变化。

> 安全是旅程不是目的地——每次 Edition 升级都是向更安全 Rust 迈出的一步。

迁移清单：

```bash
# 1. 查看当前 edition
rg 'edition' Cargo.toml

# 2. 更新为 2024
# Cargo.toml: edition = "2024"

# 3. 运行自动修复
cargo fix --edition --allow-dirty

# 4. 手动检查 unsafe extern
rg '^\s*extern\s*\{' src/

# 5. 运行全量检查
cargo clippy --all-targets --all-features

# 6. 运行测试
cargo test
```

安全检查清单：

| 检查项 | 命令 | 期望 |
|--------|------|------|
| extern 块均标 unsafe | `rg 'extern\s*\{'` | 零输出（extern "C" 为合法形式） |
| 无隐式 ABI | `cargo clippy` | 无 `missing_abi` 警告 |
| MaybeUninit 前后均有 write | 人工审查 | 每个 assume_init 前有 write |
| repr(C) 类型 FFI 对齐 | 审查与 C 头文件的对齐 | 字段偏移一致 |
| unsafe 块有安全文档 | 人工审查 | 每个 unsafe 有解释注释 |

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 忘记在 Edition 2024 中给 extern 块加 unsafe | 旧代码迁移后 extern 块缺少 unsafe 前缀 | 用 `cargo fix --edition` 自动添加 |
| MaybeUninit 跳过 write 直接 assume_init | 读取未初始化内存是未定义行为 | 严格遵循 write -> assume_init 流程 |
| repr(packed) 下取字段引用 | packed 字段可能未对齐，引用要求对齐 | 用 `ptr::read_unaligned` 安全读取 |
| ZST 指针运算 | ZST 的指针不包含实际内存地址 | 避免对 ZST 类型的裸指针进行 offset 运算 |
| 误用 `!` 类型掩盖逻辑错误 | ! 可以让编译器"放过"不完整分支但并非永远正确 | 确保 `!` 的来源分支确实不可达 |
| repr(transparent) 用于多字段结构体 | 仅单字段非 ZST 结构体可用 transparent | 多字段场景改用其它 repr 方案 |
| 静态变量初始化中移动 MaybeUninit | MaybeUninit<T> 不实现 Copy（除 T: Copy 外） | 静态初始化中使用 const 或 transmute 技巧 |

---

> **详见测试**: `tests/rust_features/33_edition_2024_safety.rs`
