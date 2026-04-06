// ---------------------------------------------------------------------------
// 4.5 FFI 与外部函数接口
// ---------------------------------------------------------------------------

#[test]
/// 测试: Edition 2024 unsafe extern 块要求
fn test_unsafe_extern_block() {
    // 语法: Edition 2024 中 extern 块必须标记 unsafe; 调用外部函数也需要 unsafe
    // 避坑: 旧版不需要 unsafe extern; 外部函数签名必须与 C ABI 完全匹配
    unsafe extern "C" {
        fn abs(i: i32) -> i32;
    }
    assert_eq!(unsafe { abs(-42) }, 42);
}

#[test]
/// 测试: Edition 2024 #[unsafe(...)] 属性语法
fn test_unsafe_attributes() {
    // 语法: Edition 2024 中 no_mangle/link_section/export_name 等属性必须用 #[unsafe(...)]
    // 避坑: 旧版直接写 #[no_mangle]; 迁移后需改为 #[unsafe(no_mangle)]
    #[unsafe(no_mangle)]
    pub extern "C" fn my_exported_function() -> i32 {
        42
    }
    assert_eq!(my_exported_function(), 42);
}

#[test]
/// 测试: extern "C" 函数签名中使用 i128/u128 (1.89+)
fn test_i128_extern_c() {
    // 语法: 1.89+ i128/u128 可用于 extern "C" 函数签名
    // 避坑: 旧版 i128 不能用于 extern "C"; 跨语言调用需确认对方支持 128 位整数
    unsafe extern "C" {
        fn external_128_function(x: i128) -> i128;
    }
    let big_num: i128 = 170141183460469231731687303715884105727;
    assert!(big_num > i64::MAX as i128);
}

#[test]
/// 测试: #[target_feature] 安全函数与 CPU 特性检测 (1.86+)
fn test_safe_target_feature() {
    // 语法: 1.86+ #[target_feature] 可用于安全函数, 但调用方仍需 unsafe 确保特性可用
    // 避坑: 调用前必须用 is_x86_feature_detected! 等检查 CPU 支持; 否则 UB
    #[target_feature(enable = "sse2")]
    fn requires_sse2() -> bool {
        true
    }
    if is_x86_feature_detected!("sse2") {
        assert!(unsafe { requires_sse2() });
    }
}

#[test]
/// 测试: repr(C) 结构体内存布局
fn test_repr_c() {
    // 语法: #[repr(C)] 保证结构体字段按 C 语言顺序和对齐方式布局
    // 避坑: 不加 repr(C) 时 Rust 不保证字段顺序; 跨 FFI 必须加 repr(C)
    #[repr(C)]
    struct Point {
        x: f64,
        y: f64,
    }

    assert_eq!(std::mem::size_of::<Point>(), 16);
    assert_eq!(std::mem::align_of::<Point>(), 8);

    let p = Point { x: 1.0, y: 2.0 };
    let ptr = &p as *const Point as *const f64;
    unsafe {
        assert_eq!(*ptr, 1.0);
        assert_eq!(*ptr.add(1), 2.0);
    }
}

#[test]
/// 测试: repr(transparent) 单字段 FFI 类型
fn test_repr_transparent() {
    // 语法: #[repr(transparent)] 保证单字段结构体与内部字段有相同的 ABI
    // 避坑: 只能有一个非零大小字段; 用于 newtype 模式包装 FFI 类型
    #[repr(transparent)]
    struct MyHandle(*mut std::ffi::c_void);

    assert_eq!(std::mem::size_of::<MyHandle>(), std::mem::size_of::<*mut std::ffi::c_void>());
}

#[test]
/// 测试: extern 函数指针类型
fn test_extern_fn_pointer() {
    // 语法: extern "C" fn(i32) -> i32 是 C ABI 函数指针类型
    // 避坑: 普通 fn 指针和 extern "C" fn 指针 ABI 不同, 不能混用
    type CCallback = extern "C" fn(i32) -> i32;

    extern "C" fn double(x: i32) -> i32 {
        x * 2
    }

    let callback: CCallback = double;
    let result = callback(21);
    assert_eq!(result, 42);
}

#[test]
/// 测试: CStr 和 CString 互操作
fn test_cstr_cstring() {
    // 语法: CStr 是借用的 C 字符串 (&CStr); CString 是 owned C 字符串
    // 避坑: CStr 必须以 \0 结尾; from_bytes_with_nul 需要手动加 \0
    //       to_str() 检查 UTF-8 有效性, 失败返回 Err
    use std::ffi::{CStr, CString};

    // Rust String → C 字符串
    let cstring = CString::new("hello from Rust").unwrap();
    let c_ptr = cstring.as_ptr();
    unsafe {
        let cstr = CStr::from_ptr(c_ptr);
        assert_eq!(cstr.to_str().unwrap(), "hello from Rust");
    }

    // C 字符串 → Rust &str
    let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(b"hello C\0") };
    assert_eq!(cstr.to_str().unwrap(), "hello C");
}

#[test]
/// 测试: 不同 ABI 的 extern 块
fn test_extern_abi_variants() {
    // 语法: extern 支持多种 ABI: "C", "system", "cdecl", "stdcall", "fastcall", "aapcs", "win64"
    // 避坑: 不同平台支持的 ABI 不同; 不指定 ABI 默认是 "Rust" (不稳定)
    // "system" 是平台默认的外部调用 ABI (Windows=stdcall, Linux=C)
    unsafe extern "system" {
        fn GetLastError() -> u32;
    }
    // 仅演示语法, 不实际调用
    assert!(true);
}

#[test]
/// 测试: #[link] 链接外部库
fn test_link_attribute() {
    // 语法: #[link(name = "libname")] 告诉 rustc 链接指定库
    // 避坑: #[link] 只能用于 extern 块; 库名不加 lib 前缀和扩展名
    //       也可以用 RUSTFLAGS=-l 或 build.rs 配置
    assert!(true);
}
