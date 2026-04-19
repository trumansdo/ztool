// ---------------------------------------------------------------------------
// 2.2 错误处理 - Result/Option/Try
// ---------------------------------------------------------------------------

#[test]
/// 测试: Result 基础操作 (Ok/Err/unwrap/is_ok/is_err)
fn test_result_basics() {
    // 语法: Result<T, E> 表示成功(T)或失败(E); unwrap() 成功时返回值, 失败时 panic
    // 避坑: 生产代码避免 unwrap, 用 ? 或 match 处理; unwrap() 的 panic 信息不含上下文
    fn returns_result() -> Result<i32, String> {
        Ok(42)
    }
    let result: Result<i32, _> = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);

    // Err 情况
    let err: Result<i32, &str> = Err("not found");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err(), "not found");
}

#[test]
/// 测试: Option 基础操作 (Some/None/unwrap_or/map/and_then)
fn test_option_basics() {
    // 语法: Option<T> 表示存在(T)或不存在(None)
    //
    // 安全取值:
    //   - unwrap_or(default)     None 时返回默认值
    //   - unwrap_or_else(f)      None 时用闭包计算默认值(惰性)
    //   - expect(msg)            Some 时返回值, None 时 panic(带消息)
    //   - unwrap_or_default()    None 时返回 T::default()
    //
    // 转换:
    //   - map(f)                 Some 时应用 f, None 时不变
    //   - and_then(f)            Some 时应用 f(返回 Option), None 时不变 (扁平化 map)
    //   - or(opt)                Some 时返回自身, None 时返回 opt
    //   - or_else(f)             Some 时返回自身, None 时用闭包计算
    //   - filter(pred)           Some 且满足条件时保留, 否则变 None
    //   - zip(other)             组合两个 Option 为 Option<(T, U)>
    //   - ok_or(err)             Some→Ok, None→Err
    //   - ok_or_else(f)          Some→Ok, None→Err(惰性)
    //
    // 避坑:
    //   - unwrap_or 的参数总是被求值, 即使 Some; 用 unwrap_or_else 惰性求值
    //   - and_then 的闭包必须返回 Option, 不是普通值
    //   - expect 的 panic 消息在 None 时才显示, 不要滥用
    //
    let some: Option<i32> = Some(10);
    let none: Option<i32> = None;

    // 安全取值
    assert_eq!(some.unwrap_or(0), 10);
    assert_eq!(none.unwrap_or(0), 0);
    assert_eq!(none.unwrap_or_else(|| 40 + 2), 42); // 惰性求值
    assert_eq!(none.unwrap_or_default(), 0);

    // map
    assert_eq!(some.map(|x| x * 2), Some(20));
    assert_eq!(none.map(|x| x * 2), None);

    // and_then (扁平化)
    fn safe_sqrt(x: f64) -> Option<f64> {
        if x >= 0.0 { Some(x.sqrt()) } else { None }
    }
    assert_eq!(Some(4.0).and_then(safe_sqrt), Some(2.0));
    assert_eq!(Some(-1.0).and_then(safe_sqrt), None);
    assert_eq!(None.and_then(safe_sqrt), None);

    // or / or_else
    assert_eq!(some.or(Some(0)), Some(10));
    assert_eq!(none.or(Some(0)), Some(0));
    assert_eq!(none.or_else(|| Some(99)), Some(99));

    // filter
    assert_eq!(Some(10).filter(|&x| x > 5), Some(10));
    assert_eq!(Some(3).filter(|&x| x > 5), None);

    // ok_or
    assert_eq!(Some(42).ok_or("missing"), Ok(42));
    assert_eq!(None::<i32>.ok_or("missing"), Err("missing"));
}

