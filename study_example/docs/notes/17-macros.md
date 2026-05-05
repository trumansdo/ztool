# 声明宏 macro_rules!

> 宏不是函数，宏是编译器前端的一个文本替换引擎——它在编译时展开，在语义分析之前就已运行完毕。

## 1. macro_rules! 基本语法

声明宏通过 `macro_rules!` 定义，核心结构为 `(模式) => { 展开结果 }`，支持多个匹配臂：

```rust
macro_rules! say_hello {
    () => {
        println!("你好，世界！");
    };
    ($name:expr) => {
        println!("你好，{}！", $name);
    };
}

fn main() {
    say_hello!();              // 你好，世界！
    say_hello!("张三");        // 你好，张三！
}
```

> 宏的匹配臂从上到下依次尝试，第一个匹配成功的分支即为最终展开结果——这与 match 表达式的语义一致。

多个匹配臂之间用分号分隔，每个臂的左侧是模式(pattern)，右侧是展开模板(template)。展开模板中可以包含 Rust 代码，也可以再次调用宏（形成递归）。

### 1.1 宏的调用语法

声明宏支持三种调用方式：

```rust
// 标准形式
say_hello!("参数");

// 花括号形式（常用于 vec! 等）
say_hello! { "参数" }

// 方括号形式（常用于 cfg_if! 等）
say_hello!["参数"];
```

> 三种调用方式在语义上完全等价，选择哪种取决于代码风格和上下文——花括号适合多行展开，方括号适合配置类宏。

## 2. 片段说明符详解

片段说明符(fragment specifier)是宏模式中的核心概念，它决定了 `$` 占位符可以匹配何种 Rust 语法片段。Rust 共定义了 14 种片段说明符：

| 说明符 | 匹配内容 | 示例 |
|--------|----------|------|
| `ident` | 标识符/关键字 | `x`, `self`, `fn` |
| `expr` | 表达式 | `1+2`, `foo()`, `{...}` |
| `ty` | 类型 | `i32`, `Vec<T>`, `&str` |
| `stmt` | 语句（不含尾部分号） | `let x = 1` |
| `pat` | 模式（edition 2024支持 \|） | `Some(x)`, `1..=10` |
| `pat_param` | 模式（不允许 \|） | `Some(x)`, `1..=10` |
| `item` | 顶层定义项 | `fn`, `struct`, `impl` |
| `meta` | 属性内部内容 | `derive(Debug)`, `test` |
| `tt` | 令牌树（单令牌/配对分隔符） | 任意单独令牌 |
| `block` | 花括号块 | `{ ... }` |
| `vis` | 可见性修饰符 | `pub`, `pub(crate)` |
| `lifetime` | 生命周期标注 | `'a`, `'static` |
| `literal` | 字面量 | `42`, `"hello"`, `true` |
| `path` | 路径 | `std::collections::HashMap` |

```rust
macro_rules! demo_fragment {
    ($id:ident, $e:expr, $t:ty, $p:pat) => {
        let $id: $t = $e;
        match $id {
            $p => true,
            _ => false,
        }
    };
}

fn main() {
    demo_fragment!(x, 5 + 3, i32, 8..=10);
}
```

> 片段说明符是宏世界的类型系统——选错了说明符，匹配就会失败，不会像泛型那样有类型推导的容错空间。

### 2.1 tt 片段的特殊作用

`tt` (token tree) 是最灵活的说明符，它匹配任意单令牌或配对分隔符内的全部内容：

```rust
macro_rules! capture_tt {
    ($($t:tt)*) => {
        // $($t)* 捕获了所有输入令牌
        stringify!($($t)*)
    };
}

fn main() {
    let s = capture_tt!(fn foo() { println!("hello"); });
    println!("{}", s); // fn foo() { println!("hello"); }
}
```

> tt 片段不关心语法语义，只关心令牌层面——它是宏中最"宽容"的匹配工具，也是实现递归宏的核心武器。

## 3. 重复模式

宏的重复模式通过 `$( ... ) 重复操作符 分隔符` 实现，支持三种重复操作符和可选的尾部分隔符：

| 语法 | 含义 |
|------|------|
| `$(...)*` | 零次或多次 |
| `$(...)+` | 一次或多次 |
| `$(...)?` | 零次或一次 |
| `$(...),*` | 逗号分隔的零次或多次 |
| `$(...);+` | 分号分隔的一次或多次 |

