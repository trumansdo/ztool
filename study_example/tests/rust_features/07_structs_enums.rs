// ---------------------------------------------------------------------------
// 1.3 结构体与枚举
// ---------------------------------------------------------------------------

use std::mem::{size_of, align_of};

#[test]
/// 测试: 结构体定义与实例化
fn test_struct_definition() {
    // 语法: struct 定义自定义数据类型
    //
    // 三种结构体:
    //   - 命名字段结构体: struct User { name: String, age: u8 }
    //   - 元组结构体: struct Color(i32, i32, i32)
    //   - 单元结构体: struct AlwaysEqual;
    //
    // 实例化:
    //   - 字段可以任意顺序
    //   - 可变实例必须 mut
    //   - 字段简写: email 等价于 email: email
    //
    // 避坑:
    //   - 结构体字段默认私有, 模块外不可见
    //   - 所有字段必须初始化, 不能部分赋值
    //   - 更新语法 .. 会 move 或 copy 字段
    //
    struct User {
        username: String,
        email: String,
        sign_in_count: u64,
        active: bool,
    }

    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        sign_in_count: 1,
        active: true,
    };
    assert_eq!(user.username, "alice");
    assert!(user.active);

    // 字段简写
    fn build_user(email: String, username: String) -> User {
        User {
            email,
            username,
            active: true,
            sign_in_count: 1,
        }
    }
    let user = build_user(
        String::from("bob@example.com"),
        String::from("bob"),
    );
    assert_eq!(user.email, "bob@example.com");

    // 更新语法
    let user2 = User {
        email: String::from("charlie@example.com"),
        ..user
    };
    assert_eq!(user2.username, "bob");
}

#[test]
/// 测试: 元组结构体与单元结构体
fn test_tuple_unit_structs() {
    // 语法:
    //   - 元组结构体: 命名元组包装, 创建新类型
    //   - 单元结构体: 无字段, 用于标记/类型状态
    //
    // 避坑:
    //   - 元组结构体字段通过索引访问
    //   - 不同元组结构体即使内部类型相同也是不同类型
    //

    struct Color(i32, i32, i32);
    struct Point(i32, i32, i32);

    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);

    assert_eq!(black.0, 0);
    assert_eq!(origin.1, 0);

    // 单元结构体
    struct AlwaysEqual;
    let _subject = AlwaysEqual;
}

#[test]
/// 测试: 结构体方法 (impl 块/self/关联函数)
fn test_struct_methods() {
    // 语法: impl 块定义方法
    //
    // self 参数:
    //   - &self: 不可变借用(读)
    //   - &mut self: 可变借用(修改)
    //   - self: 获取所有权(消耗)
    //   - 无 self: 关联函数(类似静态方法)
    //
    // 避坑:
    //   - 关联函数用 :: 调用, 方法用 . 调用
    //   - Rust 自动解引用: &self 和 &mut self 自动处理
    //

    struct Rectangle {
        width: u32,
        height: u32,
    }

    impl Rectangle {
        fn area(&self) -> u32 {
            self.width * self.height
        }

        fn can_hold(&self, other: &Rectangle) -> bool {
            self.width > other.width && self.height > other.height
        }

        fn square(size: u32) -> Rectangle {
            Rectangle {
                width: size,
                height: size,
            }
        }

        // 消耗型方法
        fn destroy(self) -> (u32, u32) {
            (self.width, self.height)
        }
    }

    let rect = Rectangle {
        width: 30,
        height: 50,
    };
    assert_eq!(rect.area(), 1500);

    let other = Rectangle {
        width: 20,
        height: 40,
    };
    assert!(rect.can_hold(&other));
    assert!(!other.can_hold(&rect));

    let sq = Rectangle::square(10);
    assert_eq!(sq.area(), 100);

    // 消耗型方法: self 获取所有权
    let (w, h) = sq.destroy();
    assert_eq!(w, 10);
    assert_eq!(h, 10);
}

