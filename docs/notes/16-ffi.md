# 16 - FFI 外部函数接口

## 核心概念

Rust 可以与其他语言交互：

### extern "C"

```rust
extern "C" {
    fn printf(format: *const c_char) -> c_int;
}
```

### #[no_mangle]

防止名称修饰:

```rust
#[no_mangle]
pub extern "C" fn exported_function() {}
```

### repr(C)

C 布局:

```rust
#[repr(C)]
struct CStruct {
    field: c_int,
}
```

### repr(transparent)

透明包装:

```rust
#[repr(transparent)]
struct Wrapper(Inner);
```

### 函数指针

```rust
type Callback = extern "C" fn(i32) -> i32;
```

### CStr / CString

C 字符串互转:

```rust
use std::ffi::{CStr, CString};
let c_str = CString::new("hello").unwrap();
let ptr = c_str.as_ptr();
```

## 单元测试

详见 `tests/rust_features/16_ffi.rs`
