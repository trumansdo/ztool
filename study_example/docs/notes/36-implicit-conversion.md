# Rust 隐式转换（Implicit Conversion / Coercion）完全清单

本文从宽泛视角收录 Rust 中所有"编译器自动触发、开发者无需显式书写"的类型转换机制，分为三大部分：

- **上篇：编译器自动调用的转换 Trait** —— 语法糖展开时隐式调用（`?` / `for` / `.await` / `let` / `f()`）
- **中篇：开发者可实现的隐式类型强制** —— `Deref` / `DerefMut`
- **下篇：编译器硬编码的隐式规则** —— 开发者无法实现，纯编译器行为

---

# 上篇：编译器自动调用的转换 Trait

以下 trait 的调用由编译器在语法糖展开时**自动插入**，开发者无需手写 `.into()` / `.into_iter()` 等，但它们本质上是标准 trait 调用，不属于类型强制（type coercion）。

---

## 1. From / Into —— 配合 `?` 运算符隐式调用

```rust
pub trait From<T>: Sized {
    fn from(value: T) -> Self;
}

pub trait Into<T>: Sized {
    fn into(self) -> T;
}

// 标准库 blanket impl：只要你实现了 From，就自动获得 Into
impl<T, U> Into<U> for T where U: From<T> {
    fn into(self) -> U { U::from(self) }
}
```

### 1.1 ? 运算符隐式调用 `From`

`?` 运算符展开时会隐式调用 `From::from()` 来转换错误类型：

```rust
fn foo() -> Result<i32, Box<dyn Error>> {
    let f = File::open("foo.txt")?;   // io::Error → Box<dyn Error>
    // 编译器展开为：
    // let f = match File::open("foo.txt") {
    //     Ok(v) => v,
    //     Err(e) => return Err(From::from(e)),  // 隐式调用
    // };
    Ok(0)
}
```

开发者只需要为错误类型实现 `From<SourceErr> for TargetErr`，之后所有 `?` 处都会自动转换。

### 1.2 `.into()` 仍需显式书写

除 `?` 展开外，`.into()` 本身需要开发者显式调用：
```rust
let x: i32 = 42u8.into();  // .into() 需要写出来，不会自动插入
```

---

## 2. IntoIterator —— `for` 循环隐式调用

```rust
pub trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}
```

`for x in expr` 在编译器展开时会隐式调用 `IntoIterator::into_iter(expr)`：

```rust
for x in vec![1, 2, 3] {
    // 编译器展开为：
    // let mut iter = IntoIterator::into_iter(vec![1, 2, 3]);
    // loop {
    //     match Iterator::next(&mut iter) {
    //         Some(x) => { /* 循环体 */ }
    //         None => break,
    //     }
    // }
}
```

标准库为 `Vec<T>`、`&[T]`、`&mut [T]`、`HashMap` 等都实现了 `IntoIterator`，这就是为什么它们可以直接用于 `for` 循环。

`IntoIterator` 还有三个重要的 blanket impl，**决定了 `for` 循环中迭代的行为**：

| 迭代对象 | 实际调用 | 行为 |
|---------|---------|------|
| `for x in v` (v: Vec<T>) | `v.into_iter()` (消耗) | 获取所有权，逐个产出 `T` |
| `for x in &v` | `(&v).into_iter()` → `v.iter()` | 不可变借用，产出 `&T` |
| `for x in &mut v` | `(&mut v).into_iter()` → `v.iter_mut()` | 可变借用，产出 `&mut T` |

---

## 3. IntoFuture —— `.await` 隐式调用

```rust
pub trait IntoFuture {
    type Output;
    type IntoFuture: Future<Output = Self::Output>;
    fn into_future(self) -> Self::IntoFuture;
}
```

`.await` 展开时会隐式调用 `IntoFuture::into_future()`：

```rust
let result = some_expr.await;
// 编译器展开为：
// let mut future = IntoFuture::into_future(some_expr);
// loop {
//     match Future::poll(Pin::new(&mut future), cx) {
//         Poll::Ready(output) => break output,
//         Poll::Pending => yield,
//     }
// }
```