#[test]
/// 测试: 结构体内存布局 (对齐/padding/repr(C) vs repr(Rust))
fn test_struct_memory_layout() {
    // 关键概念:
    //   - repr(Rust): 编译器可重排字段以优化大小 (默认)
    //   - repr(C): 按声明顺序排列, 遵循 C ABI 对齐规则
    //   - 编译器会在字段间插入 padding 以满足对齐要求
    //
    // 为什么要关心:
    //   - FFI 交互必须使用 repr(C)
    //   - 对内存敏感的场景了解每种表示的代价
    //

    // ---- repr(Rust) 默认布局: 编译器可以优化 ----
    struct RustLayout {
        a: u8,   // 1 字节
        b: u32,  // 4 字节, 对齐要求 4
        c: u16,  // 2 字节, 对齐要求 2
    }

    // ---- repr(C): 固定顺序, C ABI 兼容 ----
    #[repr(C)]
    struct CLayout {
        a: u8,   // offset 0
        b: u32,  // offset 4 (需要 4 字节对齐, 3 字节 padding)
        c: u16,  // offset 8
    }

    // repr(C) 布局计算:
    // offset 0-0:  a (u8,    1 字节)
    // offset 1-3:  [padding, 3 字节]
    // offset 4-7:  b (u32,   4 字节)
    // offset 8-9:  c (u16,   2 字节)
    // offset 10-11: [padding, 2 字节] (总大小对齐到最大对齐值 4)
    // 总大小: 12 字节

    assert_eq!(size_of::<CLayout>(), 12);
    assert_eq!(align_of::<CLayout>(), 4);

    // repr(Rust) 编译器可以重排字段, 通常更紧凑
    // 但具体布局由编译器决定, 这里只验证它不会比 C 布局更大
    assert!(size_of::<RustLayout>() <= 12,
        "repr(Rust) 应等于或优于 repr(C) 的大小");

    // ---- 紧凑的最佳顺序: 手动重排 ----
    #[repr(C)]
    struct OptimizedLayout {
        b: u32,  // 4 字节, offset 0
        c: u16,  // 2 字节, offset 4
        a: u8,   // 1 字节, offset 6
    }
    // offset 0-3:  b (u32)
    // offset 4-5:  c (u16)
    // offset 6:    a (u8)
    // offset 7:    [padding, 1 字节] → 总大小 8
    assert_eq!(size_of::<OptimizedLayout>(), 8);

    // ---- 对齐示例 ----
    #[repr(C)]
    struct WeirdAlign {
        x: u8,
        y: u64,  // 8 字节对齐
    }
    // offset 0: x, offset 1-7: 7 字节 padding, offset 8-15: y
    assert_eq!(size_of::<WeirdAlign>(), 16);

    // ---- 单元结构体 ---- 零大小
    struct Nothing;
    assert_eq!(size_of::<Nothing>(), 0);

    // ---- 单元元组结构体 ----
    struct AlsoNothing();
    assert_eq!(size_of::<AlsoNothing>(), 0);
}

#[test]
/// 测试: 结构体更新语法及其 Move 语义陷阱
fn test_struct_update_syntax() {
    // 核心陷阱:
    //   ..base 会逐字段赋值。对于 Copy 类型复制, 对于非 Copy 类型 move。
    //   被 move 的字段在原结构体中失效, 但 Copy 字段仍然可用。
    //
    // 关键规则:
    //   - 显式覆盖的字段: 不使用 .. 获取, 对应原字段不受 move 影响
    //   - 被 .. 传递的字段: 非 Copy 的会 move, Copy 的会复制
    //

    #[derive(Debug)]
    struct Config {
        name: String,       // 非 Copy (会 move)
        version: u32,       // Copy
        enabled: bool,      // Copy
        description: String, // 非 Copy (会 move)
    }

    let base = Config {
        name: String::from("myapp"),
        version: 1,
        enabled: true,
        description: String::from("一个应用"),
    };

    // 情况 1: 覆盖非 Copy 字段, .. 传递剩余字段
    let updated = Config {
        description: String::from("更新后的描述"),  // 覆盖此字段
        ..base  // name 被 move, version 和 enabled 被复制
    };
    assert_eq!(updated.name, "myapp");
    assert_eq!(updated.version, 1);
    assert!(updated.enabled);
    assert_eq!(updated.description, "更新后的描述");

    // base.name 和 base.description 被 move 了
    // println!("{}", base.name); // 编译错误!
    // 但 Copy 字段仍然可用
    assert_eq!(base.version, 1);  // OK: u32 是 Copy
    assert!(base.enabled);        // OK: bool 是 Copy

    // 情况 2: 先 clone 需要保留的字段
    let base2 = Config {
        name: String::from("app2"),
        version: 2,
        enabled: false,
        description: String::from("第二应用"),
    };

    let updated2 = Config {
        name: base2.name.clone(),  // 显式 clone, base2.name 不会被 move
        ..base2
    };
    // base2.name 未被 move (我们 clone 了), description 被 move
    assert_eq!(base2.name, "app2");        // OK: 我们 clone 了
    assert_eq!(updated2.name, "app2");     // clone 的值

    // 情况 3: 全部 Copy 字段的结构体不受影响
    #[derive(Debug, Clone, Copy)]
    struct Point {
        x: i32,
        y: i32,
    }

    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 99, ..p1 };
    assert_eq!(p2.x, 99);
    assert_eq!(p2.y, 20);
    // p1 的所有字段仍然可用
    assert_eq!(p1.x, 10);
    assert_eq!(p1.y, 20);
}

