// ---------------------------------------------------------------------------
// 1.1 所有权与借用
// ---------------------------------------------------------------------------

#[test]
/// 测试: 所有权规则 (每个值只有一个所有者/离开作用域时被 drop)
fn test_ownership_rules() {
    // 语法: Rust 的所有权系统
    //
    // 三条核心规则:
    //   1. 每个值都有一个所有者(owner)
    //   2. 同一时间只有一个所有者
    //   3. 所有者离开作用域时值被自动丢弃(drop)
    //
    // 避坑:
    //   - String 和 &str 的所有权行为不同
    //   - 离开作用域自动调用 drop, 不要手动释放
    //
    let s = String::from("hello");
    assert_eq!(s, "hello");
    // s 在此被 drop

    // 边界: 嵌套作用域中的所有权
    {
        let inner = String::from("world");
        assert_eq!(inner, "world");
    } // inner 在此被 drop, 外部的 s 不受影响

    // 边界: 变量遮蔽（shadowing）不影响所有权逻辑
    let _s = String::from("shadowed");
    let s = String::from("reshadowed"); // 前一个 _s 被 drop
    assert_eq!(s, "reshadowed");

    // 边界: 基本类型在作用域结束时也被 drop, 但无堆内存释放（平凡的）
    {
        let _n = 42_i32;
    } // 栈上数据弹出, 无特殊清理逻辑
}

#[test]
/// 测试: Move 语义 (堆类型赋值转移所有权)
fn test_move_semantics() {
    // 语法: 堆类型赋值时转移所有权(浅拷贝栈数据+使原变量失效)
    //
    // 区分:
    //   - Copy: 栈上数据按位复制, 原变量仍有效
    //   - Move: 堆数据浅拷贝指针, 原变量失效
    //
    // 避坑:
    //   - Move 后原变量不能再使用
    //   - 函数传参也会转移所有权
    //   - 返回值可以转回所有权
    //
    let s1 = String::from("hello");
    let s2 = s1;
    assert_eq!(s2, "hello");
    // assert_eq!(s1, "hello"); // 编译错误: s1 已被 move

    // 基本类型(Copy)的赋值不会 move
    let x = 5;
    let y = x;
    assert_eq!(x, 5); // OK: i32 实现了 Copy
    assert_eq!(y, 5);

    // 边界: Vec<T> 也是 move 语义
    let v1 = vec![1, 2, 3];
    let v2 = v1;
    assert_eq!(v2.len(), 3);
    // assert_eq!(v1.len(), 3); // 编译错误

    // 边界: Box<T> 是 move 语义
    let b1 = Box::new(42);
    let b2 = b1;
    assert_eq!(*b2, 42);
    // assert_eq!(*b1, 42); // 编译错误

    // 边界: 元组中包含 move 类型时整个元组是 move 语义
    let t1 = (String::from("a"), String::from("b"));
    let t2 = t1;
    assert_eq!(t2.0, "a");
    // assert_eq!(t1.0, "a"); // 编译错误

    // 边界: [T; N] 中 T 是 Copy 则数组是 Copy, 否则是 move
    let arr1 = [1, 2, 3];       // [i32; 3] 是 Copy
    let arr2 = arr1;
    assert_eq!(arr1[0], 1);      // OK: 数组实现了 Copy
    assert_eq!(arr2[0], 1);

    // 边界: Box 解引用移动（deref move）
    let b = Box::new(String::from("boxed"));
    let s: String = *b;          // 所有权从 Box 转移到 s, b 失效
    assert_eq!(s, "boxed");
    // let _ = *b;               // 编译错误: b 已被移动
}

#[test]
/// 测试: 函数传参的所有权转移
fn test_function_move() {
    // 语法: 函数参数会获取变量的所有权, 返回值可以归还
    //
    // 避坑:
    //   - 函数接收堆类型后, 原变量失效
    //   - 使用引用(&)避免所有权转移
    //
    fn takes_ownership(s: String) -> String {
        s
    }

    fn makes_copy(x: i32) -> i32 {
        x
    }

    let s = String::from("hello");
    let s = takes_ownership(s); // 所有权转回
    assert_eq!(s, "hello");

    let n = 5;
    let _ = makes_copy(n);
    assert_eq!(n, 5); // OK: i32 是 Copy

    // 边界: Vec<T> 传参
    fn take_vec(v: Vec<i32>) -> Vec<i32> {
        v
    }
    let v = vec![1, 2, 3];
    let v = take_vec(v);
    assert_eq!(v.len(), 3);

    // 边界: 多参数函数同时接收 move 和 copy 参数
    fn mixed(s: String, n: i32) -> (String, i32) {
        (s, n)
    }
    let s = String::from("mixed");
    let n = 10;
    let (s_back, n_back) = mixed(s, n);
    assert_eq!(s_back, "mixed");
    assert_eq!(n_back, 10);
    assert_eq!(n, 10); // OK: n 是 Copy

    // 边界: 将 move 类型传给接收引用的函数——需要先创建引用
    fn takes_ref(_s: &String) {}
    let s = String::from("ref");
    takes_ref(&s);
    assert_eq!(s, "ref"); // s 所有权未转移
}

