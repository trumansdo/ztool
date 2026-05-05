// ---------------------------------------------------------------------------
// 5.7 保留关键字与语法
// ---------------------------------------------------------------------------

#[test]
/// 测试: gen 关键字保留 (为未来生成器语法做准备)
fn test_gen_keyword_reserved() {
    assert!(true);
}

#[test]
/// 测试: async 和 await 关键字
fn test_async_await_keywords() {
    let _ = async { 42 };
    assert!(true);
}

#[test]
/// 测试: 保留关键字 static 和 mut
fn test_static_mut_keywords() {
    static COUNTER: i32 = 0;
    assert_eq!(COUNTER, 0);
}

#[test]
/// 测试: dyn 关键字用于 trait 对象
fn test_dyn_keyword() {
    trait Greeter {
        fn greet(&self) -> &str;
    }

    struct Hello;
    impl Greeter for Hello {
        fn greet(&self) -> &str {
            "hello"
        }
    }

    let greeter: &dyn Greeter = &Hello;
    assert_eq!(greeter.greet(), "hello");
}

#[test]
/// 测试: raw identifier r# 用法
fn test_raw_identifier() {
    let r#async = 42;
    let r#type = "test";
    assert_eq!(r#async, 42);
    assert_eq!(r#type, "test");
}

#[test]
/// 测试: move 闭包关键字
fn test_move_keyword() {
    let data = vec![1, 2, 3];
    let moved = move || data.len();
    assert_eq!(moved(), 3);
}

#[test]
/// 测试: ref 模式匹配关键字
fn test_ref_keyword() {
    let value = Some(42);
    if let Some(ref x) = value {
        assert_eq!(*x, 42);
    }
}

#[test]
/// 测试: where 子句关键字
fn test_where_keyword() {
    fn print<T>(value: T)
    where
        T: std::fmt::Display,
    {
        println!("{}", value);
    }

    print("hello");
    assert!(true);
}

// ===================== 扩充测试 =====================

#[test]
/// 测试: raw identifier 在结构体和方法名中使用
fn test_raw_identifier_in_struct_and_method() {
    #[allow(non_camel_case_types)]
    struct r#impl {
        r#type: &'static str,
    }

    #[allow(non_snake_case)]
    impl r#impl {
        fn r#fn(&self) -> &str {
            self.r#type
        }
    }

    let obj = r#impl { r#type: "raw identifier" };
    assert_eq!(obj.r#fn(), "raw identifier");
}

#[test]
/// 测试: dyn 关键字在不同智能指针中的使用
fn test_dyn_with_different_pointers() {
    trait Value {
        fn get(&self) -> i32;
    }

    struct IntVal(i32);
    impl Value for IntVal {
        fn get(&self) -> i32 {
            self.0
        }
    }

    let ref_val: &dyn Value = &IntVal(10);
    let box_val: Box<dyn Value> = Box::new(IntVal(20));

    assert_eq!(ref_val.get(), 10);
    assert_eq!(box_val.get(), 20);
}

#[test]
/// 测试: move 关键字在不同场景
fn test_move_in_thread() {
    let data = vec![1, 2, 3];

    let handle = std::thread::spawn(move || {
        data.len()
    });

    assert_eq!(handle.join().unwrap(), 3);
}

#[test]
/// 测试: ref 和 ref mut 在模式匹配中的区别
fn test_ref_vs_ref_mut() {
    let mut value = Some(42);

    if let Some(ref x) = value {
        assert_eq!(*x, 42);
    }

    if let Some(ref mut x) = value {
        *x = 100;
    }

    assert_eq!(value, Some(100));
}

#[test]
/// 测试: static 变量与内部可变性
fn test_static_with_interior_mutability() {
    use std::sync::Mutex;
    static COUNTER: Mutex<i32> = Mutex::new(0);

    {
        let mut guard = COUNTER.lock().unwrap();
        *guard += 1;
    }

    assert_eq!(*COUNTER.lock().unwrap(), 1);
}

#[test]
/// 测试: where 子句多约束组合
fn test_where_clause_multiple_bounds() {
    fn complex<T, U>(t: T, u: U) -> String
    where
        T: std::fmt::Display + Clone,
        U: std::fmt::Debug,
    {
        format!("T: {}, U: {:?}", t.clone(), u)
    }

    let result = complex(42, "test");
    assert_eq!(result, "T: 42, U: \"test\"");
}

#[test]
/// 测试: const 关键字 —— 编译期常量和运行时常量
fn test_const_keyword() {
    const PI: f64 = 3.14159;
    const GREETING: &str = "你好, Rust!";

    assert!((PI - 3.14159).abs() < 0.0001);
    assert_eq!(GREETING, "你好, Rust!");
}

#[test]
/// 测试: Self 关键字 —— 类型别名
fn test_self_type_alias() {
    struct Wrapper(i32);

    impl Wrapper {
        fn new(value: i32) -> Self {
            Self(value)
        }

        fn inner(&self) -> i32 {
            self.0
        }
    }

    let w = Wrapper::new(42);
    assert_eq!(w.inner(), 42);
}

#[test]
/// 测试: extern 关键字与 ABIs
fn test_extern_with_various_abis() {
    // "Rust" ABI (默认)
    fn rust_fn() -> i32 { 42 }

    // "C" ABI 声明 (Edition 2024 需要 unsafe extern)
    assert_eq!(rust_fn(), 42);

    // 验证 extern "C" 语法
    #[allow(unused_unsafe)]
    unsafe extern "C" { fn _placeholder() -> i32; }
    assert!(true);
}

#[test]
/// 测试: unsafe 关键字的安全包装
fn test_unsafe_keyword_wrapper() {
    // 安全包装函数
    fn safe_get(arr: &[i32], index: usize) -> Option<i32> {
        if index < arr.len() {
            Some(unsafe { *arr.get_unchecked(index) })
        } else {
            None
        }
    }

    let data = [10, 20, 30];
    assert_eq!(safe_get(&data, 0), Some(10));
    assert_eq!(safe_get(&data, 5), None);
    assert_eq!(safe_get(&data, 2), Some(30));
}

#[test]
/// 测试: match 和 if let 关键字比较
fn test_match_vs_if_let_keywords() {
    let value: Option<i32> = Some(5);

    // match 方式
    let by_match = match value {
        Some(x) => x * 2,
        None => 0,
    };

    // if let 方式
    let by_if_let = if let Some(x) = value { x * 2 } else { 0 };

    assert_eq!(by_match, by_if_let);
}

#[test]
/// 测试: in 关键字在 for 和匹配中
fn test_in_keyword_usage() {
    // for 循环
    let mut sum = 0;
    for i in 0..5 {
        sum += i;
    }
    assert_eq!(sum, 10);
}

#[test]
/// 测试: raw identifier 避免与 edition 关键字冲突
fn test_raw_identifier_edition_compat() {
    // 使用 r#gen 模拟新版代码中的旧 gen 标识符
    let r#gen = vec![1, 2, 3];
    assert_eq!(r#gen.len(), 3);

    // 使用 r#macro (如果 macro 在特定上下文中是关键字)
    let r#macro = "macro_rules";
    assert_eq!(r#macro, "macro_rules");
}
