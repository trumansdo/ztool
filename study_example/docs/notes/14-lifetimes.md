# Rust 生命周期：借用检查器的底层法则

## 一、生命周期标注语法

> **金句引用**："`'a` 不是计时器，是关系声明——它标注引用间的存活关联，编译器据此拒绝悬垂指针。"

### 1.1 基本语法

```rust
// 生命周期参数以 ' 开头，通常用短小写字母
// 'a、'b、'static、'input、'output

// 函数签名中的标注：标注输入引用和输出引用之间的关系
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
// 解读：返回引用的生命周期是参数中较短的那个

let s1 = String::from("长字符串");
let s2 = String::from("短");
let result = longest(&s1, &s2); // 返回 s1 或 s2 的引用
println!("{}", result);          // 可用：s1 和 s2 都存活
// 离开此块：result、&s1、&s2 全部释放
```

### 1.2 三条省略规则

编译器在以下条件下自动推断生命周期：

1. **每个引用参数获得独立生命周期**：`fn foo(x: &T, y: &T)` → `fn foo<'a, 'b>(x: &'a T, y: &'b T)`
2. **单输入→输出**：若仅一个输入生命周期，输出获得相同生命周期
3. **&self / &mut self 方法**：输出生命周期等于 self 的生命周期

```rust
// 规则2示例：编译器自动补全
fn first(s: &str) -> &str { &s[..1] }
// 反编译等价：
fn first<'a>(s: &'a str) -> &'a str { &s[..1] }

// 规则3示例：方法自动推导
impl MyStruct {
    fn get(&self) -> &i32 { ... }       // 输出 = &self 生命周期
    fn get_mut(&mut self) -> &mut i32 { ... } // 输出 = &mut self 生命周期
}
```

---

## 二、输入 vs 输出生命周期

```rust
// 多输入时输出生命周期取决于实际返回的引用
fn choose<'a, 'b>(first: &'a str, second: &'b str, pick_first: bool) -> &'a str {
    if pick_first { first } else { second }  // ⚠️ second 的生命周期为 'b 非 'a
}
// 编译错误：输出标注为 'a，但可能返回 'b 的引用

// 正确版本：
fn choose<'a>(first: &'a str, second: &'a str, pick_first: bool) -> &'a str {
    if pick_first { first } else { second }
}
// 或分别标注且各归各路
```

---

## 三、结构体与方法中的生命周期

> **金句引用**："结构体含引用必带 `'a`，impl 块紧跟【<'a>】——这是 Rust 的呼吸节奏。"

### 3.1 结构体标注

```rust
struct Excerpt<'a> {
    part: &'a str,  // 结构体实例不能比它借用的数据活得更久
}

fn main() {
    let novel = String::from("从前有座山...");
    let first = novel.split('.').next().unwrap();
    let excerpt = Excerpt { part: first };
    // excerpt 无法比 novel 活得更久
}
```

### 3.2 方法 impl 块标注

```rust
impl<'a> Excerpt<'a> {
    fn announce_and_return(&self, announcement: &str) -> &str {
        println!("请注意: {}", announcement);
        self.part  // 省略规则3：返回生命周期 = &self 的生命周期 = 'a
    }
    // 等价于显式标注版：
    fn announce_and_return_explicit<'b>(&'a self, announcement: &'b str) -> &'a str {
        println!("请注意: {}", announcement);
        self.part
    }
}
```

---

## 四、NLL 非词法生命周期

> **金句引用**："NLL 是引用的大赦令——从块级释放降至语句级释放，编译器更懂你的自由。"

```rust
// 旧版（NLL 之前）：借用到作用域结束
// NLL（1.31+）：借用到最后一次使用即结束

let mut data = vec![1, 2, 3];
let r1 = &data;             // 不可变借用开始
println!("{:?}", r1);       // r1 最后一次使用 → 借用结束
let r2 = &mut data;         // 可变借用：无冲突，r1 已释放
r2.push(4);                 // OK

// NLL 基于 MIR（中级中间表示）的控制流图分析
// 而非词法作用域
```

### MIR 层面的分析

```
fn example(cond: bool) {
    let mut v = vec![1, 2];
    let r = &mut v;       // borrow at BB0
    if cond {             // fork BB1 / BB2
        r.push(3);        // use r at BB1
    }
    // NLL: r 仅在 BB1 被使用，BB2 中借用无效
}
```

---

## 五、变型（Variance）