#[test]
/// 测试: 不可变引用 (&T 借用/不获取所有权)
fn test_immutable_references() {
    // 语法: &T 创建不可变引用, 借用但不获取所有权
    //
    // 特性:
    //   - 可以同时有多个不可变引用
    //   - 引用离开作用域时不 drop 指向的值
    //   - 引用必须始终有效 (no dangling references)
    //
    // 避坑:
    //   - 引用不能比指向的值活得更久
    //   - 不可变引用不能修改值
    //
    fn calculate_length(s: &String) -> usize {
        s.len()
    }

    let s = String::from("hello");
    let len = calculate_length(&s);
    assert_eq!(len, 5);
    assert_eq!(s, "hello"); // s 仍然可用

    // 边界: 多个不可变引用共存
    let r1 = &s;
    let r2 = &s;
    let r3 = &s;
    assert_eq!(*r1, "hello");
    assert_eq!(*r2, "hello");
    assert_eq!(*r3, "hello");

    // 边界: 不可变引用不能修改值
    // let r = &s;
    // r.push_str("!"); // 编译错误: 不能通过不可变引用修改

    // 边界: 引用传递给函数（自动再借用）
    fn get_len(s: &str) -> usize {
        s.len()
    }
    assert_eq!(get_len(&s), 5);
    assert_eq!(get_len(r1), 5); // &String 自动强制为 &str

    // 边界: 引用类型的引用
    let r = &s;
    let rr: &&String = &r;
    assert_eq!(**rr, "hello");

    // 边界: 引用作为 &T 本身实现了 Copy
    let r1 = &s;
    let r2 = r1;   // 引用是 Copy, r1 仍可用
    assert_eq!(*r1, "hello");
    assert_eq!(*r2, "hello");
}

#[test]
/// 测试: 可变引用 (&mut T/修改借用的值)
fn test_mutable_references() {
    // 语法: &mut T 创建可变引用, 可以修改值
    //
    // 规则:
    //   - 同一时间只能有一个可变引用
    //   - 有可变引用时不能有不可变引用
    //
    // 避坑:
    //   - 可变引用的作用域从创建到最后一次使用
    //   - 不可变引用的作用域结束后才能创建可变引用
    //
    let mut s = String::from("hello");

    fn change(s: &mut String) {
        s.push_str(", world");
    }

    change(&mut s);
    assert_eq!(s, "hello, world");

    // 边界: 通过可变引用替换整个值
    let mut s = String::from("old");
    let r = &mut s;
    *r = String::from("new");
    assert_eq!(s, "new");

    // 边界: swap 模式 —— 用两个可变引用交换值需要 unsafe 或 split_at_mut
    let mut v = vec![1, 2, 3, 4, 5];
    let (left, right) = v.split_at_mut(2);
    left[0] = 10;
    right[0] = 20;
    assert_eq!(v, vec![10, 2, 20, 4, 5]);

    // 边界: 可变引用不能同时存在多个
    // let r1 = &mut s;
    // let r2 = &mut s; // 编译错误: 不能同时有两个可变借用

    // 边界: 有可变引用时不能读取原始变量
    let mut s = String::from("test");
    let r = &mut s;
    r.push_str("!");
    // println!("{}", s); // 编译错误: 原始变量被可变借用"冻结"
    assert_eq!(r, "test!");
}

#[test]
/// 测试: 借用规则 (多个不可变/单个可变/作用域)
fn test_borrowing_rules() {
    // 语法: 借用规则防止数据竞争
    //
    // 规则:
    //   - 多个不可变引用可以共存 (&T)
    //   - 同一时间只能有一个可变引用 (&mut T)
    //   - 不可变和可变引用不能同时存在
    //
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    assert_eq!(r1, "hello");
    assert_eq!(r2, "hello");
    // r1, r2 在此之后不再使用

    let r3 = &mut s;
    r3.push_str(" world");
    assert_eq!(r3, "hello world");

    // 边界: NLL 让借用更灵活 —— 不可变引用在最后一次使用后自动失效
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    assert_eq!(*first, 1);    // first 最后一次使用
    // first 在此后不再存活
    v.push(4);               // OK: NLL 知道 first 已不再使用
    assert_eq!(v, vec![1, 2, 3, 4]);

    // 边界: 不可变引用作为函数参数传递时也算"使用"
    let mut s = String::from("hello");
    let r = &s;
    fn read_only(_: &String) {}
    read_only(r);            // r 最后一次使用
    let _rm = &mut s;       // OK: NLL 下通过

    // 边界: 新绑定的可变借用可以与之前的不可变借用共存, 只要不重叠存活
    let mut s = String::from("borrow");
    {
        let _r1 = &s;
        let _r2 = &s;
    }
    let _r3 = &mut s;  // OK: r1 和 r2 的作用域已结束
}

#[test]
/// 测试: 悬垂引用 (编译器阻止)
fn test_dangling_references() {
    // 语法: Rust 编译器保证引用始终有效
    //
    // 避坑:
    //   - 函数不能返回局部变量的引用
    //   - 编译器通过生命周期检查阻止悬垂引用
    //

    // 以下代码无法编译:
    // fn dangle() -> &String {
    //     let s = String::from("hello");
    //     &s
    // }

    // 正确做法: 返回 String 本身
    fn no_dangle() -> String {
        let s = String::from("hello");
        s
    }

    assert_eq!(no_dangle(), "hello");

    // 边界: 在嵌套作用域中创建的引用不能逃逸到外部
    // let r;
    // {
    //     let s = String::from("temp");
    //     r = &s;  // 编译错误: s 的生命周期不够长
    // }
    // println!("{}", r);

    // 边界: 通过参数传递的引用可以返回（因为生命周期来自调用者）
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }
    let s1 = String::from("short");
    let s2 = String::from("longer");
    let result = longest(&s1, &s2);
    assert_eq!(result, "longer");

    // 边界: 返回静态引用总是安全的
    fn static_ref() -> &'static str {
        "hello"
    }
    assert_eq!(static_ref(), "hello");
}

