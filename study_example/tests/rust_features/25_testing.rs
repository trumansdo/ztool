// ---------------------------------------------------------------------------
// 5.2 测试
// ---------------------------------------------------------------------------

// 辅助函数: 测试私有函数
fn private_helper(x: i32) -> i32 {
    x * 2
}

#[test]
/// 测试: 基本断言宏 (assert!/assert_eq!/assert_ne!)
fn test_basic_assertions() {
    // 语法: 测试使用 assert!(条件) 和 assert_eq!(左, 右)
    //
    // 断言宏:
    //   - assert!(cond, "msg")     布尔断言
    //   - assert_eq!(a, b, "msg")  相等断言
    //   - assert_ne!(a, b, "msg")  不等断言
    //
    // 避坑:
    //   - 断言失败会 panic
    //   - 自定义错误信息使用 format! 语法
    //   - Debug 模式下性能开销无影响
    //
    assert!(true);
    assert_eq!(2 + 2, 4);
    assert_ne!(2 + 2, 5);

    // 自定义错误信息
    assert!(true, "这个应该通过: {}", "OK");
    assert_eq!(
        2 + 2,
        4,
        "2 + 2 应该等于 4, 而不是 {}",
        2 + 2
    );
}

#[test]
/// 测试: should_panic 属性 (预期 panic)
#[should_panic(expected = "出错了")]
fn test_should_panic() {
    // 语法: #[should_panic(expected = "...")] 标记预期 panic 的测试
    //
    // 避坑:
    //   - expected 是子串匹配, 不要求完全相等
    //   - 测试函数中没有 panic 时测试失败
    //   - 不要过度使用, 优先使用 Result<T, E>
    //
    panic!("出错了: 位置信息");
}

#[test]
/// 测试: 使用 Result<T, E> 返回值的测试
fn test_with_result() -> Result<(), String> {
    // 语法: 测试函数可以返回 Result<T, E>, 不再需要 panic
    //
    // 优势:
    //   - 支持 ? 操作符
    //   - 测试代码更符合正常控制流
    //   - 错误信息更清晰
    //
    // 避坑:
    //   - 不能同时使用 #[should_panic]
    //   - 返回 Err 时测试失败
    //

    if 2 + 2 == 4 {
        Ok(())
    } else {
        Err(String::from("2 + 2 != 4"))
    }
}

#[test]
/// 测试: 测试私有函数
fn test_private_function() {
    // 语法: 单元测试可以直接测试私有函数
    //
    // 这是 Rust 相比其他语言的独特优势:
    //   其他语言通常需要通过反射或特殊手段才能测试私有方法
    //

    assert_eq!(private_helper(5), 10);
    assert_eq!(private_helper(0), 0);
    assert_eq!(private_helper(-3), -6);
}

#[test]
/// 测试: 忽略测试 (#[ignore])
#[ignore]
fn test_ignored() {
    // 语法: #[ignore] 标记的测试默认跳过, 用 cargo test -- --ignored 运行
    //
    // 使用场景:
    //   - 长时间运行的测试
    //   - 需要外部资源的测试
    //   - 临时禁用的失败测试
    //
    unreachable!("这个测试不会运行");
}

#[test]
/// 测试: 条件编译 (#[cfg])
fn test_cfg_attribute() {
    // 语法: #[cfg(condition)] 按平台/特性条件编译
    //
    // 常用条件:
    //   - target_os = "linux"/"windows"/"macos"
    //   - target_arch = "x86_64"/"aarch64"
    //   - feature = "some_feature"
    //   - debug_assertions (仅 debug 模式)
    //
    // 避坑:
    //   - 条件编译的测试不会在所有平台上运行
    //   - 需要 #[cfg(test)] 包裹 test module
    //

    #[cfg(target_os = "windows")]
    {
        assert!(std::env::consts::OS == "windows");
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert!(std::env::consts::OS != "windows");
    }

    // debug_assertions 在 debug 模式下启用
    #[cfg(debug_assertions)]
    {
        assert!(true, "debug 模式");
    }
}

