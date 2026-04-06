# 25 - Edition 2024 安全特性

## 核心概念

Edition 2024 引入多项安全相关变更：

### unsafe extern

Edition 2024 要求 extern 块必须标记 unsafe：

```rust
// 旧版
extern "C" { fn foo(); }

// Edition 2024
unsafe extern "C" { fn foo(); }
```

### 显式 ABI

建议显式指定 ABI：

```rust
unsafe extern "C" { fn foo(); }
```

### #[repr(packed)]

去除字段对齐：

```rust
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,  // 不对齐
}
```

### #[repr(align(n))]

控制对齐：

```rust
#[repr(align(64))]
struct CacheAligned {
    data: [u8; 64],
}
```

### MaybeUninit<T>

未初始化内存：

```rust
let mut data: MaybeUninit<i32> = MaybeUninit::uninit();
unsafe { data.as_mut_ptr().write(42); }
let value = unsafe { data.assume_init() };
```

### Never Type !

发散函数不返回：

```rust
fn panic_fn() -> ! {
    panic!("never returns");
}
```

## 单元测试

详见 `tests/rust_features/25_edition_2024_safety.rs`
