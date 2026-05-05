// ---------------------------------------------------------------------------
// 3.2 泛型与常量泛型
// ---------------------------------------------------------------------------

// ============================================================================
// 基础泛型测试
// ============================================================================

#[test]
/// 测试: 泛型函数基础
fn test_generic_functions() {
    // 语法: fn foo<T>(x: T) 定义泛型函数, T 是类型占位符
    // 避坑: 泛型参数在编译期单态化, 每个具体类型生成独立代码; 不是运行时多态
    // 金句: 泛型函数是对类型的抽象——一份代码, 编译器为每种调用生成专用版本
    fn identity<T>(x: T) -> T {
        x
    }

    assert_eq!(identity(42), 42);
    assert_eq!(identity("hello"), "hello");

    // turbofish 语法显式指定泛型参数
    assert_eq!(identity::<f64>(3.14), 3.14);

    // 多参数泛型
    fn pair<A, B>(a: A, b: B) -> (A, B) {
        (a, b)
    }
    assert_eq!(pair(1, "two"), (1, "two"));

    // 泛型 + 闭包: 泛型参数可以是闭包类型
    fn apply_twice<T, F: Fn(T) -> T>(x: T, f: F) -> T {
        f(f(x))
    }
    assert_eq!(apply_twice(3, |n| n * 2), 12);
}

#[test]
/// 测试: 泛型结构体
fn test_generic_structs() {
    // 语法: struct Foo<T> { field: T } 定义泛型结构体
    // 避坑: impl 块需要重复声明泛型参数; 方法可以有自己的泛型参数
    // 金句: 泛型结构体一份定义适配无限种具体类型, 但同一实例中每个泛型参数只能对应一种类型
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

    // 为具体类型实现方法 —— 只有 Point<f64> 能调用
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
    // pi.distance_from_origin(); // 编译错误! Point<i32> 没有此方法

    // 多泛型参数结构体
    struct Pair<A, B>(A, B);
    impl<A: std::fmt::Display, B: std::fmt::Display> Pair<A, B> {
        fn describe(&self) -> String {
            format!("({}, {})", self.0, self.1)
        }
    }
    let pair = Pair(10, "hello");
    assert_eq!(pair.describe(), "(10, hello)");
}

#[test]
/// 测试: 泛型枚举
fn test_generic_enums() {
    // 语法: enum Foo<T> { Variant1(T), Variant2 } 泛型枚举
    // 避坑: 每个变体可以有不同的泛型使用方式; Option/Result 就是标准库的泛型枚举
    // 金句: Option<T> 和 Result<T, E> 用类型系统替代空值检查和异常处理
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
    // 金句: trait bound 是泛型的"合约"——没有约束的 T 什么都做不了
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

    // impl Trait 语法糖: 函数参数可用 impl Trait 代替泛型声明
    fn print_impl(x: &impl Display) -> String {
        format!("impl: {}", x)
    }
    assert_eq!(print_impl(&99), "impl: 99");

    // 返回类型 impl Trait: 所有返回路径必须是同一具体类型
    fn returns_display() -> impl Display {
        42
    }
    assert_eq!(format!("{}", returns_display()), "42");
}