标准库为所有 `T: Future` 实现了 `IntoFuture`（identity impl），所以普通的 `Future` 可以直接 `.await`。但自定义类型只实现 `IntoFuture` 而不实现 `Future` 也是合法的。

---

## 4. FromResidual —— `?` 运算符 v2

```rust
pub trait FromResidual<R = <Self as Try>::Residual> {
    fn from_residual(residual: R) -> Self;
}
```

`?` 运算符的完整展开依赖 `Try` + `FromResidual` 两个 trait（try_trait_v2）。当函数返回 `Result<T, E>` 时：

```rust
fn foo() -> Result<i32, MyError> {
    let x = bar()?;  // bar() 返回 Result<i32, OtherError>
    // ? 展开大致为：
    // let x = match bar().branch() {
    //     ControlFlow::Continue(v) => v,
    //     ControlFlow::Break(e) => return FromResidual::from_residual(e),
    // };
    Ok(x)
}
```

其中 `FromResidual::from_residual` 会调用 `From` 来转换错误类型。`FromResidual` 决定了哪些类型可以用 `?`：`Result<T, E>`、`Option<T>`、`Poll<T>`、`ControlFlow` 等都实现了它。

---

## 5. Copy —— let 绑定隐式复制

```rust
pub trait Copy: Clone { }
```

`Copy` 是一个标记 trait（marker trait），无方法。当一个类型实现了 `Copy`，编译器会在赋值、传参、闭包捕获等场景**隐式复制**而非移动（move）：

```rust
let x: i32 = 42;
let y = x;     // x 仍然可用！编译器隐式调用 bitwise copy
println!("{x}");  // 正常

let s1 = String::from("hello");
let s2 = s1;      // s1 被移动，所有权转移
// println!("{s1}");  // 编译错误
```

**隐式发生的位置**：

| 场景 | 表现 |
|------|------|
| `let y = x;` | x 是 Copy 则复制，否则 move |
| `f(x)` 传参 | x 是 Copy 则复制，否则 move |
| 闭包 `move \|\| x` | x 是 Copy 则按值复制，否则 move |
| `[x; N]` 数组初始化 | x 必须是 Copy |

**开发者如何实现**：使用 `#[derive(Copy, Clone)]` 或手动 `impl Copy for T {}`（要求所有字段都实现 Copy）。

```rust
#[derive(Copy, Clone)]
struct Point { x: i32, y: i32 }

let p1 = Point { x: 1, y: 2 };
let p2 = p1;    // Copy！p1 仍然可用
```

**标准库中常见的 Copy 类型**：所有基础数值类型（`i32`、`f64` 等）、`bool`、`char`、`&T`（引用）、`*const T` / `*mut T`、`fn` 指针、元素全 Copy 的数组和元组。

> `Clone` 是显式 trait，必须调用 `.clone()`。`Copy` 是隐式的——编译器决定何时复制。

---

## 6. Fn / FnMut / FnOnce —— 函数调用隐式分发

```rust
pub trait FnOnce<Args> {
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

pub trait FnMut<Args>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

pub trait Fn<Args>: FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
```

当你写 `f(arg)` 时，编译器**隐式展开**为对应 trait 方法的调用：

```rust
let add = |a, b| a + b;
let result = add(1, 2);
// 编译器展开为（大致）：
// let result = Fn::call(&add, (1, 2));
```

**编译器根据捕获方式自动选择 trait**：

| 捕获方式 | 编译器实现的 trait | 调用方式 |
|---------|-------------------|---------|
| 不可变借用 | `Fn` + `FnMut` + `FnOnce` | 可以多次调用（`&self`） |
| 可变借用 | `FnMut` + `FnOnce` | 可以多次调用但需 `&mut self` |
| 获取所有权 | `FnOnce` only | 只能调用一次 |

**开发者几乎不需要关心**：这是编译器为闭包自动生成的实现。常规开发中无需手动为自定义类型实现 Fn 系列 trait。

> 函数指针 `fn(T) -> U` 也实现了 `Fn` / `FnMut` / `FnOnce`，故 `f(args)` 对函数指针同样有效。

---

## 7. 隐式 trait 调用的边界

