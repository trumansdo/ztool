# 00 - 字符串类型 String 与 &str

## 核心概念

Rust 中的字符串是 UTF-8 编码的字符序列，有两种主要类型：

- **String**: 拥有所有权的动态字符串，可变长
- **&str**: 字符串切片，借用自其他字符串

## String vs &str

| 特性 | String | &str |
|------|--------|------|
| 所有权 | 拥有数据 | 借用数据 |
| 大小 | 动态 | 固定(指针+长度) |
| 可变性 | 可变 | 不可变 |
| 用途 | 数据存储 | 函数参数/切片 |

## 主要操作

### 创建 String

```rust
let s = String::new();           // 空字符串
let s = String::from("hello");   // 从 &str 创建
let s = "hello".to_string();     // to_string 方法
let s = format!("{} {}", a, b);  // 格式化
```

### 字符串切片

```rust
let s = String::from("hello world");
let slice = &s[0..5];            // &str 切片
let slice = &s[..5];            // 从开头
let slice = &s[6..];            // 到结尾
```

### 常用方法

- `push_str()`: 追加字符串
- `push()`: 追加字符
- `len()`: 字节长度(非字符数)
- `chars()`: 按字符迭代
- `bytes()`: 按字节迭代
- `contains()`: 子串查找
- `starts_with()` / `ends_with()`: 前缀/后缀
- `split()`: 分割
- `replace()`: 替换
- `trim()`: 去除空白

## 避坑指南

1. **索引操作**: 不能用整数索引 String，需使用切片
2. **中文处理**: `len()` 返回字节数，非字符数
3. **UTF-8**: Rust 字符串必须是有效 UTF-8

## 单元测试

详见 `tests/rust_features/00_strings.rs`

## 参考资料

- [Rust String 与 &str 基础教程](https://cloud.tencent.com/developer/article/2484991)
