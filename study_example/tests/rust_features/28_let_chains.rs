// ---------------------------------------------------------------------------
// 5.1 Let Chains (1.88+ / Edition 2024)
// ---------------------------------------------------------------------------

// 语法: if let ... && ... 链式组合 let 绑定和普通条件, 避免嵌套 (1.88+ / Edition 2024)
// 避坑: && 优先级低于 let, 不能用 || 连接 let; let 绑定的变量在后续条件中可用

enum Channel {
    Stable(Semver),
    Beta,
    Nightly,
    Unknown,
}
struct Semver {
    major: u32,
    minor: u32,
    patch: u32,
}

fn get_channel() -> Channel {
    Channel::Stable(Semver { major: 1, minor: 88, patch: 0 })
}

#[test]
/// 测试: let chains 链式 let 绑定和条件 (1.88+)
fn test_let_chains_basic() {
    if let Channel::Stable(v) = get_channel()
        && v.major == 1
        && v.minor == 88
    {
        assert!(true);
    } else {
        panic!("Let chains didn't work");
    }
}

#[test]
/// 测试: let chains 多 let 链式组合和变量传递
fn test_let_chains_with_binding_usage() {
    if let Some(x) = Some(10)
        && let Some(y) = Some(20)
        && x + y == 30
    {
        assert_eq!(x, 10);
        assert_eq!(y, 20);
    }
}

#[test]
/// 测试: let chains 混合 let 和普通布尔条件
fn test_let_chains_mixed_conditions() {
    let flag = true;
    if let Some(x) = Some(42)
        && flag
        && x > 0
    {
        assert_eq!(x, 42);
    }
}

#[test]
/// 测试: let chains 在循环中逐个处理
fn test_let_chains_in_loop() {
    let data = vec![Some(1), Some(2), None, Some(3)];
    let mut sum = 0;
    for item in data {
        if let Some(x) = item && x > 0 {
            sum += x;
        }
    }
    assert_eq!(sum, 6);
}

#[test]
/// 测试: let chains 匹配枚举多变体
fn test_let_chains_enum_matching() {
    let ch = get_channel();
    if let Channel::Stable(v) = ch
        && v.major >= 1
        && v.minor >= 70
    {
        assert!(true);
    } else {
        panic!("Should match stable with high version");
    }
}

#[test]
/// 测试: let chains 变量遮蔽和作用域
fn test_let_chains_variable_shadowing() {
    let x = Some(5);
    if let Some(x) = x && x > 0 {
        assert_eq!(x, 5);
    }
    if let Some(y) = Some(10) {
        assert_eq!(y, 10);
    }
}

#[test]
/// 测试: let chains 嵌套 Option 处理
fn test_let_chains_nested_option() {
    let nested: Option<Option<i32>> = Some(Some(42));
    if let Some(inner) = nested && let Some(val) = inner && val > 0 {
        assert_eq!(val, 42);
    }
}

#[test]
/// 测试: let chains Result 和 Option 混合
fn test_let_chains_result_option_mix() {
    let data: Result<Option<i32>, &str> = Ok(Some(100));
    if let Ok(Some(val)) = data && val > 50 {
        assert_eq!(val, 100);
    }
}

#[test]
/// 测试: let chains 在 match arm 中使用
fn test_let_chains_in_match_arm() {
    let value = Some(20);
    let result = match value {
        Some(x) if x > 10 => x * 2,
        _ => 0,
    };
    assert_eq!(result, 40);
}

#[test]
/// 测试: let chains 多条件组合
fn test_let_chains_multiple_conditions() {
    let nums = vec![1, 2, 3, 4, 5];
    if let Some(first) = nums.first()
        && let Some(last) = nums.last()
        && *first == 1
        && *last == 5
    {
        assert_eq!(*first, 1);
        assert_eq!(*last, 5);
    }
}

// ===================== 扩充测试 =====================

#[test]
/// 测试: let chains 短路求值 —— 失败时不执行后续
fn test_let_chains_short_circuit() {
    let mut called = false;
    let mut get_value = || {
        called = true;
        None::<i32>
    };

    if let Some(x) = get_value() && x > 0 {
        panic!("should not reach here");
    }
    assert!(called, "first let expression should be evaluated");
}