以下 trait 虽然常用于类型转换，但**从不被编译器隐式调用**，开发者必须显式写：

| Trait | 说明 |
|-------|------|
| `TryFrom` / `TryInto` | 必须调用 `.try_into()` 或 `T::try_from()` |
| `AsRef` / `AsMut` | 必须调用 `.as_ref()` / `.as_mut()` |
| `Borrow` / `BorrowMut` | 用于约束，不自动调用（如 `HashMap::get` 接受 `&Q where K: Borrow<Q>`，但需你传入 `&Q`） |
| `FromStr` | 必须调用 `.parse()` |
| `ToString` / `Display` | 必须调用 `.to_string()` 或 `format!()` |

但它们仍属于 Rust 的"转换体系"，只是走显式路径。

---

# 中篇：开发者可实现的隐式类型强制

## 8. Deref / DerefMut —— 唯一的隐式类型强制入口

```rust
pub trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

**`Deref` / `DerefMut` 是 Rust 中唯一开放给开发者实现的隐式类型强制机制。**

### 8.1 基本规则

| Trait | 隐式强制 | 典型示例 |
|-------|---------|---------|
| `Deref` | `&T` → `&T::Target` | `&String` → `&str`, `&Vec<T>` → `&[T]`, `&Box<T>` → `&T` |
| `DerefMut` | `&mut T` → `&mut T::Target` | `&mut String` → `&mut str`, `&mut Box<T>` → `&mut T` |

### 8.2 Deref 链（可连续多次）

```rust
fn foo(s: &str) {}
let b: Box<String> = Box::new("hello".into());
foo(&b);  // &Box<String> → &String → &str（两步 Deref）
```

### 8.3 自定义 Deref 实现

```rust
use std::ops::Deref;

struct MyPtr<T> { data: T }

impl<T> Deref for MyPtr<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.data }
}

let p = MyPtr { data: 42i32 };
fn takes_ref(x: &i32) { println!("{x}"); }
takes_ref(&p);  // &MyPtr<i32> → &i32，自动发生
```

### 8.4 设计约束

- **一个类型只能有一个 `Target`**：因为 `Deref` 的关联类型是唯一的
- **标准库 blanket impl**：`impl<T: ?Sized> Deref for &T { type Target = T; }` —— 这就是 `*&T` 能解引用出 `T` 的原因
- **不要滥用**：官方建议仅在类型是"智能指针"或"透明包装"时才实现 `Deref`

---

# 下篇：编译器硬编码的隐式规则

以下所有规则由编译器硬编码或自动生成，**开发者可以享受其效果，但无法实现或修改**。

---

## 9. 基础公理

Rust Reference 定义类型强制的三条基础规则：

1. **自反性（Reflexive）**：`T` 始终可强制为 `T`
2. **传递性（Transitive）**：若 `T → V` 且 `V → U` 均合法，则 `T → U` 亦合法
3. **强制发生位置**（详见第 10 节）

这意味着多种强制可以链式组合：
```
&Box<String> → &String (Deref) → &str (Deref) → &dyn Display (unsizing)
```

---

## 10. 强制发生位置（Coercion Sites）

| 位置 | 说明 |
|------|------|
| `let x: T = expr;` | expr 强制为 T |
| `foo(expr)` | expr 强制为参数类型 |
| `return expr` | expr 强制为函数返回类型 |
| `S { field: expr }` | expr 强制为字段类型 |
| 块尾 `{ expr }` | expr 强制为块类型 |
| 数组字面量元素 | 各元素强制为统一元素类型 |
| `if` / `match` 分支 | LUB 强制合并为公共超类型 |

---

## 11. Unsizing 体系 — 编译器专属 Trait

### 11.1 Sized — 根基

`Sized` 标记类型的大小编译期可知。`?Sized` 放宽约束，允许 DST（动态大小类型）。

### 11.2 Unsize `[stable]`

`std::marker::Unsize<U>` — 标记 `T` 可以"去大小化"为 `U`。由编译器自动生成，开发者不可实现。

| 源类型 | 目标类型 |
|--------|---------|
| `[T; N]` | `[T]` |
| 具体类型 `T` | `dyn Trait`（T 实现了该 Trait） |
| `Struct<..., T>` | `Struct<..., U>`（T 是尾字段） |

### 11.3 CoerceUnsized `[unstable]`

`std::ops::CoerceUnsized<U>` — 为 `Box`, `Rc`, `Arc`, `Cow`, `RefCell`, `Cell`, `Mutex`, `RwLock` 等指针/包装类型提供 unsizing 能力。

标准库内置的 CoerceUnsized impl 可以**单步组合多种强制**：

```rust
// 引用 unsizing — 生命周期缩短 + unsizing 一步完成
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<&U> for &T {}
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<&mut U> for &mut T {}

