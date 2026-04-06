# 10 - 宏与元编程

## 核心概念

### 声明宏 macro_rules!

```rust
macro_rules! my_macro {
    ($name:expr) => {
        println!("{}", $name);
    };
    ($name:expr, $($rest:expr),*) => {
        // 多个参数
    };
}

my_macro!("hello");
```

### 片段说明符

| 说明符 | 匹配 |
|--------|------|
| `expr` | 表达式 |
| `stmt` | 语句 |
| `pat` | 模式 |
| `ty` | 类型 |
| `ident` | 标识符 |
| `path` | 路径 |
| `block` | 代码块 |
| `tt` | Token Tree |
| `meta` | 属性内容 |

### 重复修饰符

- `$($item:expr),*` - 逗号分隔
- `$($item:expr);*` - 分号分隔

### 内置宏

- `println!` / `format!`: 格式化
- `vec!`: 创建 Vec
- `panic!`: 触发 panic
- `assert!`: 断言
- `stringify!`: 转字符串
- `concat!`: 连接字面量
- `env!` / `option_env!`: 环境变量
- `include!`: 包含文件

### derive 宏

```rust
#[derive(Debug, Clone, Copy)]
struct Point { x: i32, y: i32 }
```

## 卫生性

宏展开时，宏内部的标识符不会与外部冲突。

## 单元测试

详见 `tests/rust_features/10_macros.rs`
