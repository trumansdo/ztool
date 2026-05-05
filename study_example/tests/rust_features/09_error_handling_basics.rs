// ---------------------------------------------------------------------------
// 2.2 错误处理 - Result/Option/Try
// ---------------------------------------------------------------------------

use std::panic::{catch_unwind, AssertUnwindSafe};

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

// ==========================================================================
// 以下是增强版 panic/assert 测试: 覆盖 assert!/assert_eq!/assert_ne!/
//   debug_assert!/unreachable!/unimplemented!/todo! 完整家族
// ==========================================================================

#[test]
/// 测试: panic 断言家族 (assert!/assert_eq!/assert_ne!/debug_assert!/unreachable!/unimplemented!/todo!)
fn test_panic_assert_family() {
    // 语法:
    //   - panic!(msg)            立即终止当前线程
    //   - assert!(cond)          条件为 false 时 panic
    //   - assert!(cond, fmt, ..) 条件为 false 时 panic 带格式化消息
    //   - assert_eq!(a, b)       不等时 panic, 显示两边值
    //   - assert_ne!(a, b)       相等时 panic
    //   - debug_assert!(cond)    仅 debug 模式生效, release 被优化掉
    //   - debug_assert_eq!(a, b) 仅 debug 模式生效
    //   - debug_assert_ne!(a, b) 仅 debug 模式生效
    //   - unreachable!(fmt, ..)  标记不可能到达的代码 (返回 ! 类型)
    //   - unimplemented!(fmt, ..)标记未实现的代码 (返回 ! 类型)
    //   - todo!(fmt, ..)         同 unimplemented! 但语义更温和
    //
    // 避坑:
    //   - debug_assert 在 release (--release) 中完全跳过, 不能有副作用
    //   - assert_eq! 的参数必须实现 Debug 和 PartialEq
    //   - panic! 默认展开 unwind, 可配置为 abort (Cargo.toml: panic = "abort")
    //   - unreachable! 用于告诉编译器某分支不可能到达, 到达则 panic
    //   - 所有 ! 返回类型的宏可以用于任何期望任意类型的位置

    // assert 宏
    assert!(true);
    assert!(1 + 1 == 2, "基本算术不应失败");
    assert_eq!(1 + 1, 2);
    assert_eq!(2 * 3, 6, "乘法: 2*3 应等于 6");
    assert_ne!(1, 2);
    assert_ne!("hello", "world", "两个字符串不应相等");

    // debug_assert (release 模式下不执行)
    debug_assert!(true);
    debug_assert_eq!(2 + 2, 4);
    debug_assert_ne!(3, 5);

    // unreachable: 标记不可能到达的分支
    let x: Option<i32> = Some(5);
    match x {
        Some(n) => assert_eq!(n, 5),
        None => unreachable!("逻辑上此分支不可达"),
    }

    // unimplemented! / todo! 不会在此直接调用 (它们会 panic),
    // 但验证它们在 helper 函数内部可以存在:
    fn _placeholder() -> String {
        todo!("等待实现")
    }
    fn _placeholder2() -> i32 {
        unimplemented!("函数未完成")
    }
}

#[test]
/// 测试: catch_unwind 捕获 panic 与不安全展开边界 (AssertUnwindSafe)
fn test_catch_unwind() {
    // 语法: std::panic::catch_unwind(f) 捕获 unwind 模式的 panic, 返回 Result
    // 避坑:
    //   - 只能捕获 unwind 模式的 panic; panic = "abort" 时无法捕获
    //   - 捕获到 panic 后线程状态可能不一致 (部分写入、锁未释放)
    //   - 可变引用需要 AssertUnwindSafe 包装才能传入闭包
    //   - catch_unwind 不应替代正常错误处理

    // 不 panic 的情况
    let result = catch_unwind(|| 42);
    assert_eq!(result.unwrap(), 42);

    // panic 的情况 —— 基础捕获
    let result = catch_unwind(|| {
        panic!("intentional panic");
    });
    assert!(result.is_err());

    // 从 Box<dyn Any> 中提取 panic 消息
    let result = catch_unwind(|| {
        panic!("具体的错误消息");
    });
    match result {
        Ok(_) => unreachable!(),
        Err(e) => {
            // 尝试解析为 &str
            if let Some(msg) = e.downcast_ref::<&str>() {
                assert_eq!(*msg, "具体的错误消息");
            } else if let Some(msg) = e.downcast_ref::<String>() {
                assert_eq!(msg, "具体的错误消息");
            }
        }
    }

    // AssertUnwindSafe: 包装可变状态使其可通过 catch_unwind 捕获
    // (注意: 这样做绕过了编译器安全检查, 仅在确信展开不会破坏状态时使用)
    let mut counter = 0;
    let result = catch_unwind(AssertUnwindSafe(|| {
        counter += 1;   // 即使 panic 前部分修改了 counter, 仍被捕获
        42
    }));
    assert_eq!(result.unwrap(), 42);
    // counter 可能被修改也可能不变, 取决于闭包是否正常完成

    // 验证嵌套 panic 也能被捕获
    let result = catch_unwind(|| {
        let _ = catch_unwind(|| {
            panic!("内层 panic");
        });
        "外层继续"
    });
    assert_eq!(result.unwrap(), "外层继续");
}