#[test]
/// 测试: 元组结构体作为 newtype (类型安全包装)
fn test_tuple_struct_newtype() {
    // newtype 模式:
    //   - 用元组结构体包装单一底层类型
    //   - 零运行时开销, 编译期类型安全
    //   - 可绕过孤儿规则为包装类型实现外部 trait
    //

    // ---- 基本 newtype: 类型安全的度量单位 ----
    #[derive(Debug, PartialEq)]
    struct Meters(f64);

    #[derive(Debug, PartialEq)]
    struct Kilometers(f64);

    impl Meters {
        fn to_kilometers(&self) -> f64 {
            self.0 / 1000.0
        }

        fn add(&self, other: Meters) -> Meters {
            Meters(self.0 + other.0)
        }
    }

    impl Kilometers {
        fn to_meters(&self) -> f64 {
            self.0 * 1000.0
        }
    }

    let distance_m = Meters(1500.0);
    let distance_km = Kilometers(1.5);

    assert!((distance_m.to_kilometers() - 1.5).abs() < f64::EPSILON);
    assert!((distance_km.to_meters() - 1500.0).abs() < f64::EPSILON);

    // Meters 和 Kilometers 是不同的类型, 不能互相赋值
    // let wrong: Kilometers = distance_m; // 编译错误!

    let a = Meters(100.0);
    let b = Meters(200.0);
    assert_eq!(a.add(b), Meters(300.0));

    // ---- newtype 绕过孤儿规则 ----
    // 对 Vec 包装实现自定义方法 (Vec 和 Display trait 都是外部的)
    struct MyVec<T>(Vec<T>);

    impl<T: std::fmt::Display> std::fmt::Display for MyVec<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}]", self.0.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", "))
        }
    }

    let mv = MyVec(vec![1, 2, 3]);
    assert_eq!(format!("{}", mv), "[1, 2, 3]");

    // ---- 零开销验证: newtype 不增加大小 ----
    #[repr(transparent)]
    struct Wrapper(u64);

    assert_eq!(size_of::<Wrapper>(), size_of::<u64>());
}

#[test]
/// 测试: 枚举定义与数据变体
fn test_enum_definition() {
    // 语法: enum 定义枚举类型, 变体可以携带数据
    //
    // 变体数据类型:
    //   - enum IpAddr { V4(u8, u8, u8, u8), V6(String) }
    //   - enum Message { Quit, Move { x: i32, y: i32 }, Write(String) }
    //
    // 避坑:
    //   - 枚举大小 = 最大变体大小 + 判别式标记
    //   - Option<T> 和 Result<T,E> 是标准库最重要的枚举
    //

    enum IpAddr {
        V4(u8, u8, u8, u8),
        V6(String),
    }

    let home = IpAddr::V4(127, 0, 0, 1);
    let _loopback = IpAddr::V6(String::from("::1"));

    // 解构枚举
    match home {
        IpAddr::V4(a, _, _, d) => {
            assert_eq!(a, 127);
            assert_eq!(d, 1);
        }
        _ => panic!("期望 V4"),
    }

    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        ChangeColor(u8, u8, u8),
    }

    let msg = Message::Move { x: 10, y: 20 };
    match msg {
        Message::Move { x, y } => {
            assert_eq!(x, 10);
            assert_eq!(y, 20);
        }
        _ => panic!("期望 Move"),
    }
}

#[test]
/// 测试: 枚举判别式与内存布局
fn test_enum_memory_layout() {
    // 核心概念:
    //   - 判别式 (discriminant): 一个整数标记, 标识当前变体
    //   - 枚举大小 ≈ 最大变体大小 + 判别式大小 + padding
    //   - 判别式默认从 0 开始, isize 大小 (但有优化)
    //

    // ---- 简单枚举: 只有判别式, 无数据 ----
    enum Color {
        Red,
        Green,
        Blue,
    }
    assert!(size_of::<Color>() <= 1); // 通常 1 字节
    assert_eq!(Color::Red as u8, 0);
    assert_eq!(Color::Green as u8, 1);
    assert_eq!(Color::Blue as u8, 2);

    // ---- 显式判别式 ----
    #[repr(u8)]
    enum Status {
        Ok = 0,
        NotFound = 1,
        InternalError = 2,
    }
    assert_eq!(size_of::<Status>(), 1);

    // 可以通过 as 转换获取判别值
    assert_eq!(Status::Ok as u8, 0);
    assert_eq!(Status::NotFound as u8, 1);
    assert_eq!(Status::InternalError as u8, 2);

    // ---- 带数据的枚举: 判别式 + 最大变体大小 ----
    enum RichEnum {
        A,                      // 无数据
        B(u8),                  // 1 字节数据
        C(u64),                 // 8 字节数据 (最大)
    }
    // 大小 ≈ 8 (u64) + 判别式 + padding
    let enum_size = size_of::<RichEnum>();
    assert!(enum_size >= 8 && enum_size <= 16,
        "枚举大小包含判别式和最大变体数据, 实际大小: {}", enum_size);

    // ---- 有数据变体的判别式 ----
    #[repr(u8)]
    #[derive(Debug, PartialEq)]
    enum DataEnum {
        A = 10,
        B = 20,
    }
    assert_eq!(DataEnum::A as u8, 10);
    assert_eq!(DataEnum::B as u8, 20);

    // ---- 判别式自动递增 ----
    #[repr(u8)]
    enum AutoInc {
        First = 3,
        Second,     // 4
        Third = 10,
        Fourth,     // 11
    }
    assert_eq!(AutoInc::Second as u8, 4);
    assert_eq!(AutoInc::Fourth as u8, 11);
}

