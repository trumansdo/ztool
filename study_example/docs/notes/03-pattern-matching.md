# 03 - 模式匹配

## 核心概念

模式匹配是 Rust 最强大的特性之一：

### match 表达式

```rust
match value {
    pattern1 => result1,
    pattern2 => result2,
    _ => default,  // 通配符
}
```

### 模式类型

- **字面量**: `1`, `"hello"`, `true`
- **范围**: `1..=5`, `'a'..='z'`
- **枚举**: `Option::Some(x)`, `Result::Ok(_)`
- **元组**: `(a, b, c)`
- **结构体**: `Point { x, y }`
- **或**: `A | B | C`
- **@ 绑定**: `x @ 1..=10`

### 匹配守卫

```rust
match value {
    Some(x) if x > 0 => { /* ... */ }
    _ => { /* ... */ }
}
```

## 解构

### 元组解构

```rust
let (a, b, c) = (1, 2, 3);
```

### 结构体解构

```rust
let Point { x, y } = point;
let Point { x: px, y: py } = point;
```

### 嵌套解构

```rust
let Some(Some(x)) = nested;
```

## 避坑指南

1. **穷尽性**: match 必须覆盖所有可能
2. **绑定模式**: `ref` 借用，`mut` 可变借用
3. **优先级**: 模式按声明顺序匹配

## 单元测试

详见 `tests/rust_features/03_pattern_matching.rs`

## 参考资料

- [Rust Book - 模式语法](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html)
- [Pattern Matching RFC](https://rust-lang.github.io/rfcs/3637-guard-patterns.html)