```rust
macro_rules! vec_simple {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

fn main() {
    let v = vec_simple![1, 2, 3, 4, 5];
    println!("{:?}", v); // [1, 2, 3, 4, 5]
}
```

> 重复模式中的 `$()` 必须成对出现：模式中定义了几组捕获，展开中就需对应几组重复。

### 3.1 多组重复组合

单组重复中可包含多个捕获变量，它们必须具有相同的重复次数：

```rust
macro_rules! key_value_pairs {
    ( $( $key:expr => $value:expr ),* ) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

fn main() {
    let m = key_value_pairs!("a" => 1, "b" => 2, "c" => 3);
    println!("{:?}", m); // {"a": 1, "b": 2, "c": 3}
}
```

### 3.2 分隔符的使用

分隔符放在 `)` 与重复操作符之间，可以是任意非字母的令牌：

```rust
macro_rules! sum_list {
    ( $($x:expr),+ ) => {
        {
            let mut total = 0;
            $(
                total += $x;
            )+
            total
        }
    };
    ( $($x:expr);+ ) => {
        {
            let mut total = 0;
            $(
                total += $x;
            )+
            total
        }
    };
}
```

> 分隔符选择没有硬性规定，但要保持与 Rust 生态一致——逗号用于元素列表，分号用于语句列表，花括号和方括号跟随标准语法习惯。

## 4. 卫生性

Rust 宏是**卫生的**(hygienic)，这意味着宏内部定义的变量不会与调用处的变量发生非预期的冲突：

```rust
macro_rules! create_var {
    () => {
        let x = 42;
        println!("宏内部: x = {}", x);
    };
}

fn main() {
    let x = 100;
    create_var!();
    println!("外部: x = {}", x);
    // 输出:
    // 宏内部: x = 42
    // 外部: x = 100
}
```

### 4.1 卫生性的三条规则

1. **变量名自动重命名**：宏内部引入的变量名会被编译器自动赋予唯一标识
2. **宏内定义不泄漏**：宏展开中 `let` 的变量不会污染调用作用域
3. **局部变量遮蔽遵循普通规则**：如果宏引用了外部的 `ident`，则按正常作用域规则处理

```rust
macro_rules! use_outer_var {
    ($var:expr) => {
        // 这里的变量名就是字面使用，不做重命名
        println!("{}", $var);
    };
}
```

> 卫生性是 Rust 宏区别于 C 语言宏的关键特性——C 宏的变量名冲突需要靠全大写命名和 `__` 前缀来手动规避。

## 5. 递归宏

递归宏是在展开体中再次调用自身，通常配合 `tt` 片段说明符实现：

```rust
macro_rules! count_tts {
    () => { 0usize };
    ($odd:tt $($rest:tt)*) => {
        1usize + count_tts!($($rest)*)
    };
}

fn main() {
    let n = count_tts!(a b c d e);
    println!("令牌数量: {}", n); // 5
}
```

### 5.1 tt muncher 模式

tt muncher(令牌咀嚼器)是一种递归消费输入令牌的设计模式，每次递归"咀嚼"掉一个或多个前缀令牌，直到消耗完毕：

```rust
macro_rules! html {
    // 终止条件：所有标签处理完毕
    ($($tree:tt)*) => {
        html_inner!($($tree)*)
    };
}

macro_rules! html_inner {
    // 递归基：空输入
    () => { String::new() };
    // 匹配一个标签对
    (<$tag:ident> $($body:tt)* </$tag:ident> $($rest:tt)*) => {
        format!(
            "<{0}>{1}</{0}>{2}",
            stringify!($tag),
            html_inner!($($body)*),
            html_inner!($($rest)*)
        )
    };
    // 匹配纯文本
    ($text:tt $($rest:tt)*) => {
        format!("{}{}", stringify!($text), html_inner!($($rest)*))
    };
}
```

> tt muncher 是声明宏领域最强大的递归工具——它可以把令牌序列当作链表一样遍历处理，代价是编译时间随输入长度增长。

## 6. 内部规则模式

通过 `@` 约定，可以在宏内部定义辅助匹配分支：

```rust
macro_rules! tuple_len {
    // 公共入口：调用内部辅助规则
    ($ty:ty; $($value:expr),*) => {
        tuple_len!(@count $($value),*)
    };
    // 内部计数规则
    (@count $first:expr $(, $rest:expr)*) => {
        1 + tuple_len!(@count $($rest),*)
    };
    (@count) => { 0 };
}

fn main() {
    let len = tuple_len!(i32; 1, 2, 3, 4);
    println!("长度: {}", len); // 4
}
```

