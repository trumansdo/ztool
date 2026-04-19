// ---------------------------------------------------------------------------
// 5.8 Edition 2024 安全特性
// ---------------------------------------------------------------------------

#[test]
/// 测试: Edition 2024 extern 块必须标记 unsafe
fn test_unsafe_extern_required() {
    // 语法: Edition 2024 中所有 extern 块必须标记 unsafe
    // 避坑: 从旧版迁移时所有 extern "C" { ... } 需改为 unsafe extern "C" { ... }
    unsafe extern "C" {
        fn some_c_function() -> i32;
    }
    assert!(true);
}

#[test]
/// 测试: Edition 2024 显式指定 ABI (extern "C")
fn test_missing_abi_warning() {
    // 语法: Edition 2024 中省略 ABI(extern { })会触发 lint 警告, 建议显式写 extern "C"
    // 避坑: 旧版默认 extern "C"; 新版建议始终显式指定 ABI, 避免歧义
    unsafe extern "C" {
        fn explicit_abi_fn() -> i32;
    }
    assert!(true);
}

#[test]
/// 测试: #[repr(packed)] 结构体布局
fn test_repr_packed() {
    // 语法: #[repr(packed)] 去除结构体字段对齐
    // 避坑: 访问未对齐字段可能产生未定义行为; 性能敏感且字段已对齐的场景使用
    #[repr(packed)]
    struct Packed {
        a: u8,
        b: u32,
    }

    let packed = Packed { a: 1, b: 2 };
    assert_eq!(std::mem::size_of::<Packed>(), 5);
}

#[test]
/// 测试: #[repr(align(n))] 对齐控制
fn test_repr_alignment() {
    // 语法: #[repr(align(n))] 控制结构体对齐方式
    // 避坑: 过度对齐会浪费内存; 与硬件缓存行对齐可提升性能
    #[repr(align(64))]
    struct CacheAligned {
        data: [u8; 64],
    }

    let aligned = CacheAligned { data: [0; 64] };
    assert_eq!(std::mem::align_of::<CacheAligned>(), 64);
}

#[test]
/// 测试: 未初始化内存 MaybeUninit
fn test_maybe_uninit() {
    // 语法: MaybeUninit<T> 表示可能未初始化的类型
    // 避坑: 调用 assume_init() 前必须确保已正确初始化, 否则产生未定义行为
    let mut data: MaybeUninit<i32> = MaybeUninit::uninit();

    unsafe {
        data.as_mut_ptr().write(42);
    }

    let initialized = unsafe { data.assume_init() };
    assert_eq!(initialized, 42);
}

use std::mem::MaybeUninit;

#[test]
/// 测试: 零大小类型 Zero-Sized Types
fn test_zero_sized_types() {
    // 语法: ZST (Zero-Sized Type) 不占用内存
    // 避坑: 编译器可能优化掉 ZST 相关操作; 需注意边界情况
    #[derive(Clone, Copy)]
    struct Empty;

    let v: Vec<Empty> = vec![Empty; 100];
    assert_eq!(v.len(), 100);
    assert_eq!(std::mem::size_of::<Vec<Empty>>(), std::mem::size_of::<Vec<()>>());
}

#[test]
/// 测试: Never 类型 ! (发散函数)
fn test_never_type() {
    // 语法: ! (never type) 表示函数不返回 (panic, continue, exit 等)
    // 避坑: never type 可自动 coercion 到任何类型
    fn panic_fn() -> ! {
        panic!("this function never returns");
    }

    let _result: i32 = if true {
        42
    } else {
        panic_fn();
    };

    assert_eq!(_result, 42);
}
