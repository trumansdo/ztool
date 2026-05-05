# 异步闭包

> 异步闭包是闭包和 async 块的合体——它捕获环境变量，并返回一个 Future，而自身不会立即执行任何异步操作。

## 1. 异步闭包语法

### 1.1 基本形式

```rust
use std::future::Future;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // 异步闭包：async || { ... }
    let async_closure = async || {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        42
    };

    let result = rt.block_on(async_closure());
    println!("结果: {}", result); // 42
}
```

> 异步闭包本质上是一个返回 Future 的普通闭包——`async || expr` 等价于 `|| async { expr }`，但前者更紧凑。

### 1.2 带参数的异步闭包

```rust
#[tokio::main]
async fn main() {
    let multiply = async |x: i32, y: i32| -> i32 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        x * y
    };

    let result = multiply(6, 7).await;
    println!("积: {}", result); // 42
}
```

### 1.3 async move 闭包

```rust
#[tokio::main]
async fn main() {
    let owned_string = String::from("被移动的数据");

    // move 将所有权转移到闭包内
    let closure = async move || {
        println!("闭包使用了: {}", owned_string);
    };

    // owned_string 已不可用
    closure().await;
}
```

## 2. 异步闭包捕获环境

### 2.1 借用捕获

```rust
#[tokio::main]
async fn main() {
    let data = vec![1, 2, 3, 4, 5];

    // 默认按需借用
    let print_data = async || {
        for item in &data {
            println!("{}", item);
        }
    };

    println!("调用之前 data 仍可用: {:?}", data);
    print_data().await;
    println!("调用之后 data 仍可用: {:?}", data);
}
```

> 异步闭包的捕获规则与普通闭包一致——编译器按最小权限原则进行不可变借用或可变借用，除非你显式使用 move。

### 2.2 可变借用捕获

```rust
#[tokio::main]
async fn main() {
    let mut counter = 0;

    let mut increment = async || {
        counter += 1;
        println!("计数: {}", counter);
    };

    increment().await;
    increment().await;
    increment().await;
    // 输出: 计数: 1, 计数: 2, 计数: 3
}
```

### 2.3 跨 .await 引用的陷阱

```rust
#[tokio::main]
async fn main() {
    let mut data = vec![1, 2, 3];

    // 危险：跨 .await 的可变引用
    // 以下代码无法编译（如果闭包内有 .await）
    // let mut modify = async || {
    //     data.push(4);  // 可变借用 data
    //     tokio::time::sleep(...).await; // 暂停点
    //     // data 的可变借用跨越了 .await
    // };
}
```

> 异步闭包中跨 .await 的引用受限，原理与 async fn 完全一致——.await 是暂停点，编译器必须确保暂停期间引用的数据未被移动或释放。

## 3. 异步闭包作为参数

### 3.1 泛型方式

```rust
use std::future::Future;

// Fn() -> Future 的泛型签名
async fn with_async_closure<F, Fut>(f: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = String>,
{
    let result = f().await;
    println!("闭包结果: {}", result);
}

#[tokio::main]
async fn main() {
    let name = "李四".to_string();

    with_async_closure(|| async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        format!("你好, {}", name)
    }).await;
}
```

### 3.2 使用 trait object

```rust
use std::pin::Pin;
use std::future::Future;

// 使用 Pin<Box<dyn Future>> 包装
async fn call_dyn(f: Box<dyn Fn() -> Pin<Box<dyn Future<Output = i32>>>>) {
    let result = f().await;
    println!("动态分发结果: {}", result);
}

#[tokio::main]
async fn main() {
    let f = Box::new(|| Box::pin(async { 100 }));
    call_dyn(f).await;
}
```

> 异步闭包作为参数时，泛型版本性能更好但写起来啰嗦，trait object 更灵活但有一层间接调用开销——选择取决于你的 API 设计需求。

## 4. 异步闭包多次调用

```rust
#[tokio::main]
async fn main() {
    let multiply = async |x: i32| {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        x * 2
    };

    // 多次调用同一个闭包
    let r1 = multiply(1).await;
    let r2 = multiply(2).await;
    let r3 = multiply(3).await;

    println!("{} {} {}", r1, r2, r3); // 2 4 6
}
```

### 4.1 FuturesUnordered 批量执行

```rust
use futures::stream::FuturesUnordered;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let async_op = async |x: i32| {
        tokio::time::sleep(std::time::Duration::from_millis((100 - x * 10) as u64)).await;
        x * x
    };

    let mut tasks = FuturesUnordered::new();
    for i in 1..=8 {
        tasks.push(async_op(i));
    }

    while let Some(result) = tasks.next().await {
        println!("任务完成: {}", result);
    }
}
```

## 5. 异步闭包组合

### 5.1 嵌套闭包

```rust
#[tokio::main]
async fn main() {
    let outer = async || {
        println!("外层开始");
        let inner = async || {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            "内层结果"
        };
        let inner_result = inner().await;
        println!("外层结束: {}", inner_result);
        42
    };

    outer().await;
}
```

### 5.2 链式组合

```rust
#[tokio::main]
async fn main() {
    let step1 = async |x: i32| -> i32 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        x + 1
    };

    let step2 = async |x: i32| -> i32 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        x * 2
    };

    let r1 = step1(5).await;
    let r2 = step2(r1).await;
    println!("流水线结果: {}", r2); // 12
}
```

### 5.3 join! 并发组合

```rust
#[tokio::main]
async fn main() {
    let task = async |label: &str, delay_ms: u64| {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        format!("{} 完成", label)
    };

    let (r1, r2, r3) = tokio::join!(
        task("A", 200),
        task("B", 100),
        task("C", 300),
    );

    println!("{}, {}, {}", r1, r2, r3);
}
```

> 闭包 + join! 的组合可以将"要并发执行的任务集合"表述得非常紧凑——这比单独定义多个 async fn 更灵活。

## 6. 生命周期与状态机

```rust
#[tokio::main]
async fn main() {
    let shared = String::from("共享数据");

    // 异步闭包的生命周期绑定到 shared
    let closure = async || {
        println!("使用共享数据: {}", shared);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        // shared 的不可变借用可以跨越 .await
        println!("仍在使用: {}", shared);
    };

    // shared 在闭包使用期间不能被释放或可变借用
    // 以下代码会被编译器拒绝：
    // let _mutable = &mut shared;
    // closure().await;

    closure().await;
}
```

> 异步闭包的状态机遵守与 async fn 相同的生命周期规则——编译器会在状态机中保存引用元数据，确保悬挂点前后的借用关系正确。

---

## 避坑指南

| 陷阱 | 原因 | 正确做法 |
|------|------|----------|
| 异步闭包忘记 .await | 不调用 .await 的 Future 不会执行 | IDE 会给出未使用 Future 的警告，务必 .await |
| 闭包内可变引用了数据又在外部使用 | 闭包持有可变引用，外部不可再用 | 在闭包调用前后用花括号限定作用域 |
| async move 后外部变量不可用 | move 将所有权移入闭包 | 如需后续使用，预先 clone |
| 异步闭包作为函数参数的签名太复杂 | 泛型签名嵌套了 Future | 抽成类型别名或使用 trait object |
| 在循环中创建异步闭包但未 move | 循环变量被借用的生命周期问题 | 在循环体内 `async move` 捕获值的所有权 |
| 忘记闭包捕获了 Arc/引用导致内存泄漏 | 循环引用或 Arc 未及时 drop | 检查捕获列表中是否有 `Arc`/`Rc`，必要时使用 `Weak` |

> **详见测试**: `tests/rust_features/21_async_closures.rs`
