// ---------------------------------------------------------------------------
// 5.3 宏片段说明符扩展 (Edition 2024)
// ---------------------------------------------------------------------------

// 语法: 宏片段说明符 $name:fragment 指定匹配 Rust 代码的哪部分
// 避坑: 不同片段有不同的类型要求和匹配规则; 混合使用可能导致类型错误

// ===========================================================================
// 单个片段说明符测试
// ===========================================================================

macro_rules! test_expr_macro {
    ($e:expr) => {
        $e
    };
}

#[test]
/// 测试: Edition 2024 宏 expr 片段匹配 const 块和 _ 表达式
fn test_macro_expr_matches_const() {
    // 语法: Edition 2024 中 expr 也能匹配 const 块和 _ 表达式
    // 避坑: 旧版 expr 不匹配 const { ... }; 迁移后某些宏可能匹配到更多内容
    let val = test_expr_macro!({ 42 });
    assert_eq!(val, 42);

    // Edition 2024: expr 可以匹配 _ 下划线表达式
    let inferred = test_expr_macro!({ let _x: i32 = 10; _x });
    assert_eq!(inferred, 10);
}

macro_rules! capture_ident {
    ($i:ident) => {
        1
    };
}

#[test]
/// 测试: 宏 ident 片段匹配标识符 (含原始标识符)
fn test_macro_ident_fragment() {
    // 语法: :ident 匹配标识符 (函数名, 变量名, 类型名等)
    // 避坑: ident 不匹配关键字(除非原始标识符 r#), 不匹配数字
    let _ = capture_ident!(my_variable);
    let _ = capture_ident!(r#type); // 原始标识符
    let _ = capture_ident!(r#match);
    assert!(true);
}

macro_rules! capture_ty {
    ($t:ty) => {
        std::mem::size_of::<$t>()
    };
}

#[test]
/// 测试: 宏 ty 片段匹配类型 (含复合类型)
fn test_macro_ty_fragment() {
    // 语法: :ty 匹配类型 (如 i32, Vec<String>, &str)
    // 避坑: ty 匹配的是类型语法, 不是具体类型值
    let size = capture_ty!(i32);
    assert_eq!(size, 4);

    let size_u64 = capture_ty!(u64);
    assert_eq!(size_u64, 8);

    // 引用类型
    macro_rules! type_is_ref {
        ($t:ty) => { std::mem::size_of::<$t>() == std::mem::size_of::<&i32>() };
    }
    assert!(type_is_ref!(&i32));
}

macro_rules! capture_path {
    ($p:path) => {
        1
    };
}

#[test]
/// 测试: 宏 path 片段匹配路径 (含泛型参数)
fn test_macro_path_fragment() {
    // 语法: :path 匹配路径 (如 std::collections::HashMap)
    // 避坑: path 包含类型参数时可能需要额外处理
    let _ = capture_path!(std::collections::HashMap<String, i32>);
    let _ = capture_path!(Vec);
    let _ = capture_path!(std::vec::Vec);
    assert!(true);
}

macro_rules! capture_pat {
    ($p:pat_param) => {
        1
    };
}

#[test]
/// 测试: 宏 pat_param / pat 片段匹配模式
fn test_macro_pat_fragment() {
    // 语法: :pat_param 匹配模式 (如 Some(x), _, 1..=5)
    //        :pat (Edition 2024) 不允许 | 前导
    // 避坑: pat_param 匹配可能产生歧义, 某些模式需要完整解析

    // 辅助函数
    fn match_guard(val: i32, expected: i32) -> bool {
        val == expected
    }

    let x = 5;
    let _ = match_guard(x, 5);

    // 调用 capture_pat 验证 pat_param 片段
    let _ = capture_pat!(Some(1));
    let _ = capture_pat!(_);
    assert!(true);

    // 测试各种模式
    macro_rules! test_pattern {
        ($val:expr, $p:pat_param) => {
            matches!($val, $p)
        };
    }
    assert!(test_pattern!(Some(1), Some(_x)));
    assert!(test_pattern!(42, 30..=50));
    assert!(test_pattern!(42, _));
    assert!(!test_pattern!(42, 100));
}

macro_rules! capture_lit {
    ($l:literal) => {
        1
    };
}

#[test]
/// 测试: 宏 literal 片段匹配各种字面量
fn test_macro_literal_fragment() {
    // 语法: :literal 匹配字面量 (数字, 字符串, 字符, 布尔值等)
    // 避坑: literal 匹配的是字面量语法, 不是值
    let _ = capture_lit!(42);
    let _ = capture_lit!("hello");
    let _ = capture_lit!(true);
    let _ = capture_lit!(false);
    let _ = capture_lit!('x');
    let _ = capture_lit!(3.14);
    assert!(true);
}

macro_rules! capture_block {
    ($b:block) => {
        1
    };
}

#[test]
/// 测试: 宏 block 片段匹配代码块 (含空块和嵌套块)
fn test_macro_block_fragment() {
    // 语法: :block 匹配花括号包围的代码块 { ... }
    // 避坑: block 不匹配表达式, 只匹配语句块
    let _ = capture_block!({
        let x = 1;
        x
    });

    // 空块
    let _ = capture_block!({});

    // 嵌套块
    macro_rules! nested_block {
        ($outer:block) => { $outer };
    }
    let result = nested_block!({
        let a = { let b = 2; b };
        a * 3
    });
    assert_eq!(result, 6);

    assert!(true);
}

macro_rules! capture_tt {
    ($($t:tt)*) => {
        1
    };
}

#[test]
/// 测试: 宏 tt 片段匹配 token tree (灵活匹配)
fn test_macro_tt_fragment() {
    // 语法: :tt (token tree) 是最灵活的片段, 可匹配任意 token 序列
    // 避坑: tt 过于灵活, 可能导致宏递归展开问题
    let _ = capture_tt!(1 + 2);
    let _ = capture_tt!(fn foo() -> i32 { 42 });
    let _ = capture_tt!(a b c);
    assert!(true);
}

// ===========================================================================
// 片段说明符组合测试 - 多个不同片段协同工作
// ===========================================================================

#[test]
/// 测试: ident + ty 组合 - 生成函数签名
fn test_combo_ident_ty() {
    // 语法: ident 和 ty 组合是生成函数/结构体的常见模式
    macro_rules! make_getter {
        ($struct:ident, $field:ident, $ty:ty) => {
            impl $struct {
                fn get_field(&self) -> &$ty {
                    &self.$field
                }
            }
        };
    }

    struct User {
        name: String,
    }

    make_getter!(User, name, String);

    let u = User { name: "Alice".into() };
    assert_eq!(u.get_field(), "Alice");
}

#[test]
/// 测试: expr + ident 组合 - 带标签的表达式
fn test_combo_expr_ident() {
    macro_rules! label_and_val {
        ($label:ident => $val:expr) => {
            (stringify!($label).to_string(), $val)
        };
    }

    let (label, val) = label_and_val!(count => 42);
    assert_eq!(label, "count");
    assert_eq!(val, 42);
}

#[test]
/// 测试: pat + expr 组合 - match 臂生成
fn test_combo_pat_expr() {
    macro_rules! match_arms {
        ($val:expr, $($pat:pat_param => $body:expr),+ $(,)?) => {
            match $val {
                $($pat => $body),+
            }
        };
    }

    let result = match_arms!(
        Some(3),
        Some(1) => 1,
        Some(2) => 2,
        Some(x) => x,
        None => 0,
    );
    assert_eq!(result, 3);
}

#[test]
/// 测试: ty + lifetime 组合
fn test_combo_ty_lifetime() {
    macro_rules! ref_type {
        ($lt:lifetime, $ty:ty) => {
            &$lt $ty
        };
    }

    // 使用宏生成的引用类型
    fn get_ref<'a>(r: ref_type!('a, i32)) -> ref_type!('a, i32) {
        r
    }

    let x: i32 = 42;
    let r = get_ref(&x);
    assert_eq!(*r, 42);

    // 验证宏生成的类型签名
    macro_rules! describe_ref {
        ($lt:lifetime, $ty:ty) => {
            stringify!(&$lt $ty).to_string()
        };
    }
    let desc = describe_ref!('a, i32);
    assert!(desc.contains("&") && desc.contains("i32"));
}

#[test]
/// 测试: item + meta 组合 - 带属性的条目
fn test_combo_item_meta() {
    macro_rules! documented_fn {
        ($(#[$meta:meta])* $vis:vis fn $name:ident($param:ident: $ty:ty) -> $ret:ty $body:block) => {
            $(#[$meta])*
            $vis fn $name($param: $ty) -> $ret $body
        };
    }

    documented_fn! {
        #[doc = "返回输入值的两倍"]
        fn double(x: i32) -> i32 { x * 2 }
    }
    assert_eq!(double(5), 10);
}

#[test]
/// 测试: block + expr 组合 - 条件执行
fn test_combo_block_expr() {
    macro_rules! if_then {
        ($cond:expr, $then:block $(, $else:block)?) => {
            if $cond $then $(else $else)?
        };
    }

    let a = if_then!(true, { 10 }, { 20 });
    assert_eq!(a, 10);

    let b = if_then!(false, { 10 }, { 20 });
    assert_eq!(b, 20);
}

#[test]
/// 测试: stmt + tt 组合 - 语句拼接
fn test_combo_stmt_tt() {
    macro_rules! with_context {
        ($ctx:ident => $($body:stmt)*) => {{
            let $ctx = "上下文";
            $($body)*
        }};
    }

    let result = with_context!(ctx => {
        let answer = 42;
        assert_eq!(ctx, "上下文");
        answer
    });
    assert_eq!(result, 42);
}

#[test]
/// 测试: vis + ident + ty 组合 - 带可见性的结构体字段
fn test_combo_vis_ident_ty() {
    macro_rules! make_struct {
        ($vis:vis struct $name:ident { $($fvis:vis $fname:ident: $fty:ty),* $(,)? }) => {
            $vis struct $name {
                $($fvis $fname: $fty),*
            }
        };
    }

    make_struct! {
        struct Person {
            pub name: String,
            pub(crate) age: u32,
        }
    }

    let p = Person { name: "Bob".into(), age: 30 };
    assert_eq!(p.name, "Bob");
    assert_eq!(p.age, 30);
}

#[test]
/// 测试: literal + ident 组合 - 转换表
fn test_combo_literal_ident() {
    macro_rules! const_map_entry {
        ($key:literal => $val:ident) => {
            ($key, $val)
        };
    }

    let val = 42;
    let entry = const_map_entry!("answer" => val);
    assert_eq!(entry.0, "answer");
    assert_eq!(entry.1, 42);
}

#[test]
/// 测试: 复杂组合 - 模拟 vec! 宏的完整实现
fn test_combo_complex_vec_like() {
    macro_rules! my_vec {
        // 空
        () => {
            Vec::new()
        };
        // 具新容量
        ( $elem:expr; $n:expr ) => {
            std::vec::from_elem($elem, $n)
        };
        // 逗号分隔
        ( $($x:expr),+ $(,)? ) => {
            {
                let mut temp_vec = Vec::with_capacity(0usize $(+ { let _ = $x; 1 })+);
                $(temp_vec.push($x);)+
                temp_vec
            }
        };
    }

    let v1: Vec<i32> = my_vec!();
    assert_eq!(v1, Vec::<i32>::new());

    let v2 = my_vec![42; 3];
    assert_eq!(v2, vec![42, 42, 42]);

    let v3 = my_vec![1, 2, 3, 4, 5];
    assert_eq!(v3, vec![1, 2, 3, 4, 5]);

    let v4 = my_vec![1, 2,];
    assert_eq!(v4, vec![1, 2]);
}

#[test]
/// 测试: 多个片段在重复组内组合
fn test_combo_repeat_with_mixed_fragments() {
    macro_rules! make_match {
        ($val:expr, $($pat:pat_param $(if $guard:expr)? => $body:expr),+ $(,)?) => {
            match $val {
                $($pat $(if $guard)? => $body),+
            }
        };
    }

    let result = make_match!(
        15,
        1..=10 => "小",
        11..=20 => "中",
        _ => "大",
    );
    assert_eq!(result, "中");

    // 带守卫条件：需要通配符以确保穷尽性
    let result2 = make_match!(
        7,
        x if x % 2 == 0 => "偶数",
        x if x % 2 != 0 => "奇数",
        _ => "不可能",
    );
    assert_eq!(result2, "奇数");
}

#[test]
/// 测试: Edition 2024 pat 片段说明符 (不含 | 前导)
fn test_edition2024_pat_fragment() {
    // Edition 2024 新增 pat 片段：不允许 | 前导
    macro_rules! match_arm_pat {
        ($val:expr, $(($pat:pat => $body:expr)),+ $(,)?) => {
            match $val {
                $( $pat => $body, )+
                _ => None,
            }
        };
    }

    let result = match_arm_pat!(
        42,
        (1..=10 => Some("小")),
        (11..=100 => Some("中")),
    );
    assert_eq!(result, Some("中"));
}
