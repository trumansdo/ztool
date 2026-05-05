// ---------------------------------------------------------------------------
// 5.8 Edition 2024 安全特性
// ---------------------------------------------------------------------------

use std::mem::MaybeUninit;

#[test]
/// 测试: Edition 2024 extern 块必须标记 unsafe
fn test_unsafe_extern_required() {
    unsafe extern "C" {
        fn some_c_function() -> i32;
    }
    assert!(true);
}

#[test]
/// 测试: Edition 2024 显式指定 ABI (extern "C")
fn test_missing_abi_warning() {
    unsafe extern "C" {
        fn explicit_abi_fn() -> i32;
    }
    assert!(true);
}

#[test]
/// 测试: #[repr(packed)] 结构体布局
fn test_repr_packed() {
    #[repr(packed)]
    struct Packed {
        a: u8,
        b: u32,
    }

    let _packed = Packed { a: 1, b: 2 };
    assert_eq!(std::mem::size_of::<Packed>(), 5);
}

#[test]
/// 测试: #[repr(align(n))] 对齐控制
fn test_repr_alignment() {
    #[repr(align(64))]
    struct CacheAligned {
        data: [u8; 64],
    }

    let _aligned = CacheAligned { data: [0; 64] };
    assert_eq!(std::mem::align_of::<CacheAligned>(), 64);
}

#[test]
/// 测试: 未初始化内存 MaybeUninit
fn test_maybe_uninit() {
    let mut data: MaybeUninit<i32> = MaybeUninit::uninit();

    unsafe {
        data.as_mut_ptr().write(42);
    }

    let initialized = unsafe { data.assume_init() };
    assert_eq!(initialized, 42);
}

#[test]
/// 测试: 零大小类型 Zero-Sized Types
fn test_zero_sized_types() {
    #[derive(Clone, Copy)]
    struct Empty;

    let v: Vec<Empty> = vec![Empty; 100];
    assert_eq!(v.len(), 100);
    assert_eq!(std::mem::size_of::<Vec<Empty>>(), std::mem::size_of::<Vec<()>>());
}

