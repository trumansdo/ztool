// ---------------------------------------------------------------------------
// 4.4 Unsafe 编程
// ---------------------------------------------------------------------------

#[test]
/// 测试: Edition 2024 unsafe fn 内 unsafe 块要求
fn test_unsafe_op_in_unsafe_fn() {
    // 语法: Edition 2024 中 unsafe fn 内的 unsafe 操作仍需 unsafe 块
    // 避坑: 旧版 Edition 中 unsafe fn 内操作自动 unsafe; 迁移后需补加 unsafe 块
    unsafe fn unsafe_operation(ptr: *const i32) -> i32 {
        unsafe { *ptr }
    }
    let val = 123;
    let ptr = &val as *const i32;
    assert_eq!(unsafe { unsafe_operation(ptr) }, 123);
}

#[test]
/// 测试: 调试模式空指针解引用 panic (1.86+)
fn test_pointer_null_debug_assertions() {
    // 语法: 1.86+ 调试模式下, 解引用空指针会触发 panic
    // 避坑: 仅在 debug assertions 启用时生效; release 模式仍是 UB
}

#[test]
/// 测试: UnsafeCell 内部可变性基石
fn test_unsafe_cell_single_thread() {
    // 语法: UnsafeCell<T> 是内部可变性的基石, Cell/RefCell 都基于它
    // 避坑: UnsafeCell 不实现 Sync, 不能跨线程共享; 访问必须 unsafe; 需自行保证别名安全
    use std::cell::UnsafeCell;
    let cell: UnsafeCell<i32> = UnsafeCell::new(0);
    unsafe {
        assert_eq!(*cell.get(), 0);
        *cell.get() = 42;
        assert_eq!(*cell.get(), 42);
    }
}

#[test]
/// 测试: AtomicI32 无锁原子操作和 static 变量
fn test_atomic_static() {
    // 语法: AtomicI32 提供无锁原子操作, 可安全用于 static 变量
    // 避坑: 原子操作有内存序(Ordering)选择, SeqCst 最安全但最慢; 不支持 128 位原子操作(部分平台)
    use std::sync::atomic::{AtomicI32, Ordering};
    static COUNTER: AtomicI32 = AtomicI32::new(0);
    assert_eq!(COUNTER.load(Ordering::SeqCst), 0);
    COUNTER.store(42, Ordering::SeqCst);
    assert_eq!(COUNTER.load(Ordering::SeqCst), 42);
}

#[test]
/// 测试: 裸指针基础 (*const T / *mut T)
fn test_raw_pointers() {
    // 语法: *const T (不可变裸指针) 和 *mut T (可变裸指针)
    // 避坑: 裸指针不保证有效性/对齐/非空; 不能通过裸指针绕过借用检查
    //       裸指针不自动释放内存; 创建裸指针是安全的, 解引用才需要 unsafe
    let mut x = 42;
    let ptr_const: *const i32 = &x as *const i32;
    let ptr_mut: *mut i32 = &mut x as *mut i32;

    // 创建裸指针是安全的
    assert!(!ptr_const.is_null());
    assert!(!ptr_mut.is_null());

    // 解引用需要 unsafe
    unsafe {
        assert_eq!(*ptr_const, 42);
        *ptr_mut = 100;
        assert_eq!(*ptr_const, 100);
    }
}

#[test]
/// 测试: 指针算术与 offset
fn test_pointer_arithmetic() {
    // 语法: ptr.offset(n) / ptr.add(n) / ptr.sub(n) 进行指针算术
    // 避坑: 指针算术必须在同一分配对象内; 不能越界; wrapping_offset 允许溢出
    let arr = [10, 20, 30, 40, 50];
    let base = arr.as_ptr();

    unsafe {
        assert_eq!(*base.offset(0), 10);
        assert_eq!(*base.offset(2), 30);
        assert_eq!(*base.add(4), 50);

        // 指针相减
        let end = base.add(5);
        assert_eq!(end.offset_from(base), 5);
    }
}

#[test]
/// 测试: std::ptr::read / write / copy
fn test_ptr_read_write_copy() {
    // 语法:
    //   - ptr::read(ptr)    按位复制值, 不运行 drop (move 语义)
    //   - ptr::write(ptr, v) 按位写入值, 不 drop 旧值
    //   - ptr::copy(src, dst, count) 内存拷贝 (类似 memcpy)
    // 避坑: read 后原位置的值不应再被使用(双重 drop); write 会覆盖旧值而不 drop
    use std::ptr;

    let mut x = 42;
    let ptr = &mut x as *mut i32;

    unsafe {
        // read 移动值
        let val = ptr::read(ptr);
        assert_eq!(val, 42);

        // write 写入新值 (不 drop 旧值)
        ptr::write(ptr, 100);
        assert_eq!(*ptr, 100);
    }

    // copy (内存拷贝)
    let src = [1, 2, 3, 4, 5];
    let mut dst = [0; 5];
    unsafe {
        ptr::copy(src.as_ptr(), dst.as_mut_ptr(), 3);
    }
    assert_eq!(dst, [1, 2, 3, 0, 0]);
}