#[test]
/// 测试: Option 组合操作 (filter/zip/flatten/is_some_and)
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

// ==========================================================================
// 新增测试: Option 方法全集测试 (map_or/map_or_else/and/xor/as_deref/zip_with/take/replace 等)
// ==========================================================================

#[test]
/// 测试: Option 完整方法族 (map_or / map_or_else / and / xor / as_deref / zip_with / take / replace / is_some_and)
fn test_option_methods_comprehensive() {
    // map_or: Some 时映射, None 时返回默认值
    assert_eq!(Some("42").map_or(0, |s| s.len()), 2);
    assert_eq!(None::<&str>.map_or(0, |s| s.len()), 0);

    // map_or_else: None 时用闭包生成默认值 (惰性)
    assert_eq!(None::<i32>.map_or_else(|| 100, |x| x * 2), 100);
    assert_eq!(Some(5).map_or_else(|| 100, |x| x * 2), 10);

    // and: Some 时返回第二个 Option, 丢弃自己的值
    assert_eq!(Some(1).and(Some("hello")), Some("hello"));
    assert_eq!(None::<i32>.and(Some("hello")), None);
    assert_eq!(Some(1).and(None::<&str>), None);

    // xor: 恰好一个为 Some 时返回它
    assert_eq!(Some(1).xor(Some(2)), None);
    assert_eq!(Some(1).xor(None), Some(1));
    assert_eq!(None::<i32>.xor(Some(2)), Some(2));
    assert_eq!(None::<i32>.xor(None::<i32>), None);

    // as_deref: Option<String> → Option<&str>
    let opt: Option<String> = Some("hello".into());
    assert_eq!(opt.as_deref(), Some("hello"));
    assert_eq!(None::<String>.as_deref(), None);

    // zip_with 为不稳定特性 (option_zip #70086), 注释保留供参考
    // 可用 zip + map 替代: a.zip(b).map(|(x, y)| x * y)

    // take: 从可变引用中取出值, 留下 None
    let mut opt = Some(42);
    let taken = opt.take();
    assert_eq!(taken, Some(42));
    assert_eq!(opt, None);

    // replace: 替换旧值, 返回旧值
    let mut opt = Some(10);
    let old = opt.replace(20);
    assert_eq!(old, Some(10));
    assert_eq!(opt, Some(20));

    // insert: 插入值并返回可变引用
    let mut opt = None::<i32>;
    let r = opt.insert(99);
    assert_eq!(*r, 99);
    assert_eq!(opt, Some(99));

    // get_or_insert_with: None 时用闭包插入 (惰性)
    let mut opt = None::<i32>;
    let r = opt.get_or_insert_with(|| 42 * 2);
    assert_eq!(*r, 84);
    assert_eq!(opt, Some(84));
}

// ==========================================================================
// 新增测试: Result 方法全集测试 (unwrap_or_default/map_or/map_or_else/and/or/
//   ok/err/as_deref/copied/cloned/inspect_err 等)
// ==========================================================================