#[test]
/// 测试: Option 的 null 指针优化 (NPO)
fn test_option_null_pointer_optimization() {
    // Option<T> 当 T 存在"非法值"时无需额外判别式空间:
    //   - &T / &mut T: 引用永远不能为空 (全零), None 用全零表示
    //   - Box<T>: 同引用, 不能为空
    //   - NonZeroU8 / NonZeroI32 等: 0 是非法值
    //   - NonNull<T>: 指针包装, 保证非空
    //
    // 这就是 Rust 的"零成本抽象"——Option 不额外占用空间!
    //

    // ---- 引用类型: Option 大小 == 指针大小 ----
    assert_eq!(size_of::<Option<&i32>>(), size_of::<&i32>());     // 8 字节 (64位)
    assert_eq!(size_of::<Option<&mut i32>>(), size_of::<&mut i32>());
    assert_eq!(size_of::<Option<Box<i32>>>(), size_of::<Box<i32>>());

    // ---- NonZero 类型: Option 大小 == 底层类型大小 ----
    use std::num::{NonZeroU8, NonZeroU32};
    assert_eq!(size_of::<Option<NonZeroU8>>(), size_of::<u8>());    // 1 字节
    assert_eq!(size_of::<Option<NonZeroU32>>(), size_of::<u32>());  // 4 字节

    // ---- 普通类型需要额外的判别式 ----
    assert_eq!(size_of::<Option<u8>>(), 2);   // 1 字节数据 + 1 字节标记
    assert_eq!(size_of::<Option<u32>>(), 8);  // 4 字节数据 + 对齐后 8 字节
    assert_eq!(size_of::<Option<i64>>(), 16); // 8 字节数据 + 对齐后 16 字节

    // ---- bool 也有 NPO (bool 只有 0 和 1 是合法值) ----
    assert_eq!(size_of::<Option<bool>>(), 1);     // 1 字节
    assert_eq!(size_of::<Option<Option<bool>>>(), 1); // 嵌套也只需 1 字节
    // 解释: bool 用 0/1, Option<bool> 用 None=2, 还有空间放更多

    // ---- 验证运行时不额外分配 ----
    let some_ref: Option<&i32> = Some(&42);
    assert!(some_ref.is_some());

    let none_ref: Option<&i32> = None;
    assert!(none_ref.is_none());
}

#[test]
/// 测试: Option 枚举 (替代 null/安全处理)
fn test_option_enum() {
    // 语法: Option<T> = Some(T) | None
    //
    // 特性:
    //   - 消灭了 null 引用("十亿美元的错误")
    //   - 编译器强制处理 None 情况
    //   - 泛型 T 可以是任何类型
    //
    // 常用方法:
    //   - unwrap() / expect("msg")    取出值(可能 panic)
    //   - is_some() / is_none()       检查
    //   - map() / and_then() / or_else()
    //   - unwrap_or(default)          有默认值
    //   - take() / replace()          替换/取出
    //
    // 避坑:
    //   - unwrap 在 None 时 panic, 仅在确定有值时使用
    //   - expect 提供比 unwrap 更好的错误信息
    //
    let some_number = Some(5);
    let absent_number: Option<i32> = None;

    assert_eq!(some_number.unwrap(), 5);
    assert_eq!(absent_number.unwrap_or(0), 0);

    // Option 运算
    assert!(some_number.is_some());
    assert!(absent_number.is_none());

    // map
    let doubled = some_number.map(|x| x * 2);
    assert_eq!(doubled, Some(10));

    let _doubled_none: Option<i32> = None;
    assert_eq!(None::<i32>.map(|x| x * 2), None);

    // ---- 新增: and_then (链式调用) ----
    fn try_parse(s: &str) -> Option<i32> {
        s.parse::<i32>().ok()
    }

    let result = Some("42")
        .and_then(try_parse)
        .map(|n| n + 1);
    assert_eq!(result, Some(43));

    let result_none: Option<i32> = None
        .and_then(try_parse)
        .map(|n| n + 1);
    assert_eq!(result_none, None);

    // ---- 新增: take() / replace() ----
    let mut opt = Some(String::from("hello"));
    let taken = opt.take();
    assert_eq!(taken, Some(String::from("hello")));
    assert_eq!(opt, None);

    let mut opt2 = Some(10);
    let old = opt2.replace(20);
    assert_eq!(old, Some(10));
    assert_eq!(opt2, Some(20));

    // ---- 新增: unwrap_or_else (懒计算默认值) ----
    let counter = std::cell::Cell::new(0);
    let val = None::<i32>.unwrap_or_else(|| {
        counter.set(counter.get() + 1);
        42
    });
    assert_eq!(val, 42);
    assert_eq!(counter.get(), 1); // 只在 None 时才调用闭包

    // Some 时不会调用闭包
    let _val2 = Some(7).unwrap_or_else(|| {
        counter.set(counter.get() + 1);
        0
    });
    assert_eq!(counter.get(), 1); // 仍然是 1
}