#[test]
/// 测试: where 子句
fn test_where_clause() {
    // 语法: fn foo<T>() where T: Trait 将约束移到函数签名后
    // 避坑: where 子句更适合复杂约束; 提高可读性; 支持关联类型约束
    // 金句: 当约束超过一个时就应迁移到 where 子句
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
    // 金句: 默认类型参数让常见用法零配置
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
    // 避坑: 生命周期参数必须以 ' 开头; 编译器通常能自动推导
    // 金句: 生命周期是一种特殊的泛型——描述存活范围而非数据类型
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
    // 金句: PhantomData 让编译器"看到"实际不存在的类型关系——零字节代价换取类型安全
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
    let _miles: Length<Miles> = Length::new(1.0);

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
    // 金句: 关联类型是一对一抽象, 泛型参数是一对多抽象——选错了 API 就难用了

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
    // 金句: 方法的泛型参数独立于 impl 块——方法的泛型参数只在该方法调用时确定
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

// ============================================================================
// const 泛型测试
// ============================================================================

#[test]
/// 测试: const generics 常量泛型参数
fn test_const_generics() {
    // 语法: const N: usize 将值作为类型参数, 实现编译期多态
    // 避坑: const 参数必须是编译期常量; 默认只支持整数和 bool 类型
    // 金句: const 泛型把"值"提升到类型层面——数组大小、矩阵维度的编译期类型安全
    fn array_sum<const N: usize>(arr: [i32; N]) -> i32 {
        arr.iter().sum()
    }
    assert_eq!(array_sum([1, 2, 3, 4, 5]), 15);

    // 同时对多个 const 参数进行泛型化
    fn array_product<const N: usize>(arr: [i32; N]) -> i32 {
        arr.iter().product()
    }
    assert_eq!(array_product([2, 3, 4]), 24);
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

    // N = 0 边界情况
    let empty: [i32; 0] = default_array();
    assert_eq!(empty.len(), 0);
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
    // 金句: const 泛型在结构体中实现编译期维度检查——错误的矩阵乘法在写代码时就暴露
    struct Matrix<T, const R: usize, const C: usize> {
        data: [[T; C]; R],
    }

    impl<T: Default + Copy, const R: usize, const C: usize> Matrix<T, R, C> {
        fn new(fill: T) -> Self {
            Self { data: [[fill; C]; R] }
        }

        fn rows(&self) -> usize { R }
        fn cols(&self) -> usize { C }
    }

    let m: Matrix<i32, 2, 3> = Matrix::new(0);
    assert_eq!(m.data[0][0], 0);
    assert_eq!(m.data[1][2], 0);
    assert_eq!(m.rows(), 2);
    assert_eq!(m.cols(), 3);

    // 不同维度的矩阵是不同的类型
    let _m23: Matrix<i32, 2, 3> = Matrix::new(1);
    let _m32: Matrix<i32, 3, 2> = Matrix::new(1);
    // 类型系统阻止维度错误的运算:
    // fn multiply<const M: usize, const N: usize, const P: usize>(
    //     a: Matrix<f64, M, N>, b: Matrix<f64, N, P>) -> Matrix<f64, M, P> { ... }
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

    // 类型 + const 泛型的混合使用: 用 const 泛型作为类型参数的数组维度
    fn fill_slice<T: Default + Copy, const N: usize>() -> [T; N] {
        [T::default(); N]
    }
    let floats: [f64; 4] = fill_slice();
    assert_eq!(floats, [0.0, 0.0, 0.0, 0.0]);
}

// ============================================================================
// 新增测试: 单态化证据
// ============================================================================

#[test]
/// 测试: 单态化证据 —— 编译器为每种具体类型生成独立代码
fn test_monomorphization_evidence() {
    // 语法: 泛型代码在编译期被展开为多个独立的具体类型版本
    // 避坑: 每种类型组合生成一份代码, 编译时间和二进制大小都会增加
    // 金句: 单态化是零成本抽象的基石——编译期付出时间, 运行时零开销
    use std::mem::size_of;

    // 证据1: 不同泛型实例有独立的内存布局
    struct Wrapper<T>(T);

    // 不同 T 的 Wrapper<T> 大小与 T 的大小相同（单态化后独立计算）
    assert_eq!(size_of::<Wrapper<i32>>(), size_of::<i32>());
    assert_eq!(size_of::<Wrapper<i64>>(), size_of::<i64>());
    assert_eq!(size_of::<Wrapper<[u8; 16]>>(), 16);
    // 注意: ZST (零大小类型) 的 Wrapper 也是零大小
    assert_eq!(size_of::<Wrapper<()>>(), 0);

    // 证据2: 相同函数, 不同类型调用生成不同机器码 (通过地址推断)
    // 这里我们无法直接获取函数地址, 但可以通过行为验证
    fn identity<T>(x: T) -> T { x }

    let a: i32 = identity::<i32>(100);
    let b: &str = identity::<&str>("hello");
    let c: f64 = identity::<f64>(3.14);
    assert_eq!(a, 100);
    assert_eq!(b, "hello");
    assert!((c - 3.14).abs() < 0.001);

    // 证据3: 同一泛型结构体不同参数是不同类型
    struct Tag<T>(std::marker::PhantomData<T>);
    let _i32_tag: Tag<i32> = Tag(std::marker::PhantomData);
    let _str_tag: Tag<&str> = Tag(std::marker::PhantomData);
    // Tag<i32> 和 Tag<&str> 是完全不同的类型, 不能互相赋值
    // let _: Tag<i32> = _str_tag; // 编译错误!

    // 证据4: 单态化后约束检查是"按实例"的——只有实际使用的类型才被检查
    fn requires_debug<T: std::fmt::Debug>(x: T) -> String {
        format!("{:?}", x)
    }
    assert!(requires_debug(vec![1, 2, 3]).contains("1"));
}

// ============================================================================
// 新增测试: const 泛型全面测试
// ============================================================================

#[test]
/// 测试: const 泛型完整用法 —— const fn 配合、多参数、默认值
fn test_const_generics_comprehensive() {
    // 语法: const fn 在编译期计算, 结果可作为 const 泛型参数
    // 避坑: const fn 只能包含编译期可求值的操作; const 泛型仅支持整数/bool/char
    // 金句: const fn + const 泛型 = 编译期计算 + 编译期类型安全
    use std::mem::size_of;

    // const fn 配合 const 泛型
    const fn buffer_size(multiplier: usize) -> usize {
        multiplier * 1024
    }

    struct Buffer<const SIZE: usize> {
        data: [u8; SIZE],
    }

    impl<const SIZE: usize> Buffer<SIZE> {
        fn new() -> Self {
            Self { data: [0u8; SIZE] }
        }

        fn capacity(&self) -> usize { SIZE }
    }

    let buf: Buffer<{ buffer_size(4) }> = Buffer::new();
    assert_eq!(buf.capacity(), 4096);
    assert_eq!(buf.data.len(), 4096);

    // 多个 const 泛型参数, 含默认值
    struct FixedArray<T, const N: usize = 1> {
        data: [T; N],
    }

    // 使用默认 N = 1
    let arr: FixedArray<i32> = FixedArray { data: [42; 1] };
    assert_eq!(arr.data[0], 42);

    // 显式指定 N
    let arr3: FixedArray<i32, 3> = FixedArray { data: [1, 2, 3] };
    assert_eq!(arr3.data.len(), 3);

    // const 泛型在函数中的应用: 编译期保证长度安全
    fn pairwise_sum<const N: usize>(a: [i32; N], b: [i32; N]) -> [i32; N] {
        let mut result = [0; N];
        for i in 0..N {
            result[i] = a[i] + b[i];
        }
        result
    }

    let sum = pairwise_sum([1, 2, 3], [4, 5, 6]);
    assert_eq!(sum, [5, 7, 9]);

    // 不同长度的数组无法调用 (编译期阻止):
    // pairwise_sum([1, 2, 3], [4, 5]); // 编译错误! N 必须统一

    // const 泛型与 size_of 的组合
    fn size_of_array<T, const N: usize>() -> usize {
        size_of::<T>() * N
    }
    assert_eq!(size_of_array::<i32, 8>(), 32);
    assert_eq!(size_of_array::<u8, 4>(), 4);
}

// ============================================================================
// 新增测试: 默认类型参数
// ============================================================================

#[test]
/// 测试: 默认泛型参数 —— trait 和结构体级别的默认类型
fn test_default_type_parameter() {
    // 语法: trait Add<Rhs = Self> 提供 trait 级别的默认类型参数
    // 避坑: 默认参数只能在参数列表末尾; trait 默认参数在标准库中广泛使用
    // 金句: 默认类型参数让常见用法零配置, 特殊需求显式覆盖

    // 模拟标准库的 Add trait: 默认 Rhs = Self
    trait Add<Rhs = Self> {
        type Output;
        fn add(self, rhs: Rhs) -> Self::Output;
    }

    // i32 + i32 (默认 Rhs = Self)
    impl Add for i32 {
        type Output = i32;
        fn add(self, rhs: i32) -> i32 { self + rhs }
    }

    assert_eq!(1i32.add(2), 3);

    // i32 + f64 (显式指定 Rhs = f64)
    impl Add<f64> for i32 {
        type Output = f64;
        fn add(self, rhs: f64) -> f64 { self as f64 + rhs }
    }

    assert!((1i32.add(2.5_f64) - 3.5).abs() < 0.001);

    // 结构体默认泛型参数 —— 多个默认参数
    struct Config<
        T = String,
        U = i32,
    > {
        name: T,
        value: U,
    }

    // 完全使用默认
    let c1 = Config { name: String::from("key"), value: 42 };
    assert_eq!(c1.value, 42);

    // 部分覆盖 (第一个使用默认)
    let c2 = Config::<String, f64> { name: String::from("pi"), value: 3.14 };
    assert!((c2.value - 3.14).abs() < 0.001);
}

// ============================================================================
// 新增测试: where 子句高级用法
// ============================================================================

#[test]
/// 测试: where 子句高级用法 —— 关联类型约束、生命周期混合、多重约束
fn test_where_clause_advanced() {
    // 语法: where 子句可以约束关联类型、生命周期, 并组合多个复杂 bound
    // 避坑: 关联类型约束语法为 T::AssocType: Trait; 生命周期在 where 中写为 T: 'a
    // 金句: where 子句让复杂约束可读、可维护

    // 场景1: 约束关联类型实现特定 trait
    fn sum_items<C>(container: C) -> C::Item
    where
        C: IntoIterator,
        C::Item: std::ops::Add<Output = C::Item> + Default + Copy,
    {
        container.into_iter().fold(C::Item::default(), |acc, x| acc + x)
    }

    assert_eq!(sum_items(vec![1, 2, 3, 4]), 10);
    assert_eq!(sum_items(vec![1.5_f64, 2.5]), 4.0);

    // 场景2: 多个关联类型约束, 包括关联类型相等约束
    fn merge_and_sort<A, B>(a: A, b: B) -> Vec<A::Item>
    where
        A: IntoIterator,
        B: IntoIterator<Item = A::Item>,
        A::Item: Ord,
    {
        let mut result: Vec<_> = a.into_iter().chain(b).collect();
        result.sort();
        result
    }

    let merged = merge_and_sort(vec![3, 1, 4], vec![2, 6, 5]);
    assert_eq!(merged, vec![1, 2, 3, 4, 5, 6]);

    // 场景3: 生命周期 + 类型约束的组合
    fn longest_with_display<'a, T>(x: &'a T, y: &'a T) -> &'a T
    where
        T: std::cmp::PartialOrd + std::fmt::Display,
    {
        if x > y { x } else { y }
    }

    let r = longest_with_display(&10, &20);
    assert_eq!(*r, 20);

    // 场景4: ?Sized 约束 —— 允许动态大小类型
    fn debug_sized<T: std::fmt::Debug + ?Sized>(x: &T) -> String {
        format!("{:?}", x)
    }
    assert_eq!(debug_sized(&42), "42");
}

// ============================================================================
// 新增测试: Blanket Implementation
// ============================================================================

#[test]
/// 测试: blanket implementation —— 为所有满足条件的类型批量实现 trait
fn test_blanket_implementation() {
    // 语法: impl<T: SomeTrait> MyTrait for T {} 为所有实现 SomeTrait 的类型自动实现 MyTrait
    // 避坑: blanket impl 可能与具体类型 impl 冲突; 遵循孤儿规则
    // 金句: blanket impl 是一行代码为无穷多种类型赋予能力——trait 系统最优雅的杀手锏

    // 自定义 trait: 为所有迭代器提供便捷方法
    trait IterExt: Iterator {
        fn my_collect_vec(self) -> Vec<Self::Item>
        where
            Self: Sized,
        {
            self.collect()
        }
    }

    // blanket impl: 为所有 Iterator 实现 IterExt
    impl<I: Iterator> IterExt for I {}

    // 使用: 任何迭代器自动获得 my_collect_vec
    let v1 = (0..5).my_collect_vec();
    assert_eq!(v1, vec![0, 1, 2, 3, 4]);

    let v2 = vec![10, 20, 30].into_iter().my_collect_vec();
    assert_eq!(v2, vec![10, 20, 30]);

    // 标准库示例模式: 为所有 Display 实现自定义 trait
    trait Describe {
        fn describe(&self) -> String;
    }

    impl<T: std::fmt::Display> Describe for T {
        fn describe(&self) -> String {
            format!("Value: {}", self)
        }
    }

    // i32, &str, f64 等都自动获得了 describe 方法
    assert_eq!(42.describe(), "Value: 42");
    assert_eq!("hello".describe(), "Value: hello");
    assert_eq!(3.14_f64.describe(), "Value: 3.14");
}

// ============================================================================
// 新增测试: 条件 trait 实现
// ============================================================================

#[test]
/// 测试: 条件 trait 实现 —— 只有泛型参数满足条件时才实现 trait
fn test_conditional_trait_impl() {
    // 语法: impl<T: PreCondition> TargetTrait for Wrapper<T> {}
    // 避坑: 条件 impl 不会自动传递——子字段满足不代表外层自动满足
    // 金句: 条件 impl 让类型有条件地获得能力——T 有什么 trait, Wrapper<T> 就有什么 trait

    #[derive(Debug)]
    struct Wrapper<T>(T);

    // 条件实现 Clone: 只有 T: Clone 时 Wrapper<T> 才能克隆
    impl<T: Clone> Clone for Wrapper<T> {
        fn clone(&self) -> Self {
            Wrapper(self.0.clone())
        }
    }

    // 条件实现 Display: 只有 T: Display 时才能格式化
    impl<T: std::fmt::Display> std::fmt::Display for Wrapper<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}]", self.0)
        }
    }

    // Wrapper<i32> 满足 Clone + Display（因为 i32 满足）
    let w = Wrapper(42);
    let w2 = w.clone();
    assert_eq!(w2, Wrapper(42));
    assert_eq!(format!("{}", w), "[42]");

    // 条件实现 PartialEq: 只有 T: PartialEq 时才能比较
    impl<T: PartialEq> PartialEq for Wrapper<T> {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    assert_eq!(Wrapper(1), Wrapper(1));
    assert_ne!(Wrapper(1), Wrapper(2));

    // 另一种条件模式: 可选的功能门控
    trait ToJson {
        fn to_json(&self) -> String;
    }

    impl ToJson for i32 {
        fn to_json(&self) -> String { self.to_string() }
    }

    // 条件实现: 只有 T: ToJson 时 Wrapper<T> 才可转 JSON
    impl<T: ToJson> ToJson for Wrapper<T> {
        fn to_json(&self) -> String {
            format!("{{\"value\": {}}}", self.0.to_json())
        }
    }

    let w = Wrapper(999);
    assert_eq!(w.to_json(), "{\"value\": 999}");
}

