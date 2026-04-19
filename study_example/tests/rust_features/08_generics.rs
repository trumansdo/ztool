// ---------------------------------------------------------------------------
// 3.2 泛型与常量泛型
// ---------------------------------------------------------------------------

#[test]
/// 测试: 泛型函数基础
fn test_generic_functions() {
    // 语法: fn foo<T>(x: T) 定义泛型函数, T 是类型占位符
    // 避坑: 泛型参数在编译期单态化, 每个具体类型生成独立代码; 不是运行时多态
    fn identity<T>(x: T) -> T {
        x
    }

    assert_eq!(identity(42), 42);
    assert_eq!(identity("hello"), "hello");

    // 多参数泛型
    fn pair<A, B>(a: A, b: B) -> (A, B) {
        (a, b)
    }
    assert_eq!(pair(1, "two"), (1, "two"));
}

#[test]
/// 测试: 泛型结构体
fn test_generic_structs() {
    // 语法: struct Foo<T> { field: T } 定义泛型结构体
    // 避坑: impl 块需要重复声明泛型参数; 方法可以有自己的泛型参数
    struct Point<T> {
        x: T,
        y: T,
    }

    impl<T> Point<T> {
        fn new(x: T, y: T) -> Self {
            Self { x, y }
        }

        fn x(&self) -> &T {
            &self.x
        }
    }

    // 为具体类型实现方法
    impl Point<f64> {
        fn distance_from_origin(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }

    let p = Point::new(3.0, 4.0);
    assert_eq!(*p.x(), 3.0);
    assert!((p.distance_from_origin() - 5.0).abs() < 0.001);

    let pi = Point::new(1, 2);
    assert_eq!(*pi.x(), 1);
}

#[test]
/// 测试: 泛型枚举
fn test_generic_enums() {
    // 语法: enum Foo<T> { Variant1(T), Variant2 } 泛型枚举
    // 避坑: 每个变体可以有不同的泛型使用方式; Option/Result 就是标准库的泛型枚举
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }

    let ok: Result<i32, &str> = Result::Ok(42);
    let err: Result<i32, &str> = Result::Err("error");

    match ok {
        Result::Ok(v) => assert_eq!(v, 42),
        Result::Err(_) => panic!(),
    }

    match err {
        Result::Ok(_) => panic!(),
        Result::Err(e) => assert_eq!(e, "error"),
    }
}

#[test]
/// 测试: trait 约束 (Trait Bounds)
fn test_trait_bounds() {
    // 语法: fn foo<T: Trait>(x: T) 约束 T 必须实现 Trait
    // 避坑: 多个约束用 + 连接; 约束越多越灵活但越难满足
    use std::fmt::Display;

    fn display_pair<T: Display>(a: &T, b: &T) -> String {
        format!("({}, {})", a, b)
    }

    assert_eq!(display_pair(&1, &2), "(1, 2)");
    assert_eq!(display_pair(&"a", &"b"), "(a, b)");

    // 多个约束
    fn debug_and_display<T: Display + std::fmt::Debug>(x: &T) -> String {
        format!("display: {}, debug: {:?}", x, x)
    }
    assert!(debug_and_display(&42).contains("42"));
}

#[test]
/// 测试: where 子句
fn test_where_clause() {
    // 语法: fn foo<T>() where T: Trait 将约束移到函数签名后
    // 避坑: where 子句更适合复杂约束; 提高可读性; 支持关联类型约束
    use std::fmt::Display;

    // 简单约束可以直接写在 <> 中
    fn simple<T: Display>(x: &T) -> String {
        format!("{}", x)
    }
    assert_eq!(simple(&42), "42");

    // 复杂约束用 where 更清晰
    fn complex<T, U>(t: &T, u: &U) -> String
    where
        T: Display + Clone,
        U: Display + std::fmt::Debug,
    {
        format!("{} and {:?}", t, u)
    }
    assert!(complex(&"hello", &42).contains("hello"));

    // 关联类型约束
    fn first_element<I>(iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
    {
        iter.into_iter().next()
    }
    assert_eq!(first_element(vec![1, 2, 3]), Some(1));
}

#[test]
/// 测试: 默认泛型参数
fn test_default_generic_params() {
    // 语法: struct Foo<T = DefaultType> 提供默认类型
    // 避坑: 默认参数只能放在参数列表末尾; 调用方可以省略
    struct Container<T = i32> {
        value: T,
    }

    impl<T> Container<T> {
        fn new(value: T) -> Self {
            Self { value }
        }
    }

    // 使用默认类型
    let c1 = Container::new(42);
    assert_eq!(c1.value, 42);

    // 显式指定类型
    let c2 = Container::<String>::new(String::from("hello"));
    assert_eq!(c2.value, "hello");
}

#[test]
/// 测试: 泛型生命周期
fn test_generic_lifetimes() {
    // 语法: fn foo<'a, T>(x: &'a T) -> &'a T 生命周期也是泛型参数
    // 避坑: 生命周期参数必须以 ' 开头; 编译器通常能自动推导, 不需要显式标注
    fn first_word<'a>(s: &'a str) -> &'a str {
        let bytes = s.as_bytes();
        for (i, &item) in bytes.iter().enumerate() {
            if item == b' ' {
                return &s[0..i];
            }
        }
        s
    }

    assert_eq!(first_word("hello world"), "hello");
    assert_eq!(first_word("hello"), "hello");
}

