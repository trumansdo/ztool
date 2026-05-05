# 条件编译

> 条件编译不是"有条件的代码"，而是"根据条件，代码本身就是不同的"——它在编译前就已决定了哪些代码进入编译流程，哪些被丢弃。

## 1. #[cfg(...)] 属性与 cfg!() 宏

### 1.1 核心区别

| 维度 | `#[cfg(...)]` | `cfg!(...)` |
|------|---------------|-------------|
| 作用阶段 | 编译时（条件编译） | 运行时（布尔值） |
| 影响 | 代码被编译或丢弃 | 代码始终编译，返回 bool |
| 适用场景 | 平台特定代码、feature 切换 | 运行时检测配置 |
| 典型用法 | `#[cfg(target_os = "windows")]` | `if cfg!(target_os = "windows")` |

```rust
// #[cfg]：此函数仅在 Windows 平台被编译
#[cfg(target_os = "windows")]
fn platform_specific() {
    println!("这是 Windows 平台代码");
}

// cfg!()：所有平台都编译，运行时判断
fn main() {
    if cfg!(target_os = "windows") {
        println!("正在 Windows 上运行!");
    } else if cfg!(target_os = "linux") {
        println!("正在 Linux 上运行!");
    } else {
        println!("其他操作系统");
    }
}
```

> cfg!() 中的代码会被编译，只是不在目标平台执行——所以其中的语法错误在所有平台上都会导致编译失败。

### 1.2 #[cfg] 的各种使用位置

```rust
// 条件编译整个模块
#[cfg(feature = "extra")]
mod extra_features {
    pub fn advanced() { println!("高级功能"); }
}

// 条件编译函数
#[cfg(debug_assertions)]
fn debug_only() {
    println!("仅在 debug 模式下编译");
}

// 条件编译语句（使用 cfg!()）
fn flexible() {
    if cfg!(debug_assertions) {
        println!("debug 模式的活代码");
    }
}

// 条件编译 impl 块
struct MyStruct;

#[cfg(target_os = "linux")]
impl MyStruct {
    fn linux_method(&self) { /* Linux 特有实现 */ }
}

#[cfg(target_os = "windows")]
impl MyStruct {
    fn windows_method(&self) { /* Windows 特有实现 */ }
}
```

## 2. 全部条件谓词

### 2.1 target_os

| 值 | 平台 |
|----|------|
| `"windows"` | Windows |
| `"macos"` | macOS |
| `"linux"` | Linux |
| `"android"` | Android |
| `"ios"` | iOS |
| `"freebsd"` | FreeBSD |
| `"dragonfly"` | DragonFly BSD |
| `"netbsd"` | NetBSD |
| `"openbsd"` | OpenBSD |

### 2.2 target_arch

| 值 | 架构 |
|----|------|
| `"x86"` | x86 32位 |
| `"x86_64"` | x86 64位 |
| `"arm"` | ARM 32位 |
| `"aarch64"` | ARM 64位 |
| `"riscv64"` | RISC-V 64位 |
| `"mips"` | MIPS |

### 2.3 其他谓词

| 谓词 | 示例值 | 说明 |
|------|--------|------|
| `target_family` | `"windows"`, `"unix"`, `"wasm"` | 操作系统家族 |
| `target_env` | `"gnu"`, `"msvc"`, `"musl"` | ABI/环境 |
| `target_endian` | `"little"`, `"big"` | 字节序 |
| `target_pointer_width` | `"32"`, `"64"` | 指针宽度 |
| `target_vendor` | `"apple"`, `"pc"`, `"unknown"` | 供应商 |
| `target_abi` | `""`, `"eabihf"`, `"gnu"` | ABI 子类型 |
| `target_has_atomic` | `"8"`, `"16"`, `"32"`, `"64"` | 原子操作支持 |

```rust
// 多条件组合
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn linux_x64_specific() {
    println!("仅在 Linux x86_64 编译");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_specific() {
    println!("在 Unix-like 平台可用");
}

#[cfg(not(target_os = "windows"))]
fn non_windows() {
    println!("非 Windows 平台");
}
```

> target 谓词集合涵盖了 Rust 支持的所有目标平台——掌握它们等于掌握了一次编写、随处编译的关键。

## 3. feature 标志

### 3.1 Cargo.toml 定义

```toml
[package]
name = "my-app"
version = "0.1.0"

[features]
default = ["std"]
std = []
extra = ["dep:serde"]    # 可选依赖项
advanced = ["extra"]     # advanced 隐含 extra
gpu = []
```

### 3.2 代码中使用 feature

```rust
#[cfg(feature = "std")]
use std::io;

#[cfg(not(feature = "std"))]
use core::fmt;

// 条件编译整个模块
#[cfg(feature = "extra")]
pub mod extra {
    pub fn special_feature() {
        println!("需要 extra feature 才能使用");
    }
}

// 编译时断言
#[cfg(not(any(feature = "gpu", feature = "advanced")))]
compile_error!("至少需要 gpu 或 advanced 特性之一");
```

### 3.3 feature 依赖传递