#[test]
/// 测试: Result 枚举 (错误处理基础)
fn test_result_enum() {
    // 语法: Result<T, E> = Ok(T) | Err(E)
    //
    // 避坑:
    //   - Result 必须被处理, 编译器警告未使用的 Result
    //   - ? 操作符自动传播 Err
    //

    fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err(String::from("零除错误"))
        } else {
            Ok(a / b)
        }
    }

    assert!(safe_divide(10.0, 2.0).is_ok());
    assert!(safe_divide(10.0, 0.0).is_err());

    assert_eq!(safe_divide(10.0, 2.0).unwrap(), 5.0);
    assert_eq!(
        safe_divide(10.0, 0.0).unwrap_or(-1.0),
        -1.0
    );

    // ---- 新增: Result 链式调用 ----
    let chained = safe_divide(20.0, 4.0)
        .map(|x| x * 3.0)
        .and_then(|x| safe_divide(x, 3.0));
    assert!((chained.unwrap() - 5.0).abs() < f64::EPSILON);

    // ---- 新增: Result 的 map_err ----
    let result: Result<i32, String> = Err(String::from("出错了"));
    let mapped = result.map_err(|e| format!("包装: {}", e));
    assert_eq!(mapped, Err(String::from("包装: 出错了")));
}

#[test]
/// 测试: 枚举方法 (impl 枚举) — 增强版
fn test_enum_methods() {
    // 语法: 枚举也可以有方法, 与结构体一样用 impl 块
    // 方法内部通常用 match 根据变体分发逻辑
    //

    enum Message {
        Quit,
        Write(String),
        Move { x: i32, y: i32 },
        ChangeColor(u8, u8, u8),
    }

    impl Message {
        fn call(&self) -> Option<&str> {
            match self {
                Message::Quit => None,
                Message::Write(s) => Some(s.as_str()),
                Message::Move { .. } => None,
                Message::ChangeColor(..) => None,
            }
        }

        // 判断是否为 Quit
        fn is_quit(&self) -> bool {
            matches!(self, Message::Quit)
        }

        // 获取消息的文本表示
        fn describe(&self) -> String {
            match self {
                Message::Quit => "退出".to_string(),
                Message::Write(s) => format!("写入: {}", s),
                Message::Move { x, y } => format!("移动到 ({}, {})", x, y),
                Message::ChangeColor(r, g, b) => {
                    format!("颜色 ({}, {}, {})", r, g, b)
                }
            }
        }

        // 关联函数: 创建常用消息
        fn hello() -> Message {
            Message::Write(String::from("你好, 世界!"))
        }

        fn origin() -> Message {
            Message::Move { x: 0, y: 0 }
        }

        // 消耗型方法
        fn into_text(self) -> Option<String> {
            match self {
                Message::Write(s) => Some(s),
                _ => None,
            }
        }
    }

    let msg = Message::Write(String::from("hello"));
    assert_eq!(msg.call(), Some("hello"));

    let quit = Message::Quit;
    assert_eq!(quit.call(), None);

    // 新增断言
    let move_msg = Message::Move { x: 5, y: 10 };
    assert_eq!(move_msg.describe(), "移动到 (5, 10)");
    assert!(!move_msg.is_quit());
    assert!(quit.is_quit());

    let color_msg = Message::ChangeColor(255, 128, 0);
    assert_eq!(color_msg.describe(), "颜色 (255, 128, 0)");

    // 关联函数
    let hello_msg = Message::hello();
    assert_eq!(hello_msg.call(), Some("你好, 世界!"));

    let origin_msg = Message::origin();
    match origin_msg {
        Message::Move { x, y } => {
            assert_eq!(x, 0);
            assert_eq!(y, 0);
        }
        _ => panic!("期望 Move"),
    }

    // 消耗型方法: into_text 会消耗 self
    let write_msg = Message::Write(String::from("消耗我"));
    assert_eq!(write_msg.into_text(), Some(String::from("消耗我")));
    // write_msg 已被 move, 不能再使用

    // matches! 宏——快速布尔判断
    let msg2 = Message::Write(String::from("测试"));
    assert!(matches!(msg2, Message::Write(_)));
    assert!(!matches!(msg2, Message::Quit));
}

#[test]
/// 测试: match 与枚举的穷尽性匹配
fn test_match_with_enum() {
    // 语法: match 必须穷举所有变体
    //
    // 避坑:
    //   - 漏掉变体会编译失败
    //   - 使用 _ 通配符处理剩余情况
    //

    enum Coin {
        Penny,
        Nickel,
        Dime,
        Quarter,
    }

    fn value_in_cents(coin: Coin) -> u8 {
        match coin {
            Coin::Penny => 1,
            Coin::Nickel => 5,
            Coin::Dime => 10,
            Coin::Quarter => 25,
        }
    }

    assert_eq!(value_in_cents(Coin::Penny), 1);
    assert_eq!(value_in_cents(Coin::Quarter), 25);

    // ---- 新增: 枚举+守卫条件 (match guard) ----
    enum Rating {
        Score(u8),
        NoRating,
    }

    fn rating_to_label(r: Rating) -> &'static str {
        match r {
            Rating::Score(s) if s >= 90 => "优秀",
            Rating::Score(s) if s >= 60 => "合格",
            Rating::Score(_) => "不合格",
            Rating::NoRating => "未评分",
        }
    }

    assert_eq!(rating_to_label(Rating::Score(95)), "优秀");
    assert_eq!(rating_to_label(Rating::Score(70)), "合格");
    assert_eq!(rating_to_label(Rating::Score(30)), "不合格");
    assert_eq!(rating_to_label(Rating::NoRating), "未评分");
}