#[test]
/// 测试: Result 转换方法 (map/map_err/and_then/or_else)
fn test_result_transform() {
    // 语法: Result 提供丰富的转换方法, 类似 Option 但有 Ok/Err 两侧
    //
    // 方法:
    //   - map(f)                 Ok 时应用 f, Err 时不变
    //   - map_err(f)             Err 时应用 f, Ok 时不变
    //   - and_then(f)            Ok 时应用 f(返回 Result), Err 时不变
    //   - or_else(f)             Err 时应用 f(返回 Result), Ok 时不变
    //   - inspect(f)             Ok 时执行副作用, 不改变值 (1.76+)
    //   - inspect_err(f)         Err 时执行副作用, 不改变值 (1.76+)
    //   - transpose()            Result<Option<T>, E> → Option<Result<T, E>>
    //   - flatten()              Result<Result<T, E>, E> → Result<T, E>
    //
    // 避坑:
    //   - map_err 常用于统一错误类型, 配合 ? 操作符
    //   - and_then 的闭包必须返回 Result, 不是普通值
    //   - inspect/inspect_err 不消费值, 仅用于日志/调试

    // map
    let ok: Result<i32, &str> = Ok(5);
    assert_eq!(ok.map(|x| x * 2), Ok(10));

    let err: Result<i32, &str> = Err("error");
    assert_eq!(err.map(|x| x * 2), Err("error"));

    // map_err (转换错误类型)
    let err: Result<i32, &str> = Err("parse error");
    let converted: Result<i32, String> = err.map_err(|e| e.to_string());
    assert_eq!(converted, Err("parse error".to_string()));

    // and_then
    fn parse_int(s: &str) -> Result<i32, String> {
        s.parse::<i32>()
            .map_err(|e: std::num::ParseIntError| e.to_string())
    }
    let ok_str: Result<&str, String> = Ok("42");
    assert_eq!(ok_str.and_then(parse_int), Ok(42));

    let err_str: Result<&str, String> = Ok("abc");
    assert!(
        err_str
            .and_then(parse_int)
            .is_err()
    );

    let err_data: Result<&str, String> = Err("no data".to_string());
    assert_eq!(err_data.and_then(parse_int), Err("no data".to_string()));

    // or_else
    let result: Result<i32, &str> = Err("primary failed");
    let fallback: Result<i32, &str> = result.or_else(|_| Ok(0));
    assert_eq!(fallback, Ok(0));

    // inspect (1.76+)
    let mut logged = false;
    let val: Result<i32, &str> = Ok(42).inspect(|x| {
        if *x == 42 {
            logged = true;
        }
    });
    assert!(logged);
    assert_eq!(val, Ok(42));

    // transpose
    let ro: Result<Option<i32>, &str> = Ok(Some(42));
    let or: Option<Result<i32, &str>> = ro.transpose();
    assert_eq!(or, Some(Ok(42)));

    let ro: Result<Option<i32>, &str> = Ok(None);
    let or: Option<Result<i32, &str>> = ro.transpose();
    assert_eq!(or, None);

    let ro: Result<Option<i32>, &str> = Err("fail");
    let or: Option<Result<i32, &str>> = ro.transpose();
    assert_eq!(or, Some(Err("fail")));
}

#[test]
/// 测试: ? 操作符在 Result 上的链式调用
fn test_try_operator() {
    // 语法: ? 操作符: Ok 时解包, Err 时提前返回; 只能用在返回 Result/Option 的函数中
    // 避坑: ? 会调用 From::from 转换错误类型, 确保错误类型可转换; main 函数返回 Result 时可用 ?
    fn fallible_operation(x: i32) -> Result<i32, String> {
        if x < 0 { Err("negative".to_string()) } else { Ok(x * 2) }
    }

    fn chained_operations(x: i32) -> Result<i32, String> {
        let a = fallible_operation(x)?;
        let b = fallible_operation(a)?;
        Ok(b)
    }

    assert_eq!(chained_operations(2), Ok(8));
    assert!(chained_operations(-1).is_err());
}

#[test]
/// 测试: ? 操作符在 Option 上的链式调用
fn test_try_operator_option() {
    // 语法: ? 同样适用于 Option: Some 时解包, None 时提前返回 None
    // 避坑: 不能在返回 Result 的函数中对 Option 用 ?, 反之亦然
    fn safe_divide(a: f64, b: f64) -> Option<f64> {
        if b == 0.0 { None } else { Some(a / b) }
    }

    fn compute() -> Option<f64> {
        let x = safe_divide(10.0, 2.0)?;
        let y = safe_divide(x, 5.0)?;
        Some(y)
    }

    assert_eq!(compute(), Some(1.0));
}

#[test]
/// 测试: ? 操作符的错误类型转换 (From trait)
fn test_try_operator_error_conversion() {
    // 语法: ? 自动调用 From::from 转换错误类型, 实现错误类型统一
    // 避坑: 错误类型必须实现 From<源错误> 到目标错误的转换; 否则编译失败

    use std::num::ParseIntError;

    fn parse_and_double(s: &str) -> Result<i32, String> {
        // parse 返回 ParseIntError, ? 自动通过 From 转为 String
        let n: i32 = s
            .parse::<i32>()
            .map_err(|e: ParseIntError| e.to_string())?;
        Ok(n * 2)
    }

    assert_eq!(parse_and_double("21"), Ok(42));
    assert!(parse_and_double("abc").is_err());
}

#[test]
/// 测试: main 函数返回 Result
fn test_main_returns_result() {
    // 语法: main 函数可以返回 Result<(), E>, 其中 E: Debug
    // 避坑: 返回 Err 时程序退出码非零; E 必须实现 Debug trait

    fn example_main() -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string("/nonexistent")?;
        println!("{}", content);
        Ok(())
    }

    // 模拟调用
    let result = example_main();
    assert!(result.is_err()); // 文件不存在
}