> 内部规则使用 `@name` 命名只是社区约定，并非编译器要求——但它能清晰地分离公共 API 和内部实现。

## 7. 过程宏概览

声明宏是"基于模式匹配的代码替换"，而过程宏是"基于 Rust 代码的 AST 操作"。三种过程宏对比如下：

| 类型 | 标记方式 | 调用方式 | 适用场景 |
|------|----------|----------|----------|
| **派生宏** | `#[derive(MyMacro)]` | 自动应用于 struct/enum | Serialize/Deserialize |
| **属性宏** | `#[my_macro]` | 标注在项上 | 路由注解、AOP |
| **函数式宏** | `my_macro!(...)` | 函数式调用 | SQL 查询、HTML 模板 |

> 声明宏解决的是"减少样板代码"的问题，过程宏解决的是"生成在语义上无法简单表达的代码"的问题。

## 8. 常见内置宏家族

```rust
// 集合类
vec![1, 2, 3];              // 创建 Vec
format!("{}+{}={}", a, b, c); // 格式化字符串

// 调试类
println!("值: {}", x);      // 打印到标准输出
dbg!(&complicated_expr);   // 打印表达式和值
eprintln!("错误!");         // 打印到标准错误

// 测试类
assert!(x > 0);             // 断言为真
assert_eq!(a, b);           // 断言相等

// 特殊操作
todo!();                    // 标记未实现
unreachable!();             // 标记不可达
include_str!("file.txt");  // 编译时包含文件内容
compile_error!("stop");    // 编译时报错
cfg!(target_os = "windows")// 编译时条件检查

// 日志类
log::info!("事件");         // 日志宏(需引入 log crate)

// future 相关
ready!(poll_result);        // 轮询就绪宏
```

> 内置宏是编译器本身实现的"特权设施"，声明宏无法做到 `compile_error!` 或 `cfg!` 这种事——它们是编译器的原生能力。

## 9. 宏与函数对比

| 维度 | 宏 | 函数 |
|------|-----|------|
| 求值时机 | 编译时展开 | 运行时调用 |
| 参数类型 | 任意 Rust 语法片段 | 具体类型 |
| 可变参数 | 天然支持 | 需要重载或 trait |
| 代码生成 | 可在调用处生成任意代码 | 只能返回一个值 |
| 命名空间 | 全局（macro_export） | 模块作用域 |
| 调试难度 | 难（展开后代码不可见） | 易（正常调试） |
| 错误消息 | 展开后上下文，晦涩 | 类型系统检查，清晰 |

> 能用函数解决的问题优先用函数——宏是最后的武器，只在函数无法表达可变参数数量或需要在调用处注入代码时才考虑。

## 10. 编译时展开原理

宏展开发生在编译过程的早期阶段：

```
源代码文本
  ↓ 词法分析
令牌流 (TokenStream)
  ↓ 宏展开（多次迭代，直至没有宏调用）
扩展后的令牌流
  ↓ 语法分析
AST（抽象语法树）
  ↓ 语义分析 / 类型检查
HIR → MIR → LLVM IR → 机器码
```

> 宏的展开和类型检查是两个独立阶段——宏中不检查类型，只会报告语法层面的问题。要理解宏，就必须理解这个阶段分离。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 宏内部 `let` 变量与外部冲突 | 以为会冲突，实际卫生性保护了 | 放心在宏内定义变量，但要显式传参引用的标识符 |
| `$($x:expr),*` 要求最后不带逗号 | 逗号是分隔符，尾随逗号导致额外匹配 | 如需支持尾随逗号，多加一个 `$(,)?` |
| 在宏里使用 `return` | return 作用于整个函数，不是宏本身 | 用块表达式返回值，或改为闭包 |
| `tt` 匹配了意料之外的括号 | tt 会把 `[` 到 `]` 整体当作一个令牌 | 明确匹配时用具体的说明符而非 tt |
| 递归宏无限展开 | 缺少递归终止条件 | 总是先写终止分支（空输入或单一条件） |
| `macro_export` 污染全局命名空间 | 导出宏对所有下游 crate 可见 | 确定要导出的才加该属性，内部宏放在模块中 |
| 宏展开错误位置信息不准确 | 编译器报告展开后代码的错，非宏定义处 | 用 `stringify!` 调试，用 `compile_error!` 尽早报错 |

> **详见测试**: `tests/rust_features/17_macros.rs`