#[test]
/// 测试: 常见 Trait 派生 (#[derive])
fn test_derive_traits() {
    // 语法: #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    //
    // 避坑:
    //   - 所有字段必须实现了对应的 trait
    //   - Copy 要求所有字段都是 Copy
    //

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let p1 = Point { x: 1, y: 2 };
    let p2 = p1.clone();
    assert_eq!(p1, p2);

    let p3 = p1; // Copy
    assert_eq!(p1, p3);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Status {
        Active,
        Inactive,
    }

    let s1 = Status::Active;
    let s2 = Status::Active;
    assert_eq!(s1, s2);

    // ---- 新增: Hash derive ----
    use std::collections::HashSet;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum OrderStatus {
        Pending,
        Shipped,
        Delivered,
        Cancelled,
    }

    let mut set = HashSet::new();
    set.insert(OrderStatus::Pending);
    set.insert(OrderStatus::Pending); // 重复
    set.insert(OrderStatus::Shipped);
    assert_eq!(set.len(), 2); // Pending 和 Shipped
}

#[test]
/// 测试: 结构体模式 (解构/匹配)
fn test_struct_destructuring() {
    // 语法: 解构结构体取出字段值
    //
    // 模式:
    //   - let Point { x, y } = p;     全部解构
    //   - let Point { x, .. } = p;    部分解构
    //   - match p { Point { x: 0, y } => ... }
    //

    struct Point {
        x: i32,
        y: i32,
    }

    let p = Point { x: 0, y: 7 };

    // 解构
    let Point { x, y } = p;
    assert_eq!(x, 0);
    assert_eq!(y, 7);

    // match 解构
    match (Point { x: 3, y: 4 }) {
        Point { x: 0, y } => assert_eq!(y, -1),
        Point { x, y } => {
            assert_eq!(x, 3);
            assert_eq!(y, 4);
        }
    }

    // ---- 新增: 嵌套解构 ----
    struct Line {
        start: Point,
        end: Point,
    }

    let line = Line {
        start: Point { x: 0, y: 0 },
        end: Point { x: 5, y: 10 },
    };

    let Line { start: Point { x: x1, y: y1 }, end: Point { x: x2, y: y2 } } = line;
    assert_eq!((x1, y1), (0, 0));
    assert_eq!((x2, y2), (5, 10));

    // ---- 新增: if let 解构 ----
    let p = Point { x: 3, y: 9 };
    if let Point { x, y: 9 } = p {
        assert_eq!(x, 3);
    } else {
        panic!("应匹配 y=9");
    }
}

#[test]
/// 测试: 带泛型的结构体与枚举
fn test_enum_generics() {
    // 泛型结构体和枚举: 类型安全的代码复用
    // 泛型参数在类型定义和 impl 块中需要声明
    //

    // ---- 泛型结构体 ----
    #[derive(Debug, PartialEq)]
    struct Point<T> {
        x: T,
        y: T,
    }

    let int_pt = Point { x: 1, y: 2 };
        let float_pt: Point<f64> = Point { x: 1.5, y: 3.2 };
        assert_eq!(int_pt.x, 1);
        assert!((float_pt.y - 3.2).abs() < f64::EPSILON);

    // 多泛型参数
    #[derive(Debug, PartialEq)]
    struct Pair<A, B> {
        first: A,
        second: B,
    }

    let pair = Pair { first: 42u32, second: "hello" };
    assert_eq!(pair.first, 42);
    assert_eq!(pair.second, "hello");

    // impl 泛型结构体
    impl<T: Clone> Point<T> {
        fn new(x: T, y: T) -> Self {
            Point { x, y }
        }

        fn x(&self) -> T {
            self.x.clone()
        }
    }

    let pt = Point::new(10, 20);
    assert_eq!(pt.x(), 10);

    // ---- 泛型枚举: Option ----
    let opt_int: Option<i32> = Some(42);
    let opt_str: Option<&str> = Some("hello");
    assert_eq!(opt_int, Some(42));
    assert_eq!(opt_str, Some("hello"));

    // ---- 泛型枚举: Result ----
    fn parse_or_default<T: std::str::FromStr>(s: &str, default: T) -> T {
        s.parse::<T>().unwrap_or(default)
    }
    assert_eq!(parse_or_default("42", 0), 42);
    assert_eq!(parse_or_default("abc", 10), 10);

    // ---- 自定义泛型枚举: Either ----
    #[derive(Debug, PartialEq)]
    enum Either<L, R> {
        Left(L),
        Right(R),
    }

    impl<L, R> Either<L, R> {
        fn is_left(&self) -> bool {
            matches!(self, Either::Left(_))
        }

        fn left(self) -> Option<L> {
            match self {
                Either::Left(l) => Some(l),
                Either::Right(_) => None,
            }
        }

        fn right(self) -> Option<R> {
            match self {
                Either::Left(_) => None,
                Either::Right(r) => Some(r),
            }
        }
    }

    let left_val: Either<i32, &str> = Either::Left(42);
    let right_val: Either<i32, &str> = Either::Right("world");

    assert!(left_val.is_left());
    assert!(!right_val.is_left());
    assert_eq!(left_val.left(), Some(42));
    assert_eq!(right_val.right(), Some("world"));

    // ---- 泛型 + where 子句 ----
    #[derive(Debug)]
    enum ComputationResult<T>
    where
        T: std::fmt::Debug + Clone,
    {
        Success(T),
        Retryable(T, u32),      // 值 + 重试次数
        Failed(String),
    }

    let success = ComputationResult::Success(vec![1, 2, 3]);
    let failed: ComputationResult<Vec<i32>> = ComputationResult::Failed(String::from("网络错误"));
    match success {
        ComputationResult::Success(v) => assert_eq!(v, vec![1, 2, 3]),
        _ => panic!(),
    }
    match &failed {
        ComputationResult::Failed(msg) => assert_eq!(msg, "网络错误"),
        _ => panic!(),
    }
}