#[test]
/// 测试: PhantomData 标记类型
fn test_phantom_data() {
    // 语法: PhantomData<T> 告诉编译器"我拥有 T"或"我引用 T", 但实际不存储
    // 避坑: 不用 PhantomData 时编译器会报 unused type parameter; 选择正确的 PhantomData 变体
    use std::marker::PhantomData;
    use std::ptr::NonNull;

    // 场景1: 泛型结构体实际存储裸指针, 需要标记类型参数
    struct MyBox<T> {
        ptr: NonNull<T>,
        _marker: PhantomData<T>, // 告诉编译器我们"拥有" T
    }

    // 场景2: 单元结构体用作类型标记 (newtype 模式)
    struct Meters;
    struct Miles;

    struct Length<Unit>(f64, PhantomData<Unit>);

    impl<Unit> Length<Unit> {
        fn new(value: f64) -> Self {
            Self(value, PhantomData)
        }
    }

    let meters: Length<Meters> = Length::new(100.0);
    let miles: Length<Miles> = Length::new(1.0);

    // 类型系统阻止混淆单位
    // let wrong: Length<Meters> = miles; // 编译错误!
    assert!((meters.0 - 100.0).abs() < 0.001);
}

#[test]
/// 测试: 关联类型 vs 泛型参数
fn test_associated_types_vs_generics() {
    // 语法: trait 可以使用泛型参数或关联类型定义抽象
    // 避坑: 关联类型每个实现只能有一个; 泛型参数可以有多个实现
    //       选择: 一个类型对应该用关联类型, 多个类型用泛型参数

    // 关联类型风格 (Iterator 模式)
    trait Iterator {
        type Item;
        fn next(&mut self) -> Option<Self::Item>;
    }

    struct Counter {
        count: u32,
    }
    impl Iterator for Counter {
        type Item = u32;
        fn next(&mut self) -> Option<Self::Item> {
            if self.count < 5 {
                self.count += 1;
                Some(self.count)
            } else {
                None
            }
        }
    }

    let mut c = Counter { count: 0 };
    assert_eq!(c.next(), Some(1));
    assert_eq!(c.next(), Some(2));

    // 泛型参数风格 (From 模式)
    trait From<T> {
        fn from(value: T) -> Self;
    }

    impl From<i32> for f64 {
        fn from(v: i32) -> Self {
            v as f64
        }
    }
    let f: f64 = From::from(42i32);
    assert_eq!(f, 42.0);
}

#[test]
/// 测试: 泛型方法上的额外泛型参数
fn test_generic_methods_with_extra_generics() {
    // 语法: impl 块已有泛型参数, 方法可以添加额外的泛型参数
    // 避坑: 方法的泛型参数和 impl 块的泛型参数是独立的
    struct Parser {
        input: String,
    }

    impl Parser {
        fn parse<T: std::str::FromStr>(&self) -> Result<T, T::Err> {
            self.input.parse()
        }
    }

    let p = Parser {
        input: "42".to_string(),
    };
    assert_eq!(p.parse::<i32>().unwrap(), 42);
    assert_eq!(p.parse::<f64>().unwrap(), 42.0);
}

#[test]
/// 测试: const generics 常量泛型参数
fn test_const_generics() {
    // 语法: const N: usize 将值作为类型参数, 实现编译期多态
    // 避坑: const 参数必须是编译期常量; 默认只支持整数和 bool 类型
    fn array_sum<const N: usize>(arr: [i32; N]) -> i32 {
        arr.iter().sum()
    }
    assert_eq!(array_sum([1, 2, 3, 4, 5]), 15);
}

#[test]
/// 测试: const 泛型默认值数组 ([T::default(); N])
fn test_const_default() {
    // 语法: [T::default(); N] 用默认值填充数组, 要求 T: Default + Copy
    // 避坑: 元素类型必须实现 Copy, 否则无法重复 N 次; N 为 0 时返回空数组
    fn default_array<T: Default + Copy, const N: usize>() -> [T; N] {
        [T::default(); N]
    }
    let arr: [i32; 3] = default_array();
    assert_eq!(arr, [0, 0, 0]);
}

#[test]
/// 测试: 推导常量泛型数组长度 (1.89+)
fn test_inferred_const_generic() {
    // 语法: [T::default(); _] 让编译器推导数组长度 (1.89+)
    // 避坑: _ 只能在能从返回类型推导 N 的场景使用; 不能用于局部变量推导
    fn create_array<T: Default + Copy, const N: usize>() -> [T; N] {
        [T::default(); _]
    }
    let arr: [i32; 5] = create_array();
    assert_eq!(arr, [0, 0, 0, 0, 0]);

    let arr_bool: [bool; 3] = create_array();
    assert_eq!(arr_bool, [false, false, false]);
}

#[test]
/// 测试: const 泛型在结构体中的应用
fn test_const_generics_in_structs() {
    // 语法: struct Matrix<T, const R: usize, const C: usize> 编译期确定维度
    // 避坑: const 参数不能用于运行时计算; 矩阵运算可以在编译期检查维度
    struct Matrix<T, const R: usize, const C: usize> {
        data: [[T; C]; R],
    }

    impl<T: Default + Copy, const R: usize, const C: usize> Matrix<T, R, C> {
        fn new(fill: T) -> Self {
            Self { data: [[fill; C]; R] }
        }
    }

    let m: Matrix<i32, 2, 3> = Matrix::new(0);
    assert_eq!(m.data[0][0], 0);
    assert_eq!(m.data[1][2], 0);
}

#[test]
/// 测试: 泛型 + const 泛型组合
fn test_generics_with_const() {
    // 语法: fn foo<T, const N: usize>() 同时使用类型泛型和 const 泛型
    // 避坑: const 参数放在类型参数之后; 两者可以独立推导
    fn repeat<T: Clone, const N: usize>(value: T) -> [T; N] {
        [(); N].map(|_| value.clone())
    }

    let arr: [String; 3] = repeat(String::from("hi"));
    assert_eq!(arr[0], "hi");
    assert_eq!(arr[2], "hi");
}
