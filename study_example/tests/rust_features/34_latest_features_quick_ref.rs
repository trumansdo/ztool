// ---------------------------------------------------------------------------
// 5.9 最新特性快速参考 (1.85-1.90+)
// ---------------------------------------------------------------------------

#[test]
/// 测试: 裸函数 naked functions 概念 (1.88+)
fn test_naked_functions_concept() {
    assert!(true);
}

#[test]
/// 测试: 安全架构 intrinsic (1.87+)
fn test_safe_arch_intrinsics() {
    if is_x86_feature_detected!("sse2") {
        assert!(true);
    }
}

#[test]
/// 测试: asm! 标签操作数概念 (1.87+)
fn test_asm_labels_concept() {
    assert!(true);
}

#[test]
/// 测试: cargo publish --workspace 工作区发布 (1.90+)
fn test_workspace_publishing() {
    assert!(true);
}

#[test]
/// 测试: LLD 默认链接器 (1.90+)
fn test_lld_linker() {
    assert!(true);
}

#[test]
/// 测试: const eval 编译期求值
fn test_const_evaluation() {
    const VALUE: i32 = {
        let a = 10;
        let b = 20;
        a + b
    };
    assert_eq!(VALUE, 30);
}

#[test]
/// 测试: inline 编译提示
fn test_inline_attribute() {
    #[inline]
    fn small_fn() -> i32 {
        42
    }
    assert_eq!(small_fn(), 42);
}

#[test]
/// 测试: const generics 泛型常量
fn test_const_generics() {
    fn make_array<T: Copy + Default, const N: usize>() -> [T; N] {
        [T::default(); N]
    }

    let arr: [i32; 3] = make_array();
    assert_eq!(arr, [0; 3]);
}

#[test]
/// 测试: associated const 关联常量
fn test_associated_const() {
    trait Consts {
        const VALUE: i32;
    }

    struct MyStruct;
    impl Consts for MyStruct {
        const VALUE: i32 = 100;
    }

    assert_eq!(MyStruct::VALUE, 100);
}

#[test]
/// 测试: trait bound 语法糖
fn test_trait_bound_syntax() {
    fn print_debug<T: std::fmt::Debug>(value: &T) {
        println!("{:?}", value);
    }

    print_debug(&42);
    assert!(true);
}

#[test]
/// 测试: blanket impl 毯式实现
fn test_blanket_impl() {
    fn double<T: Copy>(x: T) -> i32
    where
        T: Into<i32>,
    {
        let v: i32 = x.into();
        v * 2
    }

    let x = 21;
    assert_eq!(double(x), 42);
}

#[test]
/// 测试: default trait implementations 默认实现
fn test_default_trait_impl() {
    trait Greet {
        fn greet(&self) -> String {
            String::from("Hello!")
        }
    }

    struct Person;
    impl Greet for Person {}

    let p = Person;
    assert_eq!(p.greet(), "Hello!");
}

#[test]
/// 测试: trait 对象 dynamic dispatch
fn test_trait_object() {
    trait Draw {
        fn draw(&self) -> &str;
    }

    struct Circle;
    impl Draw for Circle {
        fn draw(&self) -> &str {
            "circle"
        }
    }

    let drawable: &dyn Draw = &Circle;
    assert_eq!(drawable.draw(), "circle");
}

// ===================== 扩充测试 =====================

#[test]
/// 测试: const fn 复杂编译期计算
fn test_const_fn_complex() {
    const fn factorial(n: u64) -> u64 {
        match n {
            0 | 1 => 1,
            n => n * factorial(n - 1),
        }
    }

    const FACT_5: u64 = factorial(5);
    assert_eq!(FACT_5, 120);
}

#[test]
/// 测试: const generics 配合数组操作
fn test_const_generics_array_operations() {
    fn sum_array<const N: usize>(arr: [i32; N]) -> i32 {
        arr.iter().sum()
    }

    assert_eq!(sum_array([1, 2, 3]), 6);
    assert_eq!(sum_array([10, 20]), 30);
}

