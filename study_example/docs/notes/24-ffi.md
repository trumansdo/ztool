# 外部函数接口 FFI

> FFI 是 Rust 与外部世界对话的舌头——通过它，Rust 可以调用 C 库，也能让 C 代码调用 Rust，把生态边界从一门语言拓展到整个系统层面。

## 1. 调用约定

调用约定(calling convention)定义了函数如何传递参数、返回值，以及调用者与被调用者谁负责清理栈。

| 调用约定 | 说明 | 常见平台 |
|----------|------|----------|
| `"C"` | C 语言默认约定 | 所有平台 |
| `"system"` | 操作系统 API 约定（Windows = stdcall, Unix = C） | 各平台 |
| `"Rust"` | Rust 默认约定（无稳定 ABI） | 所有平台 |
| `"stdcall"` | Win32 API 约定 | Windows |
| `"win64"` | Windows x64 约定 | Windows |

```rust
// 每种调用约定都需要对应的 extern 块
extern "C" {
    fn abs(input: i32) -> i32;
}

extern "system" {
    // Windows API 函数通常使用 system/CDECL/stdcall
}

fn main() {
    unsafe {
        println!("|-3| = {}", abs(-3));
    }
}
```

> "C" 调用约定是 FFI 的通用语言——几乎每种语言都能导出和导入 C 约定的函数，它是互操作的枢纽。

## 2. extern 函数声明与安全包装

### 2.1 声明外部函数

```rust
// 声明来自 C 库的函数
extern "C" {
    // 每个函数声明都必须标记 unsafe
    fn strlen(s: *const std::os::raw::c_char) -> usize;
    fn malloc(size: usize) -> *mut std::os::raw::c_void;
    fn free(ptr: *mut std::os::raw::c_void);
}

fn main() {
    let s = "hello\0";
    unsafe {
        let len = strlen(s.as_ptr() as *const std::os::raw::c_char);
        println!("字符串长度: {}", len); // 5
    }
}
```

### 2.2 安全包装

将 unsafe FFI 调用封装在安全的 Rust 函数中是最佳实践：

```rust
use std::ffi::CString;

// 安全包装：隐藏所有 unsafe 细节
pub fn safe_strlen(s: &str) -> usize {
    let c_string = CString::new(s).expect("字符串中包含空字节");
    unsafe {
        libc::strlen(c_string.as_ptr())
    }
}

pub fn safe_allocate(size: usize) -> *mut u8 {
    unsafe {
        libc::malloc(size) as *mut u8
    }
}

pub fn safe_deallocate(ptr: *mut u8) {
    unsafe {
        libc::free(ptr as *mut std::os::raw::c_void);
    }
}
```

> 安全包装是 FFI 的铁律——外部函数总是 unsafe，但调用它们的用户应该只看到安全的 Rust API。

## 3. #[link] 属性

```rust
// 使用 #[link] 属性指定要链接的库
#[link(name = "m", kind = "dylib")]  // 链接数学库 libm
extern "C" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
}

#[link(name = "z", kind = "static")] // 静态链接 libz.a
extern "C" {
    // zlib 函数
}

fn main() {
    unsafe {
        let angle = std::f64::consts::PI / 2.0;
        println!("sin(PI/2) = {}", sin(angle));
        println!("cos(PI/2) = {}", cos(angle));
    }
}
```

### 3.1 链接属性详解

```rust
#[link(name = "foo",           // 库名
       kind = "static",        // 链接方式：static/dylib/framework
       cfg(target_os = "linux") // 条件链接
      )]
```

| kind | 说明 |
|------|------|
| `"static"` | 静态链接 `.a` / `.lib` |
| `"dylib"` | 动态链接 `.so` / `.dylib` / `.dll` |
| `"framework"` | macOS Framework 链接 |

## 4. C 类型映射

### 4.1 基本类型对照

