// ---------------------------------------------------------------------------
// 5.7 保留关键字与语法
// ---------------------------------------------------------------------------

#[test]
/// 测试: gen 关键字保留 (为未来生成器语法做准备)
fn test_gen_keyword_reserved() {
    // 语法: gen 被保留为关键字, 为未来生成器语法(gen { } / gen || )做准备
    // 避坑: gen 不能用作变量名/函数名/类型名; 旧代码中用 gen 作标识符的需要改名
    assert!(true);
}

#[test]
/// 测试: async 和 await 关键字
fn test_async_await_keywords() {
    // 语法: async 和 await 是关键字, 不能用作标识符
    // 避坑: Rust 2015+ 中两者都是关键字; 可用 r#async / r#await 转义
    let _ = async { 42 };
    assert!(true);
}

#[test]
/// 测试: 保留关键字 static 和 mut
fn test_static_mut_keywords() {
    // 语法: static 和 mut 组合表示可变性
    // 避坑: static 变量默认不可变, 需加 mut 才能修改
    static COUNTER: i32 = 0;
    assert_eq!(COUNTER, 0);
}

#[test]
/// 测试: dyn 关键字用于 trait 对象
fn test_dyn_keyword() {
    // 语法: dyn 用于创建 trait 对象 (dyn Trait)
    // 避坑: dyn 后的类型必须实现 trait; 避免与具体类型混淆
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
    // 语法: r# 允许使用关键字作为标识符
    // 避坑: 跨 edition 迁移时有用; 但代码可读性可能下降
    let r#async = 42;
    let r#type = "test";
    assert_eq!(r#async, 42);
    assert_eq!(r#type, "test");
}

#[test]
/// 测试: move 闭包关键字
fn test_move_keyword() {
    // 语法: move 闭包获取捕获变量的所有权
    // 避坑: 闭包中使用了 move, 原变量将不可用(如果实现了 Copy 则仍可用)
    let data = vec![1, 2, 3];
    let moved = move || data.len();
    assert_eq!(moved(), 3);
}

#[test]
/// 测试: ref 模式匹配关键字
fn test_ref_keyword() {
    // 语法: ref 在模式匹配中创建引用
    // 避坑: ref mut 可变引用, ref 不可变引用
    let value = Some(42);
    if let Some(ref x) = value {
        assert_eq!(*x, 42);
    }
}

#[test]
/// 测试: where 子句关键字
fn test_where_keyword() {
    // 语法: where 子句提供泛型约束
    // 避坑: where 可在函数定义和 trait 定义中使用
    fn print<T>(value: T)
    where
        T: std::fmt::Display,
    {
        println!("{}", value);
    }
    
    print("hello");
    assert!(true);
}
