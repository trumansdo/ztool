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