#[test]
/// 测试: 使用 Mock (通过 trait 实现测试替身)
fn test_mock_pattern() {
    // 语法: Rust 不支持运行期动态 mock
    //       通过 trait + 测试替身(Test Double)实现
    //
    // 模式:
    //   1. 定义 trait 接口
    //   2. 产品代码实现真实逻辑
    //   3. 测试代码实现 mock 逻辑
    //   4. 将 trait 作为函数参数
    //

    trait Database {
        fn query(&self, id: u32) -> Option<String>;
    }

    struct MockDb;
    impl Database for MockDb {
        fn query(&self, _id: u32) -> Option<String> {
            Some("mock_data".to_string())
        }
    }

    fn process_user(db: &impl Database, id: u32) -> String {
        db.query(id).unwrap_or_default()
    }

    let mock = MockDb;
    assert_eq!(process_user(&mock, 1), "mock_data");
    assert_eq!(process_user(&mock, 999), "mock_data");
}

#[test]
/// 测试: 文档测试示例 (验证注释中的代码)
fn test_doc_test_example() {
    // 语法: /// ```rust 代码示例 ``` 在文档注释中编写测试
    //
    // 特点:
    //   - cargo test 自动运行文档中的代码
    //   - 确保文档中的示例始终有效
    //   - 是 Rust 特有的测试形式
    //

    /// 将两个数相加
    ///
    /// ```
    /// let result = add(2, 3);
    /// assert_eq!(result, 5);
    /// ```
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    assert_eq!(add(2, 3), 5);
}

// ===========================================================================
// 补充增强测试
// ===========================================================================

#[test]
/// 测试: assert_eq! 自定义错误信息（format! 语法）
fn test_assert_with_custom_message() {
    // 语法: 断言宏支持 format! 语法作为最后一个参数
    // 避坑: 自定义消息只在断言失败时求值，成功时没有开销
    let actual = vec![1, 2, 3];
    let expected = vec![1, 2, 3];

    // 使用 format! 语法传递上下文
    assert_eq!(
        actual, expected,
        "列表比较失败: 期望 {:?}，实际 {:?}，长度分别为 {} 和 {}",
        expected, actual,
        expected.len(), actual.len()
    );

    // assert! 支持多参数 format
    let item_count = 3;
    assert!(
        item_count >= 3,
        "item_count 应该 >= 3，但实际是 {}（差 {} 个）",
        item_count, 3 - item_count as i32
    );
}

#[test]
/// 测试: PartialEq + Debug 类型使用 assert_eq!（元组、Option、Result）
fn test_assert_eq_on_standard_types() {
    // 语法: 标准库类型已实现 PartialEq + Debug，可直接用 assert_eq!
    // 避坑: 自定义类型需 #[derive(PartialEq, Debug)] 才能用 assert_eq!

    // 元组
    assert_eq!((1, "hello"), (1, "hello"));

    // Option
    assert_eq!(Some(42), Some(42));
    assert_ne!(Some(1), None);

    // Result
    let ok_result: Result<i32, &str> = Ok(42);
    assert_eq!(ok_result, Ok(42));

    // Vec
    assert_eq!(vec![1, 2, 3], vec![1, 2, 3]);

    // HashMap（需引入 std::collections::HashMap）
    use std::collections::HashMap;
    let mut map1 = HashMap::new();
    map1.insert("key", "value");
    let mut map2 = HashMap::new();
    map2.insert("key", "value");
    assert_eq!(map1, map2);
}

#[test]
/// 测试: 集成测试的典型模式——仅测试 pub API
fn test_integration_style_pub_api() {
    // 语法: 集成测试放在 tests/ 目录，只能调用 crate 的 pub API
    //
    // 集成测试模式:
    //   1. 仅测试公开导出的函数/类型
    //   2. 模拟外部用户的视角
    //   3. 测试多个模块的协作
    //
    // 避坑: 如需测试内部函数，应放在 src/ 中的单元测试，而非 tests/

    // 模拟一个 pub API
    pub fn public_api(value: i32) -> i32 {
        value * 2
    }

    // 集成测试只能调用 pub 函数
    assert_eq!(public_api(21), 42);
    assert_eq!(public_api(0), 0);
    assert_eq!(public_api(-5), -10);
}

