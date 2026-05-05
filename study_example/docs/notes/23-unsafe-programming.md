# Unsafe Rust 编程

> Unsafe Rust 不是对安全 Rust 的背叛，而是它的基石——标准库中那些被千锤百炼的安全抽象，底层都是由 unsafe 代码精心搭建的。

## 1. Unsafe Rust 五大超能力

在 unsafe 块中，你可以做五件安全 Rust 中不能做的事：

| 超能力 | 说明 | 风险等级 |
|--------|------|----------|
| 解引用裸指针 | 读取或修改裸指针指向的值 | 高 |
| 调用 unsafe 函数 | 包括 extern FFI 函数 | 中 |
| 访问/修改可变静态变量 | 读取或写入 static mut | 高 |
| 实现 unsafe trait | 如 Send、Sync | 中 |
| 访问 union 字段 | 读取联合体的字段 | 中 |

```rust
fn main() {
    let x = 42;
    let ptr: *const i32 = &x;

    unsafe {
        // 超能力1：解引用裸指针
        println!("裸指针值: {}", *ptr);

        // 超能力2：调用 unsafe 函数
        do_unsafe_stuff();
    }

    // 以下在 unsafe 块外非法：
    // println!("{}", *ptr); // 错误！
}

unsafe fn do_unsafe_stuff() {
    println!("执行了不安全操作");
}
```

> unsafe 关键字的作用不是"关闭类型检查"，而是向编译器承诺："我已人工验证了以下操作的安全性，请信任我。"

## 2. 裸指针

### 2.1 *const T vs *mut T

```rust
fn main() {
    let mut value = 100;

    // 创建裸指针（安全操作）
    let const_ptr: *const i32 = &value;
    let mut_ptr: *mut i32 = &mut value;
    let null_ptr: *const i32 = std::ptr::null();

    // 解引用裸指针（unsafe 操作）
    unsafe {
        println!("const_ptr: {}", *const_ptr);
        *mut_ptr = 200;
        println!("mut_ptr: {}", *mut_ptr);
    }
}
```

| 特性 | 引用 `&T` / `&mut T` | 裸指针 `*const T` / `*mut T` |
|------|---------------------------|-----------------------------------|
| 创建 | 安全 | 安全 |
| 解引用 | 安全 | **unsafe** |
| 生命周期 | 有编译器检查 | **无** |
| 借用规则 | 强制执行 | **不执行** |
| 可为空 | 否 | 是 (null) |
| 自动解引用 | 是 | 否 |
| 内存大小 | 8 字节 (64位) | 8 字节 (64位) |

> 裸指针和引用的内存布局完全相同——但裸指针脱去了编译器施加的所有安全约束，你手中的不是"更自由的引用"，而是"失去了全部保护的引用"。

### 2.2 裸指针与引用的转换

```rust
fn main() {
    let mut x = 5;

    // 引用 → 裸指针（安全，隐式转换）
    let r1: *const i32 = &x;
    let r2: *mut i32 = &mut x;

    // 裸指针 → 引用（unsafe）
    unsafe {
        let ref1: &i32 = &*r1;
        let ref2: &mut i32 = &mut *r2;
        *ref2 = 10;
    }
}
```

## 3. 指针运算

### 3.1 offset 与 wrapping_offset

```rust
fn main() {
    let data = [10, 20, 30, 40, 50];
    let ptr = data.as_ptr();

    unsafe {
        // offset: 越界即 UB
        println!("ptr[0]: {}", *ptr);                // 10
        println!("ptr[2]: {}", *ptr.offset(2));      // 30

        // add / sub (1.47+)
        println!("ptr[4]: {}", *ptr.add(4));          // 50
        println!("ptr[4] 之前: {}", *ptr.add(4).sub(1)); // 40

        // wrapping_offset: 不产生 UB（但不保证有效）
        let far = ptr.wrapping_offset(1000);
        // 读写 far 很可能是 UB
    }
}
```

### 3.2 ptr::read / ptr::write

```rust
fn main() {
    let mut x = 42i32;
    let mut y = 0i32;

    unsafe {
        // 在不触发 Drop 的情况下读取
        let val = std::ptr::read(&x);
        println!("read: {}", val);

        // 在不触发 Drop 的情况下写入（覆盖旧值）
        std::ptr::write(&mut y, val);
        println!("write 后: {}", y);
    }
}
```

### 3.3 ptr::copy

```rust
fn main() {
    let src = [1u8, 2, 3, 4, 5];
    let mut dst = [0u8; 5];

    unsafe {
        // 逐字节复制（非重叠区域）
        std::ptr::copy(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    println!("目标数组: {:?}", dst); // [1, 2, 3, 4, 5]
}
```

> `ptr::copy` 和 `ptr::write` 是最底层的类型无关内存操作——它们不关心 T 是什么，不调用 Drop，不检查边界。这就是 unsafe 的"自由度"代价。

### 3.4 ptr::drop_in_place

```rust
fn main() {
    let mut s = String::from("将被销毁");
    let ptr: *mut String = &mut s;

    unsafe {
        // 仅调用 Drop，不释放内存
        std::ptr::drop_in_place(ptr);
    }

    // s 现在处于未定义状态（已被 drop）
    // 不能让 s 再次离开作用域（双重释放）
    std::mem::forget(s); // 阻止再次 drop
}
```

## 4. transmute 的危险性

