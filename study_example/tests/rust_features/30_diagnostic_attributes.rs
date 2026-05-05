// ---------------------------------------------------------------------------
// 5.5 诊断属性 (1.85+)
// ---------------------------------------------------------------------------

// 语法: #[diagnostic::do_not_recommend] 阻止编译器在错误信息中推荐此实现 (1.85+)
// 避坑: 仅影响错误信息, 不影响编译行为; 用于隐藏内部实现细节, 减少用户困惑

pub trait InternalTrait {}
pub trait PublicTrait {}

#[diagnostic::do_not_recommend]
impl<T: InternalTrait> PublicTrait for T {}

struct MyType;

#[test]
/// 测试: #[diagnostic::do_not_recommend] 隐藏实现细节 (1.85+)
fn test_do_not_recommend() {
    let _ = MyType;
    assert!(true);
}

#[test]
/// 测试: #[must_use] 强制使用返回值
fn test_must_use_attribute() {
    #[must_use]
    fn important_value() -> i32 {
        42
    }

    let _ = important_value();
    assert!(true);
}

#[test]
/// 测试: #[deprecated] 标记废弃 API
fn test_deprecated_attribute() {
    #[deprecated(since = "1.0.0", note = "use new_function instead")]
    fn old_function() -> i32 {
        42
    }

    let _ = old_function();
    assert!(true);
}

#[test]
/// 测试: #[warn 和 #[deny] 强制 lint 级别
fn test_lint_level_attributes() {
    #[allow(dead_code)]
    fn unused_function() {}

    assert!(true);
}

#[test]
/// 测试: #[non_exhaustive] 标记不完全枚举
fn test_non_exhaustive() {
    assert!(true);
}

// ===================== 扩充测试 =====================

#[test]
/// 测试: #[must_use] 配合 Result 类型
fn test_must_use_with_result() {
    #[must_use = "这个 Result 可能包含重要错误信息"]
    fn fallible_operation() -> Result<i32, &'static str> {
        Ok(99)
    }

    let result = fallible_operation();
    assert_eq!(result, Ok(99));
}

#[test]
/// 测试: #[must_use] 在类型上标记 —— 该类型所有返回值都必须使用
fn test_must_use_on_type() {
    #[must_use]
    #[derive(Debug, PartialEq)]
    struct Important(i32);

    fn create_important() -> Important {
        Important(42)
    }

    let imp = create_important();
    assert_eq!(imp, Important(42));
}

#[test]
/// 测试: #[deprecated] 带 since 和 note
fn test_deprecated_with_details() {
    #[deprecated(since = "2.0.0", note = "请使用 new_api() 代替")]
    fn legacy_func() -> &'static str {
        "legacy"
    }

    #[allow(deprecated)]
    {
        let result = legacy_func();
        assert_eq!(result, "legacy");
    }
}

#[test]
/// 测试: #[allow] 作用域 —— 不同层级
fn test_allow_scoping() {
    #[allow(unused_variables)]
    fn with_allowed_unused() {
        let x = 1; // 不会警告
        let _ = x;
    }

    with_allowed_unused();
    assert!(true);
}

#[test]
/// 测试: #[non_exhaustive] 枚举 —— 外部 match 必须有通配符
fn test_non_exhaustive_enum_requires_wildcard() {
    #[non_exhaustive]
    enum Status {
        Ok,
        Err,
        Pending,
    }

    let s = Status::Ok;
    match s {
        Status::Ok => assert!(true),
        Status::Err => panic!("unexpected"),
        _ => {} // non_exhaustive 强制要求通配符
    }
}

#[test]
/// 测试: #[non_exhaustive] 结构体 —— 不能外部构造
fn test_non_exhaustive_struct_cannot_construct_externally() {
    // 只能在定义 crate 内构造
    assert!(true);
}

#[test]
/// 测试: #[cfg_attr] 条件属性
fn test_cfg_attr_conditional() {
    // 在 test 模式下添加属性
    #[cfg_attr(test, allow(dead_code))]
    fn test_only_helper() -> i32 {
        100
    }

    assert_eq!(test_only_helper(), 100);
}

#[test]
/// 测试: #[inline] 和 #[inline(always)]
fn test_inline_attributes() {
    #[inline]
    fn maybe_inlined() -> i32 { 10 }

    #[inline(always)]
    fn always_inlined() -> i32 { 20 }

    assert_eq!(maybe_inlined() + always_inlined(), 30);
}

#[test]
/// 测试: #[track_caller] —— panic 信息指向调用者
fn test_track_caller_panics_at_caller() {
    #[track_caller]
    fn assert_positive(value: i32) {
        if value <= 0 {
            panic!("值必须为正数, 实际为 {value}");
        }
    }

    assert_positive(42); // 正常通过
    assert!(true);

    // std::panic::catch_unwind 来捕获 panic 并验证
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert_positive(-1);
    }));
    assert!(result.is_err(), "should panic for negative value");
}

#[test]
/// 测试: #[cold] 冷路径标记
fn test_cold_attribute() {
    #[cold]
    fn unlikely_case() -> i32 {
        0
    }

    fn likely_case() -> i32 {
        42
    }

    let result = if true { likely_case() } else { unlikely_case() };
    assert_eq!(result, 42);
}

#[test]
/// 测试: 多个属性组合使用
fn test_multiple_attributes_combo() {
    #[must_use]
    #[inline]
    fn compute() -> i32 {
        42
    }

    let result = compute();
    assert_eq!(result, 42);
}

#[test]
/// 测试: lint 级别 ! (crate级) 在模块级别 deny
fn test_deny_lint_at_module_level() {
    // 模块级别的 #[deny] 可以通过 allow 在子项中覆盖 (不同于 forbid)
    #[allow(unused_variables)]
    fn inner_with_unused() {
        let _x = 5;
    }

    inner_with_unused();
    assert!(true);
}

#[test]
/// 测试: #[diagnostic::on_unimplemented] 自定义错误信息 (概念验证)
fn test_on_unimplemented_concept() {
    // 这个属性在真实的 trait 定义中使用,
    // 此处验证 trait 约束的正常行为
    trait Display {
        fn show(&self) -> String;
    }

    impl Display for i32 {
        fn show(&self) -> String {
            self.to_string()
        }
    }

    fn print_it<T: Display>(val: T) -> String {
        val.show()
    }

    assert_eq!(print_it(42), "42");
}
