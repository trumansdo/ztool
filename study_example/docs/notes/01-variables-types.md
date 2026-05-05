# 变量与基本类型

## 一、let 绑定与可变性

Rust 使用 `let` 关键字声明变量绑定。默认情况下，所有变量都是**不可变的**——这是 Rust 安全哲学的根基。

```rust
let x = 5;           // 不可变绑定，x 不能再被赋值
// x = 6;            // 编译错误：cannot assign twice to immutable variable `x`

let mut y = 10;      // mut 关键字声明可变绑定
y = 20;              // 允许修改
```

> 不可变优先不是限制，而是让程序的行为变得可预测、可推理，是编译器替你阅读代码的方式。

### 变量遮蔽（Shadowing）

`let` 可以重复声明同名变量，新变量会**遮蔽**旧变量：

```rust
let x = 5;
let x = x + 1;       // x = 6，新变量遮蔽旧变量
{
    let x = x * 2;   // x = 12，在内部作用域中
    println!("内部: {x}");   // 12
}
println!("外部: {x}");       // 6
```

**遮蔽 vs mut 的区别**：

| 特性 | `let` 遮蔽 | `let mut` |
|------|-----------|-----------|
| 重新赋值 | 创建全新变量 | 修改同一变量 |
| 类型变化 | 可以改变类型 | 类型必须一致 |
| 作用域结束后 | 旧值恢复 | 修改永久生效 |
| 编译器检查 | 仍然是不可变绑定 | 允许重新赋值 |

```rust
let spaces = "   ";
let spaces = spaces.len();  // 类型从 &str 变为 usize，遮蔽允许

let mut n = "hello";
// n = n.len();              // 错误：mut 不允许改变类型
```

---

## 二、标量类型

Rust 的标量类型体系体现了对内存和性能的精确控制。

| 类型 | 大小 | 范围 | 说明 |
|------|------|------|------|
| `i8` | 1字节 | -128 ~ 127 | |
| `i16` | 2字节 | -32768 ~ 32767 | |
| `i32` | 4字节 | -2³¹ ~ 2³¹-1 | 默认整数类型 |
| `i64` | 8字节 | -2⁶³ ~ 2⁶³-1 | 默认与C long相当 |
| `i128` | 16字节 | | |
| `isize` | 指针宽度 | | 数组索引、指针运算 |
| `u8` | 1字节 | 0 ~ 255 | 字节级操作 |
| `u16` | 2字节 | 0 ~ 65535 | |
| `u32` | 4字节 | 0 ~ 2³²-1 | |
| `u64` | 8字节 | 0 ~ 2⁶⁴-1 | `std::time` 时间戳常用 |
| `u128` | 16字节 | | |
| `usize` | 指针宽度 | | 数组大小、索引 |
| `f32` | 4字节 | IEEE754单精度(6-7位有效数字) | |
| `f64` | 8字节 | IEEE754双精度(15-16位有效数字) | 默认浮点类型 |
| `bool` | 1字节 | true/false | |
| `char` | 4字节 | Unicode标量值(0x0000~0xD7FF, 0xE000~0x10FFFF) | 不是1字节！ |

```rust
let a: u8 = 255;
let b: isize = 42;
let c: f64 = 3.14159;
let d: bool = true;
let e: char = '中';    // 3字节UTF-8编码的汉字，char为4字节Unicode标量值
let f: char = '😻';    // emoji也是有效的char
```

> 精确的类型尺寸不是学究式的挑剔，而是系统编程中的安身立命之本——你不知道数据占多少字节，就无法控制内存布局。

---

## 三、数字字面量

Rust 提供了丰富的字面量表示法：

```rust
let decimal = 98_222;          // 十进制，_ 为可读分隔符
let hex = 0xff;                // 十六进制 (255)
let octal = 0o77;              // 八进制 (63)
let binary = 0b1111_0000;      // 二进制 (240)
let byte = b'A';               // 字节字面量，等价于 65u8

// 类型后缀
let x = 42u8;                  // u8 类型
let y = 3.14f32;               // f32 类型
let z = 1_000_000i32;          // i32 类型

// 科学计数法
let big = 1e6;                 // f64: 1000000.0
let small = 1e-3_f64;          // f64: 0.001
```

---

## 四、整数溢出

> 安全的边界不是阻碍，而是让你在危险的数值世界里有了一张可靠的航海图。