#[test]
/// 测试: 文档测试高级用法 (should_panic / no_run / compile_fail / 隐藏行)
fn test_doc_test_advanced_modes() {
    // 语法: 文档测试支持多种模式标记
    //
    // ```should_panic —— 期望 panic 的示例
    // ```no_run       —— 仅编译不运行（适合无限循环等）
    // ```compile_fail —— 期望编译失败（设计上不可编译的代码）
    // ```ignore       —— 跳过此示例
    // # 开头行       —— 在文档中隐藏但会被执行
    //

    /// 安全除法，当除数为 0 时返回 None
    ///
    /// ```
    /// # struct Calculator;
    /// # impl Calculator {
    /// #   fn safe_divide(&self, a: i32, b: i32) -> Option<i32> {
    /// #       if b == 0 { None } else { Some(a / b) }
    /// #   }
    /// # }
    /// # let calc = Calculator;
    /// let result = calc.safe_divide(10, 2);
    /// assert_eq!(result, Some(5));
    ///
    /// let result = calc.safe_divide(10, 0);
    /// assert_eq!(result, None);
    /// ```
    fn safe_divide(a: i32, b: i32) -> Option<i32> {
        if b == 0 {
            None
        } else {
            Some(a / b)
        }
    }

    assert_eq!(safe_divide(10, 2), Some(5));
    assert_eq!(safe_divide(10, 0), None);
}

#[test]
/// 测试: 测试组织结构——单元测试 vs 集成测试的模板
fn test_organization_patterns() {
    // 语法: 以下展示 Rust 测试的三种组织方式
    //
    // 模式1: 同文件单元测试 (src/lib.rs 中)
    //   #[cfg(test)]
    //   mod tests { use super::*; ... }
    //
    // 模式2: 独立测试模块 (src/tests/xxx.rs)
    //   // 在 lib.rs: #[cfg(test)] mod tests;
    //   // 在 tests/mod.rs: pub mod xxx;
    //
    // 模式3: 集成测试 (tests/xxx.rs)
    //   每个文件是独立 crate，只能访问 pub API
    //
    // 模式4: 共享测试辅助 (tests/common/mod.rs)
    //   Cargo 不把 tests/common/ 下的文件当测试文件

    // 本文件即模式3的示例（集成测试）
    assert!(true, "不同的测试组织方式适用于不同场景");
}

#[test]
/// 测试: 测试专用辅助函数的组织
fn test_helper_function_organization() {
    // 语法: 测试文件中可直接定义辅助函数，不属于 #[cfg(test)] 的 guard
    // 避坑: 集成测试中辅助函数不加 #[cfg(test)] 也没关系（整个文件就是测试上下文）

    // 辅助函数可在多个测试间共享
    fn setup_test_data() -> Vec<i32> {
        vec![1, 2, 3, 4, 5]
    }

    fn assert_sorted(data: &[i32]) {
        for window in data.windows(2) {
            assert!(window[0] <= window[1], "数据未排序");
        }
    }

    let data = setup_test_data();
    assert_eq!(data.len(), 5);
    assert_sorted(&data);
}

#[test]
/// 测试: assert_matches! 风格的模式匹配断言（手写模拟）
fn test_pattern_matching_assertion() {
    // 语法: 虽然没有标准库 assert_matches!，可用 match + assert! 模拟
    // 注意: nightly 有 assert_matches! 宏，stable 可用 matches! + assert!
    enum Status {
        Success(i32),
        Error(String),
        Pending,
    }

    let result = Status::Success(42);

    // 方式1: matches! + assert!
    assert!(matches!(result, Status::Success(_)));

    // 方式2: match + assert!（更细节的验证）
    match result {
        Status::Success(code) => assert_eq!(code, 42),
        Status::Error(_) => panic!("不应该到达这里"),
        Status::Pending => panic!("不应该到达这里"),
    }

    // 方式3: if let 风格
    assert!(
        matches!(result, Status::Success(n) if n > 0),
        "Success 的值应该大于 0"
    );
}

#[test]
/// 测试: 并发测试的基本模式
fn test_concurrent_safety() {
    // 语法: 使用 std::thread 在测试中验证并发安全性
    // 避坑: cargo test 默认并行运行测试，需注意测试间的共享状态隔离
    use std::sync::{Arc, Mutex};
    use std::thread;

    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 10);
}

#[test]
/// 测试: 测试返回值更丰富的 Result 用法
fn test_result_with_question_mark() -> Result<(), Box<dyn std::error::Error>> {
    // 语法: 测试返回 Result 时，可使用 ? 操作符
    // 避坑: Box<dyn Error> 可以容纳任何错误类型，比固定错误类型更灵活

    // ? 可以用于任何返回 Result 的操作
    let number: i32 = "42".parse()?;
    assert_eq!(number, 42);

    // 可以串联多个可能失败的操作
    let path = std::env::current_dir()?;
    assert!(path.exists());

    Ok(())
}
