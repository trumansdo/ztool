# 控制流

## 一、if 是表达式

Rust 中 `if` 不是语句，而是**有返回值的表达式**——这是 Rust 与 C 系语言的根本分歧。

```rust
let condition = true;
let number = if condition { 5 } else { 6 };
// number = 5

// 如果 condition 是 false，number = 6
// 所有分支必须返回相同类型
```

> `if` 是表达式而非语句——这种设计让代码从"做了一件事"变成了"计算了一个结果"，函数式的优雅从控制流开始。

### 条件必须是 bool 类型

```rust
let num = 3;
// if num { ... }       // 错误！i32 不是 bool
if num != 0 {           // 正确：显式比较
    println!("非零");
}

// C/JS/Python 对照：
// C:   if (x)   → 非零为真
// JS:  if (x)   → truthy/falsy
// Python: if x: → 空/0/None 为假
// Rust:  if x   → 必须是 bool，没有隐式转换
```

| 语言 | `if` 可以接受的类型 | 隐式转换 |
|------|---------------------|----------|
| C | 整数、指针 | 非零 → true |
| JavaScript | 任意 | truthy/falsy 规则 |
| Python | 任意 | `__bool__` / `__len__` |
| **Rust** | **仅 `bool`** | **无** |

---

## 二、if let 与链式多分支

```rust
let optional = Some(7);

// if let：模式的简化匹配
if let Some(x) = optional {
    println!("值是: {x}");
} else {
    println!("是 None");
}

// 链式多分支（等价于 match）
let number = 13;
let size = if number < 0 {
    Size::Small
} else if number < 10 {
    Size::Medium
} else if number < 20 {
    Size::Large
} else {
    Size::Huge
};

// else if 和 match 的类型统一原理：
// 每个分支都必须返回相同类型，编译器逐一检查
```

---

## 三、loop — 无限循环

`loop` 是 Rust 中专用的无限循环，可以**带返回值**：

```rust
// 基础 loop
let mut counter = 0;
loop {
    counter += 1;
    if counter == 10 {
        break;               // 跳出循环，不返回值
    }
}

// break 带返回值——搜索型循环
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;   // 返回值 20
    }
};
println!("result = {result}"); // 20

// 嵌套循环标签
'outer: loop {
    println!("外层循环");
    'inner: loop {
        println!("内层循环");
        break 'outer;          // 跳出外层循环
    }
}
// 不会到达这里
```

`loop` 返回值的类型推导：

```rust
let x = loop {
    break 42;   // 编译器推断 x: i32
};
let y = loop {
    break 42;
    // break "hello";  // 错误：不兼容的 break 类型
};
```

> `loop` 是 Rust 中最诚实的无限循环——`while true` 可能在编译器眼里只是"可能无限的循环"，而 `loop` 明明白白地告诉编译器"我永远不会自然结束"。

---

## 四、while 循环

前条件循环，与 C 的区别在于 Rust 条件同样必须是 `bool`：

```rust
let mut n = 0;
while n < 5 {
    println!("{n}");
    n += 1;
}

// while let：模式匹配驱动的循环
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("弹出: {top}");  // 3, 2, 1
}

let mut iter = [1, 2, 3].into_iter();
while let Some(x) = iter.next() {
    println!("{x}");
}

// Result 消费
let mut results = vec![Ok(1), Err("oops"), Ok(3)];
while let Some(Ok(val)) = results.pop() {
    println!("成功值: {val}");
}
```

---

## 五、for...in 循环

`for...in` 是 Rust 中最常用的迭代器语法，底层展开为 `IntoIterator::into_iter()`：

```rust
let v = vec![1, 2, 3];

// 三模式：
for x in v.into_iter() { ... }   // 消耗集合，x: T
for x in v.iter() { ... }        // 不可变借用，x: &T
for x in v.iter_mut() { ... }    // 可变借用，x: &mut T

// 底层等效
let mut iter = v.into_iter();
while let Some(x) = iter.next() {
    // ...
}
```

### Range 类型全集

```rust
for i in 0..5 { }          // 0, 1, 2, 3, 4（半开区间）
for i in 0..=5 { }         // 0, 1, 2, 3, 4, 5（闭区间）
for i in (0..5).rev() { }  // 4, 3, 2, 1, 0（反转）
for i in (0..10).step_by(2) { }  // 0, 2, 4, 6, 8

// 底层原理：Range 实现了 IntoIterator
// 0..5   → Range<i32>
// 0..=5  → RangeInclusive<i32>
// (0..5).rev() → Rev<Range<i32>>

// Range 是 Copy 类型
let r = 0..10;
for _ in r.clone() { }  // r 仍在用

// char 范围
for c in 'a'..='z' {
    print!("{c} ");  // a b c ... z
}
```

---

## 六、match 表达式

`match` 具备**穷举性检查**——编译器确认所有可能性都已处理：

```rust
enum Coin { Penny, Nickel, Dime, Quarter }

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
        // 缺少分支 → 编译错误
    }
}

// 绑定模式：提取枚举内部数据
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}

fn process(msg: Message) {
    match msg {
        Message::Quit => println!("退出"),
        Message::Move { x, y } => println!("移动到 ({x}, {y})"),
        Message::Write(text) => println!("写入: {text}"),
    }
}

// 守卫条件
let num = Some(4);
match num {
    Some(x) if x < 5 => println!("小于 5: {x}"),
    Some(x) => println!("大于等于5: {x}"),
    None => (),
}

// 赋值给变量
let x = Some(5);
let y = match x {
    Some(val) => val,
    None => 0,
};
// y = 5，类型为 i32
```

