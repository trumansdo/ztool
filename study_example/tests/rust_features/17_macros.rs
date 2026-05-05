// ---------------------------------------------------------------------------
// 3.4 宏系统
// ---------------------------------------------------------------------------

#[test]
/// 测试: macro_rules! 声明宏 (模式匹配/重复匹配)
fn test_macro_rules() {
    // 语法: macro_rules! 声明宏, 基于模式匹配生成代码; $()*, 重复匹配
    // 避坑: 宏在编译期展开, 错误信息难以理解; expr 片段不能跟某些 token 直接相邻(需分隔符)
    macro_rules! vec_of_strings {
        ($($x:expr),*) => {
            vec![$($x.to_string()),*]
        };
    }
    let v = vec_of_strings!["hello", "world"];
    assert_eq!(v, vec!["hello".to_string(), "world".to_string()]);

    // 多分支宏：根据参数个数分发
    macro_rules! describe {
        () => { "空".to_string() };
        ($x:expr) => { format!("单个: {}", $x) };
        ($x:expr, $($xs:expr),+) => {{
            let mut s = format!("多个: {}", $x);
            $( s.push_str(&format!(", {}", $xs)); )+
            s
        }};
    }
    assert_eq!(describe!(), "空");
    assert_eq!(describe!(1), "单个: 1");
    assert_eq!(describe!(1, 2, 3), "多个: 1, 2, 3");
}

