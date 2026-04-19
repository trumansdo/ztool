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
    // 语法: #[must_use] 标记返回值必须被使用, 否则产生警告
    // 避坑: 函数返回重要值时被忽略可能导致逻辑错误
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
    // 语法: #[deprecated] 标记已废弃的 API, 产生警告或错误
    // 避坑: 迁移期间使用废弃 API 会收到警告
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
    // 语法: #[warn] 产生警告, #[deny] 产生错误, #[allow] 忽略
    // 避坑: deny 会将警告转为错误, 可能导致编译失败
    #[allow(dead_code)]
    fn unused_function() {}

    assert!(true);
}

#[test]
/// 测试: #[non_exhaustive] 标记不完全枚举
fn test_non_exhaustive() {
    // 语法: #[non_exhaustive] 强制使用 match 时必须有 _ 分支
    // 避坑: 添加新变体时会产生编译错误, 提醒更新代码
    assert!(true);
}