// ============================================================================
// 新增测试: 多重 trait 约束模式
// ============================================================================

#[test]
/// 测试: 多重 trait 约束的各种写法及典型错误
fn test_multi_constraint_patterns() {
    // 语法: 用 + 连接多个约束; 或用 where 子句分离
    // 避坑: 约束不足是最常见的泛型错误; impl 块忘记声明泛型参数也是常见错误
    // 金句: 多重约束=泛型的"能力清单"——缺一个都不行, 多一个都是浪费
    use std::ops::Mul;

    // 模式1: 行内多约束
    fn double_clone<T: Clone + std::fmt::Debug>(x: &T) -> (T, String) {
        (x.clone(), format!("{:?}", x))
    }
    let (v, s) = double_clone(&42);
    assert_eq!(v, 42);
    assert_eq!(s, "42");

    // 模式2: where 多约束
    fn transform<T, U>(x: T) -> U
    where
        T: std::fmt::Display + Clone,
        U: From<T> + std::fmt::Debug,
    {
        let u = U::from(x);
        u
    }
    let result: f64 = transform(42i32);
    assert!((result - 42.0).abs() < 0.001);

    // 模式3: super trait —— trait 之间建立继承关系
    trait Printable: std::fmt::Display + std::fmt::Debug {
        fn print(&self) {
            println!("Display: {}, Debug: {:?}", self, self);
        }
    }

    // 不能为不满足 super trait 的类型实现 Printable:
    // impl Printable for i32 {} // 需要 impl Display for i32 和 Debug for i32
    impl Printable for String {}
    // String 满足 Display + Debug, 因此可以

    // 模式4: 组合多个标准 trait
    fn clone_and_sort<T: Clone + Ord>(items: &[T]) -> Vec<T> {
        let mut v: Vec<T> = items.to_vec();
        v.sort();
        v
    }
    assert_eq!(clone_and_sort(&[3, 1, 2]), vec![1, 2, 3]);

    // 模式5: 关联类型 + 约束
    fn first_cloned<I>(iter: I) -> Option<I::Item>
    where
        I: IntoIterator,
        I::Item: Clone,
    {
        iter.into_iter().next().map(|x| x.clone())
    }
    assert_eq!(first_cloned(vec!["hello"]), Some("hello"));

    // 典型错误说明 (以下代码无法编译):
    // 错误1: impl 块忘记泛型参数声明
    // struct Foo<T> { val: T }
    // impl Foo { fn new(val: T) -> Self { Foo { val } } } // 错误!
    // 正确: impl<T> Foo<T> { fn new(val: T) -> Self { Foo { val } } }

    // 错误2: 约束不足
    // fn multiply<T>(a: T, b: T) -> T { a * b } // 错误! T 没有实现 Mul
    // 正确: fn multiply<T: Mul<Output = T>>(a: T, b: T) -> T { a * b }

    // 正确写法验证:
    fn multiply<T: Mul<Output = T>>(a: T, b: T) -> T { a * b }
    assert_eq!(multiply(3, 4), 12);
}