#[test]
/// 测试: repr 属性对布局的影响 (repr(C)/repr(u8)/repr(transparent))
fn test_repr_attributes() {
    // repr 属性控制内存布局:
    //   repr(C)      - C ABI 兼容布局
    //   repr(u8)     - 判别式存为 u8, 枚举大小 ≤ u8
    //   repr(usize)  - 判别式存为 usize
    //   repr(transparent) - 单字段类型与内部类型相同布局
    //   repr(packed) - 无 padding (慎用)
    //

    // ---- repr(C) vs 默认 ----
    #[repr(C)]
    struct CStruct {
        a: u8,
        b: u32,
    }
    // repr(C): a(1), padding(3), b(4) = 8 字节 (对齐到 4)
    assert_eq!(size_of::<CStruct>(), 8);

    struct DefaultStruct {
        a: u8,
        b: u32,
    }
    // 默认: 编译器可能重排
    assert!(size_of::<DefaultStruct>() <= 8);

    // ---- repr(u8): 判别式为 u8 ----
    #[repr(u8)]
    enum TinyEnum {
        A,
        B,
        C,
        D,
    }
    assert_eq!(size_of::<TinyEnum>(), 1);

    // 可以包含最多 256 个变体
    assert_eq!(TinyEnum::A as u8, 0);
    assert_eq!(TinyEnum::D as u8, 3);

    // ---- repr(u32): 大判别式 ----
    #[repr(u32)]
    enum LargeEnum {
        A = 1000,
        B = 2000,
        C = 3000,
    }
    assert!(size_of::<LargeEnum>() >= 4);

    // ---- repr(transparent): 零开销包装 ----
    #[repr(transparent)]
    struct TransparentWrapper(u64);

    // 与内部类型完全相同的大小和对齐
    assert_eq!(size_of::<TransparentWrapper>(), size_of::<u64>());
    assert_eq!(align_of::<TransparentWrapper>(), align_of::<u64>());

    // ---- repr(transparent) 对枚举 ----
    #[repr(transparent)]
    enum TransparentEnum {
        Variant(u64),
    }
    assert_eq!(size_of::<TransparentEnum>(), size_of::<u64>());

    // ---- repr(align(N)) 强制对齐 ----
    #[repr(align(16))]
    struct AlignedStruct {
        data: [u8; 4],
    }
    assert_eq!(align_of::<AlignedStruct>(), 16);

    // ---- repr(C) 对枚举: 大小有保证 ----
    #[repr(C)]
    enum CEnum {
        A = 1,
        B = 2,
    }
    // repr(C) 枚举的判别式大小至少是 int (C 的 int)
    // 在大多数平台上这等于 4
    let c_enum_size = size_of::<CEnum>();
    assert!(c_enum_size >= 4, "repr(C) 枚举至少是 C int 大小");

    // ---- 通过 transmute 的 unsafe 转换 (仅用于演示!) ----
    #[repr(i32)]
    enum SignedEnum {
        NegOne = -1,
        Zero = 0,
        PosOne = 1,
    }
    assert_eq!(SignedEnum::NegOne as i32, -1);
    assert_eq!(SignedEnum::PosOne as i32, 1);
}

