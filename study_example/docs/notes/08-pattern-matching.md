# 模式匹配

## 一、match 穷举性原理

Rust 的 `match` 要求编译器证明**所有可能性都已覆盖**——这是模式匹配的根基：

```rust
enum Color { Red, Green, Blue }

let c = Color::Red;
match c {
    Color::Red => println!("红色"),
    Color::Green => println!("绿色"),
    Color::Blue => println!("蓝色"),
    // 缺少任何一个变体 → 编译错误
}

// 通配符 _ 作为兜底
match c {
    Color::Red => println!("是红色"),
    _ => println!("不是红色"),
}

// 穷举性的威力：添加新变体自动报告所有未完整处理的 match
// 这是"让编译器替你审查代码"的典型例子
```

> 穷举性检查不是编译器的附加功能，而是类型系统的一线防线——一个没有穷举检查的模式匹配只是语法糖，有了它才是类型导向的架构工具。

### 不可反驳 vs 可反驳模式

| 上下文 | 允许的模式类型 | 示例 |
|--------|----------------|------|
| `let` 语句 | 仅不可反驳 | `let (x, y) = (1, 2);` |
| `let-else` | 可反驳 | `let Some(x) = maybe else { return; }` |
| 函数参数 | 仅不可反驳 | `fn foo((x, y): (i32, i32)) {}` |
| `if let` | 可反驳 | `if let Some(x) = maybe {}` |
| `while let` | 可反驳 | `while let Some(x) = iter.next() {}` |
| `for` 循环 | 不可反驳 | `for (k, v) in map {}` |
| `match` 臂 | 可反驳 | `Some(x) => {}` |

```rust
// 不可反驳模式：总能匹配
let (x, y) = (1, 2);          // 正确
// let Some(z) = None;        // 错误：let 不接受可反驳模式

// 可反驳模式：可能匹配失败
if let Some(z) = None {
    // 不执行
}
```

---

## 二、模式语法大全

Rust 的模式语法极其丰富，下表列出所有模式类型：

| 模式 | 语法 | 说明 | 示例 |
|------|------|------|------|
| 字面量 | `42`, `true`, `'a'` | 匹配固定值 | `1 => {}` |
| 变量 | `x` | 匹配任意值并绑定 | `x => {}` |
| 通配符 | `_` | 匹配任意值，不绑定 | `_ => {}` |
| 绑定 | `name @ pattern` | 匹配并绑定 | `n @ 1..=5 => {}` |
| 解构-元组 | `(a, b, c)` | 分解元组 | `(x, y, z) => {}` |
| 解构-结构体 | `Foo { x, y }` | 分解结构体字段 | `Point { x, y } => {}` |
| 解构-枚举 | `Enum::Variant(inner)` | 解包枚举变体 | `Some(x) => {}` |
| 解构-数组 | `[a, b, c]` | 匹配固定长度数组 | `[1, 2, 3] => {}` |
| 解构-切片 | `[first, rest @ ..]` | 匹配变长切片 | `[h, t@..] => {}` |
| 引用 | `&pattern` | 解引用匹配 | `&val => {}` |
| ref 绑定 | `ref x` / `ref mut x` | 绑定为引用 | `Some(ref x) => {}` |
| 或模式 | `A \| B` | 匹配任一个 | `Ok(v) \| Err(v) => {}` |
| 守卫 | `pattern if cond` | 附加条件 | `x if x > 0 => {}` |
| 范围 | `lo..=hi` / `lo..hi` | 值范围匹配 | `1..=5 => {}` |
| 剩余 | `..` | 忽略剩余字段/元素 | `Point { x, .. } => {}` |

---

## 三、解构模式

解构是模式匹配最强大的能力之一，可以逐层分解复杂数据结构：

```rust
// 元组解构
let pair = (0, -2);
match pair {
    (0, y) => println!("第一个是 0, y = {y}"),
    (x, 0) => println!("x = {x}, 第二个是 0"),
    _ => println!("都不为零"),
}

// 嵌套解构
enum Message {
    Hello { id: i32 },
    Move { x: i32, y: i32 },
}

let msg = Message::Move { x: 10, y: 20 };
match msg {
    Message::Hello { id: id_var } => println!("Hello id: {id_var}"),
    Message::Move { x, y } => println!("移动到 ({x}, {y})"),
}

// 结构体解构：简写与重命名
struct Point { x: i32, y: i32 }
let p = Point { x: 0, y: 7 };
match p {
    Point { x, y: 0 } => println!("在 x 轴上, x = {x}"),
    Point { x: 0, y } => println!("在 y 轴上, y = {y}"),
    Point { x, y } => println!("不在轴上: ({x}, {y})"),
}

// 使用 .. 忽略剩余字段
match p {
    Point { x, .. } => println!("x = {x}"),
}
```

---

## 四、或模式 `|` 与守卫