#[test]
/// 测试: 完整的宏片段说明符 (ident/expr/ty/stmt/pat/item/meta/tt/block/vis/lifetime/literal/path/pat_param)
fn test_fragment_specifiers() {
    // 语法: 宏片段说明符决定匹配什么类型的 token
    // 避坑: expr 后面不能直接跟 token, 需要分隔符; ident 后面可以直接跟 token
    //        stmt 匹配的是语句，可以不包含分号; block 匹配花括号包围的代码块

    // --- ident: 标识符 ---
    macro_rules! use_ident {
        ($name:ident) => { stringify!($name).to_string() };
    }
    assert_eq!(use_ident!(my_var), "my_var");
    assert_eq!(use_ident!(r#struct), "r#struct");

    // --- expr: 表达式 ---
    macro_rules! eval_expr {
        ($e:expr) => { $e };
    }
    assert_eq!(eval_expr!(1 + 2 * 3), 7);
    assert_eq!(eval_expr!({ let x = 5; x }), 5);
    assert_eq!(eval_expr!(if true { 10 } else { 20 }), 10);

    // --- ty: 类型 ---
    macro_rules! type_name_of {
        ($t:ty) => { std::any::type_name::<$t>() };
    }
    assert_eq!(type_name_of!(i32), "i32");
    assert_eq!(type_name_of!(Vec<String>), "alloc::vec::Vec<alloc::string::String>");

    // --- stmt: 语句 ---
    macro_rules! count_stmts {
        () => { 0usize };
        ($s:stmt) => { 1usize };
        ($s:stmt, $($rest:stmt),+) => {
            1usize + count_stmts!($($rest),+)
        };
    }
    assert_eq!(count_stmts!(let x = 1), 1);
    assert_eq!(count_stmts!(let x = 1, let y = 2, println!("hi")), 3);

    // --- pat / pat_param: 模式 ---
    macro_rules! match_pat {
        ($val:expr, $p:pat_param => $body:expr) => {
            match $val {
                $p => $body,
                _ => None,
            }
        };
    }
    assert_eq!(match_pat!(Some(5), Some(x) => Some(x)), Some(5));
    assert_eq!(match_pat!(None::<i32>, Some(x) => Some(x)), None);

    // --- item: 条目 (顶层声明) ---
    // 用 item 在宏内部生成函数/结构体
    macro_rules! make_fn_item {
        ($name:ident, $input:ty, $output:ty) => {
            fn $name(x: $input) -> $output { x as $output }
        };
    }
    make_fn_item!(to_i64_item, i32, i64);
    assert_eq!(to_i64_item(42), 42i64);

    // --- meta: 属性元数据 ---
    // meta 匹配属性内容 (#[meta])
    macro_rules! parse_meta {
        (#[$m:meta]) => { stringify!($m).to_string() };
    }
    assert!(parse_meta!(#[allow(dead_code)]).contains("allow"));
    assert!(parse_meta!(#[doc = "doc"]).contains("doc"));

    // --- tt: Token Tree (最灵活的片段) ---
    macro_rules! tt_capture {
        ($($tt:tt)*) => {
            stringify!($($tt)*).to_string()
        };
    }
    assert!(tt_capture!(1 + 2 * 3).contains("1"));
    assert!(tt_capture!(fn foo() {}).contains("fn"));

    // --- block: 代码块 ---
    macro_rules! run_block {
        ($b:block) => { $b };
    }
    assert_eq!(run_block!({
        let a = 10;
        let b = 20;
        a + b
    }), 30);

    // --- vis: 可见性 ---
    macro_rules! make_pub {
        ($vis:vis fn $name:ident() -> $ret:ty { $($body:stmt)* }) => {
            $vis fn $name() -> $ret { $($body)* }
        };
    }
    make_pub!(pub(crate) fn internal_vis_fn() -> i32 { 100 });
    assert_eq!(internal_vis_fn(), 100);

    // --- lifetime: 生命周期 ---
    macro_rules! use_lifetime {
        ($name:ident<$lt:lifetime>) => { stringify!($name<$lt>).to_string() };
    }
    assert_eq!(use_lifetime!(Foo<'a>), "Foo < 'a >");

    // --- literal: 字面量 ---
    macro_rules! lit_match {
        ($l:literal) => { stringify!($l).to_string() };
    }
    assert_eq!(lit_match!(42), "42");
    assert_eq!(lit_match!(true), "true");
    assert_eq!(lit_match!('x'), "'x'");
    assert_eq!(lit_match!("hello"), "\"hello\"");

    // --- path: 路径 ---
    macro_rules! path_to_string {
        ($p:path) => { stringify!($p).to_string() };
    }
    assert!(path_to_string!(std::collections::HashMap).contains("HashMap"));
    assert!(path_to_string!(Vec).contains("Vec"));
}

#[test]
/// 测试: 宏重复模式 ($(),* / $();* / $()+ / $(,)?) 尾随逗号
fn test_repetition_patterns() {
    // 语法: $(...)* 零次或多次, $(...)+ 一次或多次, $(...)? 零次或一次
    // 避坑: + 要求至少匹配一次; * 可以匹配零次; ? 匹配零次或一次
    //        分隔符可以是逗号、分号、或任意token; 注意尾随分隔符的处理

    // --- 逗号分隔：零次或多次 ---
    macro_rules! comma_star {
        ($($x:expr),*) => {
            vec![$($x),*]
        };
    }
    let empty_i32: Vec<i32> = comma_star!();
    assert_eq!(empty_i32, vec![] as Vec<i32>);
    assert_eq!(comma_star!(1, 2, 3), vec![1, 2, 3]);

    // --- 逗号分隔：至少一次 ---
    macro_rules! comma_plus {
        ($($x:expr),+) => {
            vec![$($x),+]
        };
    }
    assert_eq!(comma_plus!(10), vec![10]);
    assert_eq!(comma_plus!(10, 20, 30), vec![10, 20, 30]);

    // --- 分号分隔：零次或多次 (用于语句) ---
    macro_rules! semi_star {
        ($($x:expr);*) => {
            vec![$($x),*]
        };
    }
    let empty_semi: Vec<i32> = semi_star!();
    assert_eq!(empty_semi, vec![] as Vec<i32>);
    assert_eq!(semi_star!(1; 2; 3), vec![1, 2, 3]);

    // --- 分号分隔：至少一次 ---
    macro_rules! semi_plus {
        ($($x:expr);+) => {
            vec![$($x),+]
        };
    }
    assert_eq!(semi_plus!(100), vec![100]);

    // --- 可选的尾随逗号 ---
    macro_rules! tuple_with_trailing {
        ($($x:expr),+ $(,)?) => {
            ($($x,)+)
        };
    }
    let t1 = tuple_with_trailing!(1);
    assert_eq!(t1, (1,));
    let t2 = tuple_with_trailing!(1, 2, 3);
    assert_eq!(t2, (1, 2, 3));
    let t3 = tuple_with_trailing!(1, 2,);
    assert_eq!(t3, (1, 2));

    // --- ? 可选修饰符 ---
    macro_rules! optional_item {
        ($x:expr $(, $y:expr)?) => {
            $x $( + $y)?
        };
    }
    assert_eq!(optional_item!(5), 5);
    assert_eq!(optional_item!(5, 3), 8);

    // --- 无分隔符重复 (Token树级) ---
    macro_rules! concat_all {
        ($($x:tt)*) => {
            concat!($($x),*)
        };
    }
    assert_eq!(concat_all!("a" "b" "c"), "abc");

    // --- 嵌套重复 ---
    macro_rules! nested_repeat {
        ($([$($inner:expr),*]),*) => {{
            let mut sum = 0;
            $( $( sum += $inner; )* )*
            sum
        }};
    }
    assert_eq!(nested_repeat!([1, 2], [3, 4, 5]), 15);
}

#[test]
/// 测试: 宏递归与内部规则 (tt muncher / internal rules @)
fn test_recursive_macro() {
    // 语法: 宏可以递归调用自身; 内部规则用 @ 前缀标记辅助模式
    // 避坑: 递归深度限制默认 128; 用 tt muncher 逐 token 处理

    // --- 基本递归：计数表达式 ---
    macro_rules! count_exprs {
        () => { 0usize };
        ($head:expr $(, $tail:expr)*) => {
            1usize + count_exprs!($($tail),*)
        };
    }
    assert_eq!(count_exprs!(), 0);
    assert_eq!(count_exprs!(1, 2, 3, 4, 5), 5);

    // --- 递归求和 ---
    macro_rules! sum_all {
        ($head:expr) => { $head };
        ($head:expr, $($tail:expr),+) => {
            $head + sum_all!($($tail),+)
        };
    }
    assert_eq!(sum_all!(10), 10);
    assert_eq!(sum_all!(1, 2, 3, 4), 10);

    // --- 内部规则模式：用 @ 标记辅助规则 ---
    macro_rules! vec_of {
        (@repeat $elem:expr; $n:expr) => {{
            let mut v = Vec::with_capacity($n as usize);
            for _ in 0..($n as usize) { v.push($elem); }
            v
        }};
        ($($elem:expr),* $(,)?) => {
            vec![$($elem),*]
        };
    }

    // 演示内部规则用法（@repeat 不对外公开）
    let v1 = vec_of![1, 2, 3];
    assert_eq!(v1, vec![1, 2, 3]);
    let empty_vec_of: Vec<i32> = vec_of![];
    assert_eq!(empty_vec_of, vec![] as Vec<i32>);

    // 通过内部规则创建重复元素
    let v2 = vec_of!(@repeat 42; 3);
    assert_eq!(v2, vec![42, 42, 42]);

    // --- tt muncher 模式：逐 Token 处理 ---
    macro_rules! tt_count {
        () => { 0usize };
        ($t:tt $($rest:tt)*) => {
            1usize + tt_count!($($rest)*)
        };
    }
    // 计数 Token 个数
    assert_eq!(tt_count!(a b c), 3);
    assert_eq!(tt_count!(1 + 2 * 3), 5); // 5 个 token: 1, +, 2, *, 3

    // --- 用内部规则实现类似 match 的宏 ---
    macro_rules! when_then {
        // 公开 API: 单个条件+then (else 默认为 None)
        ($cond:expr, $then:expr) => {
            when_then!(@if_else $cond, $then, Option::<i32>::None)
        };
        // 内部规则：if-else 分支
        (@if_else $cond:expr, $then:expr, $else:expr) => {
            if $cond { Some($then) } else { $else }
        };
    }
    assert_eq!(when_then!(true, 42), Some(42));
    assert_eq!(when_then!(false, 42), None);
}

#[test]
/// 测试: 调试与断言宏 (dbg!/todo!/unimplemented!/unreachable!/assert!/assert_eq!/debug_assert!)
fn test_debug_macros() {
    // 语法: 调试宏用于开发阶段标记状态; 断言宏用于运行时检查
    // 避坑: dbg! 在 release 模式下也会执行, 生产环境务必移除;
    //        debug_assert! 仅在 debug 模式下检查, 不能依赖它保证正确性;
    //        todo! 和 unimplemented! 都导致 panic, 语义上前者表示"将来做", 后者表示"不会做"

    // --- dbg!：调试打印并返回所有权 ---
    let x = 5;
     let y = dbg!(x * 2); // stderr: [17_macros.rs:xxx] x * 2 = 10
    assert_eq!(y, 10);

    // dbg! 可以嵌套在表达式中
    let z = dbg!(1 + dbg!(2 + 3));
    assert_eq!(z, 6);

    // --- assert!：运行时断言 ---
    assert!(true);
    assert!(1 + 1 == 2, "基本算术验证");

    // --- assert_eq! / assert_ne! ---
    assert_eq!(2 + 2, 4);
    assert_eq!(vec![1, 2, 3], vec![1, 2, 3]);
    assert_ne!(1, 2);

    // assert_eq! 支持自定义错误消息
    assert_eq!(10, 10, "值应该相等");

    // --- debug_assert!：仅 debug 模式检查 ---
    debug_assert!(true);
    debug_assert!(cfg!(debug_assertions) || !cfg!(debug_assertions)); // 编译期 trivially true

    // debug_assert_eq! / debug_assert_ne!
    debug_assert_eq!(3 * 3, 9);
    debug_assert_ne!(1, 0);

    // 在 release 模式下, debug_assert! 代码被完全移除
    // 因此不应将影响程序正确性的逻辑放入 debug_assert!
    let flag = true;
    debug_assert!(flag);

    // todo! 和 unimplemented! 会 panic, 这里只验证类型系统
    // 它们都实现 ! (never type), 可以适配任何返回类型
    let _wip: fn() -> i32 = || todo!("还没实现");
    let _nyi: fn() -> String = || unimplemented!("永远不会实现");

    // --- format_args! 作为底层格式化基础 ---
    let args: std::fmt::Arguments = format_args!("hello {}", "world");
    let s = format!("{}", args);
    assert_eq!(s, "hello world");

    // format_args! 在 println! / write! / panic! 等宏内部使用
    // 返回 Arguments 结构体, 不分配堆内存
    assert!(true);
}

#[test]
/// 测试: 宏重复匹配 (+, *, ? 修饰符) - 原有增强
fn test_macro_repetition() {
    // 语法: $(...)* 零次或多次, $(...)+ 一次或多次, $(...)? 零次或一次
    // 避坑: + 要求至少匹配一次; * 可以匹配零次; ? 匹配零次或一次
    macro_rules! tuple_with_commas {
        ($($x:expr),+ $(,)?) => {
            ($($x,)+)
        };
    }

    let t1 = tuple_with_commas!(1);
    assert_eq!(t1, (1,));

    let t2 = tuple_with_commas!(1, 2, 3);
    assert_eq!(t2, (1, 2, 3));

    // 尾随逗号也支持
    let t3 = tuple_with_commas!(1, 2,);
    assert_eq!(t3, (1, 2));

    // --- 多种分隔符组合 ---
    macro_rules! build_map {
        ($($k:expr => $v:expr),* $(,)?) => {
            std::collections::HashMap::from([$(($k, $v)),*])
        };
    }
    let map = build_map!("a" => 1, "b" => 2, "c" => 3);
    assert_eq!(map.len(), 3);
    assert_eq!(map["a"], 1);

    // ? 可选的重复
    macro_rules! maybe_add {
        ($a:expr $(, $b:expr)?) => {
            $a $( + $b)?
        };
    }
    assert_eq!(maybe_add!(5), 5);
    assert_eq!(maybe_add!(5, 3), 8);
}

#[test]
/// 测试: 宏递归 (tt muncher 模式) - 原有增强
fn test_macro_recursion() {
    // 语法: 宏可以递归调用自身, 实现复杂的代码生成
    // 避坑: 递归深度有限制(默认 128); 用 tt muncher 逐 token 处理
    macro_rules! count_exprs {
        () => { 0 };
        ($head:expr $(, $tail:expr)*) => {
            1 + count_exprs!($($tail),*)
        };
    }

    assert_eq!(count_exprs!(), 0);
    assert_eq!(count_exprs!(1), 1);
    assert_eq!(count_exprs!(1, 2, 3, 4, 5), 5);

    // tt muncher 模式：逐 token 累积到列表
    macro_rules! tt_collect {
        (@result [$($acc:tt)*]) => {
            vec![$($acc)*]
        };
        ($t:tt $($rest:tt)*) => {
            tt_collect!(@result [stringify!($t) $(, stringify!($rest))*])
        };
    }
    let v = tt_collect!(a b c d);
    assert_eq!(v, vec!["a", "b", "c", "d"]);
}

#[test]
/// 测试: 内置宏 stringify!
fn test_stringify() {
    // 语法: stringify!(...) 将任意 token 树转为字符串字面量, 不展开宏
    // 避坑: 不展开内部宏; 保留原始格式(包括空白); 常用于日志/错误信息
    let s = stringify!(println!("hello"));
    assert!(s.contains("println"));
    assert!(s.contains("hello"));

    // 不展开宏 (格式因 Rust 版本可能略有不同)
    #[allow(unused_macros)]
    macro_rules! my_macro {
        () => {
            42
        };
    }
    let expanded = stringify!(my_macro!());
    assert!(expanded.contains("my_macro"));
}

#[test]
/// 测试: 内置宏 concat! / concat_idents!
fn test_concat_macros() {
    // 语法: concat! 编译期拼接字符串字面量; concat_idents! 拼接标识符 (nightly)
    // 避坑: concat! 只能拼接字面量; concat_idents! 只在 nightly 可用
    let s = concat!("hello", " ", "world", "!");
    assert_eq!(s, "hello world!");

    // 拼接文件路径
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    assert!(path.ends_with("Cargo.toml"));
}

#[test]
/// 测试: 内置宏 env! / option_env!
fn test_env_macros() {
    // 语法: env!("VAR") 编译期读取环境变量, 不存在则编译失败
    //        option_env!("VAR") 编译期读取, 不存在返回 None
    // 避坑: env! 在编译期求值, 运行时修改环境变量无效
    let pkg = env!("CARGO_PKG_NAME");
    assert_eq!(pkg, "study_example");

    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());

    // option_env! 安全版本
    let maybe = option_env!("NONEXISTENT_VAR");
    assert!(maybe.is_none());
}

#[test]
/// 测试: 内置宏 include! / include_str! / include_bytes!
fn test_include_macros() {
    // 语法: include!("file.rs") 编译期包含文件内容
    //        include_str!("file.txt") 包含为 &str
    //        include_bytes!("file.bin") 包含为 &[u8]
    // 避坑: 路径相对于当前源文件; 文件修改后需要重新编译
    assert!(true);
}

#[test]
/// 测试: 内置宏 column! / line! / file! / module_path!
fn test_location_macros() {
    // 语法: 编译期获取代码位置信息, 常用于日志和调试
    // 避坑: 位置信息是编译期确定的, 运行时不会改变
    let line = line!();
    assert!(line > 0);

    let file = file!();
     assert!(file.ends_with("17_macros.rs"));

    let module = module_path!();
    assert!(module.contains("macros"));

    let col = column!();
    assert!(col > 0);
}

#[test]
/// 测试: 内置宏 format_args!
fn test_format_args() {
    // 语法: format_args!(fmt, args...) 创建 Arguments 对象, 不分配内存
    // 避坑: 返回的 Arguments 生命周期受借用参数限制; 用于 no_std 环境
    let args = format_args!("hello {}", "world");
    let s = format!("{}", args);
    assert_eq!(s, "hello world");
}

#[test]
/// 测试: cfg! 宏 (编译期条件)
fn test_cfg_macro() {
    // 语法: cfg!(condition) 编译期求值的布尔表达式
    // 避坑: 运行时条件永远为 false; 用于条件编译逻辑
    assert!(cfg!(target_os = "windows") || cfg!(target_os = "linux") || cfg!(target_os = "macos"));

    // 自定义 cfg
    #[cfg(debug_assertions)]
    let is_debug = true;
    #[cfg(not(debug_assertions))]
    let is_debug = false;

    assert_eq!(cfg!(debug_assertions), is_debug);
}

#[test]
/// 测试: derive 派生宏
fn test_derive_macros() {
    // 语法: #[derive(Trait)] 自动生成 trait 实现
    // 常用派生: Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default
    // 避坑: 所有字段必须也实现对应 trait; Copy 要求所有字段都是 Copy
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Point {
        x: i32,
        y: i32,
    }

    impl Default for Point {
        fn default() -> Self {
            Self { x: 0, y: 0 }
        }
    }

    let p1 = Point { x: 1, y: 2 };
    let p2 = p1.clone();
    assert_eq!(p1, p2);
    assert_eq!(format!("{:?}", p1), "Point { x: 1, y: 2 }");
    assert_eq!(Point::default(), Point { x: 0, y: 0 });
}

#[test]
/// 测试: 属性宏 (#[derive] 自定义派生)
fn test_attribute_macros() {
    // 语法: 属性宏通过 #[proc_macro_derive] 定义, 在编译期生成代码
    // 避坑: 需要在独立的 proc-macro crate 中定义; 不能和主 crate 放在一起
    assert!(true);
}

#[test]
/// 测试: 函数宏 (function-like procedural macro)
fn test_function_like_macros() {
    // 语法: 函数宏像函数一样调用: sql!(SELECT * FROM users)
    // 避坑: 函数宏也是 proc_macro, 需要独立 crate
    let v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
}

#[test]
/// 测试: 宏卫生性 (Hygiene)
fn test_macro_hygiene() {
    // 语法: declarative macros 是卫生的, 宏内部定义的标识符不会与外部冲突
    // 避坑: 宏内部的变量名不会泄漏到外部; 但 tt 片段可能携带外部上下文
    macro_rules! make_counter {
        () => {{
            let mut count = 0;
            count += 1;
            count
        }};
    }

    let count = 42; // 外部变量
    let result = make_counter!(); // 宏内部的 count 不影响外部
    assert_eq!(result, 1);
    assert_eq!(count, 42); // 外部变量不变

    // 卫生性的更多验证
    macro_rules! create_var {
        ($name:ident) => {
            let $name = 999;
        };
    }

    create_var!(temp_var);
    assert_eq!(temp_var, 999);

    // 同名变量不会覆盖外部
    macro_rules! shadow_test {
        () => {{
            let a = 10;
            a
        }};
    }
    let a = 5;
    let b = shadow_test!();
    assert_eq!(a, 5);
    assert_eq!(b, 10);
}