#[test]
/// 测试: Result 完整方法族 (map_or/map_or_else/and/or/ok/err/as_deref/is_ok_and/is_err_and)
fn test_result_methods_comprehensive() {
    // map_or: Ok 时映射, Err 时返回默认值
    let ok: Result<i32, &str> = Ok(5);
    assert_eq!(ok.map_or(0, |x| x * 2), 10);

    let err: Result<i32, &str> = Err("fail");
    assert_eq!(err.map_or(0, |x| x * 2), 0);

    // map_or_else: Err 时用闭包生成默认值 (惰性)
    assert_eq!(Err::<i32, &str>("fail").map_or_else(|_| 100, |x| x), 100);

    // and: Ok 时返回第二个 Result
    let ok: Result<i32, &str> = Ok(1);
    let ok2: Result<&str, &str> = Ok("good");
    assert_eq!(ok.and(ok2), Ok("good"));

    let err: Result<i32, &str> = Err("first");
    assert_eq!(err.and(Ok("good")), Err("first"));

    // or: Err 时返回备选 Result
    let err: Result<i32, &str> = Err("first");
    assert_eq!(err.or::<&str>(Ok(0)), Ok(0));

    let ok: Result<i32, &str> = Ok(42);
    assert_eq!(ok.or::<&str>(Ok(0)), Ok(42));

    // ok(): Result<T,E> → Option<T>  (Ok→Some, Err→None)
    assert_eq!(Ok::<i32, &str>(42).ok(), Some(42));
    assert_eq!(Err::<i32, &str>("fail").ok(), None);

    // err(): Result<T,E> → Option<E>  (Err→Some, Ok→None)
    assert_eq!(Err::<i32, &str>("fail").err(), Some("fail"));
    assert_eq!(Ok::<i32, &str>(42).err(), None);

    // as_deref: &Result<String, String> → Result<&str, &str>
    let r: Result<String, String> = Ok("hello".into());
    let deref: Result<&str, &String> = r.as_deref();
    assert_eq!(deref, Ok("hello"));

    // unwrap_or_default: Err 时返回 T::default() (1.64+)
    assert_eq!(Ok::<i32, &str>(42).unwrap_or_default(), 42);
    assert_eq!(Err::<i32, &str>("fail").unwrap_or_default(), 0);

    // is_ok_and / is_err_and (1.47+)
    assert!(Ok::<i32, &str>(10).is_ok_and(|x| x > 5));
    assert!(!Ok::<i32, &str>(3).is_ok_and(|x| x > 5));
    assert!(!Err::<i32, &str>("err").is_ok_and(|x| x > 5));

    assert!(Err::<i32, &str>("fail").is_err_and(|e| e.contains("fail")));
    assert!(!Err::<i32, &str>("fail").is_err_and(|e| e.contains("ok")));
    assert!(!Ok::<i32, &str>(42).is_err_and(|_| true));

    // inspect_err: Err 时执行副作用 (1.76+)
    let mut seen = false;
    let r: Result<i32, &str> = Err("oops").inspect_err(|e| {
        if *e == "oops" { seen = true; }
    });
    assert!(seen);
    assert_eq!(r, Err("oops"));
}

// ==========================================================================
// 新增测试: Option ↔ Result 互转 (ok_or/ok_or_else/transpose 完整行为)
// ==========================================================================

#[test]
/// 测试: Option ↔ Result 互转 (ok_or / ok_or_else / transpose 完整行为)
fn test_option_result_conversion() {
    // ok_or: Some→Ok, None→Err (立即求值)
    assert_eq!(Some(42).ok_or("缺失"), Ok(42));
    assert_eq!(None::<i32>.ok_or("缺失"), Err("缺失"));

    // ok_or_else: None→Err (惰性求值)
    let mut called = false;
    let result = None::<i32>.ok_or_else(|| {
        called = true;
        "惰性错误"
    });
    assert!(called);
    assert_eq!(result, Err("惰性错误"));

    // Some 时不会调用闭包
    let mut called = false;
    let result = Some(42).ok_or_else(|| {
        called = true;
        "不应调用"
    });
    assert!(!called);
    assert_eq!(result, Ok(42));

    // Option::transpose: Option<Result<T,E>> → Result<Option<T>,E>
    let x: Option<Result<i32, &str>> = Some(Ok(42));
    assert_eq!(x.transpose(), Ok(Some(42)));

    let x: Option<Result<i32, &str>> = Some(Err("失败"));
    assert_eq!(x.transpose(), Err("失败"));

    let x: Option<Result<i32, &str>> = None;
    assert_eq!(x.transpose(), Ok(None));

    // Result::transpose: Result<Option<T>,E> → Option<Result<T,E>>
    let r: Result<Option<i32>, &str> = Ok(Some(42));
    assert_eq!(r.transpose(), Some(Ok(42)));

    let r: Result<Option<i32>, &str> = Ok(None);
    assert_eq!(r.transpose(), None);

    let r: Result<Option<i32>, &str> = Err("失败");
    assert_eq!(r.transpose(), Some(Err("失败")));

    // 综合: Option→Result→Option 往返
    let original: Option<i32> = Some(42);
    let result = original.ok_or("缺失");
    let back = result.ok();
    assert_eq!(back, Some(42));
}