```rust
let x = 1;
match x {
    1 | 2 => println!("一或二"),
    3 => println!("三"),
    _ => println!("其他"),
}

// | 模式的限制：不可绑定变量
// match x {
//     Some(1) | Some(2) => {}  // 正确
//     Some(a) | Some(b) => {}  // 错误！| 两边不能同时绑定变量
// }

// 守卫作用于整个 | 分支
let num = Some(4);
match num {
    Some(x) if x % 2 == 0 => println!("偶数: {x}"),
    Some(x) => println!("奇数: {x}"),
    None => (),
}

// 组合使用：| 和 if 守卫
match x {
    1 | 2 | 3 if true => println!("1、2 或 3"),
    _ => (),
}
// 等价于
match x {
    (1 | 2 | 3) if true => println!("1、2 或 3"),
    _ => (),
}
```

---

## 五、引用模式 ref

`ref` 在解构时**创建引用绑定**而非移动值：

```rust
let maybe_name = Some(String::from("Alice"));

// 不使用 ref：match 消耗了 maybe_name
match maybe_name {
    Some(name) => println!("{name}"),  // name: String，移出所有权
    None => (),
}
// 之后不能再用 maybe_name

let maybe_name = Some(String::from("Alice"));
// 使用 ref：match 借用
match maybe_name {
    Some(ref name) => println!("{name}"),  // name: &String
    None => (),
}
println!("{:?}", maybe_name);  // 仍可用！

// ref mut：可变引用
let mut maybe_name = Some(String::from("Alice"));
match maybe_name {
    Some(ref mut name) => name.push_str("!"),
    None => (),
}

// 使用 & 匹配：与 ref 的方向相反
// & 解构时从引用中匹配出内部值
let r: &i32 = &42;
match r {
    &val => println!("{val}"),  // val: i32 = 42
}
// 等价于：match *r { val => ... }
```

---

## 六、let-else 语句

```rust
enum Shape {
    Circle(f64),
    Square(f64),
}

impl Shape {
    fn radius_or_default(shape: Option<Shape>) -> f64 {
        let Some(Shape::Circle(radius)) = shape else {
            return 0.0;
        };
        radius  // 在此可直接使用
    }
}

// let-else 的实际应用场景
fn parse_port(input: &str) -> Result<u16, &'static str> {
    let Ok(port) = input.parse::<u16>() else {
        return Err("端口号无效");
    };

    let valid @ 1..=65535 = port else {
        return Err("端口号超出范围");
    };

    Ok(valid)
}

// let-else 链式使用
fn complex_parse(input: &str) -> Option<i32> {
    let first = input.lines().next()?;
    // 等价于：
    let Some(first) = input.lines().next() else { return None; };
    let Ok(num) = first.parse::<i32>() else { return None; };
    Some(num * 2)
}
```

> `let-else` 是早期返回模式的终极抽象——一个关键字替换了整整三行样板代码，而代码越短，bug越少。

---

## 七、matches! 宏

快速返回布尔值的模式匹配：

```rust
let x = Some(5);
assert!(matches!(x, Some(_)));
assert!(matches!(x, Some(1..=10)));
assert!(!matches!(x, None));

// matches! 支持守卫
assert!(matches!(x, Some(v) if v > 0));

// 实际使用：过滤和验证
let data = vec![
    Some(1),
    None,
    Some(2),
    Some(3),
    None,
];
let count = data.iter().filter(|v| matches!(v, Some(x) if x % 2 == 0)).count();
assert_eq!(count, 1);
```

---

## 八、切片模式

切片模式匹配变长数组，语法丰富：

```rust
let arr = [1, 2, 3, 4, 5];

// 精确长度匹配
match arr {
    [1, _, 3, _, _] => println!("匹配精确模式"),
    _ => (),
}

// 前缀/后缀/中间
match &arr[..] {
    [1, rest @ ..] => println!("以 1 开头, 剩余: {rest:?}"),
    _ => (),
}

match &arr[..] {
    [first, middle @ .., last] => {
        println!("首: {first}, 中: {middle:?}, 尾: {last}");
    }
    _ => (),
}

// 多元素前缀
match &arr[..] {
    [a, b, tail @ ..] => println!("前两个: {a}, {b}, 剩余: {tail:?}"),
    _ => (),
}

// 多元素后缀
match &arr[..] {
    [head @ .., x, y] => println!("后两个: {x}, {y}, 前面: {head:?}"),
    _ => (),
}

// 空切片和单元素
match &arr[..] {
    [] => println!("空"),
    [single] => println!("单元素: {single}"),
    [first, second, ..] => println!("前两个: {first}, {second}"),
}

// 字符串切片模式（仅支持固定长度）
let s = "hello";
match &s.as_bytes() {
    [b'h', rest @ ..] => println!("以 h 开头的字节切片"),
    _ => (),
}
```

---

## 九、范围模式

