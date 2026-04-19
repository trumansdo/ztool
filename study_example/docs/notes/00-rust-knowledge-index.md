# Rust 语法特性知识体系

本目录收集了 Rust 语法特性的学习笔记，按学习曲线从入门到精通排序。

## 知识模块索引

| 模块 | 文件 | 主要内容 |
|------|------|----------|
| 00 | [00-strings.md](00-strings.md) | 字符串类型 String 与 &str |
| 01 | [01-arrays-vecs.md](01-arrays-vecs.md) | 数组与动态数组 Vec |
| 02 | [02-control-flow.md](02-control-flow.md) | 控制流 if/loop/match |
| 03 | [03-pattern-matching.md](03-pattern-matching.md) | 模式匹配与解构 |
| 04 | [04-error-handling.md](04-error-handling.md) | Option/Result 错误处理 |
| 05 | [05-iterators.md](05-iterators.md) | 迭代器与适配器 |
| 06 | [06-collections.md](06-collections.md) | 集合类型 HashMap/BTreeMap |
| 07 | [07-type-system.md](07-type-system.md) | 类型系统与 trait |
| 08 | [08-generics.md](08-generics.md) | 泛型编程 |
| 09 | [09-smart-pointers.md](09-smart-pointers.md) | 智能指针 Box/Rc/Arc |
| 10 | [10-macros.md](10-macros.md) | 宏与元编程 |
| 11 | [11-tuple-iterator.md](11-tuple-iterator.md) | 元组迭代器与 collect |
| 12 | [12-async-basics.md](12-async-basics.md) | 异步编程基础 |
| 13 | [13-async-closures.md](13-async-closures.md) | async 闭包 |
| 14 | [14-lifetimes.md](14-lifetimes.md) | 生命周期与引用 |
| 15 | [15-unsafe-programming.md](15-unsafe-programming.md) | unsafe 编程 |
| 16 | [16-ffi.md](16-ffi.md) | FFI 与外部函数接口 |
| 17 | [17-file-io.md](17-file-io.md) | 文件与 IO 操作 |
| 18 | [18-let-chains.md](18-let-chains.md) | Let Chains (1.88+) |
| 19 | [19-conditional-compilation.md](19-conditional-compilation.md) | 条件编译 cfg |
| 20 | [20-macro-fragments.md](20-macro-fragments.md) | 宏片段说明符 |
| 21 | [21-if-let-scope.md](21-if-let-scope.md) | if let 临时作用域 |
| 22 | [22-diagnostic-attributes.md](22-diagnostic-attributes.md) | 诊断属性 |
| 23 | [23-precise-capturing.md](23-precise-capturing.md) | 精确捕获 RPIT |
| 24 | [24-reserved-keywords.md](24-reserved-keywords.md) | 保留关键字 |
| 25 | [25-edition-2024-safety.md](25-edition-2024-safety.md) | Edition 2024 安全特性 |
| 26 | [26-latest-features.md](26-latest-features.md) | 最新特性快速参考 |
| 27 | [27-comprehensive.md](27-comprehensive.md) | 综合测试 |

## 学习路径建议

### 入门阶段 (00-06)
- 掌握 Rust 基本数据类型和字符串
- 理解所有权系统基础
- 熟悉控制流和模式匹配
- 学习错误处理方式

### 进阶阶段 (07-12)
- 深入类型系统和 trait
- 掌握泛型编程
- 理解智能指针和内存管理
- 学习异步编程基础

### 高级阶段 (13-17)
- 生命周期和 HRTB
- unsafe 编程和 FFI
- 文件和网络 IO

### 现代特性阶段 (18-27)
- Rust 1.85+ 最新特性
- Edition 2024 新增语法

## 相关资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
