// ---------------------------------------------------------------------------
// 3.1 类型系统 - impl Trait / dyn Trait
// ---------------------------------------------------------------------------

#[test]
/// 测试: impl Trait 返回隐藏具体类型 (静态分发)
fn test_impl_trait() {
    // 语法: impl Trait 作为返回值, 隐藏具体类型(静态分发, 零开销)
    // 避坑: 函数所有返回路径必须返回同一具体类型; 调用方无法知道具体类型
    fn returns_impl_trait() -> impl std::fmt::Display {
        42
    }
    assert_eq!(format!("{}", returns_impl_trait()), "42");
}

#[test]
/// 测试: impl Trait 作为函数参数 (隐式泛型)
fn test_impl_trait_as_param() {
    // 语法: impl Trait 作为参数是隐式泛型参数的语法糖
    // 避坑: 每个 impl Trait 参数是独立的泛型参数, 不能假设两个 impl Trait 是同一类型
    fn print_it(x: impl std::fmt::Display) {
        let _ = format!("{}", x);
    }
    print_it(42);
    print_it("hello");

    // 两个 impl Trait 是不同类型
    fn add_display(a: impl std::fmt::Display, b: impl std::fmt::Display) {
        // a 和 b 可以是不同类型
        let _ = format!("{} {}", a, b);
    }
    add_display(42, "hello");
}

#[test]
/// 测试: impl Trait 多 trait 约束 (impl Trait1 + Trait2)
fn test_impl_trait_multiple() {
    // 语法: impl Trait1 + Trait2 要求类型同时实现多个 trait
    // 避坑: 只能有一个主 trait, 其余必须是 auto trait (Send, Sync, Unpin 等)
    //       或使用 where 子句约束
    fn process<T: std::fmt::Display + std::fmt::Debug>(x: T) -> String {
        format!("{}, {:?}", x, x)
    }
    let result = process(42i32);
    assert!(result.contains("42"));
}

#[test]
/// 测试: impl Trait 在闭包中的使用
fn test_impl_trait_in_closure() {
    // 语法: impl Trait 可用于闭包参数和返回值 (Rust 1.78+)
    // 避坑: 闭包的 impl Trait 参数也是隐式泛型
    fn apply(f: impl Fn(i32) -> i32, x: i32) -> i32 {
        f(x)
    }
    assert_eq!(apply(|x| x * 2, 5), 10);
}

#[test]
/// 测试: dyn Trait 动态分发 (trait 对象/虚表)
fn test_dyn_trait() {
    // 语法: dyn Trait 是 trait 对象(动态分发), 通过虚表调用方法
    // 避坑: trait 必须对象安全(不能有泛型方法/返回 Self); &dyn Trait 是胖指针(数据指针+虚表指针)
    trait Foo {
        fn bar(&self) -> i32;
    }

    struct Baz;
    impl Foo for Baz {
        fn bar(&self) -> i32 {
            42
        }
    }

    let obj: &dyn Foo = &Baz;
    assert_eq!(obj.bar(), 42);
}

#[test]
/// 测试: dyn Trait 作为函数参数和返回值
fn test_dyn_trait_as_param_and_return() {
    // 语法: dyn Trait 可作为参数(接受任何实现该 trait 的类型)和返回值
    // 避坑: dyn Trait 返回值必须是 Box<dyn Trait> 或 &dyn Trait, 不能直接返回
    trait Shape {
        fn area(&self) -> f64;
    }

    struct Circle {
        radius: f64,
    }
    impl Shape for Circle {
        fn area(&self) -> f64 {
            std::f64::consts::PI * self.radius * self.radius
        }
    }

    struct Rectangle {
        width: f64,
        height: f64,
    }
    impl Shape for Rectangle {
        fn area(&self) -> f64 {
            self.width * self.height
        }
    }

    fn print_area(s: &dyn Shape) -> f64 {
        s.area()
    }

    let c = Circle { radius: 1.0 };
    let r = Rectangle {
        width: 2.0,
        height: 3.0,
    };
    assert!((print_area(&c) - std::f64::consts::PI).abs() < 0.001);
    assert_eq!(print_area(&r), 6.0);

    // 返回 Box<dyn Trait>
    fn create_shape(is_circle: bool) -> Box<dyn Shape> {
        if is_circle {
            Box::new(Circle { radius: 1.0 })
        } else {
            Box::new(Rectangle {
                width: 1.0,
                height: 1.0,
            })
        }
    }

    let s = create_shape(true);
    assert!((s.area() - std::f64::consts::PI).abs() < 0.001);
}