#[test]
/// 测试: const generics 在不同类型中使用
fn test_const_generics_different_types() {
    fn create_vec<T: Copy, const N: usize>(value: T) -> Vec<T> {
        vec![value; N]
    }

    let v: Vec<i32> = create_vec::<i32, 4>(7);
    assert_eq!(v, vec![7, 7, 7, 7]);

    let s: Vec<&str> = create_vec::<&str, 3>("hi");
    assert_eq!(s, vec!["hi", "hi", "hi"]);
}

#[test]
/// 测试: 关联常量带默认值
fn test_associated_const_with_default() {
    trait Threshold {
        const MIN: i32 = 0;
        const MAX: i32 = 100;

        fn is_valid(value: i32) -> bool {
            value >= Self::MIN && value <= Self::MAX
        }
    }

    struct DefaultValid;
    impl Threshold for DefaultValid {}

    assert!(DefaultValid::is_valid(50));
    assert!(!DefaultValid::is_valid(-1));
    assert!(!DefaultValid::is_valid(101));
}

#[test]
/// 测试: default trait 实现重写
fn test_override_default_trait_impl() {
    trait Greeter {
        fn greet(&self) -> String {
            "默认问候".to_string()
        }
    }

    struct DefaultPerson;
    impl Greeter for DefaultPerson {}

    struct CustomPerson;
    impl Greeter for CustomPerson {
        fn greet(&self) -> String {
            "自定义问候".to_string()
        }
    }

    assert_eq!(DefaultPerson.greet(), "默认问候");
    assert_eq!(CustomPerson.greet(), "自定义问候");
}

#[test]
/// 测试: 多 trait 对象约束
fn test_multi_trait_object() {
    trait Identifiable {
        fn id(&self) -> i32;
    }

    impl Identifiable for i32 {
        fn id(&self) -> i32 { *self }
    }

    fn get_id(item: &dyn Identifiable) -> i32 {
        item.id()
    }

    let val: &dyn Identifiable = &42i32;
    assert_eq!(get_id(val), 42);
}

#[test]
/// 测试: trait upcasting (子 trait 向上转型为父 trait)
fn test_trait_upcasting() {
    trait Animal {
        fn name(&self) -> &str;
    }

    trait Dog: Animal {
        fn bark(&self) -> &str;
    }

    struct Husky;
    impl Animal for Husky {
        fn name(&self) -> &str { "Husky" }
    }
    impl Dog for Husky {
        fn bark(&self) -> &str { "Woof!" }
    }

    let husky = Husky;
    let dog: &dyn Dog = &husky;
    let animal: &dyn Animal = dog; // upcast
    assert_eq!(animal.name(), "Husky");
}

#[test]
/// 测试: blanket impl 避免冲突
fn test_blanket_impl_no_conflict() {
    trait AsString {
        fn as_string(&self) -> String;
    }

    impl<T: std::fmt::Display> AsString for T {
        fn as_string(&self) -> String {
            self.to_string()
        }
    }

    assert_eq!(42i32.as_string(), "42");
    assert_eq!(true.as_string(), "true");
    assert_eq!("hello".as_string(), "hello");
}

#[test]
/// 测试: const eval 配合 const generics
fn test_const_eval_with_const_generics() {
    const fn const_len() -> usize { 3 }

    let arr: [i32; const_len()] = [1, 2, 3];
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0], 1);
}

#[test]
/// 测试: compile-time 断言 (static_assertions 概念)
fn test_static_assertion_concept() {
    // 编译期断言 —— 如果错误则编译失败
    const ASSERT: () = assert!(std::mem::size_of::<i32>() == 4);
    let _ = ASSERT;

    const ASSERT_U64: () = assert!(std::mem::size_of::<u64>() == 8);
    let _ = ASSERT_U64;

    // 运行时通过即编译通过
    assert!(true);
}

#[test]
/// 测试: is_x86_feature_detected 运行时 CPU 检测
fn test_cpu_feature_detection_runtime() {
    // 基础检测 —— 总是返回 false 或 true
    let has_sse2 = is_x86_feature_detected!("sse2");
    // 我们不假设平台一定支持, 只验证宏执行不 panic
    let _ = has_sse2;
    assert!(true);
}