#[test]
/// 测试: 切片 (字符串切片/数组切片)
fn test_slices() {
    // 语法: 切片是连续内存区域一部分数据的引用, 不拥有数据
    //
    // 字符串切片:
    //   - &s[0..5]    字节索引范围 (必须在字符边界!)
    //
    // 避坑:
    //   - 切片索引必须是有效 UTF-8 字符边界
    //   - 切片返回 &str, 不是 String
    //   - 切片长度在编译时未知
    //
    let s = String::from("hello world");
    let hello = &s[0..5];
    let world = &s[6..11];
    assert_eq!(hello, "hello");
    assert_eq!(world, "world");

    // 简写形式
    assert_eq!(&s[..5], "hello");
    assert_eq!(&s[6..], "world");
    assert_eq!(&s[..], "hello world");

    // 数组切片
    let a = [1, 2, 3, 4, 5];
    let slice = &a[1..3];
    assert_eq!(slice, &[2, 3][..]);

    // 边界: 空切片
    let empty_arr: &[i32] = &[];
    assert_eq!(empty_arr.len(), 0);

    // 边界: 切片的切片
    let a = [1, 2, 3, 4, 5, 6, 7, 8];
    let s1 = &a[1..7];
    let s2 = &s1[2..4]; // &[4, 5]
    assert_eq!(s2, &[4, 5]);

    // 边界: 切片分界 (split_at)
    let a = [1, 2, 3, 4];
    let (left, right) = a.split_at(2);
    assert_eq!(left, &[1, 2]);
    assert_eq!(right, &[3, 4]);

    // 边界: 从 Vec 创建切片
    let v = vec![10, 20, 30, 40, 50];
    let slice = &v[1..4];
    assert_eq!(slice, &[20, 30, 40]);
    assert_eq!(v.len(), 5);  // v 仍可用 (切片只是借用)

    // 边界: 字符串中包含非 ASCII 多字节字符时切片必须在字符边界上
    let s = String::from("你好世界");
    // 中文字符每个占 3 字节
    let ni = &s[0..3];  // "你" (字节 0-3)
    assert_eq!(ni, "你");
    // &s[0..1];  // panic! 不在 UTF-8 字符边界
}

#[test]
/// 测试: Copy 与 Clone 的区别
fn test_copy_vs_clone() {
    // 语法:
    //   - Copy: 栈上数据的按位复制, 自动发生
    //   - Clone: 堆数据的深拷贝, 显式调用 .clone()
    //
    // 实现了 Copy 的类型:
    //   整数/浮点/布尔/字符/仅含 Copy 类型的元组
    //
    // 避坑:
    //   - String 不是 Copy, 是 Clone
    //   - Clone 可能开销很大
    //   - 不要在性能敏感路径随意 clone
    //
    let x = 5;
    let y = x; // Copy
    assert_eq!(x, 5);
    assert_eq!(y, 5);

    let s1 = String::from("hello");
    let s2 = s1.clone(); // 深拷贝
    assert_eq!(s1, s2); // 都有效

    // 边界: Clone 后的值是独立的
    let mut s1 = String::from("hello");
    let s2 = s1.clone();
    s1.push_str(" world");
    assert_eq!(s2, "hello");     // s2 不受 s1 修改的影响
    assert_eq!(s1, "hello world");

    // 边界: Vec 的 Clone 也是深拷贝
    let mut v1 = vec![1, 2, 3];
    let v2 = v1.clone();
    v1.push(4);
    assert_eq!(v2, vec![1, 2, 3]); // v2 不变

    // 边界: 引用类型实现了 Copy (不需要显式 Clone)
    let s = String::from("ref");
    let r1 = &s;
    let r2 = r1; // Copy, r1 仍可用
    assert_eq!(*r1, "ref");
    assert_eq!(*r2, "ref");
}

#[test]
/// 测试: 常见借用模式 (函数参数/返回值)
fn test_borrowing_patterns() {
    // 语法: 优先使用引用作为函数参数
    //
    // 模式:
    //   - &str 比 &String 更通用 (Deref 自动转换)
    //   - 避免不必要的 clone
    //   - 使用引用遍历集合
    //
    fn process(s: &str) -> usize {
        s.len()
    }

    let s = String::from("hello");
    let len = process(&s); // &String 自动转为 &str
    assert_eq!(len, 5);

    // 避免不必要 clone
    let data = vec![1, 2, 3];
    let mut results = Vec::new();
    for item in &data {
        results.push(item * 2);
    }
    assert_eq!(data.len(), 3); // data 仍可用
    assert_eq!(results, vec![2, 4, 6]);

    // 边界: 使用 .iter() 和 .iter_mut()
    let mut v = vec![10, 20, 30];
    for x in v.iter() {
        assert!(*x >= 10);
    }
    assert_eq!(v.len(), 3); // v 仍可用

    for x in v.iter_mut() {
        *x *= 2;
    }
    assert_eq!(v, vec![20, 40, 60]);

    // 边界: 按值迭代（消耗集合）
    let v = vec![1, 2, 3];
    let mut sum = 0;
    for x in v {   // v 被消耗
        sum += x;
    }
    assert_eq!(sum, 6);
    // assert_eq!(v.len(), 3); // 编译错误: v 已被移动

    // 边界: &Option<T> 模式 —— Some(ref x) 借用而不获取所有权
    let opt = Some(String::from("hello"));
    if let Some(ref s) = opt {
        assert_eq!(s, "hello");
    }
    assert!(opt.is_some()); // opt 仍可用

    // 边界: 返回引用需要标注生命周期
    fn pick_first<'a>(a: &'a str, _b: &str) -> &'a str {
        a
    }
    let s1 = String::from("first");
    let s2 = String::from("second");
    assert_eq!(pick_first(&s1, &s2), "first");
}