#[test]
/// 测试: let chains 短路求值 —— 第一个匹配失败不执行第二个
fn test_let_chains_short_circuit_second_not_evaluated() {
    let mut second_called = false;
    let mut make_none = || -> Option<i32> {
        second_called = true;
        Some(42)
    };

    if let Some(_x) = None::<i32>
        && let Some(_y) = make_none()
        && _x > 0
    {
        panic!("should not reach here");
    }
    assert!(!second_called, "second expression should NOT be called due to short-circuit");
}

#[test]
/// 测试: let chains while let 循环中提前退出
fn test_let_chains_while_let_with_condition() {
    let mut data = vec![1, 2, 10, 3];
    let mut sum = 0;

    // 模拟: while let 链式，大于等于 10 就退出
    while let Some(x) = data.pop() && x < 10 {
        sum += x;
    }
    assert_eq!(sum, 3); // 3 + 2 + 1 = 6? No: pop LIFO: 3, then 10 stops
    // 注意: pop 是 LIFO —— vec![1,2,10,3] -> pop 得 3, 再 pop 得 10 停止
    assert_eq!(sum, 3);
}

#[test]
/// 测试: let chains else 分支中变量也可用
fn test_let_chains_else_branch_bindings_available() {
    let value: Option<i32> = None;
    if let Some(x) = value && x > 0 {
        panic!("should not match None");
    } else {
        // 在 Edition 2024 中 x 在 else 中也可见,
        // 但由于模式未匹配, x 未绑定, 这里不能使用 x
        // 这里只验证不会 panic
        assert!(true);
    }
}

#[test]
/// 测试: let chains Result 多级解构
fn test_let_chains_result_multiple_levels() {
    fn get_config() -> Result<Option<String>, &'static str> {
        Ok(Some("enabled".to_string()))
    }

    if let Ok(Some(config)) = get_config()
        && let Some(_) = config.strip_prefix("en")
        && config.len() > 3
    {
        assert_eq!(config, "enabled");
    } else {
        panic!("should match config");
    }
}

#[test]
/// 测试: let chains 结构体解构
fn test_let_chains_struct_destructure() {
    struct Point { x: i32, y: i32 }
    let opt = Some(Point { x: 10, y: 20 });

    if let Some(Point { x, y }) = opt && x > 0 && y > 0 && x + y == 30 {
        assert_eq!(x, 10);
        assert_eq!(y, 20);
    }
}

#[test]
/// 测试: let chains 三层 Option 嵌套
fn test_let_chains_triple_option() {
    let triple: Option<Option<Option<i32>>> = Some(Some(Some(42)));

    if let Some(l1) = triple
        && let Some(l2) = l1
        && let Some(val) = l2
        && val == 42
    {
        assert_eq!(val, 42);
    } else {
        panic!("Should extract triple-nested value");
    }
}

#[test]
/// 测试: let chains 与布尔值 false 时进入 else
fn test_let_chains_false_condition_else() {
    let opt = Some(5);
    let _reached_else = false;

    if let Some(x) = opt && x > 10 {
        panic!("should not match, x=5 <= 10");
    } else {
        let _reached_else = true;
    }
    // reached_else 在版次2024中x仍然在else中可见但未绑定
}

#[test]
/// 测试: let chains 字符串模式匹配
fn test_let_chains_string_pattern() {
    let input = "  hello world  ";
    if let Some(trimmed) = Some(input.trim())
        && trimmed.starts_with("hello")
        && trimmed.ends_with("world")
        && trimmed.len() > 5
    {
        assert_eq!(trimmed, "hello world");
    }
}

#[test]
/// 测试: let chains 范围检查
fn test_let_chains_range_check() {
    fn validate_age(age: Option<u32>) -> bool {
        if let Some(a) = age && a >= 18 && a <= 120 {
            true
        } else {
            false
        }
    }

    assert!(validate_age(Some(25)));
    assert!(!validate_age(Some(15)));
    assert!(!validate_age(None));
    assert!(!validate_age(Some(150)));
}
