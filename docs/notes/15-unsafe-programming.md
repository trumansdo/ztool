# 15 - unsafe 编程

## 概述

unsafe Rust 允许绕过 Rust 的安全检查，访问底层系统能力。这带来更大权力也意味着更大责任——程序员必须确保内存安全。

## unsafe 块

```rust
unsafe {
    // 裸指针操作
}
```

## unsafe 函数

```rust
unsafe fn dangerous() {
    // 允许不安全操作
}

pub unsafe fn extern_fn() {}
```

## 裸指针

```rust
let mut num = 5;
let r1 = &num as *const i32;
let r2 = &mut num as *mut i32;

unsafe {
    println!("{}", *r1);
    *r2 = 10;
}
```

## 指针算术

```rust
let arr = [1, 2, 3, 4, 5];
let ptr = arr.as_ptr();

unsafe {
    let ptr2 = ptr.offset(1);
    println!("{}", *ptr2);
}
```

## MaybeUninit<T>

未初始化内存：

```rust
use std::mem::MaybeUninit;

let mut data: MaybeUninit<i32> = MaybeUninit::uninit();
unsafe {
    data.as_mut_ptr().write(42);
}
let value = unsafe { data.assume_init() };
```

## transmute

类型强制转换：

```rust
let bytes: [u8; 4] = unsafe { std::mem::transmute(42u32) };

let val: u32 = unsafe { std::mem::transmute([0, 0, 0, 1]) };
```

## union

```rust
#[repr(C)]
union MyUnion {
    i: i32,
    f: f32,
}

unsafe {
    let mut u = MyUnion { i: 0 };
    u.f = 3.14;
    println!("{}", u.i);
}
```

## NonNull<T>

非空指针：

```rust
use std::ptr::NonNull;

let ptr = NonNull::new(Box::into_raw(Box::new(42))).unwrap();
unsafe {
    println!("{}", *ptr.as_ptr());
}
Box::from_raw(ptr.as_ptr());
```

## 避坑指南

1. **最小化 unsafe**：尽量将 unsafe 代码隔离在小型函数中
2. **文档注释**：标记哪些函数是 unsafe 以及原因
3. **避免 transmute**：类型不兼容的转换是未定义行为
4. **Valid 的数据**：使用 MaybeUninit 后必须正确初始化

## 单元测试

详见 `tests/rust_features/15_unsafe_programming.rs`

## 参考资料

- [Rust Unsafe Code Guidelines](https://rust-lang.github.io/unsafe-code-guidelines/)
- [Rust Book - Unsafe Rust](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)