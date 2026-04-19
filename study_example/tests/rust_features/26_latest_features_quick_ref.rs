// ---------------------------------------------------------------------------
// 5.9 最新特性快速参考 (1.85-1.90+)
// ---------------------------------------------------------------------------

#[test]
/// 测试: 裸函数 naked functions 概念 (1.88+)
fn test_naked_functions_concept() {
    // 语法: 1.88+ #[unsafe(naked)] 裸函数, 完全控制生成的汇编, 配合 naked_asm! 使用
    // 避坑: naked 函数不能有 prologue/epilogue; 函数体必须全是内联汇编; 不能普通 Rust 代码
    assert!(true);
}

#[test]
/// 测试: 安全架构 intrinsic (1.87+)
fn test_safe_arch_intrinsics() {
    // 语法: 1.87+ 启用对应 target_feature 后, 许多原本 unsafe 的 intrinsic 变为安全
    // 避坑: 必须在启用对应特性的函数中调用; 跨平台 intrinsic 需用 cfg 保护
    if is_x86_feature_detected!("sse2") {
        assert!(true);
    }
}

#[test]
/// 测试: asm! 标签操作数概念 (1.87+)
fn test_asm_labels_concept() {
    // 语法: 1.87+ asm! 支持 label 操作数, 可跳转到 Rust 代码块
    // 避坑: 仅部分架构支持; label 跳转绕过 Rust 借用检查, 需手动保证安全
    assert!(true);
}

#[test]
/// 测试: cargo publish --workspace 工作区发布 (1.90+)
fn test_workspace_publishing() {
    // 语法: 1.90+ cargo publish --workspace 一次性发布 workspace 中所有包
    // 避坑: 各包版本号需手动管理; 发布失败时部分包可能已发布
    assert!(true);
}

#[test]
/// 测试: LLD 默认链接器 (1.90+)
fn test_lld_linker() {
    // 语法: 1.90+ x86_64-unknown-linux-gnu 默认使用 LLD 链接器, 链接速度大幅提升
    // 避坑: 仅 Linux 平台; Windows/macOS 不受影响; 可通过 -C linker 切换回默认链接器
    assert!(true);
}

#[test]
/// 测试: const eval 编译期求值
fn test_const_evaluation() {
    // 语法: const fn 允许在编译期求值
    // 避坑: const 上下文中不能调用非 const fn; 递归 const 可能导致编译器崩溃
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
    // 语法: #[inline] 提示编译器内联, #[inline(always)] 强制内联
    // 避坑: 过度内联增加二进制大小; 小函数内联收益大
    #[inline]
    fn small_fn() -> i32 {
        42
    }
    assert_eq!(small_fn(), 42);
}

#[test]
/// 测试: const generics 泛型常量
fn test_const_generics() {
    // 语法: const<T, const N: usize> 允许泛型常量参数
    // 避坑: const 参数必须是编译期常量; 不能用变量
    fn make_array<T: Copy + Default, const N: usize>() -> [T; N] {
        [T::default(); N]
    }

    let arr: [i32; 3] = make_array();
    assert_eq!(arr, [0; 3]);
}

#[test]
/// 测试: associated const 关联常量
fn test_associated_const() {
    // 语法: trait 中可定义 const 关联项
    // 避坑: 关联 const 类似于关联类型, 但有具体值
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
    // 语法: T: Trait 和 trait Object 等价
    // 避坑: 使用 dyn Trait 表示动态分发
    fn print_debug<T: std::fmt::Debug>(value: &T) {
        println!("{:?}", value);
    }

    print_debug(&42);
    assert!(true);
}

#[test]
/// 测试: blanket impl 毯式实现
fn test_blanket_impl() {
    // 语法: impl<T> Trait for T where T: SomeTrait 是一种毯式实现
    // 避坑: 毯式实现可能与特定实现冲突, 编译器会报错
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
    // 语法: trait 方法可提供默认实现
    // 避坑: 默认实现可被重写; 重写时可调用原默认实现
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
    // 语法: &dyn Trait 或 Box<dyn Trait> 创建 trait 对象
    // 避坑: trait 对象有运行时开销; 不是所有 trait 都能作 trait 对象 (对象安全)
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