#[test]
/// 测试: MaybeUninit 未初始化内存
fn test_maybe_uninit() {
    // 语法: MaybeUninit<T> 表示可能未初始化的内存, 用于手动初始化
    // 避坑: 读取未初始化的值是 UB; 必须确保初始化后才能 assume_init()
    //       不要对 MaybeUninit<Drop 类型> 使用 assume_init 如果未初始化
    use std::mem::MaybeUninit;

    // 场景1: 数组初始化 (使用 transmute 替代 unstable array_assume_init)
    let mut arr: [MaybeUninit<i32>; 5] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..5 {
        arr[i] = MaybeUninit::new(i as i32 * 10);
    }
    let arr: [i32; 5] = unsafe { std::mem::transmute::<_, [i32; 5]>(arr) };
    assert_eq!(arr, [0, 10, 20, 30, 40]);

    // 场景2: 单个值初始化
    let mut uninit = MaybeUninit::<String>::uninit();
    unsafe {
        uninit.write(String::from("hello"));
        let s = uninit.assume_init_read();
        assert_eq!(s, "hello");
    }
}

#[test]
/// 测试: std::mem::transmute 类型转换
fn test_mem_transmute() {
    // 语法: transmute<T, U>(t) 将 T 按位 reinterpret 为 U
    // 避坑: 极度危险! 大小必须相同; 不检查有效性; 优先用 as 或 from_bits
    //       transmute 是最后的 resort, 99% 的情况有更好的替代方案
    use std::mem;

    // 安全替代: f32 <-> u32 用 to_bits/from_bits
    let f: f32 = 1.0;
    let bits: u32 = f.to_bits();
    assert_eq!(bits, 0x3F800000);
    let f2 = f32::from_bits(bits);
    assert_eq!(f2, 1.0);

    // transmute 示例 (仅演示, 实际应使用 from_bits)
    let bits: u32 = unsafe { mem::transmute::<f32, u32>(1.0) };
    assert_eq!(bits, 0x3F800000);
}

#[test]
/// 测试: union 联合体
fn test_union() {
    // 语法: union 所有字段共享同一内存, 写入一个字段后读取另一个字段是 unsafe
    // 避坑: union 字段不能实现 Drop; 读取非最后写入的字段是 UB; 需要手动跟踪活跃字段
    #[repr(C)]
    union IntOrFloat {
        i: i32,
        f: f32,
    }

    let mut u = IntOrFloat { i: 42 };
    unsafe {
        assert_eq!(u.i, 42);
        u.f = 3.14;
        // 注意: 读取 u.i 现在是 UB, 因为最后写入的是 f
    }
}

#[test]
/// 测试: NonNull<T> 非空裸指针
fn test_non_null() {
    // 语法: NonNull<T> 保证非空的 *mut T, 可用于 Option<NonNull<T>> 优化 (和裸指针一样大小)
    // 避坑: NonNull 不保证指向有效内存; 不实现 Deref; 协变于 T
    use std::ptr::NonNull;

    let mut x = 42;
    let ptr = NonNull::new(&mut x as *mut i32).unwrap();

    unsafe {
        assert_eq!(*ptr.as_ptr(), 42);
        *ptr.as_ptr() = 100;
        assert_eq!(x, 100);
    }

    // Option<NonNull<T>> 和 *mut T 同样大小 (空指针优化)
    assert_eq!(std::mem::size_of::<Option<NonNull<i32>>>(), std::mem::size_of::<*mut i32>());
}

#[test]
/// 测试: unsafe trait 和 unsafe impl
fn test_unsafe_trait() {
    // 语法: unsafe trait 要求实现者保证某些不变量; unsafe impl 声明已满足
    // 避坑: unsafe trait 的不变量由实现者保证, 使用者可以安全调用
    unsafe trait SafeToSend {
        // 实现者必须保证此类型可以安全地发送到其他线程
    }

    struct MyType(i32);

    // 实现者声明 MyType 可以安全发送
    unsafe impl SafeToSend for MyType {}

    // 使用者可以安全地依赖这个保证
    fn assert_safe_to_send<T: SafeToSend>() {}
    assert_safe_to_send::<MyType>();
}
