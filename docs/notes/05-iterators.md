# 05 - 迭代器

## 核心概念

迭代器是 Rust 中处理序列数据的重要抽象：

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

迭代器是**惰性的**，不消费数据直到调用终端操作。

## 适配器方法

返回新迭代器：

- `map()`: 转换元素
- `filter()`: 过滤元素
- `take(n)` / `skip(n)`: 取/跳过元素
- `rev()`: 反向
- `enumerate()`: 附加索引
- `zip(other)`: 合并两个迭代器
- `chain(other)`: 链接
- `flatten()`: 展平嵌套

## 终端操作

消费迭代器返回结果：

- `collect()`: 收集到集合
- `fold(acc, f)`: 归约
- `reduce(f)`: 类似 fold，但返回 Option
- `sum()` / `product()`: 求和/求积
- `find()` / `position()`: 查找
- `any()` / `all()`: 布尔判断
- `count()` / `last()`: 计数/最后元素

## 避坑指南

1. **惰性**: 迭代器方法不执行，直到调用终端操作
2. **生命周期**: 借用迭代器需注意生命周期
3. **性能**: 链式调用可能被优化

## 单元测试

详见 `tests/rust_features/05_iterators.rs`

## 参考资料

- [Iterators in Rust](https://dev.to/francescoxx/iterators-in-rust-2o0b)