// ============================================================================
// 新增测试: 类型状态 (Type State) 模式
// ============================================================================

#[test]
/// 测试: 类型状态模式 —— 运行时状态机变为编译期类型检查
fn test_type_state_pattern() {
    // 语法: 使用 PhantomData + 泛型标记状态, 配合为特定状态实现的 impl 块
    // 避坑: 状态转换需要消费 self 或 &mut self; 同一资源不能同时在两个状态
    // 金句: 类型状态让非法操作在编译时就暴露——运行时错误变成编译错误
    use std::marker::PhantomData;

    // 状态标记: 零大小类型
    struct Initialized;
    struct Running;
    struct Stopped;

    // 进程抽象: 状态通过泛型参数编码
    struct Process<State = Initialized> {
        id: u64,
        _state: PhantomData<State>,
    }

    // 所有状态通用的方法
    impl<State> Process<State> {
        fn id(&self) -> u64 { self.id }
    }

    // 仅在 Initialized 状态下可用: 启动
    impl Process<Initialized> {
        fn new(id: u64) -> Self {
            Process { id, _state: PhantomData }
        }

        fn start(self) -> Process<Running> {
            Process { id: self.id, _state: PhantomData }
        }
    }

    // 仅在 Running 状态下可用: 执行和停止
    impl Process<Running> {
        fn execute(&self, cmd: &str) -> String {
            format!("proc[{}] 执行: {}", self.id, cmd)
        }

        fn stop(self) -> Process<Stopped> {
            Process { id: self.id, _state: PhantomData }
        }
    }

    // 仅在 Stopped 状态下可用: 查看
    impl Process<Stopped> {
        fn result(&self) -> &str { "已完成" }
    }

    // 正确流程: Initialized -> Running -> Stopped
    let proc = Process::new(1);
    assert_eq!(proc.id(), 1);

    let running = proc.start();
    assert_eq!(running.execute("echo hello"), "proc[1] 执行: echo hello");

    let stopped = running.stop();
    assert_eq!(stopped.result(), "已完成");

    // 编译期阻止非法操作:
    // let p = Process::new(1);
    // p.execute("...");       // 编译错误! Process<Initialized> 没有 execute
    // let r = p.start();
    // p.start();              // 编译错误! p 已经被 move
    // r.result();             // 编译错误! Process<Running> 没有 result
}