#[test]
/// 测试: 对象安全性 (Object Safety)
fn test_object_safety() {
    // 语法: trait 要作为 dyn Trait 使用, 必须对象安全:
    //   - 不能有泛型方法
    //   - 不能有返回 Self 的方法 (Self: Sized 除外)
    //   - 不能有 Self 作为参数的方法
    //   - 不能要求 Self: Sized
    //
    // 避坑:
    //   - 可以用 where Self: Sized 标记不对象安全的方法, 其余方法仍可用于 dyn Trait
    //   - 泛型方法可以用, 但不能通过 dyn Trait 调用

    trait MyTrait {
        fn regular_method(&self) -> i32;

        // 标记为 Self: Sized 的方法不能通过 dyn Trait 调用
        fn generic_method<T>(&self, _x: T) -> i32
        where
            Self: Sized,
        {
            0
        }
    }

    struct MyType;
    impl MyTrait for MyType {
        fn regular_method(&self) -> i32 {
            42
        }
    }

    // 可以通过 dyn Trait 调用 regular_method
    let obj: &dyn MyTrait = &MyType;
    assert_eq!(obj.regular_method(), 42);

    // generic_method 有 Self: Sized 约束，必须用具体类型调用
    let concrete = MyType;
    assert_eq!(concrete.generic_method(42), 0);
    // 或者显式指定类型参数
    assert_eq!(concrete.generic_method::<i32>(42), 0);
}

#[test]
/// 测试: trait upcasting 子trait向上转型为超trait (1.86+)
fn test_trait_upcasting() {
    // 语法: 子 trait 对象可自动向上转型为超 trait 对象 (1.86+)
    // 避坑: 必须有明确的超 trait 关系(Sub: Super); 向上转型后只能调用超 trait 的方法
    trait Super {
        fn super_method(&self) -> i32;
    }

    trait Sub: Super {
        fn sub_method(&self) -> i32;
    }

    struct MyStruct;
    impl Super for MyStruct {
        fn super_method(&self) -> i32 {
            10
        }
    }
    impl Sub for MyStruct {
        fn sub_method(&self) -> i32 {
            20
        }
    }

    fn upcast(x: &dyn Sub) -> &dyn Super {
        x
    }

    let obj = MyStruct;
    let sub: &dyn Sub = &obj;
    assert_eq!(sub.sub_method(), 20);

    // Rust 中 trait upcasting 是隐式的，不需要强制转换语法
    // &dyn Sub 可以直接赋值给 &dyn Super
    let super_ref: &dyn Super = sub;
    assert_eq!(super_ref.super_method(), 10);

    // 也可以通过函数参数隐式转换
    assert_eq!(upcast(sub).super_method(), 10);

    // 如果需要显式转换（类似 Java 的 (Super) obj），Rust 1.86+ 可以用 as
    // 但其实隐式转换已经足够了，这是 Rust 和 Java 的主要区别
    let explicit_super = sub as &dyn Super;
    assert_eq!(explicit_super.super_method(), 10);
}

#[test]
/// 测试: Any upcasting 和 downcast_ref 类型向下转型
fn test_any_upcast() {
    // 语法: 自定义 trait 继承 Any 后, 可通过 upcast 实现 downcast_ref
    // 避坑: 必须在 trait 对象上实现 downcast_ref, 利用 as &dyn Any 向上转型
    use std::any::Any;

    trait MyAny: Any {}
    impl dyn MyAny {
        fn downcast_ref<T: Any>(&self) -> Option<&T> {
            (self as &dyn Any).downcast_ref()
        }
    }
    impl MyAny for i32 {}

    let val: i32 = 42;
    let my_any: &dyn MyAny = &val;
    assert_eq!(my_any.downcast_ref::<i32>(), Some(&42));
}

