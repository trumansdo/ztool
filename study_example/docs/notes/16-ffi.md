# 16 - FFI 外部函数接口

## 概述

FFI (Foreign Function Interface) 允许 Rust 与其他语言（如 C、C++、Python）交互。Rust 的 FFI 支持让它成为系统编程和性能敏感应用的理想选择。

## extern "C"

```rust
extern "C" {
    fn printf(format: *const c_char) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}
```

## #[no_mangle]

防止名称修饰：

```rust
#[no_mangle]
pub extern "C" fn exported_function() -> i32 {
    42
}
```

## repr(C)

C 布局：

```rust
#[repr(C)]
struct CStruct {
    field1: c_int,
    field2: c_float,
}

#[repr(C, packed)]
struct Packed {
    a: u8,
    b: u32,
}
```

## repr(transparent)

透明包装：

```rust
#[repr(transparent)]
struct Wrapper(Inner);

fn rust_to_c(w: Wrapper) -> Inner {
    w.0
}
```

## 函数指针

```rust
type Callback = extern "C" fn(i32) -> i32;

unsafe extern "C" fn my_callback(x: i32) -> i32 {
    x * 2
}

fn register_callback(cb: Callback) { /* ... */ }
```

## CStr / CString

C 字符串互转：

```rust
use std::ffi::{CStr, CString};

let c_str = CString::new("hello").unwrap();
let ptr = c_str.as_ptr();

let from_c = unsafe {
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
};
```

## 链接属性

```rust
#[link(name = "curl")]
extern "C" {
    fn curl_easy_init() -> *mut c_void;
}
```

## 避坑指南

1. **内存管理**：跨边界传递时明确谁负责分配/释放
2. **错误处理**：C 没有 Result，需检查返回值
3. **线程安全**：C 库通常不是线程安全的
4. **panic**：unsafe 代码中 panic 可能导致未定义行为

## 单元测试

详见 `tests/rust_features/16_ffi.rs`

## 参考资料

- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [Rust extern "C"](https://doc.rust-lang.org/reference/items/extern-blocks.html)