```rust
let mut n: u8 = 255;

// debug 模式：panic at runtime
// n + 1;  // debug 模式下会 panic!

// release 模式：wrapping（环绕）
// n + 1 == 0u8  // release 模式下静默环绕

// 四族溢出处理函数
let result = n.checked_add(1);        // Option<u8> -> None
let result = n.saturating_add(1);     // u8 -> 255 (饱和)
let result = n.wrapping_add(1);       // u8 -> 0 (环绕)
let (result, overflowed) = n.overflowing_add(1);  // (0, true)
```

| 方法族 | 返回值 | 行为 |
|--------|--------|------|
| `checked_*` | `Option<T>` | 溢出时返回 `None` |
| `saturating_*` | `T` | 溢出时返回最大/最小值 |
| `wrapping_*` | `T` | 环绕（模运算） |
| `overflowing_*` | `(T, bool)` | 环绕 + 溢出标志 |

---

## 五、浮点类型

```rust
let x = 2.0;          // f64 默认
let y: f32 = 3.0;     // f32 显式标注

// 特殊值
let nan = f64::NAN;              // 非数值
let inf = f64::INFINITY;         // 正无穷
let neg_inf = f64::NEG_INFINITY; // 负无穷

// NaN 判定：NaN 不等于任何值，包括自身
assert!(nan.is_nan());
assert!(nan != nan);             // NaN != NaN 为 true!

// EPSILON：相邻两浮点数的最小间距
let eps = f64::EPSILON;
assert!(0.1 + 0.2 - 0.3 < eps * 10.0);  // 不是严格相等!

// 精确比较
assert!((0.1_f64 + 0.2_f64 - 0.3_f64).abs() < 1e-10);
```

> 浮点数不是实数，它们是有限精度的二进制近似——忘记这一点是一切浮点bug的根源。

---

## 六、复合类型

### 6.1 元组

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);

// 解构
let (x, y, z) = tup;
println!("x = {x}, y = {y}, z = {z}");

// 索引访问
let five_hundred = tup.0;
let six_point_four = tup.1;

// 单元元组（空元组，0大小类型）
let unit: () = ();
// () 类型只有一个值，用于不返回值的表达式
```

### 6.2 数组 [T; N]

数组在栈上分配，长度在编译期确定，是 RUST 最低层级的定长序列。

```rust
let arr: [i32; 5] = [1, 2, 3, 4, 5];
let zeros = [0; 100];                   // 100个0
let matrix: [[i32; 3]; 2] = [[1,2,3], [4,5,6]];

// 访问
let first = arr[0];
// let bad = arr[100];                  // 运行时 panic，数组越界

// 常量泛型：长度可以是编译期常量表达式
fn print_array<const N: usize>(arr: [i32; N]) {
    println!("数组长度: {}", N);
    for item in arr.iter() {
        println!("{}", item);
    }
}

// 访问切片
let slice: &[i32] = &arr[1..4];
```

> 数组长度是类型的一部分——`[i32; 3]` 和 `[i32; 4]` 是两种完全不同的类型。这不是语法糖，而是内存安全的基石。

---

## 七、常量与静态

```rust
// const：编译期求值，内联到使用处，无固定内存地址
const MAX_POINTS: u32 = 100_000;
const SECONDS_PER_HOUR: u32 = 60 * 60;  // 编译期计算

// static：有固定内存地址，'static 生命周期
static LANGUAGE: &str = "Rust";
static mut COUNTER: u32 = 0;  // 可变静态，unsafe 才能修改

fn main() {
    unsafe {
        COUNTER += 1;  // 修改可变静态必须在 unsafe 中
    }
}
```

| 特性 | `const` | `static` |
|------|---------|----------|
| 内存地址 | 无（内联展开） | 有固定地址 |
| 生命周期 | 使用处的作用域 | `'static` |
| 可变性 | 始终不可变 | 可用 `static mut`（unsafe） |
| 内联 | 编译器可内联 | 不内联，跨编译单元引用同一地址 |
| 适用场景 | 魔法数、编译期常量 | 全局状态、FFI导出 |

---

## 八、类型推断、别名与转换

```rust
// 类型推断：大多数情况下无需标注
let v = 42;                // 推断为 i32
let mut vec = Vec::new();  // 编译错误：类型不明确！
let mut vec: Vec<i32> = Vec::new();  // 必须标注

// 必须标注类型的情况：
// 1. 集合的 new() 调用
// 2. parse() 方法
let num: u32 = "42".parse().expect("不是数字");
// 3. 函数参数和返回值类型
// 4. trait 对象的类型

// 类型别名
type Kilometers = i32;
type Thunk = Box<dyn Fn() + Send + 'static>;
let distance: Kilometers = 5;

// as 转换：安全子集
let a: i32 = 42;
let b: u32 = a as u32;        // 整数间转换
let c: i8 = 300 as i8;        // 截断，c = 44 (300 % 256 - 128)
let d = 3.14 as i32;          // 浮点→整数，截断小数部分 (3)
let e = b'a' as u8;           // 97，字符到u8
let f = std::char::from_u32(0x1F600).unwrap();

// as 不能做的事：
// 不能跨指针类型转换（用 From/Into trait）
// 不能保证无损转换（用 TryFrom/TryInto）
// 不能做 trait 对象转换
```

