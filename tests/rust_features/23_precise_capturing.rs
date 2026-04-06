// ---------------------------------------------------------------------------
// 5.6 精确捕获 (Precise Capturing)
// ---------------------------------------------------------------------------

// 语法: impl Trait + use<> 精确控制 RPIT 精确捕获哪些生命周期/类型参数 (RFC 3498)
// 避坑: trait 定义中 use<> 必须包含 Self; 函数定义中可用 use<> 排除不需要的捕获

trait MyTrait {
    fn method(&self) -> impl std::fmt::Debug;
}

struct MyType(i32);

impl MyTrait for MyType {
    fn method(&self) -> impl std::fmt::Debug {
        self.0
    }
}

#[test]
/// 测试: precise capturing use<> 精确控制 RPIT 捕获
fn test_precise_capturing() {
    let obj = MyType(42);
    assert_eq!(format!("{:?}", obj.method()), "42");
}

trait LifecycleTrait {
    fn lifetimes_method<'a>(&'a self) -> impl std::fmt::Display + 'a;
}

impl LifecycleTrait for MyType {
    fn lifetimes_method<'a>(&'a self) -> impl std::fmt::Display + 'a {
        self.0.to_string()
    }
}

#[test]
/// 测试: 生命周期精确捕获
fn test_lifetime_precise_capturing() {
    // 语法: impl Trait + 'a 绑定生命周期参数
    // 避坑: 捕获的生命周期会影响返回类型的静态/动态行为
    trait LifecycleTrait {
        fn lifetimes_method<'a>(&'a self) -> impl std::fmt::Display + 'a;
    }

    struct MyType(i32);

    impl LifecycleTrait for MyType {
        fn lifetimes_method<'a>(&'a self) -> impl std::fmt::Display + 'a {
            self.0.to_string()
        }
    }

    let obj = MyType(100);
    let result = obj.lifetimes_method();
    assert_eq!(result.to_string(), "100");
}

#[test]
/// 测试: impl Trait 在返回值位置
fn test_impl_trait_in_return_position() {
    // 语法: fn foo() -> impl Trait 隐藏具体返回类型
    // 避坑: 调用者只能看到 trait 方法, 不能 downcast 具体类型
    fn create_adder() -> impl Fn(i32) -> i32 {
        move |x| x + 10
    }

    let adder = create_adder();
    let result = adder(5);
    assert_eq!(result, 15);
}

#[test]
/// 测试: impl Trait 在 trait 定义中
fn test_impl_trait_in_trait_definition() {
    // 语法: trait 中可用 impl Trait 声明方法签名
    // 避坑: 实现者必须提供具体返回类型, 且所有实现返回相同类型
    trait Producer {
        fn produce() -> impl Iterator<Item = i32>;
    }

    struct Counter;
    impl Producer for Counter {
        fn produce() -> impl Iterator<Item = i32> {
            0..5
        }
    }

    let count: Vec<_> = Counter::produce().collect();
    assert_eq!(count, vec![0, 1, 2, 3, 4]);
}

#[test]
/// 测试: impl Trait 捕获泛型参数
fn test_impl_trait_captures_generics() {
    // 语法: impl Trait 自动捕获使用的泛型参数
    // 避坑: 需要小心捕获了哪些泛型, 避免不必要的大小增加
    fn process<T: std::fmt::Debug>(val: T) -> impl std::fmt::Debug {
        val
    }

    let result = process(42);
    assert_eq!(format!("{:?}", result), "42");
}

#[test]
/// 测试: use<> 排除捕获
fn test_use_excludes_capture() {
    // 语法: use<> 语法可以显式指定捕获哪些参数
    // 避坑: 排除捕获可减小返回类型大小, 但需确保不依赖被排除的参数
    struct Container<'a> {
        data: &'a i32,
    }

    impl<'a> Container<'a> {
        fn access(&self) -> impl std::fmt::Debug + use<'a> {
            self.data
        }
    }

    let value = 42;
    let container = Container { data: &value };
    assert_eq!(format!("{:?}", container.access()), "42");
}
