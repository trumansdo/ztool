# 02 - 控制流

## 核心概念

Rust 提供丰富的控制流结构：

### if 表达式

```rust
let x = if condition { value1 } else { value2 };
```

### 循环

- `loop`: 无限循环，需手动 break
- `while`: 条件循环
- `for ... in`: 迭代器循环

```rust
loop { /* ... */ break; }
while condition { /* ... */ }
for item in collection { /* ... */ }
```

### 标签和退出

```rust
'outer: for i in 0..3 {
    for j in 0..3 {
        if condition { break 'outer; }
        if condition { continue 'outer; }
    }
}
```

### return 值

- 最后一个表达式的值作为返回值
- `;` 结尾不返回值

## 避坑指南

1. **if 返回值**: 分支必须返回相同类型
2. **loop 返回值**: `break value` 可返回值
3. **标签**: 嵌套循环时使用标签精确控制

## 单元测试

详见 `tests/rust_features/02_control_flow.rs`

## 参考资料

- [Rust Control Flow](https://runtimepanic.com/rust/control-flow)
- [Control flow - Learn Rust](https://rustfinity.com/learn/rust/the-programming-basics/control-flow)
