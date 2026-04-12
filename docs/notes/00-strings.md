# 00 - 字符串类型 String 与 &str

## 概述

在 Rust 编程语言中，处理文本数据是日常开发的核心部分。与其他许多语言不同，Rust 对字符串的处理方式有着独特的区分，主要体现在两种基本类型上：`String` 和 `&str`（字符串切片）。理解它们之间的差异、联系以及各自的适用场景，对于编写高效、安全且符合 Rust 哲学（尤其是所有权系统）的代码至关重要。

## 核心概念：所有权与借用

在深入字符串之前，必须先理解 Rust 的核心特性：**所有权（Ownership）**和**借用（Borrowing）**。

1. **所有权**：Rust 中的每个值都有一个被称为其**所有者**（owner）的变量。在任何时候，一个值只能有一个所有者。当所有者离开作用域时，该值将被丢弃（dropped），其占用的资源（如内存）会被释放。

2. **借用**：我们可以创建对值的引用（references），这被称为**借用**。借用允许我们在不获取所有权的情况下访问值。借用分为不可变借用（`&T`）和可变借用（`&mut T`）。Rust 强制执行严格的借用规则：在任何给定时间，你要么只能有一个可变引用，要么可以有任意数量的不可变引用，但不能同时拥有。

`String` 和 `&str` 的区别与联系，很大程度上就是所有权和借用规则在字符串类型上的具体体现。

## String：拥有所有权的可变字符串

`String` 类型是 Rust 标准库提供的一个**拥有所有权**的、**堆分配**的、**UTF-8 编码**的、**可增长**的字符串类型。

### 特点

1. **拥有所有权（Owned）**：当你创建一个 `String` 或将 `String` 赋值给一个变量时，该变量就成为了这个字符串数据的所有者。当变量离开作用域时，`String` 会自动释放其在堆上分配的内存。

2. **堆分配（Heap Allocated）**：`String` 的内容存储在内存的堆（heap）上。这意味着 `String` 的大小可以在运行时动态增长或缩小。堆分配涉及一定的开销（分配和释放内存），但提供了灵活性。

3. **可变（Mutable）**：你可以修改一个 `String` 的内容，例如追加字符、插入子串、清空等，前提是该 `String` 变量被声明为可变的（使用 `mut` 关键字）。

4. **UTF-8 编码**：Rust 的 `String` 保证其内容始终是有效的 UTF-8 编码。这意味着它可以表示世界上几乎所有的字符。

5. **结构**：一个 `String` 实例在栈上通常存储三个信息：
   - 指向堆上字节序列的指针（pointer）
   - 字符串当前的长度（length），即包含的字节数
   - 字符串当前的容量（capacity），即在不重新分配内存的情况下可以容纳的总字节数

### 创建 String 的常用方法

```rust
fn main() {
    // 1. 从空字符串创建
    let mut s1 = String::new();
    println!("s1 (empty): '{}', len: {}, capacity: {}", s1, s1.len(), s1.capacity());

    // 2. 从字符串字面量 (&str) 创建
    let data = "initial contents";
    let s2 = String::from(data); // 使用 String::from()
    let s3 = data.to_string(); // 使用 .to_string() 方法 (更通用)

    // 3. 使用 format! 宏创建 (类似其他语言的 sprintf 或 f-string)
    let name = "Alice";
    let age = 30;
    let s5 = format!("My name is {} and I am {} years old.", name, age);
    println!("s5: '{}'", s5);
}
```

### 修改 String

因为 `String` 是可变的（如果用 `mut` 声明），你可以修改它：

```rust
fn main() {
    let mut s = String::from("foo");
    println!("Original: '{}'", s);

    // 追加 &str
    s.push_str("bar");
    println!("After push_str: '{}'", s);

    // 追加单个字符 char
    s.push('!');
    println!("After push: '{}'", s);

    // 替换子串 (replace 返回新的 String，不修改原 String)
    let s_replaced = s.replace("foo", "baz");
    println!("Original s after replace call: '{}'", s);
    println!("s_replaced: '{}'", s_replaced);
}
```