> **金句引用**："变型是类型关系的传递规则——子类型关系如何穿透容器传播。"

| 变型类别 | 含义 | 示例 |
|----------|------|------|
| **协变** | `Sub` 是 `Super` 的子类型 ⇒ `Container<Sub>` 是 `Container<Super>` 的子类型 | `&T`, `Box<T>`, `Vec<T>`（不修改元素） |
| **逆变** | `Sub` 是 `Super` 的子类型 ⇒ `Container<Super>` 是 `Container<Sub>` 的子类型 | `fn(T)` 参数位置 |
| **不变** | 没有子类型关系传递 | `&mut T`, `Cell<T>`, `UnsafeCell<T>`, `*mut T` |

```rust
// 协变示例
let x: &'static str = "永久字符串";
let y: &'a str = x;            // &'static str → &'a str，协变成立

// 逆变示例（函数参数）
fn takes_fn(f: fn(&'static str)) { }
let short_fn: fn(&'a str) = |_| {};
// takes_fn(short_fn); // 错误：逆变要求 fn(&'static str) ≤ fn(&'a str)

// 不变示例
fn assign_mut<T>(target: &mut T, val: T) { *target = val; }
let mut a = String::from("a");
let b = &mut a;
let c: &mut String = b;          // &mut 不变，直接兼容
```

### 'long: 'short 子类型关系

```rust
// 'long: 'short 表示 'long 存活至少和 'short 一样长
// 'static 是最长的生命周期

let a: &'static str = "永久";  // 'static 是所有生命周期的子类型
let b: &'a str = a;           // &'static T → &'a T：协变 + 子类型
```

---

## 六、HRTB：高阶 Trait 限定

> **金句引用**："for<'a> 是生命周期的万能钥匙——闭包不关心引用寿命，接受任意生命期。"

```rust
// for<'a> Fn(&'a str) → bool
// F 必须适用于任意生命周期参数，而非某一特定生命周期

// 示例：泛化的闭包参数
fn apply_conditionally<F>(data: &[i32], condition: F) -> Vec<i32>
where
    F: for<'a> Fn(&'a i32) -> bool,
{
    data.iter().filter(|x| condition(x)).copied().collect()
}

// Fn trait 定义本身就包含了 HRTB：
// pub trait Fn<Args>: FnMut<Args> { ... }
// 等价于：for<'a> Fn(&'a i32) ...

// 闭包自动满足 HRTB，因为闭包不具名生命周期
let _ = apply_conditionally(&[1, 2, 3, 4], |&&x| x > 2);
```

---

## 七、'static 的三种含义

| 含义 | 说明 | 示例 |
|------|------|------|
| **&'static str** | 引用在整个程序运行期间有效 | 字符串字面量 |
| **T: 'static** | T 自身不包含非 `'static` 引用 | 拥有的值 `String` / `i32` |
| **内存泄漏产生 'static** | `Box::leak` / `String::leak` 制造的永久引用 | 配置初始化 |

```rust
// 含义1：&'static str
let greeting: &'static str = "你好，世界！";

// 含义2：T: 'static
fn static_check<T: 'static>(val: T) { /* val 不包含短生命周期引用 */ }
static_check(42);          // OK：i32: 'static
static_check("hello");     // OK：&'static str: 'static
let s = String::from("test");
static_check(s);           // OK：String: 'static（不包含非static引用）

// 含义3：泄漏产生 'static
use std::io;
let config_singleton: &'static io::Error = {
    let e = io::Error::new(io::ErrorKind::Other, "初始化错误");
    Box::leak(Box::new(e))  // 堆泄露 → 'static 引用
};
// config_singleton 永不释放，程序结束由 OS 回收
```

---

## 八、常见生命周期错误诊断表

| 错误码 | 描述 | 常见原因 | 修复方案 |
|--------|------|----------|----------|
| E0106 | 缺少生命周期标注 | 返回引用但编译器无法推断 | 添加 `'a` 标注 |
| E0495 | 生命周期不够长 | 返回的引用活得不够久 | 检查借用关系是否匹配 |
| E0506 | 修改借用的值 | 在引用存在时修改原值 | 缩小引用作用域或重新排序操作 |
| E0597 | 临时值释放 | 引用的值被立即丢弃 | 将值绑定到 let 变量中延长生命周期 |
| E0623 | 生命周期不匹配 | 尝试把短引用赋值给长引用 | 调整生命周期参数的约束 |
| E0716 | 临时值在表达式结束释放 | `match` 或 `if let` 中临时变量立即释放 | 用 let 绑定保存临时值 |
| E0310 | 借用检查器拒绝 | 实现与标注不匹配 | 检查 impl 块中的生命周期参数是否匹配 trait 定义 |

