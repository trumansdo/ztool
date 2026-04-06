# 01 - 数组与动态数组 Vec

## 核心概念

### 数组 [T; N]

- 固定长度，栈分配
- 编译时确定大小
- 默认不可变

```rust
let arr: [i32; 3] = [1, 2, 3];
let arr = [0; 10];  // 重复初始化
```

### 切片 &[T]

- 动态大小的引用类型
- 指向数组或 Vec 的一部分
- 不拥有数据

```rust
let slice = &arr[1..3];
```

### Vec<T> 动态数组

- 堆分配，可变长
- 自动扩容

```rust
let mut v = Vec::new();
v.push(1);
let v = vec![1, 2, 3];
```

## 主要操作

### Vec 方法

- `push()`: 末尾添加
- `pop()`: 末尾删除
- `insert()` / `remove()`: 指定位置操作
- `len()` / `capacity()`: 长度和容量
- `with_capacity(n)`: 预分配容量

### 切片方法

- `chunks(n)`: 按固定大小分组
- `windows(n)`: 滑动窗口
- `split_at(index)`: 指定位置分割

## 避坑指南

1. **数组索引**: 超出范围会 panic
2. **容量**: Vec 容量翻倍扩容，有性能开销
3. **切片**: 索引是左闭右开区间
4. **生命周期**: 切片必须 outlive 原数据

## 单元测试

详见 `tests/rust_features/01_arrays_and_vecs.rs`

## 参考资料

- [Rust By Example - 数组与切片](https://doc.rust-lang.org/rust-by-example/zh/primitives/array.html)
- [理解 Rust 中的数组、切片、Vector、Map](https://blog.wangjunfeng.com/post/2025/rust-array-slice-vector/)