// ---------------------------------------------------------------------------
// 新增测试: 部分移动、再借用、NLL、闭包、多所有权等
// ---------------------------------------------------------------------------

#[test]
/// 测试: 部分移动 (Partial Move) —— 从结构体中只移动部分字段
fn test_partial_move() {
    // 语法: Rust 允许从一个结构体中只移动部分字段
    //
    // 规则:
    //   - 被移动的字段不可再用
    //   - 未移动的 Copy 字段仍可用
    //   - 部分移动后整体结构体不能再使用 (有例外: 仅移动 Copy 字段不影响)
    //
    struct Person {
        name: String,
        age: u32,
        metadata: String,
    }

    let p = Person {
        name: String::from("Alice"),
        age: 30,
        metadata: String::from("developer"),
    };

    // 只移动 name
    let name = p.name;
    assert_eq!(name, "Alice");

    // age 是 Copy 类型, 仍可访问
    assert_eq!(p.age, 30);

    // 整体结构体不能再使用 (因为 name 已被移动)
    // let p2 = p; // 编译错误: p 被部分移动, 不能整体使用

    // 但未被移动的非 Copy 字段仍可用
    // metadata 未被移动
    // println!("{}", p.metadata); // 实际上这也会编译错误,
    // 因为部分移动使结构体"损坏", 访问未移动的字段也取决于编译器版本

    // 正确的部分移动模式: 模式匹配解构
    let p2 = Person {
        name: String::from("Bob"),
        age: 25,
        metadata: String::from("designer"),
    };

    // 使用模式匹配同时移动 name 和 metadata
    let Person { name, age, metadata } = p2;
    assert_eq!(name, "Bob");
    assert_eq!(age, 25);
    assert_eq!(metadata, "designer");

    // 边界: 更灵活的部分移动 —— 使用 ref
    let p3 = Person {
        name: String::from("Carol"),
        age: 28,
        metadata: String::from("manager"),
    };

    // 只借用 age, 移动 name
    let Person { name, ref age, .. } = p3;
    assert_eq!(name, "Carol");
    assert_eq!(*age, 28);
    // p3.age 仍可用 (因为是 Copy + ref 借用)
    // p3.metadata 仍可用 (未被移动)
    // 但 p3 整体不能使用, 因为 name 已被移动

    // 边界: 元组的部分移动
    let t = (String::from("a"), String::from("b"), 42);
    let (x, _, _) = t;
    assert_eq!(x, "a");

    // 不能这样做:
    // let a = t.0;
    // println!("{}", t.1); // 编译错误: t 被部分移动
}

#[test]
/// 测试: 引用再借用 (Reborrowing) —— &mut *ref 的自动发生
fn test_reborrowing() {
    // 语法: 当函数接收 &mut T 时, 已有 &mut T 会被自动再借用
    //
    // 原理:
    //   - 从 &mut T 可以创建新的 &mut T (再借用)
    //   - 再借用期间, 原引用暂时不可用
    //   - 再借用结束后, 原引用恢复可用
    //
    fn add_one(r: &mut i32) {
        *r += 1;
    }

    let mut x = 42;
    let r = &mut x;
    add_one(r);        // r 被自动再借用给 add_one
    add_one(r);        // 可以多次再借用, 因为每次再借用都会归还
    *r += 1;           // r 本身仍然可用
    assert_eq!(*r, 45);
    assert_eq!(x, 45);

    // 边界: 多次再借用的生命周期不重叠
    let mut x = 10;
    let r = &mut x;
    add_one(r); // r 被再借用, add_one 返回后归还
    add_one(r); // 再次借用 (上次已归还)
    assert_eq!(*r, 12);

    // 边界: 手动再借用 &mut *r
    let mut x = 100;
    let r = &mut x;
    {
        let r2 = &mut *r;  // 手动再借用
        *r2 += 50;
    } // r2 离开作用域, r 恢复
    *r += 1;
    assert_eq!(*r, 151);

    // 边界: 再借用为不可变引用
    let mut x = 20;
    let r = &mut x;
    {
        let r2 = &*r;       // 不可变再借用
        let r3 = &*r;       // 多个不可变再借用可以共存
        assert_eq!(*r2, 20);
        assert_eq!(*r3, 20);
    } // r2, r3 离开作用域
    *r += 1;               // 可变引用恢复
    assert_eq!(*r, 21);

    // 边界: 再借用 Vec 元素
    let mut v = vec![1, 2, 3];
    let r = &mut v;
    r.push(4);             // 自动再借用

    fn push_item(v: &mut Vec<i32>) {
        v.push(99);
    }
    push_item(r);          // 再次自动再借用
    assert_eq!(*r, vec![1, 2, 3, 4, 99]);
}

