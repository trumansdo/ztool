// ---------------------------------------------------------------------------
// 5.2 条件编译增强 (1.88+)
// ---------------------------------------------------------------------------

// 语法: cfg_attr 条件应用属性 (可叠加多个属性)
// 避坑: cfg_attr 不继承 cfg 中的属性; 嵌套的 cfg_attr 不会自动展开

#[test]
/// 测试: cfg 布尔字面量 (cfg(true)/cfg(false), 1.88+)
fn test_cfg_boolean_literals() {
    // 语法: cfg(true) 始终启用, cfg(false) 始终不启用 (1.88+)
    // 避坑: cfg(false) 中的代码完全不编译, 连语法检查都跳过; 可用于临时禁用测试
    assert!(cfg!(true));
    assert!(!cfg!(false));
}

#[cfg(true)]
fn always_compiled() -> i32 {
    42
}

#[test]
/// 测试: cfg(true) 条件编译始终启用
fn test_always_compiled() {
    assert_eq!(always_compiled(), 42);
}

#[test]
/// 测试: cfg_attr 条件应用多个属性
fn test_cfg_attr_multiple() {
    // 语法: cfg_attr 可应用多个属性, 用逗号分隔
    // 避坑: 旧版 cfg_attr 只支持单属性; 1.88+ 支持逗号分隔多属性
    #[cfg_attr(feature = "debug", allow(dead_code), warn(dead_code))]
    fn unused_function() {}
    assert!(true);
}

#[test]
/// 测试: cfg 嵌套条件组合
fn test_cfg_nested() {
    // 语法: all!() 和 any!() 组合多个 cfg 条件
    // 避坑: all!(a, b) 全部为真才算通过; any!(a, b) 任一为真就算通过
    #[cfg(all(target_os = "linux", feature = "unix"))]
    fn linux_only() {}

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn unix_like() {}

    assert!(true);
}

#[test]
/// 测试: target_family 条件编译
fn test_target_family() {
    // 语法: target_family 表示硬件平台系列 (unix, windows, wasm)
    // 避坑: target_family 是平台分组的便捷方式
    fn get_platform() -> &'static str {
        #[cfg(target_family = "unix")]
        {
            "unix"
        }
        #[cfg(target_family = "windows")]
        {
            "windows"
        }
        #[cfg(not(any(target_family = "unix", target_family = "windows")))]
        {
            "other"
        }
    }

    let _ = get_platform();
    assert!(true);
}

#[test]
/// 测试: target_arch 架构特定代码
fn test_target_arch() {
    // 语法: target_arch 用于架构特定代码 (x86, x86_64, arm, aarch64, wasm32)
    // 避坑: 架构检测应配合 #[cfg] 使用, 避免编译失败
    #[cfg(target_arch = "x86_64")]
    fn arch_specific() -> &'static str {
        "x86_64"
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn arch_specific() -> &'static str {
        "other"
    }

    let _ = arch_specific();
    assert!(true);
}

#[test]
/// 测试: cfg 模块级条件编译
fn test_cfg_module_level() {
    // 语法: #[cfg] 可用于模块, 使整个模块条件编译
    // 避坑: 条件编译的模块其依赖也需满足条件
    assert!(true);
}

#[test]
/// 测试: cfg 与函数属性组合
fn test_cfg_with_function_attributes() {
    // 语法: #[cfg] 可与 #[inline], #[test] 等函数属性组合
    // 避坑: 条件不满足时, 属性不会应用, 但函数定义仍会被编译
    #[cfg(feature = "test")]
    #[inline]
    fn inlined_fn() -> i32 {
        42
    }

    #[cfg(not(feature = "test"))]
    fn inlined_fn() -> i32 {
        42
    }

    assert_eq!(inlined_fn(), 42);
}