### 所有权转移

当 `String` 被赋值给另一个变量或作为函数参数传递（按值传递）时，所有权会发生转移。

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1 的所有权转移给 s2

    // println!("s1 is: {}", s1); // 编译错误！s1 不再有效，因为所有权已转移
    println!("s2 is: {}", s2); // s2 现在拥有数据

    takes_ownership(s2); // s2 的所有权转移给函数 takes_ownership
    // println!("s2 after function call: {}", s2); // 编译错误！s2 的所有权已被函数拿走
}

fn takes_ownership(some_string: String) {
    println!("Inside function: {}", some_string);
} // 这里 some_string 离开作用域，String 的内存被释放
```

## &str：不可变的字符串切片

`&str` 类型，通常称为"字符串切片"（string slice），是对某处存储的 **UTF-8 编码**字符串数据的**不可变引用**。

### 特点

1. **借用（Borrowed）**：`&str` 本身并不拥有它所指向的字符串数据。它只是一个"视图"或"引用"，指向存储在其他地方（如 `String`、二进制文件的数据段、或其他 `&str`）的有效 UTF-8 字节序列。

2. **不可变（Immutable by default）**：通过 `&str` 引用，你通常不能修改它所指向的字符串数据。这是 Rust 借用规则的一部分，保证了数据安全。

3. **固定大小视图**：`&str` 指向的是一段固定长度的字节序列。它本身的大小在编译时是已知的（通常是一个指针和长度，占用两个 `usize` 的空间）。

4. **UTF-8 编码**：和 `String` 一样，`&str` 也保证指向有效的 UTF-8 数据。

5. **来源多样**：`&str` 可以指向：
   - **字符串字面量**：如 `"hello"`。字符串字面量直接存储在程序编译后的二进制文件中，具有 `'static` 生命周期
   - **`String` 的一部分或全部**：你可以从一个 `String` 中创建一个或多个 `&str` 切片
   - **其他 `&str` 的一部分**

### 创建 &str

```rust
fn main() {
    // 1. 字符串字面量本身就是 &str 类型
    let literal: &str = "Hello, world!";
    println!("Literal: {}", literal);

    // 2. 从 String 创建 &str (借用)
    let s = String::from("你好，Rust");

    // 创建指向整个 String 的 &str
    let s_slice_full: &str = &s;

    // 创建指向 String 部分内容的 &str (切片)
    // 注意：切片索引是基于字节的，需要小心 UTF-8 字符边界
    // "你好，Rust" -> UTF-8 bytes: [E4 BD A0, E5 A5 BD, EF BC 8C, 52, 75, 73, 74]
    // 你: 3 bytes, 好: 3 bytes, ，: 3 bytes, R: 1 byte, u: 1 byte, s: 1 byte, t: 1 byte
    let s_slice_part: &str = &s[0..6]; // [start..end) 半开区间
    println!("Slice of '你好': {}", s_slice_part);

    // 使用 .get() 方法安全获取切片，返回 Option<&str>
    let safe_slice = s.get(0..6);
    match safe_slice {
        Some(slice) => println!("Safe slice: {}", slice),
        None => println!("Invalid slice index."),
    }
}
```

### 字节索引的陷阱！

Rust 的字符串索引是基于字节的，而不是字符。对于多字节的 UTF-8 字符，直接使用索引切片可能会导致程序 panic，因为切片边界落在一个字符的中间。

```rust
fn main() {
    let s = String::from("你好世界");

    // 尝试字节索引 (危险!)
    // let invalid_char_slice = &s[0..1]; // Panic! 1 is not a char boundary

    // 正��方式：迭代字符
    for c in s.chars() {
        println!("Char: '{}'", c);
    }

    // 迭代字节
    for b in s.bytes() {
        println!("Byte: {:02X}", b);
    }
}
```

### &str 的生命周期