#[test]
/// 测试: NLL (非词法生命周期) 行为 —— 引用在最后一次使用后即失效
fn test_nll_behavior() {
    // 语法: Rust 2018+ 的 NLL 让借用检查更精确
    //
    // 核心: 引用的实际生命周期在最后一次使用处结束, 而非词法作用域末尾
    //
    // 以下代码在 Rust 2015 (词法生命周期) 中无法编译, 在 2018+ 中通过

    // 场景1: 不可变引用在使用后可以被可变引用替代
    let mut s = String::from("hello");
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);  // r1, r2 最后一次使用
    // NLL: r1 和 r2 在此之后不再存活
    let r3 = &mut s;            // OK: r1/r2 已"死亡"
    r3.push_str(", world");
    assert_eq!(r3, "hello, world");

    // 场景2: 可变借用用于函数调用后再借用
    let mut v = vec![1, 2, 3];
    let r = &mut v;
    r.push(4);          // r 使用了

    fn read_vec(_v: &Vec<i32>) -> usize {
        1
    }
    // 在 r 的最后一次使用后...
    let _len = read_vec(&v);  // OK: NLL 知道 r 已不再需要 (push 是最后一次使用)

    // 实际上 r 仍在作用域...让我组织下测试
    // 这里需要注意: NLL 判断"最后使用"是基于控制流分析的

    // 场景3: if 分支中的借用生命周期不同
    let mut x = 10;
    let r = &x;

    if *r > 5 {
        // r 在此分支中使用了
        assert!(*r > 5);
    }
    // r 在 if 后不再使用
    // NLL 可以推断 r 不再存活

    let r_mut = &mut x;  // OK: NLL
    *r_mut = 20;
    assert_eq!(*r_mut, 20);

    // 场景4: 循环中的 NLL
    let mut v = vec![1, 2, 3];
    // 在循环中, 每次迭代重新借用是允许的
    let r = &mut v;
    r.push(4);
    // r 在 push 后最后一次使用
    // 下面再借用一个不可变引用
    let r2 = &v;         // OK: NLL
    assert_eq!(r2.len(), 4);

    // 场景5: 证明引用确实在最后一次使用处结束
    let mut s = String::from("test");
    let _r = &s;
    // _r 从未被使用 -> 立即失效
    let _rm = &mut s;    // OK: _r 未被使用, 立即失效
    _rm.push_str("!");

    // 场景6: match 中的 NLL (这是 NLL 改进的经典场景)
    let mut opt = Some(String::from("hello"));
    match &opt {
        Some(s) => assert_eq!(s, "hello"),
        None => unreachable!(),
    }
    // match 后不可变借用已结束
    let ref_mut = &mut opt;
    *ref_mut = None;
    assert!(opt.is_none());
}

#[test]
/// 测试: 所有权与闭包 —— move 闭包的所有权转移
fn test_ownership_with_closures() {
    // 语法: 闭包可以通过三种方式捕获环境变量
    //
    //   - &T: 不可变借用 -> Fn trait
    //   - &mut T: 可变借用   -> FnMut trait
    //   - T: 获取所有权     -> FnOnce trait (需要 move 关键字)
    //
    // move 关键字强制闭包获取捕获变量的所有权

    // Fn: 不可变借用
    let s = String::from("hello");
    let print = || println!("{}", s);
    print();
    print();  // 可以多次调用
    assert_eq!(s, "hello"); // s 仍可用

    // FnMut: 可变借用
    let mut count = 0;
    let mut inc = || {
        count += 1;
        count
    };
    assert_eq!(inc(), 1);
    assert_eq!(inc(), 2);
    assert_eq!(inc(), 3);
    assert_eq!(count, 3); // count 仍可用(且被修改了)

    // FnOnce: 获取所有权 (move 关键字)
    let s = String::from("consumed");
    let consume = move || {
        assert_eq!(s, "consumed");
        // s 在闭包内被 drop
    };
    consume();
    // consume(); // 编译错误: FnOnce 只能调用一次
    // println!("{}", s); // 编译错误: s 已被 move 进闭包

    // 边界: move 闭包同时获取多个变量
    let s1 = String::from("a");
    let s2 = String::from("b");
    let join = move || format!("{}-{}", s1, s2);
    assert_eq!(join(), "a-b");

    // 边界: 闭包返回所有权
    let s = String::from("returned");
    let give_back = move || s;  // FnOnce: s 被 move 进闭包, 返回时交出所有权
    let s_back = give_back();
    assert_eq!(s_back, "returned");

    // 边界: move + Copy 类型 —— 值被复制进闭包
    let x = 42;
    let add_one = move || x + 1;  // x 是 Copy, 被复制进闭包
    assert_eq!(add_one(), 43);
    assert_eq!(x, 42);  // 原 x 仍可用

    // 边界: move 闭包修改捕获的变量需要 mut
    let mut counter = 0;
    let mut inc = move || {
        counter += 1;
        counter
    };
    assert_eq!(inc(), 1);
    assert_eq!(inc(), 2);
    // assert_eq!(counter, 0); // counter 已 move 进闭包

    // 边界: 闭包作为参数传递时的所有权
    fn apply_fn<F: FnOnce() -> i32>(f: F) -> i32 {
        f()
    }
    let val = String::from("unused");
    let closure = move || {
        let _ = val;
        42
    };
    assert_eq!(apply_fn(closure), 42);
}