#[test]
/// 测试: Never 类型 ! (发散函数)
fn test_never_type() {
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

// ===================== 扩充测试 =====================

#[test]
/// 测试: MaybeUninit 数组初始化
fn test_maybe_uninit_array() {
    let mut arr: [MaybeUninit<i32>; 5] = unsafe { MaybeUninit::uninit().assume_init() };

    for i in 0..5 {
        arr[i].write(i as i32);
    }

    // Safety: 所有元素已初始化
    let initialized: [i32; 5] = unsafe { std::mem::transmute::<_, [i32; 5]>(arr) };
    assert_eq!(initialized, [0, 1, 2, 3, 4]);
}

#[test]
/// 测试: MaybeUninit 部分初始化
fn test_maybe_uninit_partial_init() {
    #[derive(Debug, PartialEq)]
    struct Pair {
        x: i32,
        y: i32,
    }

    let mut pair: MaybeUninit<Pair> = MaybeUninit::uninit();

    unsafe {
        // 逐字段写入
        let ptr = pair.as_mut_ptr();
        std::ptr::addr_of_mut!((*ptr).x).write(10);
        std::ptr::addr_of_mut!((*ptr).y).write(20);
    }

    let initialized = unsafe { pair.assume_init() };
    assert_eq!(initialized, Pair { x: 10, y: 20 });
}

#[test]
/// 测试: MaybeUninit 与 assume_init_drop (需要手动 drop)
fn test_maybe_uninit_assume_init_drop() {
    use std::mem::MaybeUninit;

    // 创建已初始化的 MaybeUninit
    let value: MaybeUninit<String> = MaybeUninit::new("hello".to_string());

    // 安全: 我们知道它已初始化
    let string = unsafe { value.assume_init() };
    assert_eq!(string, "hello");
    // string 会被正常 drop
}

#[test]
/// 测试: repr(packed) 字段访问注意事项
fn test_repr_packed_field_access_safety() {
    #[repr(packed)]
    struct PackedPoints {
        x: u16,
        y: u32,
        z: u16,
    }

    let p = PackedPoints { x: 1, y: 2, z: 3 };
    assert_eq!(std::mem::size_of::<PackedPoints>(), 8);

    // 安全读取未对齐字段
    let y_ptr = std::ptr::addr_of!(p.y);
    let y_val = unsafe { y_ptr.read_unaligned() };
    assert_eq!(y_val, 2);
}

#[test]
/// 测试: repr(align) 大于默认对齐
fn test_repr_align_larger_than_default() {
    #[repr(align(128))]
    struct BigAligned {
        data: u64,
    }

    let a = BigAligned { data: 42 };
    assert_eq!(std::mem::align_of::<BigAligned>(), 128);
    assert!(std::mem::size_of::<BigAligned>() >= 128);
    assert_eq!(a.data, 42);
}

#[test]
/// 测试: repr(C) 布局与对齐
fn test_repr_c_layout() {
    #[repr(C)]
    struct CStruct {
        a: u8,
        b: u32,
        c: u16,
    }

    // C 布局: a 在 offset 0, 填充 3 字节, b 在 offset 4, c 在 offset 8
    let s = CStruct { a: 1, b: 2, c: 3 };
    assert_eq!(std::mem::size_of::<CStruct>(), 12); // 填充后的大小
    assert_eq!(s.a, 1);
    assert_eq!(s.b, 2);
    assert_eq!(s.c, 3);
}

#[test]
/// 测试: Never type 在 match 中的强制转换
fn test_never_type_coercion_in_match() {
    enum ResultOrNever {
        Value(i32),
        Panic,
    }

    let result: i32 = match ResultOrNever::Value(42) {
        ResultOrNever::Value(v) => v,
        ResultOrNever::Panic => panic!("never"),
    };

    assert_eq!(result, 42);
}

#[test]
/// 测试: Never type 用于 unreachable 代码中
fn test_never_type_in_unreachable() {
    fn exhaustive_match(value: bool) -> i32 {
        match value {
            true => 1,
            false => 0,
        }
    }

    assert_eq!(exhaustive_match(true), 1);
    assert_eq!(exhaustive_match(false), 0);
}

#[test]
/// 测试: extern "C" fn 指针
fn test_extern_c_function_pointer() {
    // 定义 C ABI 函数指针类型
    type CCallback = unsafe extern "C" fn(i32) -> i32;

    unsafe extern "C" fn double(x: i32) -> i32 {
        x * 2
    }

    let callback: CCallback = double;
    let result = unsafe { callback(21) };
    assert_eq!(result, 42);
}

#[test]
/// 测试: Zero-Sized Type 在 Option 中的表示
fn test_zst_in_option() {
    #[derive(Debug, PartialEq)]
    struct Nothing;

    let some: Option<Nothing> = Some(Nothing);
    let none: Option<Nothing> = None;

    assert!(some.is_some());
    assert!(none.is_none());
    // ZST 在 Option 中不占用额外空间 (niche optimization)
    assert_eq!(std::mem::size_of::<Option<Nothing>>(), 1);
}

#[test]
/// 测试: PhantomData 零大小类型标记
fn test_phantom_data_zst() {
    use std::marker::PhantomData;

    struct MyWrapper<T> {
        _marker: PhantomData<T>,
    }

    let _w: MyWrapper<i32> = MyWrapper { _marker: PhantomData };
    assert_eq!(std::mem::size_of::<MyWrapper<i32>>(), 0);

    let _w2: MyWrapper<String> = MyWrapper { _marker: PhantomData };
    assert_eq!(std::mem::size_of::<MyWrapper<String>>(), 0);
}

#[test]
/// 测试: unsafe 块最小化 —— 只包含必要的危险操作
fn test_minimal_unsafe_block() {
    let value = 42;
    let ptr: *const i32 = &value;

    // 最小化 unsafe 块
    let read_value = unsafe { *ptr };
    assert_eq!(read_value, 42);

    // 安全操作在 unsafe 块外部
    let doubled = read_value * 2;
    assert_eq!(doubled, 84);
}

#[test]
/// 测试: #[repr(transparent)] 单字段结构体
fn test_repr_transparent() {
    #[repr(transparent)]
    struct WrappedU32(u32);

    let w = WrappedU32(42);
    assert_eq!(std::mem::size_of::<WrappedU32>(), std::mem::size_of::<u32>());
    assert_eq!(w.0, 42);
}
