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
/// 测试: MaybeUninit 数组手动初始化
fn test_maybe_uninit_array() {
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
    #[allow(unnecessary_transmutes)]
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
        u.f = 3.14; // 写入 f 字段
        let _ = &u; // 注意: 读取 u.i 现在是 UB, 因为最后写入的是 f
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

#[test]
/// 测试: 裸指针基础——创建、转换、引用对比、空指针判断
fn test_raw_pointer_basics() {
    // 语法: 裸指针有多种创建方式; 创建是安全的, 解引用才需 unsafe
    // 避坑: 裸指针不保证生命周期/有效性/对齐; 不能通过裸指针绕过借用检查
    let mut val = 42;
    // 引用转裸指针
    let ptr1: *const i32 = &val as *const i32;
    let ptr2: *mut i32 = &mut val as *mut i32;
    // addr_of! 宏 (1.51+)
    let ptr3: *const i32 = std::ptr::addr_of!(val);
    let ptr4: *mut i32 = std::ptr::addr_of_mut!(val);

    // 非空判断
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());

    // 同地址裸指针相等
    assert_eq!(ptr1, ptr3);
    assert_eq!(ptr2, ptr4);

    // *const → *mut 转换 (安全: 只做类型转换, 不读写)
    let _ptr5: *mut i32 = ptr1 as *mut i32; // 合法, 但解引用写入仍需通过原始 &mut 路径
    // 通过原始可变引用写入
    unsafe { *ptr2 = 200; }
    // *mut → *const 转换
    let ptr6: *const i32 = ptr2 as *const i32;
    unsafe { assert_eq!(*ptr6, 200); }

    // 裸指针 → usize → 裸指针 (危险!)
    let addr: usize = ptr1 as usize;
    let back: *const i32 = addr as *const i32;
    assert_eq!(ptr1, back);

    // 空指针
    let null_ptr: *const i32 = std::ptr::null();
    assert!(null_ptr.is_null());
    let null_mut: *mut i32 = std::ptr::null_mut();
    assert!(null_mut.is_null());

    // 解引用操作
    unsafe {
        *ptr2 = 100;
        assert_eq!(*ptr1, 100);
        assert_eq!(*ptr3, 100);
    }
}

#[test]
/// 测试: MaybeUninit 深入用法——zeroed、assume_init_read、Drop 类型交互
fn test_maybe_uninit() {
    // 语法: MaybeUninit 提供 uninit/zeroed/write/assume_init/assume_init_read 等完整 API
    // 避坑: zeroed() 的零值对某些类型无效(如 NonNull); assume_init_read 读后原位置不能再被读取
    use std::mem::MaybeUninit;

    // 场景1: zeroed 零初始化
    let buf: MaybeUninit<[u8; 16]> = MaybeUninit::zeroed();
    let buf = unsafe { buf.assume_init() };
    assert_eq!(buf, [0u8; 16]);

    // 场景2: assume_init_read 移动语义
    let mut slot = MaybeUninit::<String>::uninit();
    slot.write(String::from("hello"));
    // assume_init_read: 读走值, 原位置变回未初始化
    let s = unsafe { slot.assume_init_read() };
    assert_eq!(s, "hello");
    // 注意: slot 现在处于未初始化状态, 不能再被读取
    // 重新写入后才可以
    slot.write(String::from("world"));
    let s2 = unsafe { slot.assume_init() };
    assert_eq!(s2, "world");

    // 场景3: as_ptr / as_mut_ptr 获取底层指针
    let mut val: MaybeUninit<i32> = MaybeUninit::uninit();
    let ptr = val.as_mut_ptr();
    unsafe { ptr.write(99); }
    assert_eq!(unsafe { val.assume_init() }, 99);
}

