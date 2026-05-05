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

#[test]
/// 测试: extern "C" 完整语法——多个 ABI 块、安全包装、变量声明
fn test_extern_c_basics() {
    // 语法: extern 块声明外部 C 函数; Edition 2024 必须 unsafe extern
    // 避坑: 外部函数签名必须与 C ABI 精确匹配; 类型大小错误会导致栈损坏
    use std::os::raw::c_int;

    // 声明 C 标准库函数
    unsafe extern "C" {
        fn abs(i: c_int) -> c_int;
        fn labs(i: std::os::raw::c_long) -> std::os::raw::c_long;
    }

    // 直接调用: 需 unsafe
    assert_eq!(unsafe { abs(-42) }, 42);

    // 安全包装: 将 unsafe 封装在安全接口中
    fn safe_abs(n: i32) -> i32 {
        // SAFETY: abs 对所有 i32 输入均有定义(除了 i32::MIN)
        unsafe { abs(n as c_int) }
    }
    assert_eq!(safe_abs(-100), 100);

    // 链接属性声明
    #[cfg_attr(not(target_os = "windows"), link(name = "c"))]  // libc 链接(Windows 无需显式链接)
    unsafe extern "C" {
        fn sqrt(x: f64) -> f64;
    }
    // sqrt(4.0) 约为 2.0 (浮点误差容忍)
    let val = unsafe { sqrt(4.0) };
    assert!((val - 2.0).abs() < 1e-10);
}

#[test]
/// 测试: CString 深入转换——NulError、into_bytes、from_raw
fn test_cstring_conversion() {
    // 语法: CString::new 检查内部 \0; CString::from_raw 接管 C 分配的内存
    // 避坑: 内部含 \0 的字符串应使用 from_bytes_with_nul_unchecked (需自己保证)
    use std::ffi::{CStr, CString};

    // CString::new 失败场景: 内部含空字节
    assert!(CString::new("hel\0lo").is_err());
    assert!(CString::new("正常字符串").is_ok());

    // CString 与 Vec<u8> 互转
    let cs = CString::new("hello").unwrap();
    let bytes: Vec<u8> = cs.into_bytes();            // 不含 \0
    assert_eq!(bytes, b"hello");

    // 重新从 bytes 构造
    let cs2 = CString::new(bytes).unwrap();
    assert_eq!(cs2.to_str().unwrap(), "hello");

    // 含 \0 的字节数组 (不安全方式)
    let bytes_with_nul: &[u8] = b"raw\0";
    let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(bytes_with_nul) };
    assert_eq!(cstr.to_bytes(), b"raw");

    // CString::from_raw 和 into_raw
    let cs = CString::new("接管所有权").unwrap();
    let ptr = cs.into_raw();                           // 交出所有权
    let cs_back = unsafe { CString::from_raw(ptr) };   // 重新接管
    assert_eq!(cs_back.to_str().unwrap(), "接管所有权");

    // as_bytes_with_nul 包含结尾 \0
    let cs = CString::new("test").unwrap();
    let bytes = cs.as_bytes_with_nul();
    assert_eq!(bytes, b"test\0");

    // as_c_str 获取 &CStr
    assert_eq!(cs.as_c_str().to_str().unwrap(), "test");
}

#[test]
/// 测试: C 风格回调——extern "C" fn 指针、模拟 C 调用 Rust
fn test_callback_pattern() {
    // 语法: extern "C" fn 类型作为 C 回调; Rust 闭包不能直接转为 extern fn 指针
    // 避坑: extern "C" fn 不能捕获环境; 需要上下文时使用 userdata 模式

    // 定义 C 回调类型
    type IntCallback = extern "C" fn(i32) -> i32;

    // Rust 函数作为 C 回调
    extern "C" fn double_it(x: i32) -> i32 {
        x * 2
    }

    extern "C" fn square_it(x: i32) -> i32 {
        x * x
    }

    // 模拟: C 侧注册并调用回调
    struct CallbackRegistry {
        cb: Option<IntCallback>,
    }

    impl CallbackRegistry {
        fn new() -> Self {
            Self { cb: None }
        }
        fn register(&mut self, cb: IntCallback) {
            self.cb = Some(cb);
        }
        fn invoke(&self, arg: i32) -> Option<i32> {
            self.cb.map(|f| f(arg))
        }
    }

    let mut registry = CallbackRegistry::new();

    // 注册 double_it
    registry.register(double_it);
    assert_eq!(registry.invoke(21), Some(42));

    // 切换为 square_it
    registry.register(square_it);
    assert_eq!(registry.invoke(7), Some(49));

    // 函数指针比较
    let cb1: IntCallback = double_it;
    let cb2: IntCallback = square_it;
    assert!(cb1 != cb2);
    // 函数指针相等性比较 (需转为相同类型)
    assert!(cb1 == double_it as IntCallback);

    // 模拟 null 回调 (可空函数指针)
    let null_cb: Option<IntCallback> = None;
    assert!(null_cb.is_none());
}

#[test]
/// 测试: #[no_mangle] 导出 Rust 函数给 C
fn test_no_mangle_export() {
    // 语法: #[unsafe(no_mangle)] 禁止名称修饰, pub extern "C" 指定 C ABI
    // 避坑: Edition 2024 中用 #[unsafe(no_mangle)], 旧版直接 #[no_mangle]
    //       导出名不能与其他符号冲突; 返回指针需明确所有权

    // 场景1: 导出简单函数
    #[unsafe(no_mangle)]
    pub extern "C" fn exported_add(a: i32, b: i32) -> i32 {
        a + b
    }
    assert_eq!(exported_add(3, 4), 7);

    // 场景2: 导出接收和返回 C 字符串的函数
    use std::ffi::CString;

    #[unsafe(no_mangle)]
    pub extern "C" fn exported_strlen(s: *const std::ffi::c_char) -> usize {
        if s.is_null() { return 0; }
        unsafe { std::ffi::CStr::from_ptr(s).to_bytes().len() }
    }

    let cs = CString::new("hello").unwrap();
    assert_eq!(exported_strlen(cs.as_ptr()), 5);

    // 空指针安全处理
    assert_eq!(exported_strlen(std::ptr::null()), 0);

    // 场景3: 多个导出函数的命名检查
    #[unsafe(no_mangle)]
    pub extern "C" fn exported_mul(a: i32, b: i32) -> i32 {
        a * b
    }
    assert_eq!(exported_mul(6, 7), 42);

    // 场景4: 通过函数指针间接调用 (模拟 C 侧调用)
    type FfiAddFn = extern "C" fn(i32, i32) -> i32;
    let fn_ptr: FfiAddFn = exported_add;
    assert_eq!(fn_ptr(10, 20), 30);
}