// ============================================================================
// 新增测试: 泛型枚举与 Result 模式
// ============================================================================

#[test]
/// 测试: 泛型枚举的进阶用法与 Result-like 模式
fn test_generic_enum_and_result() {
    // 语法: enum 可以组合多个泛型参数, 每个变体可独立使用泛型
    // 避坑: 枚举变体各自持有泛型参数; 枚举整体级别可 impl trait bound
    // 金句: 泛型枚举 = 类型安全的联合类型, 每个变体都是一个独立的世界

    // 定义: 一个多泛型参数枚举
    enum Either<L, R> {
        Left(L),
        Right(R),
    }

    // 为泛型枚举实现 trait —— 需要所有变体都满足条件
    impl<L: std::fmt::Display, R: std::fmt::Display> std::fmt::Display for Either<L, R> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Either::Left(l) => write!(f, "Left({})", l),
                Either::Right(r) => write!(f, "Right({})", r),
            }
        }
    }

    let left: Either<i32, &str> = Either::Left(42);
    let right: Either<i32, &str> = Either::Right("hello");
    assert_eq!(format!("{}", left), "Left(42)");
    assert_eq!(format!("{}", right), "Right(hello)");

    // 为泛型枚举实现专用方法
    impl<L, R> Either<L, R> {
        fn map_left<F, T>(self, f: F) -> Either<T, R>
        where
            F: FnOnce(L) -> T,
        {
            match self {
                Either::Left(l) => Either::Left(f(l)),
                Either::Right(r) => Either::Right(r),
            }
        }

        fn is_left(&self) -> bool {
            matches!(self, Either::Left(_))
        }
    }

    let e: Either<i32, &str> = Either::Left(10);
    assert!(e.is_left());

    let mapped = e.map_left(|x| x * 2);
    assert_eq!(format!("{}", mapped), "Left(20)");

    // 泛型 Option 模式
    enum MyOption<T> {
        Some(T),
        None,
    }

    impl<T> MyOption<T> {
        fn unwrap_or(self, default: T) -> T {
            match self {
                MyOption::Some(v) => v,
                MyOption::None => default,
            }
        }

        fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyOption<U> {
            match self {
                MyOption::Some(v) => MyOption::Some(f(v)),
                MyOption::None => MyOption::None,
            }
        }
    }

    let some_val = MyOption::Some(5);
    assert_eq!(some_val.unwrap_or(0), 5);
    assert_eq!(MyOption::<i32>::None.unwrap_or(99), 99);

    let mapped_val = MyOption::Some(3).map(|x| x * 10);
    assert_eq!(mapped_val.unwrap_or(0), 30);
}