#[test]
/// 测试: 多重所有权需求引入 —— 为 Rc/Arc 铺垫
fn test_multiple_ownership_intro() {
    // 语法: 单所有权模型的局限性, 引入 Rc<T> 和 Arc<T>
    //
    // 场景: 多个数据结构需要共享同一份数据

    // 场景1: 单所有权下无法共享
    let _config_single_owner = String::from("db_url=localhost");
    // 问题: 如果有两个结构体都需要持有 _config_single_owner, 用引用可能产生生命周期问题,
    // 用 clone 又浪费内存

    // 解决方案: Rc<T> (引用计数, 单线程)
    use std::rc::Rc;

    let config = Rc::new(String::from("db_url=localhost"));
    let db_config = Rc::clone(&config);
    let cache_config = Rc::clone(&config);

    assert_eq!(Rc::strong_count(&config), 3); // 3 个所有者
    assert_eq!(*db_config, "db_url=localhost");
    assert_eq!(*cache_config, "db_url=localhost");
    // 所有三个引用都指向同一块堆内存

    // 场景2: Rc 提供不可变共享
    // Rc 不允许可变借用（除非配合 RefCell）

    // 场景3: 引用计数演示
    let rc = Rc::new(42);
    assert_eq!(Rc::strong_count(&rc), 1);
    {
        let rc2 = Rc::clone(&rc);
        assert_eq!(Rc::strong_count(&rc), 2);
        let rc3 = Rc::clone(&rc);
        assert_eq!(Rc::strong_count(&rc), 3);
        assert_eq!(*rc2, 42);
        assert_eq!(*rc3, 42);
    } // rc2 和 rc3 drop, 引用计数减 2
    assert_eq!(Rc::strong_count(&rc), 1);

    // 场景4: Rc 中的值是不可变的
    let rc = Rc::new(vec![1, 2, 3]);
    assert_eq!(*rc, vec![1, 2, 3]);
    // rc.push(4); // 编译错误: Rc 提供不可变共享访问

    // 场景5: 不使用 Rc 时的痛苦 —— 引用生命周期纠缠
    // 想共享数据, 需要确保所有引用的生命周期重叠, 这在复杂代码中很困难
    struct App<'a> {
        config: &'a str,     // 引用, 生命周期受限于被引用的值
    }
    let config = String::from("app_config");
    let app = App { config: &config };
    assert_eq!(app.config, "app_config");
    // 问题: app 的生命周期不能超过 config 的生命周期
    // 而 Rc 没有这个限制 —— 所有权独立
}

#[test]
/// 测试: 借用检查器诊断 —— 通过注释展示典型借用检查器错误
fn test_borrow_checker_diagnostics() {
    // 语法: 以下代码用注释方式展示典型的借用检查器错误,
    // 帮助理解编译器错误信息
    //
    // 这些错误代码在注释中展示, 不会实际编译

    // -------------------------------------------------------
    // 错误1: E0382 - use of moved value (使用已移动的值)
    // -------------------------------------------------------
    // let s1 = String::from("hello");
    // let s2 = s1;
    // println!("{}", s1);  // E0382: borrow of moved value: `s1`
    //  编译器提示: "value borrowed here after move"
    //  含义: s1 的所有权已转移给 s2, 不能再使用 s1

    // -------------------------------------------------------
    // 错误2: E0502 - cannot borrow as mutable (同时存在不可变和可变借用)
    // -------------------------------------------------------
    // let mut v = vec![1, 2, 3];
    // let first = &v[0];
    // v.push(4);          // E0502: cannot borrow `v` as mutable because
    //                     //        it is also borrowed as immutable
    // println!("{}", first);

    // -------------------------------------------------------
    // 错误3: E0506 - cannot assign to `x` because it is borrowed
    // -------------------------------------------------------
    // let mut x = 5;
    // let r = &x;
    // x = 6;              // E0506: cannot assign to `x` because it is borrowed
    // println!("{}", r);

    // -------------------------------------------------------
    // 错误4: E0515 - cannot return reference to local variable
    // -------------------------------------------------------
    // fn bad() -> &String {
    //     let s = String::from("hello");
    //     &s                  // E0515: cannot return reference to local
    // }                       //        variable `s`

    // -------------------------------------------------------
    // 错误5: E0597 - borrowed value does not live long enough
    // -------------------------------------------------------
    // let r;
    // {
    //     let x = 42;
    //     r = &x;          // E0597: `x` does not live long enough
    // }
    // println!("{}", r);

    // -------------------------------------------------------
    // 正确做法演示
    // -------------------------------------------------------

    // 修复 E0382: 使用 clone 或引用
    let s1 = String::from("hello");
    let s2 = s1.clone();
    assert_eq!(s1, "hello"); // OK: clone 后 s1 仍有效
    assert_eq!(s2, "hello");

    // 修复 E0502: 缩短借用生命周期 (NLL 已自动处理一些情况)
    let mut v = vec![1, 2, 3];
    {
        let first = &v[0];
        assert_eq!(*first, 1);
    } // first 离开作用域
    v.push(4); // OK

    // 修复 E0515: 返回拥有所有权的值
    fn good() -> String {
        let s = String::from("hello");
        s // OK: 返回所有权
    }
    assert_eq!(good(), "hello");
}

