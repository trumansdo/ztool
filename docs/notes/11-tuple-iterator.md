# 11 - 元组与迭代器高级操作

## 核心概念

### FromIterator

```rust
impl FromIterator<(A, B)> for (Vec<A>, Vec<B>) {
    fn from_iter<T: IntoIterator<Item = (A, B)>>(iter: T) -> Self {
        // ...
    }
}
```

### unzip

```rust
let pairs = vec![(1, "a"), (2, "b")];
let (nums, chars): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
```

### partition

```rust
let (evens, odds): (Vec<_>, Vec<_>) = 
    (0..10).partition(|x| x % 2 == 0);
```

### reduce vs fold

- `fold`: 需要初始值
- `reduce`: 使用第一个元素作初始值

```rust
let sum = iter.fold(0, |acc, x| acc + x);
let sum = iter.reduce(|acc, x| acc + x);  // 更简洁
```

### try_fold

处理返回 Result 的归约:

```rust
let result = iter.try_fold(0, |acc, x| {
    Ok(acc + x)
});
```

## 单元测试

详见 `tests/rust_features/11_tuple_iterator.rs`