| Rust 类型 | C 类型 | 说明 |
|-----------|--------|------|
| `std::ffi::c_char` | `char` | 字符 |
| `std::ffi::c_schar` | `signed char` | 有符号字符 |
| `std::ffi::c_uchar` | `unsigned char` | 无符号字符 |
| `std::ffi::c_short` | `short` | 短整型 |
| `std::ffi::c_ushort` | `unsigned short` | 无符号短整型 |
| `std::ffi::c_int` | `int` | 整型 |
| `std::ffi::c_uint` | `unsigned int` | 无符号整型 |
| `std::ffi::c_long` | `long` | 长整型 |
| `std::ffi::c_ulong` | `unsigned long` | 无符号长整型 |
| `std::ffi::c_longlong` | `long long` | 长长整型 |
| `std::ffi::c_ulonglong` | `unsigned long long` | 无符号长长整型 |
| `std::ffi::c_float` | `float` | 单精度浮点 |
| `std::ffi::c_double` | `double` | 双精度浮点 |
| `std::ffi::c_void` | `void` | 空类型 |
| `*const T` / `*mut T` | `const T*` / `T*` | 指针 |

> C 类型映射看似琐碎，实际上是一堵防止 ABI 错配的防火墙——使用 `std::ffi::c_*` 类型能确保你的 FFI 代码跨平台。

### 4.2 复合类型

```rust
// C 结构体 → Rust 结构体
#[repr(C)]        // 必须：确保与 C 结构体内存布局一致
struct CIovec {
    pub iov_base: *mut std::ffi::c_void,
    pub iov_len: usize,
}

// C 联合体 → Rust union
#[repr(C)]
union CUnion {
    int_val: i32,
    float_val: f32,
}

// C 枚举 → Rust 整数常量
const FLAG_READ: i32 = 0x01;
const FLAG_WRITE: i32 = 0x02;
const FLAG_EXEC: i32 = 0x04;
```

## 5. CString / CStr

```rust
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

fn main() {
    // Rust String → CString（添加空终止符）
    let rust_str = "Hello, C World!";
    let c_string = CString::new(rust_str).unwrap();
    let ptr: *const c_char = c_string.as_ptr();

    // C 字符串 → Rust &str（验证 UTF-8 并去除空终止符）
    let cstr: &CStr = unsafe { CStr::from_ptr(ptr) };
    let rust_str_back: &str = cstr.to_str().unwrap();
    println!("往返转换: {}", rust_str_back);

    // 使用 CStr 直接比较
    let expected = CStr::from_bytes_with_nul(b"Hello, C World!\0").unwrap();
    assert_eq!(cstr, expected);
}
```

### 5.1 不安全的两步转换

```rust
use std::ffi::CString;

fn call_external_func(name: &str) {
    // 注意：CString 不能在外部函数调用期间被释放
    let c_name = CString::new(name).unwrap();
    unsafe {
        external_register(c_name.as_ptr()); // 危险！
    }
    // c_name 在此处被 drop，Ptr 悬垂！
}

// 正确做法：确保 CString 的生命周期覆盖所有使用
fn call_safely(name: &str) {
    let c_name = CString::new(name).unwrap();
    unsafe {
        external_register(c_name.as_ptr());
    }
    // 如果 external_register 保存了指针，则 c_name 不能 drop
    std::mem::forget(c_name); // 泄露内存以避免悬垂指针（权衡方案）
}
```

> CString 是 Rust 对 C 空终止字符串的接管——它的生命周期直接决定了 C 指针的有效性，忘记这一点等于埋下了悬垂指针的地雷。

## 6. #[no_mangle] 导出 Rust 函数

```rust
// 导出给 C 调用的 Rust 函数
#[no_mangle]  // 禁止编译器修改函数名
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn greet(name: *const std::ffi::c_char) {
    let cstr = unsafe { std::ffi::CStr::from_ptr(name) };
    let name_str = cstr.to_str().unwrap_or("未知");
    println!("你好, {}!", name_str);
}

#[no_mangle]
pub extern "C" fn create_string() -> *mut std::ffi::c_char {
    let s = std::ffi::CString::new("来自 Rust 的问候").unwrap();
    s.into_raw() // 将所有权转移给调用者，由 C 代码负责 free
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr); // 重新获得所有权并 drop
        }
    }
}
```

