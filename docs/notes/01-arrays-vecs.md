# 01 - 数组与动态数组 Vec

## 概述

Rust 用 3 种类型来表示内存中的值序列：数组（Array）、切片（Slice）和 Vector（Vec）。理解它们之间的区别和适用场景，对于编写高效、安全的 Rust 代码至关重要。

## 数组 [T; N]

数组是固定长度的值序列，在编译时已知其值，存储在栈上而非堆上。

### 特点

- **固定长度**：数组的长度必须在编译时已知，且不可改变
- **栈分配**：数组存储在栈上，访问速度快
- **类型标注**：类型签名包含长度信息，如 `[i32; 3]`

### 创建数组

```rust
fn main() {
    // 基本数组
    let a = [1, 2, 3, 4];
    assert_eq!(a.len(), 4);

    // 指定类型
    let arr: [i32; 3] = [1, 2, 3];

    // 重复初始化
    let arr = [0; 10]; // 创建包含10个0的数组

    // 字节数组
    let byte_array = [0u8; 64]; // 64字节的0数组
}
```

### 注意

数组虽然快，但长度固定，**不能**像 `Vec` 那样动态调整大小。

## 切片 &[T]

切片是任意长度的值序列，表示内存中连续区域的一种引用（指针+长度）。

### 特点

- **任意长度**：切片可以是可变长度的，在运行时确定
- **借用**：切片不拥有数据，只是引用
- **连续内存**：指向数组或 Vec 的一部分

### 创建一个切片

```rust
fn main() {
    let array = [0u8; 64]; // 数组
    let slice: &[u8] = &array; // 从数组借用

    // 分割并借用一个切片两次
    let (first_half, second_half) = slice.split_at(32);
    println!(
        "first_half.len() = {} second_half.len() = {}",
        first_half.len(),
        second_half.len()
    );
}
```

`split_at()` 函数解构了切片，并返回两个不重叠的切片。这种模式常用于解析或解码文本或二进制数据。

```rust
fn main() {
    let wordlist = "one,two,three,four";
    for word in wordlist.split(',') {
        println!("word = {}", word);
    }
}
// 输出:
// word = one
// word = two
// word = three
// word = four
```

### 切片的巧妙用法

切片的一个重要特性是：可以**多次借用**相同的切片或数组，因为切片不重叠。这是一个常见的分治策略。

```rust
fn process_data(data: &[u8]) -> u32 {
    // 分治策略：递归处理
    if data.len() == 0 {
        return 0;
    }
    if data.len() == 1 {
        return data[0] as u32;
    }

    let mid = data.len() / 2;
    let (left, right) = data.split_at(mid);
    process_data(left) + process_data(right)
}
```

## Vector

Vector 是 Rust 中最重要的数据类型，用于存储需要动态调整大小的值序列。

### 特点

- **堆分配**：Vector 在堆上分配内存
- **可调整大小**：可以在运行时添加或删除元素
- **内部优化**：有过渡分配的优化

### 创建 Vector

```rust
fn main() {
    // 方式1：Vec::new()
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);

    // 方式2：vec! 宏
    let v = vec![1, 2, 3];

    // 方式3：with_capacity 预分配
    let mut v = Vec::with_capacity(100);
    v.push(1);
    v.push(2);

    println!("v = {:?}", v);
    println!("len = {}, capacity = {}", v.len(), v.capacity());
}
```

### Vec 的 Deref

`Vec` 实现了 `Deref` trait，可以自动转换为切片。这意味着 `Vec` 的方法也是切片的方法：

```rust
fn main() {
    let mut vec = vec![1, 2, 3];
    let slice = vec.as_slice(); // 返回 &[T]

    // Vec<T> deref 为 &[T]
    let first = &vec[0..2];
}
```

### Vec 与借用

```rust
fn main() {
    let mut vec = vec![1, 2, 3];
    let slice = vec.as_slice(); // 借用

    // vec.resize(10, 0); // 这会编译失败！

    // 错误：cannot borrow `vec` as mutable because it is also borrowed as immutable
}
```

因为切片是对 Vec 的借用，所以在切片存在时不能修改 Vec。

### 包装 Vec

许多 Rust 内置类型只是包装了 `Vec`：

```rust
pub struct String {
    vec: Vec<u8>, // String 本质上是 Vec<u8>
}
```

`String` 解引用为 `str`（即 `&str`）。

## 三者对比与选择

| 特性 | 数组 [T; N] | 切片 &[T] | Vector Vec<T> |
|------|--------------|----------|----------|
| 长度 | 固定 | 动态 | 动态 |
| 内存位置 | 栈 | 借用 | 堆 |
| 所有权 | 拥有 | 借用 | 拥有 |
| 大小 | 编译时确定 | 运行时确定 | 运行时确定 |

### 使用建议

1. **数组**：当大小固定且对性能敏感时（栈分配更快）
2. **切片**：需要借用数组或 Vec 的一部分时
3. **Vector**：需要动态大小或未知大小时

## 常用操作

### 数组/切片操作

```rust
fn main() {
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // chunks: 按固定大小分组
    for chunk in arr.chunks(3) {
        println!("chunk = {:?}", chunk);
    }

    // windows: 滑动窗口
    for window in arr.windows(3) {
        println!("window = {:?}", window);
    }

    // split_at: 指定位置分割
    let (first, second) = arr.split_at(5);
    println!("first = {:?}, second = {:?}", first, second);
}
```

### Vector 操作

```rust
fn main() {
    let mut v = vec![1, 2, 3];

    // 添加元素
    v.push(4);
    v.push(5);

    // 插入
    v.insert(0, 0);

    // 删除
    v.pop();
    v.remove(0);

    // 清空
    // v.clear();

    // 扩展
    v.extend(vec![6, 7, 8]);

    println!("v = {:?}", v);
}
```

### copy_from_slice

切片还有一个优化方法 `copy_from_slice()`：

```rust
fn main() {
    let src = [1, 2, 3, 4, 5];
    let mut dst = [0, 0, 0, 0, 0];

    dst.copy_from_slice(&src);
    println!("dst = {:?}", dst); // [1, 2, 3, 4, 5]
}
```

这个方法在底层使用 `memcpy()` 函数，性能优于手动循环。

## Stack vs Heap

理解栈和堆的区别有助于选择合适的数据结构：

- **栈分配**：速度快，但大小固定
- **堆分配**：速度稍慢，但灵活

```rust
fn main() {
    // 栈上分配 - 编译时确定大小
    let array = [0u8; 64];

    // 堆上分配 - 运行时确定大小
    let mut vec = Vec::new();
    vec.push(0);

    // 所有内存都在栈上，没有堆分配 - 没有 malloc 调用
    for word in "one,two,three,four".split(',') {
        println!("word = {}", word);
    }
}
```

## 单元测试

详见 `tests/rust_features/01_arrays_and_vecs.rs`

## 参考资料

- [第3章 | 基本数据类型 | 数组、向量和切片](https://juejin.cn/post/7329573754987986981)
- [Array/Slice/Vector in Rust](http://liubin.org/blog/2021/11/19/rust-array-slice-vector)
- [理解Rust中的数组、切片、Vector、Map](https://blog.wangjunfeng.com/post/2025/rust-array-slice-vector/)
- [Rust By Example - Arrays and Slices](https://doc.rust-lang.org/rust-by-example/zh/primitives/array.html)