```rust
let x = 5;
match x {
    1..=5 => println!("一到五"),     // 闭区间
    6..10 => println!("六到九"),      // 半开区间（.. 排除右端点）
    _ => println!("其他"),
}

// 仅整数和 char 支持范围模式
let c = 'z';
match c {
    'a'..='m' => println!("前半"),
    'n'..='z' => println!("后半"),
    _ => println!("其他"),
}

// 范围 + 守卫
match x {
    n @ 1..=10 if n % 2 == 0 => println!("1-10 中的偶数: {n}"),
    n @ 1..=10 => println!("1-10 中的奇数: {n}"),
    _ => (),
}
```

---

## 十、深度模式匹配实例

```rust
// 3 层嵌套网络协议解析
enum Packet {
    Http {
        method: Method,
        path: String,
    },
    Tls {
        version: (u8, u8),
        inner: Box<Packet>,
    },
}

enum Method { Get, Post, Put, Delete }

fn route(packet: Packet) -> String {
    match packet {
        Packet::Http { method: Method::Get, path } => {
            format!("GET {}", path)
        }
        Packet::Http { method, path } => {
            format!("{:?} {}", method, path)
        }
        Packet::Tls { version, inner } => {
            match *inner {
                Packet::Http { method: Method::Get, path } => {
                    format!("[TLS {}.{}] GET {}", version.0, version.1, path)
                }
                _ => format!("[TLS {}.{}] 加密数据", version.0, version.1),
            }
        }
    }
}

// 更优雅的写法：用 @ 绑定和深层匹配
fn route_v2(packet: Packet) -> String {
    match packet {
        Packet::Http { method: m @ (Method::Get | Method::Post), path } => {
            format!("{:?} {}", m, path)
        }
        _ => "其他请求".to_string(),
    }
}
```

---

## 十一、@ 绑定

`@` 符号在模式匹配时**同时做测试和绑定**：

```rust
let x = 3;
match x {
    num @ 1..=5 => println!("{num} 在范围内"),    // num = 3
    _ => (),
}

// 深层结构体中使用
struct Point3D { x: i32, y: i32, z: i32 }
let p = Point3D { x: 1, y: 2, z: 3 };
match p {
    pt @ Point3D { x, .. } if x > 0 => {
        println!("正 x 轴上的点: ({}, {}, {})", pt.x, pt.y, pt.z);
        // pt 是整个 Point3D 的绑定
    }
    _ => (),
}

// 枚举变体中的 @
enum Message {
    Hello { id: i32 },
}

let msg = Message::Hello { id: 5 };
match msg {
    msg_ref @ Message::Hello { id: id_val @ 3..=7 } => {
        println!("Hello 消息, id = {id_val} 在 3-7 范围内");
        // msg_ref 是整个枚举值
    }
    _ => (),
}

// ! 注意：@ 绑定同样遵守所有权规则
let opt = Some(String::from("hello"));
match &opt {
    // 使用 &opt 避免消耗所有权
    s @ Some(ref text) => {  // s: &Some<String>, text: &String
        println!("{text}");
        println!("{:?}", s);
    }
    None => (),
}
```

> `@` 绑定让你既能测试深层的值，又能握住所匹配的整体——就像用显微镜观察的同时记住研究的对象本身。

---

## 十二、绑定模式（自动 ref/deref）

```rust
// match ergonomics: match 自动处理引用
let x: &Option<i32> = &Some(42);

// 传统方式：手动解引用
match *x {
    Some(ref v) => println!("{v}"),  // v: &i32
    None => (),
}

// 现代方式（Rust 2018+）：自动因引用而调整
match x {
    Some(v) => println!("{v}"),  // v: &i32 (自动 ref)
    None => (),
}

// 规则：如果 match 的 scrutinee 是引用，
// 编译器自动在所有模式前添加 & 或 ref
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `match` 遗漏分支 | 穷举性未满足 | 添加缺失分支或使用 `_` 通配 |
| `let` 使用可反驳模式 | let 只接受不可反驳模式 | 改用 `if let` 或 `let-else` |
| `ref` 和 `&` 方向混淆 | `ref` 创建引用，`&` 解构引用 | 记口诀：ref 在左侧创建引用，& 在左侧解构引用 |
| `|` 模式两侧绑定变量 | 或模式两侧变量名不能相同也不能不同 | 提取到外围 let 或在分支内各自绑定 |
| 守卫作用范围是整个 `|` 分支 | `if` 守卫属于整个 `|` 优先级 | 需要各自守卫时拆成两个 match 分支 |
| 切片模式 `[rest @ ..]` 导致编译错误 | `..` 左右变量名不能相同 | 正确语法为 `[h, rest @ ..]` |
| `let-else` 的 else 块不是发散表达式 | else 必须返回 `!` 类型 | 使用 `return`、`break`、`continue` 或 `panic!` |
| 范围模式只有整数和 char 支持 | 浮点数无 `PartialOrd` | 使用 `if` 判等加范围判断 |
| `matches!` 中忘记添加守卫括号 | 宏展开优先级问题 | `matches!(x, Some(v) if v > 0)` |
| 深层解构导致 move | 模式默认移动所有权 | 对复杂结构使用 `ref` 绑定或匹配引用 |

> **详见测试**: `tests/rust_features/08_pattern_matching.rs`
