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
    fn process<T: std::fmt::Debug>(val: T) -> impl std::fmt::Debug {
        val
    }

    let result = process(42);
    assert_eq!(format!("{:?}", result), "42");
}

#[test]
/// 测试: use<> 排除捕获
fn test_use_excludes_capture() {
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

// ===================== 扩充测试 =====================

#[test]
/// 测试: use<> 捕获多个生命周期
fn test_use_multiple_lifetimes() {
    struct Pair<'a, 'b> {
        first: &'a str,
        second: &'b str,
    }

    impl<'a, 'b> Pair<'a, 'b> {
        fn combine(&self) -> impl std::fmt::Display + use<'a, 'b> {
            format!("{} + {}", self.first, self.second)
        }
    }

    let first = "hello";
    let second = "world";
    let pair = Pair { first: &first, second: &second };
    let result = pair.combine();
    assert_eq!(result.to_string(), "hello + world");
}

#[test]
/// 测试: use<> 为空 —— 不捕获任何参数
fn test_use_empty_captures_nothing() {
    fn returns_static() -> impl std::fmt::Display + use<> {
        "static string"
    }

    let result = returns_static();
    assert_eq!(result.to_string(), "static string");
}

#[test]
/// 测试: impl Trait 作为函数参数 (impl Trait in argument position)
fn test_impl_trait_argument_position() {
    fn print_it(item: impl std::fmt::Display) -> String {
        item.to_string()
    }

    assert_eq!(print_it(42), "42");
    assert_eq!(print_it("hello"), "hello");
}

#[test]
/// 测试: 多个 impl Trait 参数
fn test_multiple_impl_trait_args() {
    fn combine(a: impl std::fmt::Display, b: impl std::fmt::Display) -> String {
        format!("{a}|{b}")
    }

    assert_eq!(combine(10, "text"), "10|text");
}

#[test]
/// 测试: impl Trait 和泛型参数混用
fn test_impl_trait_with_generics() {
    fn transform<T: std::fmt::Debug>(val: T) -> impl std::fmt::Display {
        format!("Transformed: {:?}", val)
    }

    let result = transform(vec![1, 2, 3]);
    assert!(result.to_string().contains("1, 2, 3"));
}

#[test]
/// 测试: RPIT 在不同实现中返回不同类型 (需要 trait 定义)
fn test_rpit_different_impl_different_types() {
    trait Factory {
        fn build() -> impl std::fmt::Debug;
    }

    struct IntFactory;
    impl Factory for IntFactory {
        fn build() -> impl std::fmt::Debug {
            42i32
        }
    }

    struct StrFactory;
    impl Factory for StrFactory {
        fn build() -> impl std::fmt::Debug {
            "hello"
        }
    }

    assert_eq!(format!("{:?}", IntFactory::build()), "42");
    assert_eq!(format!("{:?}", StrFactory::build()), "\"hello\"");
}

#[test]
/// 测试: 边界 —— 同一个 impl Trait 返回位置必须返回相同类型
fn test_impl_trait_same_type_per_branch() {
    fn get_value(flag: bool) -> impl std::fmt::Display {
        if flag {
            "yes".to_string()
        } else {
            "no".to_string()
        }
        // 两个分支都返回 String, 类型一致
    }

    assert_eq!(get_value(true).to_string(), "yes");
    assert_eq!(get_value(false).to_string(), "no");
}

#[test]
/// 测试: impl Trait 配合闭包类型
fn test_impl_trait_with_closure() {
    fn make_counter(start: i32) -> impl FnMut() -> i32 {
        let mut count = start;
        move || {
            count += 1;
            count
        }
    }

    let mut counter = make_counter(0);
    assert_eq!(counter(), 1);
    assert_eq!(counter(), 2);
    assert_eq!(counter(), 3);
}

#[test]
/// 测试: impl Trait + lifetime 在返回位置
fn test_impl_trait_with_lifetime() {
    fn pick_first<'a>(x: &'a str, _y: &str) -> impl std::fmt::Display + 'a {
        x
    }

    let first = "first";
    let result = pick_first(&first, "second");
    assert_eq!(result.to_string(), "first");
}

#[test]
/// 测试: impl Trait 和 Iterator 链式调用
fn test_impl_trait_iterator_chain() {
    fn evens_then_odds() -> impl Iterator<Item = i32> {
        (0..5).filter(|x| x % 2 == 0).chain((0..5).filter(|x| x % 2 != 0))
    }

    let result: Vec<_> = evens_then_odds().collect();
    assert_eq!(result, vec![0, 2, 4, 1, 3]);
}

#[test]
/// 测试: 边界 —— impl Trait 不允许动态分发转换
fn test_impl_trait_boundary_no_dyn_cast() {
    fn produce() -> impl std::fmt::Debug {
        42i32
    }

    let value = produce();
    // value 的类型是不透明的, 无法转换为 &dyn Debug 之外的动态类型
    // 但可以转换为 trait 对象：
    let as_debug: &dyn std::fmt::Debug = &value;
    assert_eq!(format!("{:?}", as_debug), "42");
}