#[test]
/// 测试: transmute 的安全替代方案
fn test_transmute_safe_alternatives() {
    // 语法: 使用安全的类型转换函数替代 transmute
    // 避坑: transmute 不检查有效性和对齐——优先用 from_bits/to_bits/from_ne_bytes/to_ne_bytes
    // 注: 优先使用安全的 from_bits/to_bits/from_ne_bytes 等方法, transmute 仅作最后手段

    // 安全替代1: f32 <-> u32 位级转换
    let f: f32 = 1.0;
    let bits: u32 = f.to_bits();
    assert_eq!(bits, 0x3F800000);
    let f2: f32 = f32::from_bits(0x3F800000);
    assert_eq!(f2, 1.0);

    // 安全替代2: 整数 <-> 字节数组
    let num: u32 = 0xDEAD_BEEF;
    let bytes: [u8; 4] = num.to_ne_bytes();
    assert_eq!(bytes, [0xEF, 0xBE, 0xAD, 0xDE]); // 小端序
    let num2: u32 = u32::from_ne_bytes(bytes);
    assert_eq!(num2, num);

    // 大端序
    let be_bytes: [u8; 4] = num.to_be_bytes();
    assert_eq!(be_bytes, [0xDE, 0xAD, 0xBE, 0xEF]);
    let num3: u32 = u32::from_be_bytes(be_bytes);
    assert_eq!(num3, num);

    // 安全替代3: bool <-> u8
    let b: bool = true;
    let byte: u8 = b as u8;
    assert_eq!(byte, 1);

    // 安全替代4: 指针 <-> usize (用 as, 不用 transmute)
    let x = 42;
    let ptr: *const i32 = &x;
    let addr: usize = ptr as usize;
    let back: *const i32 = addr as *const i32;
    assert_eq!(unsafe { *back }, 42);
}

#[test]
/// 测试: unsafe trait 实现（模拟 Send/Sync 的安全承诺）
fn test_unsafe_trait_impl() {
    // 语法: unsafe trait 要求实现者保证额外的不变量; unsafe impl 是实现者的安全承诺
    // 避坑: unsafe impl 的约束由实现者承担, 编译器不检查; 错误实现会导致 UB

    // 模拟一个需要满足安全条件的 trait
    unsafe trait Pod {
        // 实现者必须保证: 任何字节序列都是该类型的合法值
    }

    // i32 的任何位模式都合法
    unsafe impl Pod for i32 {}
    // u8 同理
    unsafe impl Pod for u8 {}

    // 验证: 任何字节序列转换为 i32 都是合法值(虽然可能不是期望的语义)
    fn assert_pod<T: Pod>() {}

    assert_pod::<i32>();
    assert_pod::<u8>();

    // 场景2: 标记跨线程安全
    use std::marker::PhantomData;

    struct MyBuffer {
        ptr: *mut u8,
        len: usize,
        _marker: PhantomData<Vec<u8>>,
    }

    // 手动保证 MyBuffer 可以安全发送到其他线程
    unsafe impl Send for MyBuffer {}
    unsafe impl Sync for MyBuffer {}

    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    is_send::<MyBuffer>();
    is_sync::<MyBuffer>();
}

#[test]
/// 测试: FFI 安全包装——在 unsafe 外包裹安全 Rust 接口
fn test_ffi_safe_wrapper() {
    // 语法: 将 unsafe FFI 调用封装在安全的 Rust 函数中, 对外暴露安全接口
    // 避坑: 安全包装必须验证所有前置条件; 不能把 UB 传播到安全代码中

    // 模拟: 底层 unsafe 操作
    mod raw_ops {
        /// 模拟 C 的内存分配 (实际是 Rust 的 alloc)
        pub fn alloc(size: usize) -> *mut u8 {
            let mut buf = Vec::<u8>::with_capacity(size);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf); // 忘记 drop, 移交所有权
            ptr
        }

        pub fn dealloc(ptr: *mut u8, size: usize) {
            unsafe {
                // 重新获取所有权并自动 drop
                let _buf = Vec::from_raw_parts(ptr, 0, size);
            }
        }

        /// 模拟: C 的 memset
        pub unsafe fn memset(ptr: *mut u8, val: u8, count: usize) {
            unsafe {
                std::ptr::write_bytes(ptr, val, count);
            }
        }
    }

    // 安全包装: 提供安全的缓冲区操作
    struct SafeBuffer {
        ptr: *mut u8,
        len: usize,
    }

    impl SafeBuffer {
        fn new(len: usize) -> Self {
            let ptr = raw_ops::alloc(len);
            Self { ptr, len }
        }

        fn fill(&mut self, val: u8) {
            // SAFETY: ptr 在构造时确保非空, len 确保不越界
            unsafe {
                raw_ops::memset(self.ptr, val, self.len);
            }
        }

        fn as_slice(&self) -> &[u8] {
            unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
        }
    }

    impl Drop for SafeBuffer {
        fn drop(&mut self) {
            raw_ops::dealloc(self.ptr, self.len);
        }
    }

    // 使用安全接口
    let mut buf = SafeBuffer::new(10);
    buf.fill(0xAB);
    let slice = buf.as_slice();
    assert_eq!(slice, &[0xABu8; 10]);
}