#[test]
/// 测试: "让非法状态不可表达" 的实践演示
fn test_illegal_state_unrepresentable() {
    // 核心思想:
    //   用类型系统在编译器层面阻止无效状态的产生。
    //   每个 boolean 都应该问自己: 是否应该是一个枚举?
    //
    // 坏设计: bool 组合爆炸
    // 好设计: 枚举精确建模合法状态
    //

    // ---- 示例 1: 连接状态 ----
    // 坏设计 (非法状态可表达):
    // struct Connection {
    //     connected: bool,
    //     stream: Option<TcpStream>,  // connected=true && stream=None 是非法状态!
    // }
    //
    // 好设计 (非法状态不可表达):
    mod good_design {
        #[derive(Debug, PartialEq)]
        pub enum ConnectionState {
            Disconnected,
            Connected(u64),  // 一旦 Connected, 必然持有有效的连接 ID
        }

        impl ConnectionState {
            pub fn new() -> Self {
                ConnectionState::Disconnected
            }

            pub fn connect(&mut self, conn_id: u64) {
                *self = ConnectionState::Connected(conn_id);
            }

            pub fn disconnect(&mut self) -> Option<u64> {
                match self {
                    ConnectionState::Connected(id) => {
                        let old_id = *id;
                        *self = ConnectionState::Disconnected;
                        Some(old_id)
                    }
                    ConnectionState::Disconnected => None,
                }
            }

            pub fn is_connected(&self) -> bool {
                matches!(self, ConnectionState::Connected(_))
            }

            pub fn conn_id(&self) -> Option<u64> {
                match self {
                    ConnectionState::Connected(id) => Some(*id),
                    ConnectionState::Disconnected => None,
                }
            }
        }
    }

    let mut conn = good_design::ConnectionState::new();
    assert!(!conn.is_connected());
    assert_eq!(conn.conn_id(), None);

    conn.connect(42);
    assert!(conn.is_connected());
    assert_eq!(conn.conn_id(), Some(42));

    let old_id = conn.disconnect();
    assert_eq!(old_id, Some(42));
    assert!(!conn.is_connected());

    // ---- 示例 2: 网络请求方法 ----
    // 坏设计:
    // struct Request {
    //     method: String,         // 可能是任意非法值
    //     body: Option<Vec<u8>>,  // GET 请求不应该有 body
    // }
    //
    // 好设计:
    mod http_model {
        #[derive(Debug, PartialEq)]
        pub enum Method {
            Get,
            Post(Vec<u8>),  // body 与 POST 强制绑定
            Put(Vec<u8>),
            Delete,
        }

        impl Method {
            pub fn is_safe(&self) -> bool {
                matches!(self, Method::Get | Method::Delete)
            }

            pub fn body(&self) -> Option<&[u8]> {
                match self {
                    Method::Post(body) | Method::Put(body) => Some(body),
                    _ => None,
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    Method::Get => "GET",
                    Method::Post(_) => "POST",
                    Method::Put(_) => "PUT",
                    Method::Delete => "DELETE",
                }
            }
        }
    }

    let get = http_model::Method::Get;
    let post = http_model::Method::Post(b"hello".to_vec());
    let delete = http_model::Method::Delete;

    assert!(get.is_safe());
    assert!(!post.is_safe());
    assert_eq!(get.body(), None);
    assert_eq!(post.body(), Some(&b"hello"[..]));
    assert_eq!(delete.as_str(), "DELETE");

    // ---- 示例 3: 表单验证状态 ----
    mod form_validation {
        #[derive(Debug, PartialEq)]
        pub struct Validated;

        #[derive(Debug, PartialEq)]
        pub struct Unvalidated;

        #[derive(Debug, PartialEq)]
        pub struct Form<State> {
            pub data: String,
            _state: std::marker::PhantomData<State>,
        }

        impl Form<Unvalidated> {
            pub fn new(data: String) -> Self {
                Form { data, _state: std::marker::PhantomData }
            }

            pub fn validate(self) -> Result<Form<Validated>, String> {
                if self.data.is_empty() {
                    Err(String::from("不能为空"))
                } else {
                    Ok(Form { data: self.data, _state: std::marker::PhantomData })
                }
            }
        }

        impl Form<Validated> {
            pub fn data(&self) -> &str {
                &self.data
            }

            pub fn submit(&self) -> String {
                format!("已提交: {}", self.data)
            }
        }
    }

    // 未验证的表单不能 submit
    let form = form_validation::Form::<form_validation::Unvalidated>::new(
        String::from("用户输入")
    );
    // form.submit(); // 编译错误: Unvalidated 上没有 submit 方法

    let validated = form.validate().expect("验证失败");
    assert_eq!(validated.submit(), "已提交: 用户输入");

    // 空表单调试验证失败
    let empty_form = form_validation::Form::<form_validation::Unvalidated>::new(
        String::new()
    );
    assert!(empty_form.validate().is_err());

    // ---- 示例 4: 状态组合——为什么枚举更好 ----
    // 2 个 bool = 4 种状态组合, 但实际只有 3 种有意义
    enum LoadState<T> {
        NotStarted,
        Loading,
        Loaded(T),
        // 不存在 "NotStarted 但有数据" 或 "Loading 且有完整数据" 的非法状态
    }

    let state: LoadState<String> = LoadState::NotStarted;
    assert!(matches!(state, LoadState::NotStarted));

    let state = LoadState::Loaded(String::from("数据"));
    match state {
        LoadState::Loaded(ref data) => assert_eq!(data, "数据"),
        _ => panic!(),
    }
}