// ==========================================================================
// 新增测试: 迭代器 collect::<Result<Vec<_>, _>>() 收集
// ==========================================================================

#[test]
/// 测试: collect::<Result<Vec<_>,_>>() 迭代器收集 —— 全有或全无
fn test_iterator_collect_result() {
    // 全部成功 → 收集成功
    let all_ok: Result<Vec<i32>, _> = ["1", "2", "3"]
        .iter()
        .map(|s| s.parse::<i32>())
        .collect();
    assert_eq!(all_ok, Ok(vec![1, 2, 3]));

    // 遇到第一个错误 → 整体失败, 停止收集
    let with_err: Result<Vec<i32>, _> = ["1", "abc", "3"]
        .iter()
        .map(|s| s.parse::<i32>())
        .collect();
    assert!(with_err.is_err());

    // 空迭代器 → OK + 空 Vec
    let empty: Result<Vec<i32>, std::num::ParseIntError> =
        Vec::<&str>::new().iter().map(|s| s.parse::<i32>()).collect();
    assert_eq!(empty, Ok(vec![]));

    // 使用 filter_map 收集成功结果, 忽略失败 (用 .ok())
    let valid: Vec<i32> = ["1", "x", "2", "y", "3"]
        .iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    assert_eq!(valid, vec![1, 2, 3]);

    // 使用 partition 分别收集成功/失败
    let (oks, errs): (Vec<_>, Vec<_>) = ["1", "x", "2", "3"]
        .iter()
        .map(|s| s.parse::<i32>())
        .partition(Result::is_ok);
    assert_eq!(oks.len(), 3);
    assert_eq!(errs.len(), 1);
}

// ==========================================================================
// 新增测试: thiserror 自定义错误类型
// ==========================================================================

#[test]
/// 测试: thiserror derive 宏自定义错误 (Display/source/From)
fn test_thiserror_macro() {
    use std::error::Error;
    use thiserror::Error as ThisError;

    // NOTE: std::io::Error 未实现 PartialEq, 所以 TestError 也无法 derive PartialEq
    #[derive(ThisError, Debug)]
    enum TestError {
        #[error("配置缺失")]
        ConfigMissing,

        #[error("值无效: {0}")]
        InvalidValue(String),

        #[error("IO错误: {source}")]
        Io {
            #[from]
            source: std::io::Error,
        },
    }

    // 测试 Display 输出
    assert_eq!(TestError::ConfigMissing.to_string(), "配置缺失");
    assert_eq!(TestError::InvalidValue("abc".into()).to_string(), "值无效: abc");

    // 测试 #[from] 自动生成 From
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
    let app_err: TestError = io_err.into();  // From<std::io::Error>
    assert!(app_err.to_string().contains("IO错误"));

    // 测试 Error::source() 由 #[from] 自动实现
    if let Some(source) = app_err.source() {
        assert!(source.to_string().contains("文件不存在"));
    } else {
        unreachable!("应存在 source");
    }

    // ConfigMissing 没有 source
    assert!(TestError::ConfigMissing.source().is_none());
}

// ==========================================================================
// 新增测试: anyhow bail! 宏
// ==========================================================================