// 引用转裸指针 + unsizing 组合
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<*const U> for &T {}
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<*const U> for &mut T {}
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<*mut U> for &mut T {}

// 裸指针之间 unsizing
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<*const U> for *const T {}
impl<T: Unsize<U> + ?Sized, U: ?Sized> CoerceUnsized<*mut U> for *mut T {}
```

### 11.4 PinCoerceUnsized `[unstable]`

`std::pin::PinCoerceUnsized<T>` — tracking issue #150112。类似 `CoerceUnsized` 但针对 `Pin` 包裹的指针：
`Pin<Box<T>>` → `Pin<Box<dyn Trait>>`、`Pin<Rc<T>>` → `Pin<Rc<dyn Trait>>` 等。

### 11.5 DispatchFromDyn `[unstable]`

`std::ops::DispatchFromDyn<T>` — trait object 动态分发时，将 wide pointer 转换为 narrow pointer。

### 11.6 各种 Unsized 强制实例

```rust
// [T; N] → [T]
let arr: [i32; 3] = [1, 2, 3];
let slice: &[i32] = &arr;                 // &[i32; 3] → &[i32]
let boxed: Box<[i32]> = Box::new(arr);    // Box<[i32; 3]> → Box<[i32]>

// 具体类型 → dyn Trait
let s = "hello".to_string();
let d1: &dyn Display = &s;                       // &String → &dyn Display
let d2: Box<dyn Display> = Box::new(s);           // Box<String> → Box<dyn Display>
let d3: Arc<dyn Display> = Arc::new(42);          // Arc<i32> → Arc<dyn Display>

// 结构体尾字段 unsizing（只有最后字段可以是 DST）
struct Foo<T: ?Sized> { x: i32, y: T }
let f: &Foo<String> = &Foo { x: 1, y: "hi".into() };
let u: &Foo<dyn Display> = f;  // &Foo<String> → &Foo<dyn Display>

// 元组尾元素 unsizing [unstable]
// #![feature(unsized_tuple_coercion)]
// let t: &(i32, [i32; 3]) = &(1, [2, 3, 4]);
// let u: &(i32, [i32]) = t;