由于 `&str` 是一个引用，它必须遵守 Rust 的生命周期规则。`&str` 不能比它所指向的数据活得更长。

- 字符串字面量 (`"hello"`) 具有 `'static` 生命周期，意味着它们在整个程序执行期间都有效，所以对它们的 `&str` 引用也基本上没有生命周期限制。
- 从 `String` 创建的 `&str`，其生命周期不能超过该 `String` 的生命周期。如果 `String` 被移动或销毁，那么指向它的 `&str` 就会变成悬垂引用（dangling reference），Rust 的编译器会阻止这种情况发生。

```rust
fn main() {
    let s = String::from("long string is long");
    let part: &str;

    {
        let s_inner = String::from("short lived");
        // part = &s_inner; // 编译错误！ `s_inner` 在这个作用域结束后就销毁了，但 part 仍然存活

        part = &s[0..4]; // 这是合法的，因为 s 的生命周期比 part 长
    }

    println!("Part from long lived string: {}", part);
} // s 在这里被 drop
```

## String 与 &str 的关系与转换

### 1. 从 String 获取 &str (借用)

这是最常见的操作，而且**非常廉价**。因为 `&str` 只是一个指向 `String` 内部数据的引用（指针+长度），不需要进行内存分配或数据复制。

```rust
fn main() {
    let my_string = String::from("Hello");

    // 获取指向整个 String 的 &str
    let s1: &str = &my_string;

    // 获取指向 String 部分内容的 &str (切片)
    let s2: &str = &my_string[0..3]; // "Hel"

    println!("s1: {}", s1);
    println!("s2: {}", s2);

    // 函数通常接收 &str 参数，这样可以同时接受 String 和 &str
    process_string_slice(s1);
    process_string_slice(&my_string);
    process_string_slice("I am a literal"); // 字符串字面量也是 &str
}

fn process_string_slice(slice: &str) {
    println!("Processing slice: {}", slice);
}
```

### 解引用强制多态（Deref Coercion）

`process_string_slice(&my_string)` 也能工作，即使函数期望 `&str` 而我们传递的是 `&String`。这是因为 Rust 的**解引用强制多态**（Deref Coercion）特性。`String` 实现了 `Deref<Target=str>` trait，这意味着 `&String` 可以被自动、隐式地转换为 `&str`。这是 Rust 设计中的一个巨大便利，使得接受 `&str` 的函数可以无缝地处理 `String` 的引用。

### 2. 从 &str 创建 String (克隆/转换)

当你需要一个拥有所有权、可变的字符串副本时（例如，要修改它或将其传给需要 `String` 的函数），你可以从 `&str` 创建一个新的 `String`。这个操作**涉及内存分配和数据复制**，相对昂贵。

```rust
fn main() {
    let my_slice: &str = "World";

    // 方法 1: 使用 .to_string()
    let mut s1: String = my_slice.to_string();
    s1.push_str("!");

    // 方法 2: 使用 String::from()
    let s2: String = String::from(my_slice);
}
```

## 实践指南：何时使用 String vs &str?

### 使用 String 当：

1. **你需要拥有字符串数据的所有权**。例如，函数需要返回一个新生成的字符串，或者结构体需要存储字符串数据。
2. **你需要在运行时构建或修改字符串**。例如，从用户输入、文件读取或网络请求中动态地组合字符串。
3. **字符串的生命周期需要独立于其原始来源**。如果你从一个临时的 `&str` 创建了 `String`，这个 `String` 可以活得比原始 `&str` 更久。

### 使用 &str 当：

1. **你只需要读取或引用字符串数据，而不需要拥有它**。这是最常见的情况，尤其是在函���参���中。
2. **你想表示字符串字面量**。 `"hello"` 本身就是 `&'static str`。
3. **你想创建一个指向 `String` 或其他 `&str` 的一部分的视图（切片）**。
4. **性能是关键考虑因素，且你不需要修改数据或拥有所有权**。传递 `&str` 避免了不必要的内存分配和复制。

### 黄金法则：函数参数优先使用 &str