```rust
// E0597 典型错误
fn broken() -> &str {
    let s = String::from("hello");
    &s  // 错误！s 在函数结束时释放
}
// 修复：
fn correct() -> String {
    String::from("hello")  // 返回所有权
}
```

---

## 九、生命周期子类型关系的实战

```rust
// 'static 是所有生命周期的子类型
fn take_static(s: &'static str) { println!("{}", s); }

fn demonstrate_subtyping<'a>(local: &'a str) {
    let permanent: &'static str = "永久的";
    take_static(permanent);  // &'static → &'static，直接兼容

    // 但 'static 不能传给期望短引用的位置
    // 实际上 &'static str 可以用于任意期望 &'b str 的地方（协变）
    let _result: &'a str = permanent; // 'static: 'a，协变放行
}

// 逆变在函数指针中的体现
fn takes_short_fn(f: fn(&str)) {
    f("短暂");
}
fn long_func(s: &'static str) { println!("{}", s); }
// takes_short_fn(long_func); // 逆变：fn(&'static) 不能赋给 fn(&str)
```

---

## 十、RPIT 捕获规则

```rust
// RPIT（返回位置 impl Trait）在生命周期方面的捕获

// 默认捕获所有泛型生命周期（Rust 2018/2021）
fn foo<'a>(x: &'a str, y: &'a str) -> impl std::fmt::Display + 'a {
    // 返回不透明类型隐式捕获 'a
    format!("{} + {}", x, y)
}

// use<> 精确捕获语法（Rust 2024 / nightly feature）
#![feature(precise_capturing)]
fn bar<'a, 'b>(x: &'a str, y: &'b str) -> impl std::fmt::Display + use<'a> {
    // 仅捕获 'a，不捕获 'b
    format!("来自x: {}", x)
}
```

**版本对比**：

| Edition | 默认捕获 | 精确控制 |
|---------|----------|----------|
| 2018 | 所有泛型参数（生命周期+类型） | 不支持 |
| 2021 | 所有泛型参数（生命周期+类型） | 不支持 |
| 2024 | 所有生命周期（不含类型参数） | `use<>` 语法 |

---

## 十、常见生命周期模式

> **金句引用**："生命周期不是限制，是指引——三个经典模式囊括 90% 的借用场景。"

### 模式1：最长生存（返回引用不短于两个输入中较短者）

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { ... }
```

### 模式2：最短借用（借用仅覆盖使用区间，不撑满作用域）

```rust
let mut v = vec![1, 2];
let r = &v;
println!("{}", r.len());  // r 最后使用 → 借用在此结束
let w = &mut v;           // OK：NLL 判断
```

### 模式3：输入输出匹配（结构体返回的引用与 self 同寿）

```rust
impl<'a> Container<'a> {
    fn get<'b>(&'a self, key: &'b str) -> &'a str {
        &self.data[key]  // 返回引用的生命周期来自 self
    }
}
```

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 多输入返回标注过宽 | 返回 `'a` 但可能返回另一个参数的数据 | 精确标注为可能导致编译错误 |
| `'static` 与 `T: 'static` 混淆 | 前者指引用永久有效，后者指类型不含非静态引用 | 区分二者语义，不混用 |
| 闭包捕获引用绕过 NLL | 闭包创建时间不影响借用持续时间 | 手动 drop 闭包或用 `move` 转移所有权 |
| &mut 不变性导致代码冗余 | &mut 不变性阻止协变传播 | 必要时重新借用：`let tmp = &mut *r;` |
| RPIT 隐式捕获过多生命周期 | 返回的不透明类型捕获了不需要的生命周期 | 用 `use<>` 语法精确控制捕获 |
| 结构体标注生命周期遗漏 | 结构体含引用字段但无生命周期参数 | 给结构体添加 `<'a>` 并在字段上标注 |
| HRTB 被误用为不必要约束 | 普通函数不需要 `for<'a>` | 仅在闭包或需要泛化的生命周期时使用 |
| `drop` 调用时机导致借用冲突 | `drop` 需要独占引用，与后续借用冲突 | 用 `{}` 块明确作用域，让 NLL 自动释放 |

> **详见测试**: `tests/rust_features/14_lifetimes.rs`
