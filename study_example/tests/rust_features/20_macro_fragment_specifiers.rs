// ---------------------------------------------------------------------------
// 5.3 宏片段说明符扩展 (Edition 2024)
// ---------------------------------------------------------------------------

// 语法: 宏片段说明符 $name:fragment 指定匹配 Rust 代码的哪部分
// 避坑: 不同片段有不同的类型要求和匹配规则; 混合使用可能导致类型错误

macro_rules! test_expr_macro {
    ($e:expr) => {
        $e
    };
}

#[test]
/// 测试: Edition 2024 宏 expr 片段匹配 const 块
fn test_macro_expr_matches_const() {
    // 语法: Edition 2024 中 expr 也能匹配 const 块和 _ 表达式
    // 避坑: 旧版 expr 不匹配 const { ... }; 迁移后某些宏可能匹配到更多内容
    let val = test_expr_macro!({ 42 });
    assert_eq!(val, 42);
}

macro_rules! capture_ident {
    ($i:ident) => {
        1
    };
}

#[test]
/// 测试: 宏 ident 片段匹配标识符
fn test_macro_ident_fragment() {
    // 语法: :ident 匹配标识符 (函数名, 变量名, 类型名等)
    // 避坑: ident 不匹配关键字(除非原始), 不匹配数字
    let _ = capture_ident!(my_variable);
    assert!(true);
}

macro_rules! capture_ty {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

#[test]
/// 测试: 宏 ty 片段匹配类型
fn test_macro_ty_fragment() {
    // 语法: :ty 匹配类型 (如 i32, Vec<String>, &str)
    // 避坑: ty 匹配的是类型语法, 不是具体类型值
    let size = capture_ty!(i32);
    assert_eq!(size, 4);
}

macro_rules! capture_path {
    ($p:path) => {
        1
    };
}

#[test]
/// 测试: 宏 path 片段匹配路径
fn test_macro_path_fragment() {
    // 语法: :path 匹配路径 (如 std::collections::HashMap)
    // 避坑: path 包含类型参数时可能需要额外处理
    let _ = capture_path!(std::collections::HashMap<String, i32>);
    assert!(true);
}

macro_rules! capture_pat {
    ($p:pat_param) => {
        1
    };
}

#[test]
/// 测试: 宏 pat_param 片段匹配模式
fn test_macro_pat_fragment() {
    // 语法: :pat_param 匹配模式 (如 Some(x), _, 1..=5)
    // 避坑: pat_param 匹配可能产生歧义, 某些模式需要完整解析
    let x = 5;
    let _ = match_guard(x, 5);
    assert!(true);
}

fn match_guard(val: i32, expected: i32) -> bool {
    val == expected
}

macro_rules! capture_lit {
    ($l:literal) => {
        1
    };
}

#[test]
/// 测试: 宏 literal 片段匹配字面量
fn test_macro_literal_fragment() {
    // 语法: :literal 匹配字面量 (数字, 字符串, 字符等)
    // 避坑: literal 匹配的是字面量语法, 不是值
    let _ = capture_lit!(42);
    let _ = capture_lit!("hello");
    assert!(true);
}

macro_rules! capture_block {
    ($b:block) => {
        1
    };
}

#[test]
/// 测试: 宏 block 片段匹配代码块
fn test_macro_block_fragment() {
    // 语法: :block 匹配花括号包围的代码块 { ... }
    // 避坑: block 不匹配表达式, 只匹配语句块
    let _ = capture_block!({
        let x = 1;
        x
    });
    assert!(true);
}

macro_rules! capture_tt {
    ($($t:tt)*) => {
        1
    };
}

#[test]
/// 测试: 宏 tt 片段匹配 token tree
fn test_macro_tt_fragment() {
    // 语法: :tt (token tree) 是最灵活的片段, 可匹配任意 token 序列
    // 避坑: tt 过于灵活, 可能导致宏递归展开问题
    let _ = capture_tt!(1 + 2);
    assert!(true);
}