#[test]
/// 测试: Result flatten 展平嵌套 Result (1.89+)
fn test_result_flatten() {
    // 语法: flatten() 展平嵌套 Result<Result<T,E>,E> → Result<T,E> (1.89+)
    // 避坑: 内外层错误类型必须一致; 外层 Err 优先返回, 内层 Err 次之
    let nested: Result<Result<i32, String>, String> = Ok(Ok(42));
    assert_eq!(nested.flatten(), Ok(42));

    let err_inner: Result<Result<i32, String>, String> = Ok(Err("inner".to_string()));
    assert_eq!(err_inner.flatten(), Err("inner".to_string()));

    let err_outer: Result<Result<i32, String>, String> = Err("outer".to_string());
    assert_eq!(err_outer.flatten(), Err("outer".to_string()));
}

#[test]
/// 测试: panic! / assert! / debug_assert! 宏
fn test_panic_and_assert() {
    // 语法:
    //   - panic!(msg)            立即终止当前线程
    //   - assert!(cond)          条件为 false 时 panic
    //   - assert_eq!(a, b)       不等时 panic, 显示两边值
    //   - assert_ne!(a, b)       相等时 panic
    //   - debug_assert!(cond)    仅 debug 模式生效, release 被优化掉
    //   - unreachable!()         标记不可能到达的代码 (返回 ! 类型)
    //   - unimplemented!()       标记未实现的代码 (返回 ! 类型)
    //
    // 避坑:
    //   - debug_assert 在 release (--release) 中完全跳过, 不能有副作用
    //   - assert_eq! 的参数必须实现 Debug 和 PartialEq
    //   - panic! 默认展开 unwind, 可配置为 abort (Cargo.toml: panic = "abort")
    //   - unreachable! 用于告诉编译器某分支不可能到达, 到达则 panic

    // assert 宏
    assert!(true);
    assert_eq!(1 + 1, 2);
    assert_ne!(1, 2);

    // debug_assert (release 模式下不执行)
    debug_assert!(true);
    debug_assert_eq!(2 + 2, 4);

    // unreachable
    let x: Option<i32> = Some(5);
    match x {
        Some(n) => assert_eq!(n, 5),
        None => unreachable!(),
    }
}

#[test]
/// 测试: catch_unwind 捕获 panic
fn test_catch_unwind() {
    // 语法: std::panic::catch_unwind(f) 捕获 panic, 返回 Result<T, Box<dyn Any + Send>>
    // 避坑: 只能捕获 unwind 模式的 panic; panic = "abort" 时无法捕获; 捕获后线程状态不确定

    use std::panic::catch_unwind;

    // 不 panic 的情况
    let result = catch_unwind(|| 42);
    assert_eq!(result.unwrap(), 42);

    // panic 的情况
    let result = catch_unwind(|| {
        panic!("intentional panic");
    });
    assert!(result.is_err());
}

#[test]
/// 测试: Option 组合操作 (filter/zip/flatten)
fn test_option_combinators() {
    // 语法: Option 提供多种组合方法, 避免嵌套 match

    // filter
    let some: Option<i32> = Some(10);
    assert_eq!(some.filter(|&x| x > 5), Some(10));
    assert_eq!(some.filter(|&x| x > 15), None);

    // zip
    let a = Some(1);
    let b = Some("hello");
    assert_eq!(a.zip(b), Some((1, "hello")));
    assert_eq!(Some(1).zip(None::<&str>), None);
    assert_eq!(None::<i32>.zip(Some("hello")), None);

    // flatten (Option<Option<T>> → Option<T>)
    let nested: Option<Option<i32>> = Some(Some(42));
    assert_eq!(nested.flatten(), Some(42));
    assert_eq!(Some(None::<i32>).flatten(), None);
    assert_eq!(None::<Option<i32>>.flatten(), None);

    // is_some_and (1.70+)
    assert!(Some(10).is_some_and(|x| x > 5));
    assert!(!Some(3).is_some_and(|x| x > 5));
    assert!(!None.is_some_and(|x: i32| x > 5));
}

#[test]
/// 测试: Result 组合操作 (transpose/copied/cloned)
fn test_result_combinators() {
    // 语法: Result 的组合方法, 处理嵌套和引用

    // transpose: Result<Option<T>, E> ↔ Option<Result<T, E>>
    let ro: Result<Option<i32>, &str> = Ok(Some(42));
    assert_eq!(ro.transpose(), Some(Ok(42)));

    let ro: Result<Option<i32>, &str> = Err("fail");
    assert_eq!(ro.transpose(), Some(Err("fail")));

    // copied: Result<&T, E> → Result<T, E> (T: Copy)
    let x = 42;
    let r: Result<&i32, &str> = Ok(&x);
    assert_eq!(r.copied(), Ok(42));

    // cloned: Result<&T, E> → Result<T, E> (T: Clone)
    let s = String::from("hello");
    let r: Result<&String, &str> = Ok(&s);
    assert_eq!(r.cloned(), Ok(String::from("hello")));
}