> `#[no_mangle]` 不是装饰品——没有它，编译器会将函数名混淆成编译器内部名称，C 代码根本找不到你的函数。

## 7. build.rs 构建脚本

```rust
// build.rs：在编译前执行的构建脚本
fn main() {
    // 指定要链接的本地库
    println!("cargo:rustc-link-lib=foo");

    // 指定库搜索路径
    println!("cargo:rustc-link-search=native=/path/to/libs");

    // 条件链接
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dl");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=CoreFoundation");

    // 传递环境变量给编译器
    println!("cargo:rustc-cfg=has_foo");

    // 重新运行条件：指定的文件变化时重新执行 build.rs
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
```

### 7.1 结合 cc crate 编译 C 代码

```rust
// Cargo.toml
// [build-dependencies]
// cc = "1"

// build.rs
fn main() {
    cc::Build::new()
        .file("src/native/my_lib.c")
        .include("src/native/include")
        .compile("mylib"); // 输出 libmylib.a

    println!("cargo:rerun-if-changed=src/native/my_lib.c");
}
```

## 8. extern 回调函数

```rust
// C 代码调用 Rust 函数作为回调
pub type CallbackFn = extern "C" fn(data: *mut std::ffi::c_void, value: i32);

// 模拟的外部 C 函数：注册回调并触发
extern "C" {
    fn register_callback(cb: CallbackFn, user_data: *mut std::ffi::c_void);
    fn trigger_callback();
}

// Rust 实现的回调函数
extern "C" fn my_callback(data: *mut std::ffi::c_void, value: i32) {
    println!("回调被调用！值: {}", value);
    // data 可以用来传递上下文
}

fn main() {
    unsafe {
        register_callback(my_callback, std::ptr::null_mut());
        trigger_callback();
    }
}
```

## 9. bindgen 绑定生成

bindgen 是一键将 C 头文件转换为 Rust FFI 绑定的工具：

```rust
// build.rs 中使用 bindgen
fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")   // C 头文件
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("无法生成绑定");

    let out_path = std::path::PathBuf::from(
        std::env::var("OUT_DIR").unwrap()
    );
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("无法写入绑定文件");
}

// src/lib.rs
// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

> bindgen 是连接 C 和 Rust 的自动化翻译器——它把"对照头文件手写 FFI"的痛苦工作变成了编译时的自动流程。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| CString 被过早 drop 导致悬垂指针 | `.as_ptr()` 返回的指针生命期绑定到 CString | 确保 CString 的变量在指针使用期间一直存在 |
| 忘记 `#[repr(C)]` | Rust 默认不保证内存布局，C 结构体要求确定布局 | 所有 FFI 结构体加 `#[repr(C)]` |
| `#[no_mangle]` 导致的符号冲突 | 多 crate 导出同名函数 | 使用有意义的前缀命名；或导出时用`#[export_name]` |
| 指针类型用错导致 ABI 不兼容 | Windows `long` 32位，Unix `long` 64位 | 使用 `std::ffi::c_*` 类型而非硬编码 Rust 原生类型 |
| 将 Rust enum（带数据的）传递给 C | C 不理解 Rust enum 的内存布局 | 使用 `#[repr(C)]` 的纯标记 enum 或整数常量 |
| 忘记 `unsafe` 包装 | FFI 函数签名中的 unsafe 被遗漏 | 编译器会报错，但建议在 FFI 函数第一行就加 unsafe 块 |
| build.rs 未声明 rerun-if-changed | 修改 C 代码后不重新编译 | 在 build.rs 中声明所有 C 源文件和头文件 |

> **详见测试**: `tests/rust_features/24_ffi.rs`