```rust
fn main() {
    // transmute: 在位级上重新解释类型
    let bytes: [u8; 4] = [0x00, 0x00, 0x80, 0x3F];
    let float: f32 = unsafe { std::mem::transmute(bytes) };
    println!("解释为 f32: {}", float); // 1.0

    // 危险示例：以下可以编译但绝对错误
    // let s: String = unsafe { std::mem::transmute(42u64) };
    // ↑ 会将整数 42 解释为 String 的指针/长度/容量，严重 UB
}
```

### 4.1 transmute 的安全替代

```rust
fn main() {
    // 安全方式 1：类型转换
    let x: u32 = 42;
    let y: i64 = x as i64;

    // 安全方式 2：from_le_bytes / from_ne_bytes
    let bytes = [0x41u8, 0x00, 0x00, 0x00];
    let val = u32::from_le_bytes(bytes);
    println!("{}", val); // 65

    // 安全方式 3：pointer cast + ptr::read
    let val: u32 = 0x3F800000;
    let float_val: f32 = f32::from_bits(val);
    println!("{}", float_val); // 1.0
}
```

> transmute 是 Rust 中最危险的操作之一——它让编译器完全闭嘴。能用 `from_bits`、`from_ne_bytes` 或 `as` 转换时，永远不要碰 transmute。

## 5. 可变静态变量

```rust
static mut COUNTER: u32 = 0;

unsafe fn increment_counter() {
    COUNTER += 1;
}

unsafe fn read_counter() -> u32 {
    COUNTER
}

fn main() {
    unsafe {
        increment_counter();
        increment_counter();
        println!("计数: {}", read_counter()); // 2
    }
}
```

> `static mut` 在多线程环境中是彻底的灾难——没有任何同步保证。现代 Rust 倾向于用 `Atomic*` 或 `Mutex` 替代它。

## 6. unsafe trait

```rust
// 定义一个需要实现者保证特定不变量的 trait
unsafe trait TrustedLen {
    fn trusted_len(&self) -> usize;
}

// unsafe impl：开发者需要人工保证实现正确
struct MySlice<'a, T> {
    data: &'a [T],
}

unsafe impl<'a, T> TrustedLen for MySlice<'a, T> {
    fn trusted_len(&self) -> usize {
        self.data.len()
    }
}
```

## 7. Union 访问

```rust
union MyUnion {
    int_val: i32,
    float_val: f32,
    bytes: [u8; 4],
}

fn main() {
    let u = MyUnion { int_val: 42 };

    unsafe {
        // 访问 union 字段是 unsafe 的
        println!("int: {}", u.int_val);
        // 以下在不同字节序下结果不同
        println!("bytes: {:?}", u.bytes);
    }
}
```

> Union 和 transmute 一样是一柄没有铡刀的铁砧——编译器对你读取的字段值是否正确不提供任何保证，你必须自己维护"哪个字段当前有效"的状态。

## 8. 构建安全抽象

编写 unsafe 代码的核心原则是**封装**——将 unsafe 隔离在最小的安全抽象层内：

```rust
// 不安全实现 + 安全 API
pub struct SafeBuffer {
    data: *mut u8,
    len: usize,
}

impl SafeBuffer {
    pub fn new(len: usize) -> Self {
        let layout = std::alloc::Layout::array::<u8>(len).unwrap();
        let data = unsafe { std::alloc::alloc_zeroed(layout) };
        SafeBuffer { data, len }
    }

    pub fn write(&mut self, index: usize, value: u8) -> Result<(), &'static str> {
        if index >= self.len {
            return Err("索引越界");
        }
        unsafe {
            *self.data.add(index) = value;
        }
        Ok(())
    }

    pub fn read(&self, index: usize) -> Result<u8, &'static str> {
        if index >= self.len {
            return Err("索引越界");
        }
        unsafe { Ok(*self.data.add(index)) }
    }
}

impl Drop for SafeBuffer {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::array::<u8>(self.len).unwrap();
        unsafe {
            std::alloc::dealloc(self.data, layout);
        }
    }
}
```

> 每一行 unsafe 代码上方都应该有一个注释，解释为什么这行代码是安全的——不是为了编译器，而是为了下一个阅读你代码的人。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 引用和裸指针混用时违反借用规则 | 裸指针无视借用规则，可能产生别名可变引用 | 将 unsafe 操作限制在最小范围内，避免与安全引用交错 |
| offset 越界 | offset 的生产性检查只在 debug 模式生效 | 总是手动检查边界；prefer `ptr.add()` + `sub()` |
| 忘记手动 drop | `ptr::write` 覆盖旧值时不会调用旧值的 Drop | 先用 `ptr::drop_in_place` 再 `ptr::write` |
| transmute 不同大小的类型 | 位级转换后读取越界 | 严禁 transmute 不同大小的类型 |
| 从裸指针创建的生命周期过长的引用 | 被引用的内存可能已被释放 | 引用的生命周期不超过裸指针指向数据的实际生命周期 |
| static mut 多线程数据竞争 | 无任何同步保证 | 使用 `Atomic*` 或 `sync::Mutex` 替代 |
| ptr::copy 用于重叠内存 | 使用 copy 在 src 和 dst 重叠时产生 UB | 使用 `ptr::copy_nonoverlapping` 检查非重叠；重叠时用 `ptr::copy` |
| alloc 的内存忘记 dealloc | 泄漏导致内存耗尽（非 UB 但严重） | 为所有手动分配实现 Drop，使用 RAII 封装 |

> **详见测试**: `tests/rust_features/23_unsafe_programming.rs`