> `match` 的穷尽性是类型系统对你的保护——编译器替你想到了所有你没想到的边界条件。一个遗漏的分支意味着一个潜在的运行时崩溃。

---

## 七、let-else 语句

`let-else` 将模式匹配失败时的处理**内联**，避免深层嵌套：

```rust
// 传统写法：深层嵌套
if let Some(val) = optional {
    // 用 val 做后续处理
} else {
    return;
}

// let-else 写法：扁平化
let Some(val) = optional else {
    return;  // 必须发散：break/continue/return/panic!
};
// val 在这里可用

// 实际用例
let Ok(config) = std::fs::read_to_string("config.toml") else {
    eprintln!("无法读取配置文件");
    return;
};

let Some(user) = find_user() else {
    panic!("用户不存在");
};

// else 块必须是发散表达式（! 类型）
// let Some(x) = maybe else { 42 };  // 错误！42 不是 !
```

---

## 八、? 操作符

`?` 是 Rust 错误传播的核心语法：

```rust
use std::{fs::File, io::{self, Read}};

fn read_username() -> Result<String, io::Error> {
    let mut file = File::open("hello.txt")?;   // 等价于下面的代码
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Ok(username)
}

// ? 的等价展开：
// let mut file = match File::open("hello.txt") {
//     Ok(file) => file,
//     Err(e) => return Err(e.into()),  // From 转换链：io::Error → io::Error
// };

// ? 也支持 Option
fn last_char(s: &str) -> Option<char> {
    s.lines().next()?.chars().last()
    // 如果 next() 返回 None，直接返回 None
}

// ? 自动调用 From::from 转换错误类型
fn do_stuff() -> Result<(), Box<dyn std::error::Error>> {
    let s = File::open("foo.txt")?;  // io::Error → Box<dyn Error>
    Ok(())
}
```

> `?` 操作符是 Rust 错误处理的脊柱——它把错误传播从手动样板简化成一个字符，却保留了所有类型安全和显式语义。

---

## 九、return 的 ! 类型统一

`return` 返回 `!` 类型，使混合分支成为可能：

```rust
fn example(flag: bool) -> i32 {
    let x = if flag {
        42           // i32
    } else {
        return 0;    // ! 类型，强制转换为 i32
    };
    // x = 42 或函数提前返回
    x + 1
}

// 可用于简化 match
let val = match result {
    Ok(v) => v,
    Err(e) => return Err(e),
};
```

---

## 十、循环控制：标签与跳转

```rust
// break 和 continue 都可以带标签跳出多层
'outer: loop {
    for i in 0..10 {
        for j in 0..10 {
            if i * j > 50 {
                break 'outer;  // 跳出外层 loop
            }
            if i + j == 5 {
                continue 'outer;  // 继续外层 loop 的下一轮
            }
        }
    }
}

// 带值跳出
let result = 'search: loop {
    for i in 0..100 {
        if i * i > 1000 {
            break 'search i;
        }
    }
};
```

---

## 十一、控制流的类型状态

所有控制流表达式都影响类型推断：

```rust
// if 的类型
let x = if true { 1 } else { 2 };   // x: i32

// match 的类型
let y = match x {
    1 | 2 => "small",   // 所有分支类型必须一致
    3..=5 => "medium",
    _ => "large",
}; // y: &str

// loop 的类型（break 的值）
let z: i32 = loop {
    break 42;
};

// while 不能有 break 返回值（因为可能不进入循环）
// let w = while true { break 42; };  // 错误
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `if x` 编译失败，x 不是 bool | Rust 不做隐式布尔转换 | 显式 `if x != 0` 或 `if !x.is_empty()` |
| `while let` 循环内 `break` 返回的值类型不统一 | `while` 可能迭代0次 | 使用 `loop` + `if let` 替代 |
| `if/else` 分支类型不一致 | 编译器要求分支返回相同类型 | 统一返回类型，或使用枚举/trait 对象 |
| `let-else` 的 else 块不是发散表达式 | else 块必须返回 `!` | 使用 `return`/`break`/`continue`/`panic!` |
| `match` 遗漏分支导致编译错误 | 穷举性检查 | 添加遗漏分支或使用 `_ =>` 通配 |
| `?` 只能用于返回 `Result`/`Option` 的函数 | `?` 依赖 `From` 和返回类型 | 确保函数返回类型正确，或使用 `match` |
| `for` 循环中修改被迭代的集合 | 迭代器持有借用 | 使用索引循环或收集后再处理 |
| `break` 后代码类型不统一 | 编译器需推断表达式类型 | 确保所有 break 返回值类型相同 |
| `Range` 反转后 start > end 不会 panic 但无迭代 | 空 Range 合法 | 使用 `if start <= end` 守卫 |
| 嵌套循环 `break` 只跳出最内层 | `break` 默认只影响最内层循环 | 使用循环标签 `'label: loop {}` |

> **详见测试**: `tests/rust_features/06_control_flow.rs`
