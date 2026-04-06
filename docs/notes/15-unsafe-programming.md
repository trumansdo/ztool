# 15 - unsafe 编程

## 核心概念

unsafe 代码绕过 Rust 的安全检查：

### unsafe 块

```rust
unsafe {
    // 裸指针操作
}
```

### unsafe 函数

```rust
unsafe fn dangerous() {
    // 允许不安全操作
}
```

### 裸指针

```rust
let mut num = 5;
let r1 = &num as *const i32;
let r2 = &mut num as *mut i32;

unsafe {
    println!("{}", *r1);
}
```

### 指针算术

```rust
unsafe {
    let ptr = arr.as_ptr();
    let next = ptr.offset(1);
}
```

### MaybeUninit<T>

未初始化内存:

```rust
use std::mem::MaybeUninit;
let mut data: MaybeUninit<i32> = MaybeUninit::uninit();
unsafe { data.as_mut_ptr().write(42); }
let value = unsafe { data.assume_init() };
```

### transmute

类型强制转换:

```rust
let bytes: [u8; 4] = unsafe { std::mem::transmute(42u32) };
```

### union

```rust
union MyUnion {
    i: i32,
    f: f32,
}
```

## 单元测试

详见 `tests/rust_features/15_unsafe_programming.rs`