#[test]
/// 测试: anyhow bail! / anyhow! / ensure! 宏
fn test_anyhow_bail() {
    use anyhow::{anyhow, bail, ensure, Context, Result};

    // bail! 宏: 创建错误并立即返回
    fn check_even(n: i32) -> Result<()> {
        if n % 2 != 0 {
            bail!("数字 {} 不是偶数", n);
        }
        Ok(())
    }
    assert!(check_even(4).is_ok());
    let err = check_even(3).unwrap_err();
    assert!(err.to_string().contains("3"));
    assert!(err.to_string().contains("不是偶数"));

    // anyhow! 宏: 创建错误对象 (不返回)
    let err = anyhow!("手动创建的错误: {}", 42);
    assert!(err.to_string().contains("手动创建"));

    // ensure! 宏: 条件为 false 时 bail
    fn validate_name(name: &str) -> Result<()> {
        ensure!(!name.is_empty(), "名称不能为空");
        ensure!(name.len() <= 10, "名称过长: {} 字符", name.len());
        Ok(())
    }
    assert!(validate_name("Alice").is_ok());
    assert!(validate_name("").is_err());
    assert!(validate_name("VeryLongNameHere").is_err());

    // .context: 给错误附加可读上下文
    let result: Result<()> = Err(anyhow!("底层错误"))
        .context("操作失败");
    let err = result.unwrap_err();
    // 应包含 "操作失败" (上下文) 和 "底层错误" (原因)
    let msg = format!("{:#}", err);
    assert!(msg.contains("操作失败"));
    assert!(msg.contains("底层错误"));

    // .with_context: 惰性版本
    let expensive = false;
    let result: Result<()> = Err(anyhow!("底层"))
        .with_context(|| if expensive { "贵的" } else { "便宜的" });
    assert!(result.unwrap_err().to_string().contains("便宜的"));
}

// ==========================================================================
// 新增测试: 错误链 Error::source 遍历
// ==========================================================================

#[test]
/// 测试: 错误链 Error::source 遍历 (标准 Error trait / thiserror / anyhow)
fn test_error_chain_traversal() {
    use std::error::Error;
    use thiserror::Error as ThisError;

    // 定义带 source 链的 thiserror 错误
    #[derive(ThisError, Debug)]
    enum Level3Error {
        #[error("最底层: {0}")]
        Root(String),
    }

    #[derive(ThisError, Debug)]
    enum Level2Error {
        #[error("中间层")]
        Mid(#[from] Level3Error),
    }

    #[derive(ThisError, Debug)]
    enum Level1Error {
        #[error("最上层")]
        Top(#[from] Level2Error),
    }

    // 构建链: Level3 → Level2 → Level1
    let root = Level3Error::Root("root cause".into());
    let mid: Level2Error = root.into();
    let top: Level1Error = mid.into();

    // 逐级遍历 source()
    let mut chain = Vec::new();
    let mut current: Option<&dyn Error> = Some(&top);
    while let Some(err) = current {
        chain.push(err.to_string());
        current = err.source();
    }
    assert_eq!(chain, vec!["最上层", "中间层", "最底层: root cause"]);

    // anyhow 的链遍历
    use anyhow::anyhow;
    let inner = anyhow!("底层原因");
    let mid: anyhow::Error = inner.context("中间上下文");
    let err: anyhow::Error = mid.context("顶层错误");
    let msg = format!("{:#}", err);
    assert!(msg.contains("顶层错误"));
    assert!(msg.contains("中间上下文"));
    assert!(msg.contains("底层原因"));
}

// ==========================================================================
// 新增测试: ? 的 From 转换链 (多类型错误自动转换)
// ==========================================================================

#[test]
/// 测试: ? 操作符的 From 转换链 —— 不同类型错误自动转换为统一错误类型
fn test_question_mark_conversion() {
    use std::num::ParseIntError;

    // 定义统一错误类型, 为每种底层错误实现 From
    #[derive(Debug, PartialEq)]
    enum AppError {
        Io(String),
        Parse(String),
    }

    impl From<std::io::Error> for AppError {
        fn from(e: std::io::Error) -> Self {
            AppError::Io(e.to_string())
        }
    }

    impl From<ParseIntError> for AppError {
        fn from(e: ParseIntError) -> Self {
            AppError::Parse(e.to_string())
        }
    }

    // 场景: 读取文件 + 解析数字, 两种错误类型通过 ? 自动转为 AppError
    fn read_number(path: &str) -> Result<i32, AppError> {
        // 实际读取临时文件来触发 io::Error 路径
        let content = std::fs::read_to_string(path)?;      // io::Error → AppError::Io
        let n: i32 = content.trim().parse()?;               // ParseIntError → AppError::Parse
        Ok(n * 2)
    }

    // 创建一个临时文件写入数字
    let mut tmp = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut tmp, b"21").unwrap();
    let path = tmp.path().to_str().unwrap();

    assert_eq!(read_number(path).unwrap(), 42);

    // 测试解析错误: 写入非数字内容
    let mut tmp2 = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut tmp2, b"not_a_number").unwrap();
    let path2 = tmp2.path().to_str().unwrap();

    let result = read_number(path2);
    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::Parse(_) => {} // 应匹配 Parse 变体
        _ => unreachable!("应为 Parse 错误"),
    }
}