// Dyn upcasting [unstable]
// #![feature(trait_upcasting)]
// let d: &dyn Derived = &42;
// let b: &dyn Base = d;            // dyn Derived → dyn Base
```

---

## 12. 生命周期子类型（Lifetime Subtyping）

Rust 的子类型**仅存在于生命周期上**：`'a: 'b` → `'a` 是 `'b` 的子类型。

| 源类型 | 目标类型 | 条件 |
|--------|---------|------|
| `&'long T` | `&'short T` | 协变 |
| `&'long mut T` | `&'short mut T` | 协变于生命周期，不变于 T |
| `fn(T) -> &'long U` | `fn(T) -> &'short U` | 协变于返回值 |
| `fn(&'short T)` | `fn(&'long T)` | 逆变于参数 |

**不变（invariant）位置不会发生**：`&mut &'long T` 不能变为 `&mut &'short T`。

---

## 13. 指针弱化（Pointer Weakening）

| 源类型 | 目标类型 |
|--------|---------|
| `&mut T` | `&T` |
| `*mut T` | `*const T` |
| `&mut T` | `*mut T` |
| `&mut T` | `*const T` |
| `&T` | `*const T` |

> `*const T → *mut T` 和 `&T → *mut T` **不可隐式转换**，必须用 `as`。

---

## 14. Never 类型强制

`!`（never 类型 / 发散类型）可隐式强制为**任意类型**。

```rust
fn diverge() -> ! { panic!() }
let x: i32 = diverge();      // ! → i32
let y: Vec<u8> = match opt {
    Some(v) => vec![v],
    None => diverge(),       // ! → Vec<u8>
};
```

---

## 15. 函数 / 闭包到函数指针

| 转换 | 条件 |
|------|------|
| 函数项 → `fn(...)` | 任何具名函数 |
| 非捕获闭包 → `fn(...)` | 闭包不捕获任何变量 |
| 捕获闭包 → `fn(...)` | **不可** |

---

## 16. LUB 强制

`if` / `match` / 数组等上下文中，编译器找"最小公共超类型"并自动强制。

```rust
let x: &dyn Display = if true { &"str" } else { &42.to_string() };
// &str → &dyn Display, &String → &dyn Display, 统一为 &dyn Display
```

数值类型之间**不存在 LUB 强制**：`let x = if true { 1u8 } else { 2u16 };` 编译错误。

---

## 17. 方法调用的 Auto-Deref & Auto-Ref

点操作符 `.` 在方法调用时有独立机制：依次尝试 auto-ref、auto-deref、unsized coercion，直到找到匹配方法。

```
x.method()
  → 依次尝试: T, &T, &mut T, *T, **T, ***T, ...
  → 每次还尝试 unsized coercion
```

---

# 总结表

| 隐式转换 | 归类 | 谁实现 | 稳定性 |
|----------|:---:|:---:|:---:|
| `From` / `Into`（`?` 隐式调用） | 上篇 语法糖 | 开发者 | 稳定 |
| `IntoIterator`（`for` 隐式调用） | 上篇 语法糖 | 开发者 | 稳定 |
| `IntoFuture`（`.await` 隐式调用） | 上篇 语法糖 | 开发者 | 稳定 |
| `FromResidual`（`?` v2） | 上篇 语法糖 | 开发者 | 稳定 |
| Copy（赋值/传参隐式复制） | 上篇 语法糖 | 开发者 | 稳定 |
| Fn/FnMut/FnOnce（调用隐式分发） | 上篇 语法糖 | 编译器 | 稳定 |
| `TryFrom` / `TryInto` | 显式 | 开发者 | 稳定 |
| `AsRef` / `AsMut` | 显式 | 开发者 | 稳定 |
| `Borrow` / `BorrowMut` | 显式 | 开发者 | 稳定 |
| `FromStr::parse()` | 显式 | 开发者 | 稳定 |
| Deref 强制 | 中篇 类型强制 | 开发者 | 稳定 |
| DerefMut 强制 | 中篇 类型强制 | 开发者 | 稳定 |
| `Unsize` | 下篇 编译器 | 编译器 | 稳定 |
| `CoerceUnsized` | 下篇 编译器 | 编/库 | unstable |
| `PinCoerceUnsized` | 下篇 编译器 | 编/库 | unstable |
| `DispatchFromDyn` | 下篇 编译器 | 编译器 | unstable |
| 生命周期子类型 | 下篇 编译器 | 编译器 | 稳定 |
| 指针弱化 | 下篇 编译器 | 编译器 | 稳定 |
| Never `!` → 任意 T | 下篇 编译器 | 编译器 | 稳定 |
| 函数项 → fn 指针 | 下篇 编译器 | 编译器 | 稳定 |
| 非捕获闭包 → fn 指针 | 下篇 编译器 | 编译器 | 稳定 |
| `[T;N]` → `[T]` | 下篇 编译器 | 编译器 | 稳定 |
| 具体类型 → `dyn Trait` | 下篇 编译器 | 编/库 | 稳定 |
| 结构体/元组尾字段 unsizing | 下篇 编译器 | 编译器 | 稳定 |
| Dyn upcasting | 下篇 编译器 | 编译器 | unstable |
| LUB 强制 | 下篇 编译器 | 编译器 | 稳定 |
| Auto-deref/auto-ref | 下篇 编译器 | 编译器 | 稳定 |
| 强制发生位置 | 下篇 编译器 | 编译器 | 稳定 |