除非函数确实需要获取字符串的所有权（比如要存储它或返回修改后的版本），否则**函数参数应尽可能接受 `&str`**。

```rust
// 不推荐：这个函数不必要地获取了所有权
fn process_string_owned(s: String) {
    println!("{}", s);
}

// 推荐：这个函数通过借用工作，更灵活
fn process_string_borrowed(s: &str) {
    println!("{}", s);
}

fn main() {
    let s = String::from("my data");
    let literal = "literal data";

    process_string_borrowed(&s); // 可以接受 &String (通过 Deref Coercion)
    process_string_borrowed(literal); // 可以接受 &str
    process_string_borrowed(&s[3..]); // 可以接受 String 切片

    println!("s still usable here: {}", s); // s 的所有权还在
}
```

通过接受 `&str`，你的函数变得更加通用，可以处理 `String`、字符串字面量以及 `String` 的切片，而无需调用者进行额外的转换或放弃所有权。

## UTF-8 和索引

这是一个关键但有时会引起混淆的点。Rust 的 `String` 和 `&str` 内部都使用 UTF-8 编码。UTF-8 是一种变长编码，意味着一个字符可能占用 1 到 4 个字节。

`'H' -> 1 byte, 'e' -> 1 byte, 'l' -> 1 byte, 'l' -> 1 byte, 'o' -> 1 byte, ' ' -> 1 byte, '世' -> 3 bytes (E4 B8 96), '界' -> 3 bytes (E7 95 8C)`

由于这种变长特性：

1. **`len()` 方法返回的是字节数，而不是字符数**。
2. **直接用整数索引（如 `s[i]`）访问字符是危险且不被允许的**。因为索引 `i` 是字节索引，它可能指向一个多字节字符的中间，这在 UTF-8 中是无效的。
3. **字符串切片 `&s[start..end]` 的索引也是字节索引**。如果 `start` 或 `end` 不是有效的 UTF-8 字符边界，程序会在运行时 panic。

## 总结对比

| 特性 | String | &str |
|------|--------|------|
| **所有权** | 拥有所有权 (Owned) | 借用 (Borrowed) |
| **内存位置** | 数据在堆上 (Heap allocated) | 数据在其他地方 (堆, 栈, 静态内存) |
| **可变性** | 可变 (需 `mut` 声明) | 默认不可变 (Immutable) |
| **大小** | 可增长 (Growable) | 固定大小视图 (Fixed size view) |
| **创建** | `String::new()`, `String::from()`, `.to_string()`, `format!` | 字面量 (`"..."`), 切片 (`&s[..]`) |
| **成本** | 创建/修改可能涉及堆分配/重分配 | 创建 (借用) 非常廉价 (指针复制) |
| **主要用途** | 存储、构建、修改字符串数据 | 引用、查看字符串数据，函数参数 |
| **生命周期** | 由所有者作用域决定 | 不能超过其指向数据的生命周期 |
| **与另一类型转换** | 通过 `&` 或切片廉价得到 `&str` | 通过 `.to_string()` 或 `String::from()` 创建 `String` (涉及分配和复制) |
| **编码** | UTF-8 | UTF-8 |

## 单元测试

详见 `tests/rust_features/00_strings.rs`

## 参考资料

- [Rust String 与 &str 基础教程](https://wkbse.com/2025/04/25/rust-string-%E4%B8%8E-str-%E5%9F%BA%E7%A1%80%E6%95%99%E7%A8%8B-wiki%E5%9F%BA%E5%9C%B0/)
- [Unlocking the Power of Strings in Rust](https://office.qz.com/unlocking-the-power-of-strings-in-rust-4193ad56f8db)
- [Understanding String vs &str in Rust: A Comprehensive Guide](https://medium.com/%40mbugraavci38/understanding-string-vs-str-in-rust-a-comprehensive-guide-19ee3eb44fea)
- [Rust By Example - Strings](https://doc.rust-lang.org/rust-by-example/zh/std/str.html)