```toml
[features]
# 激活 serde 的 derive feature
serde-support = ["serde", "serde/derive"]

# 当此 feature 激活时，同时激活依赖的特定 feature
full = ["extra", "advanced", "gpu"]
```

> feature 本质上是一种编译期多态——通过 Cargo.toml 的开关控制哪些代码参与编译，实现"一次编写，按需装配"。

## 4. cfg_attr 用法

```rust
// 条件性地应用属性
#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
#[cfg_attr(not(target_os = "linux"), path = "generic/mod.rs")]
mod platform;

// 条件性地派生 trait
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
struct MyStruct {
    data: String,
}

// 条件性地设置文档属性
#[cfg_attr(docsrs, doc(cfg(feature = "extra")))]
pub fn extra_function() { }

// 条件性地应用多个属性（Rust 1.80+）
#[cfg_attr(target_os = "linux", allow(unused_imports))]
fn some_function() { }

// 条件性地应用工具链 lint
#[cfg_attr(
    feature = "nightly",
    allow(internal_features),
    feature(core_intrinsics)
)]
mod nightly_only { }
```

> cfg_attr 的语法是 `#[cfg_attr(条件, 属性1, 属性2, ...)]`——条件为真时，后面的属性列表被视为直接写在位置上的属性。

## 5. 自定义 cfg

### 5.1 通过 rustc 传入

```bash
# 编译时传入自定义 cfg
rustc --cfg my_feature src/main.rs
rustc --cfg 'version="2.0"' src/main.rs

# Cargo 方式
cargo rustc -- --cfg my_custom_flag
```

```rust
#[cfg(my_feature)]
fn custom_feature_fn() {
    println!("通过 rustc --cfg my_feature 启用");
}

#[cfg(version = "2.0")]
fn version_2_feature() {
    println!("版本 2.0 特性");
}
```

### 5.2 build.rs 中设置

```rust
// build.rs
fn main() {
    // 检测系统特性并设置 cfg
    println!("cargo:rustc-cfg=has_openssl");

    // 设置键值对
    println!("cargo:rustc-cfg=protocol_version=\"3\"");

    // 条件性设置
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-cfg=windows_specific");
    }
}
```

## 6. 逻辑组合

```rust
// all：所有条件都满足
#[cfg(all(target_os = "linux", target_arch = "x86_64", feature = "advanced"))]
fn very_specific() { }

// any：任一条件满足
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_like() { }

// not：条件不满足
#[cfg(not(debug_assertions))]
fn release_only() { }

// 嵌套组合
#[cfg(all(
    any(target_os = "linux", target_os = "macos"),
    not(feature = "no-std"),
    target_pointer_width = "64"
))]
fn complex_condition() { }
```

> 条件组合的自由度不受限制——`all`、`any`、`not` 可以任意嵌套，形成任意复杂的布尔表达式。

## 7. 平台代码组织最佳实践

### 7.1 模块级隔离

```
src/
├── main.rs
├── lib.rs
├── platform/
│   ├── mod.rs         # #[cfg(...)] 条件导出
│   ├── windows.rs     # #[cfg(target_os = "windows")]
│   ├── linux.rs       # #[cfg(target_os = "linux")]
│   └── macos.rs       # #[cfg(target_os = "macos")]
```

```rust
// src/platform/mod.rs
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
```

### 7.2 cfg 门控

```rust
// 安全地定义平台统一接口
pub trait PlatformImpl {
    fn get_home_dir() -> PathBuf;
    fn open_file_dialog() -> Option<PathBuf>;
}

// 每个平台有独立实现文件
```

### 7.3 cfg_attr 派生

```rust
// 同一结构体在不同平台/feature下有不同派生
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(target_os = "linux", derive(Eq, PartialOrd, Ord))]
#[derive(Debug, Clone, PartialEq)]
struct CrossPlatformData {
    value: i32,
}
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| cfg!() 中的代码在所有平台编译 | cfg!() 只返回运行时 bool，内中代码必须所有平台有效 | 平台特定的代码用 `#[cfg]` 而非 `cfg!()` |
| feature 名在代码中用了但在 Cargo.toml未定义 | 编译器不会报错，但 feature 始终为 false | 用 `cargo check` 确认 feature 定义完整性 |
| 多个 `#[cfg_attr]` 顺序冲突 | 后面的属性不覆盖前面，而是叠加 | 理解属性累加规则，或将冲突条件合并到一个 cfg_attr |
| feature 依赖未传递 | Cargo feature 默认不自动传递到依赖 | 在 Cargo.toml 中显式声明 `dep:xxx/feature` |
| 条件编译代码在 IDE 中灰显 | IDE 可能按当前平台的 cfg 评估 | 理解 IDE 显示与实际编译的差异，用 `cargo expand` 验证 |
| target_family 和 target_os 混淆 | `unix` 是 family，`linux` 是 os | 区分场景：family 用于大类判断，os 用于具体平台 |
| 条件编译导致未使用导入警告 | 只在某些 cfg 下才用到导入 | 使用 `#[allow(unused_imports)]` 或 `use xxx as _` |

> **详见测试**: `tests/rust_features/27_conditional_compilation.rs`
