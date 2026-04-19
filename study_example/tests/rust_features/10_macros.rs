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
}

#[test]
/// 测试: 宏片段说明符 (expr/ident/ty/path/lifetime)
fn test_fragment_specifiers() {
    // 语法: 宏片段说明符决定匹配什么类型的 token
    //   - $e:expr    表达式
    //   - $i:ident   标识符 (变量名/函数名)
    //   - $t:ty      类型
    //   - $p:path    路径 (std::collections::HashMap)
    //   - $l:lifetime 生命周期 ('a)
    //   - $tt:tt     任意单个 token tree
    //   - $s:stmt    语句
    //   - $pat:pat   模式
    //   - $meta:meta 元数据属性
    //   - $block:block 代码块
    // 避坑: expr 后面不能直接跟 token, 需要分隔符; ident 后面可以直接跟 token
    macro_rules! make_fn {
        ($name:ident, $input:ty, $output:ty) => {
            fn $name(x: $input) -> $output {
                x as $output
            }
        };
    }

    make_fn!(to_i64, i32, i64);
    assert_eq!(to_i64(42), 42i64);
}

#[test]
/// 测试: 宏重复匹配 (+, *, ? 修饰符)
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
}

#[test]
/// 测试: 宏递归 (tt muncher 模式)
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
    assert_eq!(pkg, "ztool");

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
    // 这里只演示 include_str! 的概念
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
    assert!(file.ends_with("10_macros.rs"));

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
    // 这里演示标准库的 serde-like 概念
    assert!(true);
}

#[test]
/// 测试: 函数宏 (function-like procedural macro)
fn test_function_like_macros() {
    // 语法: 函数宏像函数一样调用: sql!(SELECT * FROM users)
    // 避坑: 函数宏也是 proc_macro, 需要独立 crate
    // 标准库示例: format!, vec!, println! 等
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
}