// ============================================================================
// 新增测试: 泛型与生命周期结合
// ============================================================================

#[test]
/// 测试: 泛型与生命周期的结合模式, 包括结构体和函数
fn test_generic_lifetime_combination() {
    // 语法: struct Foo<'a, T> 将生命周期和类型泛型组合在同一个结构体中
    // 避坑: 生命周期参数放在类型参数之前 ('a, T); T: 'a 表示 T 必须活过生命周期 'a
    // 金句: 生命周期 + 泛型 = 完整的所有权+抽象模型; 'a 描述多久, T 描述什么

    // 场景1: 结构体同时持有引用和所有值
    struct Excerpt<'a, T> {
        reference: &'a T,
        cached: Option<T>,
    }

    impl<'a, T: Clone> Excerpt<'a, T> {
        fn new(reference: &'a T) -> Self {
            Excerpt { reference, cached: None }
        }

        fn get(&self) -> &T {
            self.reference
        }

        fn clone_to_cache(&mut self) -> &T {
            self.cached = Some(self.reference.clone());
            self.cached.as_ref().unwrap()
        }
    }

    let value = 42;
    let mut excerpt = Excerpt::new(&value);
    assert_eq!(*excerpt.get(), 42);
    assert_eq!(*excerpt.clone_to_cache(), 42);
    assert!(excerpt.cached.is_some());

    // 场景2: 多个生命周期 + 类型泛型
    struct PairRef<'a, 'b, T> {
        first: &'a T,
        second: &'b T,
    }

    impl<'a, 'b, T: PartialEq> PairRef<'a, 'b, T> {
        fn are_equal(&self) -> bool {
            self.first == self.second
        }
    }

    let a = 10;
    let b = 10;
    let pair = PairRef { first: &a, second: &b };
    assert!(pair.are_equal());

    let c = 20;
    let pair2 = PairRef { first: &a, second: &c };
    assert!(!pair2.are_equal());

    // 场景3: 泛型函数中的生命周期推导
    fn first_of_pair<'a, T>(first: &'a T, _second: &T) -> &'a T {
        first
    }

    let x = String::from("first");
    let y = String::from("second");
    let result = first_of_pair(&x, &y);
    assert_eq!(result, "first");

    // 场景4: T: 'a 约束 —— T 必须活过 'a
    // 这表示 T 中不包含生命周期短于 'a 的引用
    struct Contained<'a, T: 'a> {
        value: &'a T,
    }

    let val = 99;
    let contained = Contained { value: &val };
    assert_eq!(*contained.value, 99);
}