#[test]
/// 测试: dyn Any 直接向下转型
fn test_dyn_any_downcast() {
    // 语法: std::any::Any 提供类型运行时反射, 可 downcast/downcast_ref
    // 避坑: Any 只适用于 'static 生命周期(不含借用引用); downcast 消费值, downcast_ref 不消费
    use std::any::Any;

    let value: String = String::from("hello");
    let any: &dyn Any = &value;

    // downcast_ref (不消费)
    assert_eq!(any.downcast_ref::<String>(), Some(&String::from("hello")));
    assert_eq!(any.downcast_ref::<i32>(), None);

    // downcast (消费, 需要 owned 值)
    let value2: Box<dyn Any> = Box::new(42i32);
    let unboxed = value2
        .downcast::<i32>()
        .unwrap();
    assert_eq!(*unboxed, 42);
}

#[test]
/// 测试: 返回位置 impl Trait 隐藏具体类型
fn test_impl_trait_return() {
    // 语法: impl Trait 作为返回值隐藏具体类型, 调用方不知道实现细节
    // 避坑: 函数所有返回路径必须返回同一具体类型; 不同于 type alias impl trait (unstable)
    fn create_iter() -> impl Iterator<Item = i32> {
        (0..5).into_iter()
    }

    let result: Vec<i32> = create_iter().collect();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

#[test]
/// 测试: 返回位置协变 (Return Position Impl Trait in Trait)
fn test_rpitit() {
    // 语法: trait 定义中可以使用 impl Trait 作为返回类型 (1.75+)
    // 避坑: 实现必须返回的具体类型在编译期确定; 不同实现可返回不同类型
    trait Generator {
        fn generate(&self) -> impl Iterator<Item = i32>;
    }

    struct RangeGen;
    impl Generator for RangeGen {
        fn generate(&self) -> impl Iterator<Item = i32> {
            0..3
        }
    }

    struct VecGen;
    impl Generator for VecGen {
        fn generate(&self) -> impl Iterator<Item = i32> {
            vec![10, 20, 30].into_iter()
        }
    }

    let range: Vec<i32> = RangeGen.generate().collect();
    assert_eq!(range, vec![0, 1, 2]);

    let vec_result: Vec<i32> = VecGen.generate().collect();
    assert_eq!(vec_result, vec![10, 20, 30]);
}

#[test]
/// 测试: Sized trait 和 ?Sized
fn test_sized_trait() {
    // 语法: Sized 是自动实现的标记 trait, 表示类型大小编译期已知
    //   - 所有泛型参数默认隐含 T: Sized
    //   - ?Sized 放宽此限制, 允许动态大小类型 (dyn Trait, str, [T])
    //
    // 避坑:
    //   - fn foo<T>(x: T) 等价于 fn foo<T: Sized>(x: T)
    //   - fn foo<T: ?Sized>(x: &T) 允许传入 &str, &dyn Trait 等
    //   - ?Sized 只能用于 trait 约束, 不能用于具体类型

    // 默认 T: Sized, 不能传入 str
    fn takes_sized<T: std::fmt::Display>(x: T) -> String {
        format!("{}", x)
    }
    assert_eq!(takes_sized(42), "42");

    // T: ?Sized, 可以传入 &str
    fn takes_maybe_unsized<T: std::fmt::Display + ?Sized>(x: &T) -> String {
        format!("{}", x)
    }
    assert_eq!(takes_maybe_unsized("hello"), "hello");
    assert_eq!(takes_maybe_unsized(&42), "42");
}

#[test]
/// 测试: 零大小类型 (ZST - Zero Sized Types)
fn test_zero_sized_types() {
    // 语法: ZST 是不占内存的类型, sizeof 为 0
    //   - () 单元类型
    //   - 空结构体 struct Empty;
    //   - 标记类型 (phantom types)
    //   - 函数指针类型 fn()
    //
    // 避坑:
    //   - ZST 不分配内存, 但 Vec<ZST> 的 len 仍然有效
    //   - 不能用 &ZST 做指针运算
    //   - 编译器会优化掉 ZST 的所有运行时操作

    struct Marker;
    assert_eq!(std::mem::size_of::<Marker>(), 0);
    assert_eq!(std::mem::size_of::<()>(), 0);
    assert_eq!(std::mem::size_of::<std::convert::Infallible>(), 0);

    // Vec<ZST> 仍然可以追踪长度
    let v = vec![Marker, Marker, Marker];
    assert_eq!(v.len(), 3);
}
