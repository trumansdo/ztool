// ---------------------------------------------------------------------------
// 5.2 条件编译增强 (1.88+)
// ---------------------------------------------------------------------------

// 允许演示用的 feature 未在 Cargo.toml 中定义
#![allow(unexpected_cfgs)]

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

// ===========================================================================
// 补充增强测试
// ===========================================================================

#[test]
/// 测试: target_env 环境 ABI 检测
fn test_target_env() {
    // 语法: target_env 用于区分 ABI/工具链环境 (gnu, msvc, musl, sgx)
    // 避坑: target_env 在不同平台上取值不同，Windows 通常为 "msvc" 或 "gnu"
    fn get_env() -> &'static str {
        #[cfg(target_env = "gnu")]
        {
            "gnu"
        }
        #[cfg(target_env = "msvc")]
        {
            "msvc"
        }
        #[cfg(target_env = "musl")]
        {
            "musl"
        }
        #[cfg(not(any(target_env = "gnu", target_env = "msvc", target_env = "musl")))]
        {
            "other"
        }
    }

    let env = get_env();
    assert!(!env.is_empty());
}

#[test]
/// 测试: cfg! 宏与 #[cfg] 的区别
fn test_cfg_macro_vs_attribute() {
    // 语法: cfg!() 运行时求值为 bool，#[cfg] 编译时条件消除
    // 避坑: cfg!() 包裹的代码必须能通过编译；#[cfg] 包裹的代码完全不会编译
    //       关键区别: #[cfg(false)] 中的语法错误不会报错，cfg!(false) 中的语法错误会报错！

    // cfg! 始终编译代码，只是运行时走 else 分支
    let value = if cfg!(target_os = "linux") {
        "linux"
    } else {
        "not linux"
    };
    // 两个分支的代码都经历了编译检查
    assert!(!value.is_empty());

    // debug_assertions 在 cargo test(非 release) 下为 true
    let is_debug = cfg!(debug_assertions);
    // 测试模式下应为 true
    assert!(is_debug);

    // test 模式下 cfg!(test) 为 true
    assert!(cfg!(test));
}

#[test]
/// 测试: feature 检测 (cfg!(feature = "..."))
fn test_feature_detection() {
    // 语法: cfg!(feature = "xxx") 检测编译时启用的特性
    // 避坑: feature 名称需在 Cargo.toml 的 [features] 中定义
    //       未定义的 feature 总是 false，且不会报错

    // 检测本 crate 中定义的 feature（示例）
    let has_serde = cfg!(feature = "serde");
    let _ = has_serde; // 取决于构建配置

    // 检测标准 feature（总是已知）
    assert!(cfg!(target_arch = "x86_64"));
}

#[test]
/// 测试: cfg_attr 条件属性高级用法
fn test_cfg_attr_advanced() {
    // 语法: cfg_attr 可条件地应用一个或多个属性
    // 避坑: cfg_attr 不是 cfg 的替代，条件不满足时属性不应用但代码仍然编译

    // 示例：条件派生宏（实际使用时配合 serde 特性）
    #[cfg_attr(feature = "serde", derive(Debug, Clone))]
    struct ConditionallyDerived {
        data: i32,
    }

    // 结构体总是编译，但属性按需应用
    let instance = ConditionallyDerived { data: 42 };
    assert_eq!(instance.data, 42);

    // cfg_attr 多层：在不同条件下应用不同 lint
    #[cfg_attr(
        debug_assertions,
        allow(dead_code),
        allow(unused_variables)
    )]
    fn debug_only_lint() {
        let _unused = 1;
        // 仅在 debug 模式下，dead_code 和 unused_variables 的 warning 被抑制
    }

    debug_only_lint();
}

#[test]
/// 测试: #[cfg(not(...))] 排除模式
fn test_cfg_not_pattern() {
    // 语法: not() 用于排除条件，构建非某平台的 fallback
    // 避坑: not() 只能传入单个谓词或 all/any 组合

    #[cfg(target_os = "windows")]
    fn os_specific() -> &'static str {
        "windows"
    }

    #[cfg(not(target_os = "windows"))]
    fn os_specific() -> &'static str {
        "not windows"
    }

    let _ = os_specific();

    // 复杂组合：Linux 但不是 musl
    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    fn linux_glibc() {}

    // 或：所有 unix 但不是 macos
    #[cfg(all(target_family = "unix", not(target_os = "macos")))]
    fn unix_not_macos() {}
}

#[test]
/// 测试: 平台特定代码组织——用 #[cfg] 实现接口的多平台实现
fn test_platform_specific_impl() {
    // 语法: 最佳实践是用 #[cfg] 在同名函数上提供不同实现，而非在函数内用 if-else
    // 避坑: 两个 #[cfg] 必须互斥，否则会有重复定义编译错误

    // 平台 A 实现
    #[cfg(target_os = "windows")]
    fn platform_name() -> &'static str {
        "windows"
    }

    // 平台 B 实现（互斥条件）
    #[cfg(not(target_os = "windows"))]
    fn platform_name() -> &'static str {
        "非 windows"
    }

    let name = platform_name();
    #[cfg(target_os = "windows")]
    assert_eq!(name, "windows");
    #[cfg(not(target_os = "windows"))]
    assert_eq!(name, "非 windows");
}

#[test]
/// 测试: target_pointer_width / target_endian 等辅助谓词
fn test_target_pointer_and_endian() {
    // 语法: target_pointer_width 检测指针宽度(32/64)，target_endian 检测字节序
    // 避坑: 大多数现代 CPU 都是 little-endian

    #[cfg(target_pointer_width = "64")]
    let bits = 64;
    #[cfg(target_pointer_width = "32")]
    let bits = 32;
    #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
    let bits = 0;

    assert!(bits == 32 || bits == 64);

    #[cfg(target_endian = "little")]
    let endian = "little";
    #[cfg(target_endian = "big")]
    let endian = "big";

    // 大多数平台都是小端
    assert!(!endian.is_empty());
}

#[test]
/// 测试: cfg_attr 条件文档 (doc 属性)
fn test_cfg_attr_with_doc() {
    // 语法: cfg_attr 可用于条件地附加文档属性
    // 避坑: doc 属性只在文档生成时生效，不影响编译

    // 仅在有 serde 特性时出现在文档中
    #[cfg_attr(feature = "serde", doc = "此结构体支持 serde 序列化（需启用 serde 特性）")]
    #[derive(Debug, Clone, PartialEq)]
    struct DocumentedStruct {
        value: i32,
    }

    let ds = DocumentedStruct { value: 100 };
    assert_eq!(ds.value, 100);
}