#[test]
/// 测试: 验证哪些类型实现了 Copy
fn test_copy_types_list() {
    // 语法: 验证 Rust 内置实现 Copy trait 的类型
    //
    // Copy 类型赋值后原变量仍然有效
    // 判断方法: 赋值后原变量是否还能使用

    // 整数类型 (全部实现 Copy)
    let a: i8 = 1;
    let b = a;
    assert_eq!(a, 1); // OK: i8 是 Copy
    assert_eq!(b, 1);

    let a: i32 = 42;
    let b = a;
    assert_eq!(a, 42);
    assert_eq!(b, 42);

    let a: u64 = 100;
    let b = a;
    assert_eq!(a, 100);
    assert_eq!(b, 100);

    let a: usize = 1024;
    let b = a;
    assert_eq!(a, 1024);
    assert_eq!(b, 1024);

    let a: isize = -5;
    let b = a;
    assert_eq!(a, -5);
    assert_eq!(b, -5);

    // 浮点类型 (全部实现 Copy)
    let a: f32 = 3.14;
    let b = a;
    assert!((a - 3.14).abs() < f32::EPSILON);
    assert!((b - 3.14).abs() < f32::EPSILON);

    let a: f64 = 2.718;
    let b = a;
    assert!((a - 2.718).abs() < f64::EPSILON);
    assert!((b - 2.718).abs() < f64::EPSILON);

    // 布尔类型
    let a = true;
    let b = a;
    assert!(a);
    assert!(b);

    // 字符类型
    let a = 'A';
    let b = a;
    assert_eq!(a, 'A');
    assert_eq!(b, 'A');

    // 引用类型 (实现了 Copy)
    let s = String::from("ref");
    let r1 = &s;
    let r2 = r1;  // 引用是 Copy
    assert_eq!(*r1, "ref");
    assert_eq!(*r2, "ref");

    let r1: &i32 = &42;
    let r2 = r1;
    assert_eq!(*r1, 42);
    assert_eq!(*r2, 42);

    // 原始指针 (实现了 Copy)
    // 使用引用来演示（因为 &T 自动转成 *const T）
    let x = 42;
    let p1: *const i32 = &x;
    let p2 = p1; // 原始指针是 Copy
    assert!(!p1.is_null());
    assert!(!p2.is_null());

    // 元组 (所有元素都是 Copy 时是 Copy)
    let t1 = (1, 2.0, 'a', true);
    let t2 = t1;
    assert_eq!(t1, (1, 2.0, 'a', true));
    assert_eq!(t2, (1, 2.0, 'a', true));

    // 数组 [T; N] (T 是 Copy 时数组是 Copy)
    let arr1 = [1, 2, 3, 4, 5];
    let arr2 = arr1;
    assert_eq!(arr1, [1, 2, 3, 4, 5]);
    assert_eq!(arr2, [1, 2, 3, 4, 5]);

    // 函数指针 (实现了 Copy)
    fn foo(x: i32) -> i32 { x + 1 }
    let f1: fn(i32) -> i32 = foo;
    let f2 = f1;
    assert_eq!(f1(1), 2);
    assert_eq!(f2(1), 2);

    // -------------------------------------------------------
    // 不实现 Copy 的类型:
    // -------------------------------------------------------
    // String: 不是 Copy
    let s1 = String::from("hello");
    let _s2 = s1;
    // assert_eq!(s1, "hello"); // 编译错误: s1 被 move

    // Vec: 不是 Copy
    let v1 = vec![1, 2, 3];
    let _v2 = v1;
    // assert_eq!(v1.len(), 3); // 编译错误

    // Box: 不是 Copy
    let b1 = Box::new(42);
    let _b2 = b1;
    // assert_eq!(*b1, 42); // 编译错误

    // 包含非 Copy 元素的元组: 不是 Copy
    let t1 = (String::from("a"), 42);
    let _t2 = t1;
    // assert_eq!(t1.1, 42); // 编译错误

    // 包含非 Copy 元素的数组: 不是 Copy
    // [String; 3] 不是 Copy
    // let arr = [String::from("a"), String::from("b")];
    // let _arr2 = arr;
    // let _ = arr; // 编译错误
}