> 类型推断让你不必把类型写在脸上，但类型安全让你不能在运行时装傻。

---

## 九、Never Type — `!`

`!` 类型（发散类型）表示永远不会返回值的类型：

```rust
// 这些表达式都是 ! 类型
fn endless() -> ! {
    loop {
        // 永远循环
    }
}

fn die() -> ! {
    panic!("程序崩溃");
}

// ! 类型可以强制转换为任意类型，这使得类型统一成为可能
let guess: u32 = match guess.trim().parse() {
    Ok(num) => num,          // u32
    Err(_) => continue,      // ! 类型，被强制转为 u32
};

// 实际应用场景
let val = if condition {
    42
} else {
    return 0;   // return 返回 !，整个 if 表达式类型统一为 i32
};

// 所有 ! 类型来源：
// 1. panic!()
// 2. loop {} (无 break 的无限循环)
// 3. break/continue
// 4. return
// 5. std::process::exit()
// 6. unreachable!()
// 7. todo!()
```

---

## 十、表达式 vs 语句

```rust
// 语句：以分号结尾，不返回值（返回 ()）
let x = 5;              // let 语句
println!("hello");      // 表达式语句

// 表达式：没有分号，返回值
let y = {
    let z = 3;
    z + 1               // 注意：没有分号！返回 4
};

// 分号改变类型
let a = 5;              // 语句，a: i32
let b = { 5 };          // 表达式块，b: i32 = 5
let c = { 5; };         // 表达式块 + 分号，c: () = ()

fn foo() -> i32 {
    42                  // 表达式返回
}

fn bar() -> () {
    42;                 // 分号使返回值变为 ()
}
```

> 分号是表达式到语句的转换开关——少一个分号改变了函数签名，多一个分号抹掉了返回值。

---

## 十一、变量命名规范

遵循 RFC 430 和社区约定：

```rust
let snake_case_var = 1;           // 变量：蛇形命名
const SCREAMING_SNAKE_CASE = 2;   // 常量：全大写蛇形
static GLOBAL_FLAG: bool = true;  // 静态变量：全大写蛇形
type PascalCase = i32;            // 类型别名：帕斯卡命名

// 合法但不推荐的变量名
let _ = 5;             // 下划线：明确忽略值
let _x = 10;           // 前缀下划线：抑制未使用警告
let 变量 = "中文";      // 支持非ASCII标识符（不推荐）

// 特殊前缀/后缀约定
// raw 标识符：使用关键字作为标识符
let r#type = "hello";  // r# 前缀允许使用保留字
let r#catch = 42;
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| `let mut` 不能改变变量类型 | 可变绑定类型在声明时确定 | 用 `let` 遮蔽来改变类型 |
| `f32 == f32` 可能为 false（NaN） | NaN 不等于任何值，包括自身 | 使用 `is_nan()` 判定 |
| 数组越界在运行时 panic | Rust 不做下标检查的静态分析 | 使用 `get()` 返回 Option |
| `0.1 + 0.2 != 0.3` | 浮点数二进制近似 | 使用 `(a - b).abs() < EPSILON` |
| `static mut` 访问不是线程安全的 | 无同步机制 | 使用 `Mutex<static>`、`Atomic*` 或 `OnceCell` |
| `as` 转换可能静默截断 | i32 → i8 截断高位 | 使用 `TryFrom`/`TryInto` 做安全转换 |
| `let v = Vec::new()` 编译失败 | 编译器无法推断集合类型 | 显式标注类型或给第一个元素 |
| 遮蔽导致旧值无法访问 | 旧变量被命名遮盖 | 小心使用遮蔽，避免在关键逻辑中误用 |
| `{5;}` 返回 `()` 而非 `5` | 末尾分号将表达式变为语句 | 去掉不需要的分号 |
| `const` 不能放函数调用 | `const` 必须在编译期能够求值 | 用 `lazy_static` 或 `OnceLock` |

> **详见测试**: `tests/rust_features/01_variables_types.rs`