#[test]
/// 测试: 循环中的所有权行为
fn test_ownership_and_loops() {
    // 语法: 循环中的所有权处理需要注意每次迭代的借用行为

    // 场景1: for 循环按引用遍历 —— 不获取所有权
    let v = vec![String::from("a"), String::from("b"), String::from("c")];
    let mut total_len = 0;
    for s in &v {
        total_len += s.len();
    }
    assert_eq!(total_len, 3);
    assert_eq!(v.len(), 3); // v 仍可用

    // 场景2: for 循环按值遍历 —— 消耗集合
    let v = vec![String::from("a"), String::from("b")];
    let mut collected = Vec::new();
    for s in v {  // v 被消耗
        collected.push(s);
    }
    assert_eq!(collected.len(), 2);
    // assert_eq!(v.len(), 2); // 编译错误

    // 场景3: while loop 中的借用
    let v = vec![1, 2, 3];
    let mut sum = 0;
    let mut i = 0;
    while i < v.len() {
        sum += v[i];
        i += 1;
    }
    assert_eq!(sum, 6);
    // 注意: while 中 v 被多次不可变借用, 每次借用都在表达式求值后结束

    // 场景4: loop 循环中返回所有权
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter >= 10 {
            break String::from("done");  // 返回所有权给 result
        }
    };
    assert_eq!(result, "done");
    assert_eq!(counter, 10);

    // 场景5: for 循环中可变借用
    let mut v = vec![1, 2, 3, 4, 5];
    for x in &mut v {
        *x *= 2;
    }
    assert_eq!(v, vec![2, 4, 6, 8, 10]);

    // 场景6: 循环中使用索引（避免借用冲突）
    let mut v = vec![1, 2, 3];
    for i in 0..v.len() {
        v[i] += 1;  // OK: 每次通过索引可变借用, 不冲突
    }
    assert_eq!(v, vec![2, 3, 4]);

    // 场景7: 循环中对集合的借用和修改冲突（常见错误示例）
    // 错误做法 (编译错误):
    // let mut v = vec![1, 2, 3];
    // for x in &v {       // 不可变借用整个集合
    //     v.push(*x * 2); // E0502: 不能同时不可变借用和可变借用
    // }
    // 正确做法: 用索引或收集到新容器
    let v = vec![1, 2, 3];
    let mut result = Vec::new();
    for x in &v {
        result.push(x * 2);
    }
    assert_eq!(result, vec![2, 4, 6]);
    assert_eq!(v, vec![1, 2, 3]);

    // 场景8: 有条件的 break (所有权在 break 时转移)
    let opt = Some(String::from("found"));
    let mut count = 0;
    let value = loop {
        count += 1;
        if count > 3 {
            break String::from("default");
        }
        if count == 2 {
            break opt.clone().unwrap(); // 所有权转移给 value
        }
    };
    assert_eq!(value, "found");
    assert_eq!(count, 2);

    // 场景9: while let 模式匹配中的所有权
    let mut stack = vec![1, 2, 3];
    let mut popped = Vec::new();
    while let Some(top) = stack.pop() {
        popped.push(top);
    }
    assert_eq!(popped, vec![3, 2, 1]);
    assert!(stack.is_empty());

    // 场景10: for 循环中的 move 闭包（如果需要跨线程或有特殊需求）
    let values = vec![1, 2, 3];
    let doubled: Vec<i32> = values.into_iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6]);
    // values 已被 into_iter 消耗
}

#[test]
/// 测试: 内部可变性简介 —— Cell 和 RefCell 的基本用法
fn test_interior_mutability_intro() {
    // 语法: 内部可变性允许通过不可变引用修改内部数据
    //
    // 核心类型:
    //   - Cell<T>: 用于 Copy 类型, 通过 get()/set() 访问
    //   - RefCell<T>: 用于任何类型, 通过 borrow()/borrow_mut() 运行时检查
    //
    // 用途: 在不可变上下文中实现可变行为(如 Rc<Rc<RefCell<T>>>)

    use std::cell::Cell;
    use std::cell::RefCell;

    // ---- Cell<T>: 简单的内部可变性 (仅适用于 Copy 类型) ----
    let x = Cell::new(42);
    assert_eq!(x.get(), 42);

    // 通过不可变引用修改
    let r1 = &x;
    let r2 = &x;
    r1.set(100);
    assert_eq!(r2.get(), 100); // 多个引用可同时存在, 且能看到修改

    // Cell 适用于 Copy 类型
    let b = Cell::new(true);
    b.set(false);
    assert!(!b.get());

    // ---- RefCell<T>: 运行时借用检查 ----
    let data = RefCell::new(vec![1, 2, 3]);

    // 不可变借用 (borrow)
    {
        let borrowed = data.borrow();
        assert_eq!(*borrowed, vec![1, 2, 3]);
        // 可以同时有多个不可变借用
        let borrowed2 = data.borrow();
        assert_eq!(borrowed2.len(), 3);
    } // 借用在此释放

    // 可变借用 (borrow_mut)
    {
        let mut borrowed = data.borrow_mut();
        borrowed.push(4);
        assert_eq!(*borrowed, vec![1, 2, 3, 4]);
    }

    // 验证修改持久化
    assert_eq!(*data.borrow(), vec![1, 2, 3, 4]);

    // RefCell 的运行时检查（编译通过, 运行时 panic）
    // let _r1 = data.borrow_mut();
    // let _r2 = data.borrow_mut(); // panic: already mutably borrowed

    // ---- RefCell 典型用例: Rc<RefCell<T>> ----
    // (前面讲了 Rc 提供不可变共享, RefCell 提供内部可变性 — 两者结合)
    let shared = std::rc::Rc::new(RefCell::new(0));

    let handle1 = std::rc::Rc::clone(&shared);
    let handle2 = std::rc::Rc::clone(&shared);

    *handle1.borrow_mut() += 10;
    *handle2.borrow_mut() += 20;

    assert_eq!(*shared.borrow(), 30);

    // ---- Cell 与 RefCell 的区别总结 ----
    // Cell: 零开销 (编译期保证 Copy 语义安全)
    // RefCell: 运行时检查 (适用于非 Copy 类型, 有细微开销)
}
