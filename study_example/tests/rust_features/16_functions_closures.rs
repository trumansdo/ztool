// ---------------------------------------------------------------------------
// 2.3 函数与闭包
// ---------------------------------------------------------------------------

#[test]
/// 测试: 函数定义 (参数/返回值/表达式)
fn test_function_definition() {
    // 语法: fn name(params) -> ReturnType { body }
    //
    // 规则:
    //   - 最后一个表达式是返回值(无分号)
    //   - return 提前返回
    //   - 类型注解必须
    //
    // 避坑:
    //   - 函数体最后一个表达式不能加分号
    //   - 发散函数(panic/loop)返回 ! 类型
    //   - 返回多值用元组
    //

    fn add(x: i32, y: i32) -> i32 {
        x + y
    }
    assert_eq!(add(3, 5), 8);
    assert_eq!(add(-1, 1), 0);
    assert_eq!(add(0, 0), 0);

    fn square(x: i32) -> i32 {
        x * x
    }
    assert_eq!(square(5), 25);
    assert_eq!(square(0), 0);
    assert_eq!(square(-3), 9);

    fn early_return(x: i32) -> i32 {
        if x < 0 {
            return 0;
        }
        x * x
    }
    assert_eq!(early_return(-5), 0);
    assert_eq!(early_return(5), 25);
    assert_eq!(early_return(-1), 0);

    // 返回元组
    fn divide(a: i32, b: i32) -> (i32, i32) {
        (a / b, a % b)
    }
    let (q, r) = divide(10, 3);
    assert_eq!(q, 3);
    assert_eq!(r, 1);

    let (q2, r2) = divide(20, 7);
    assert_eq!(q2, 2);
    assert_eq!(r2, 6);
}

#[test]
/// 测试: 闭包基本语法 (匿名函数/捕获环境)
fn test_closure_basics() {
    // 语法: |params| body 或 |params| -> Type { body }
    //
    // 特性:
    //   - 匿名函数, 可捕获环境变量
    //   - 类型通常可省略(编译器推断)
    //   - 管道符 || 代替圆括号
    //
    // 避坑:
    //   - 多语句闭包需要花括号
    //   - 闭包大小等于捕获变量的总和
    //

    let add_one = |x: i32| -> i32 { x + 1 };
    assert_eq!(add_one(5), 6);
    assert_eq!(add_one(100), 101);

    let add_one = |x| x + 1; // 类型推断
    assert_eq!(add_one(5), 6);

    let multiply = |x, y| x * y;
    assert_eq!(multiply(3, 4), 12);
    assert_eq!(multiply(0, 100), 0);

    // 多语句闭包
    let complex = |x: i32| {
        let y = x + 1;
        y * 2
    };
    assert_eq!(complex(5), 12);
    assert_eq!(complex(0), 2);

    // 多参数多语句闭包
    let compute = |a: i32, b: i32| {
        let sum = a + b;
        let product = a * b;
        sum + product
    };
    assert_eq!(compute(2, 3), 11); // 5 + 6
}

#[test]
/// 测试: 闭包捕获方式 (Fn/FnMut/FnOnce)
fn test_closure_capture_modes() {
    // 语法: 闭包通过三种方式捕获环境
    //
    // 捕获方式(编译器自动选择最宽松的):
    //   - Fn: 不可变借用 (&T)——不修改环境
    //   - FnMut: 可变借用 (&mut T)——修改环境
    //   - FnOnce: 获取所有权 (T)——消费环境
    //
    // 避坑:
    //   - FnMut 需要 let mut 声明闭包
    //   - 所有闭包都实现 FnOnce
    //

    let x = 10;
    let print = || println!("{}", x);
    print(); // Fn: 不可变借用
    print(); // 可多次调用
    assert_eq!(x, 10); // x 未被修改

    let mut y = 5;
    let mut increment = || y += 1;
    increment(); // FnMut: 可变借用
    increment();
    assert_eq!(y, 7);

    let z = String::from("hello");
    let consume = || drop(z); // FnOnce: 获取所有权
    consume();
    // consume(); // 编译错误: 只能调用一次

    // Fn 闭包不阻止对未捕获变量的使用
    let mut a = 1;
    let mut b = 2;
    let mut modify_b = || b += 1;
    modify_b(); // 只捕获 b
    a += 1; // a 未被捕获，仍可使用
    assert_eq!(a, 2);
    assert_eq!(b, 3);
}

#[test]
/// 测试: move 关键字 (强制所有权转移)
fn test_move_keyword() {
    // 语法: move || { ... } 强制将捕获变量移入闭包
    //
    // 使用场景:
    //   - 线程任务: thread::spawn(move || { ... })
    //   - 异步任务: tokio::spawn(move || { ... })
    //   - 返回值闭包
    //
    // 避坑:
    //   - move 后原作用域不能再用该变量
    //   - move 不影响捕获方式(Fn/FnMut/FnOnce)
    //   - move 和捕获方式是正交概念
    //

    let data = vec![1, 2, 3];
    let closure = move || {
        assert_eq!(data.len(), 3);
    };
    closure();
    // println!("{:?}", data); // 编译错误: data 已被 move

    // move + FnMut: move 不影响闭包 trait 的推导
    let mut counter = 0;
    let mut inc = move || {
        counter += 1;
        assert!(counter > 0);
    };
    inc();
    inc();
    // 实现了 FnMut

    // move 闭包可以多次调用（如果内部没有消费操作）
    let msg = String::from("hello");
    let read_msg = move || msg.len();
    assert_eq!(read_msg(), 5);
    assert_eq!(read_msg(), 5); // 可多次调用——只是读取
}

#[test]
/// 测试: 闭包作为函数参数 (泛型约束)
fn test_closure_as_parameter() {
    // 语法: 使用泛型约束接收闭包
    //
    // 约束方式:
    //   - F: Fn(i32) -> i32
    //   - where F: FnOnce() -> i32
    //   - impl Fn(i32) -> i32 (impl Trait 语法)
    //
    // 避坑:
    //   - 需要指定完整的函数签名
    //   - FnOnce 闭包只能调用一次
    //   - 闭包作为参数是零成本的(单态化)
    //

    fn apply<F>(f: F, x: i32) -> i32
    where
        F: Fn(i32) -> i32,
    {
        f(x)
    }

    assert_eq!(apply(|x| x * 2, 10), 20);
    assert_eq!(apply(|x| x + 100, 10), 110);
    assert_eq!(apply(|x| x * x, 3), 9);

    // impl Trait 语法
    fn apply_impl(f: impl Fn(i32) -> i32, x: i32) -> i32 {
        f(x)
    }
    assert_eq!(apply_impl(|x| x - 1, 10), 9);
    assert_eq!(apply_impl(|x| x * 5, 4), 20);

    // 接收不同类型的闭包
    fn call_twice(mut f: impl FnMut() -> i32) -> (i32, i32) {
        (f(), f())
    }
    let mut n = 0;
    let results = call_twice(|| {
        n += 1;
        n
    });
    assert_eq!(results, (1, 2));
}

#[test]
/// 测试: 闭包作为返回值 (impl Fn + move)
fn test_closure_as_return_value() {
    // 语法: 返回闭包使用 impl Fn 和 move
    //
    // 避坑:
    //   - 必须使用 move 关键字
    //   - 返回的闭包捕获了环境变量
    //   - 不能提前声明闭包类型(无法具名)
    //

    fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
        move |y| x + y
    }

    let add_five = make_adder(5);
    assert_eq!(add_five(3), 8);
    assert_eq!(add_five(10), 15);
    assert_eq!(add_five(0), 5);

    let add_ten = make_adder(10);
    assert_eq!(add_ten(0), 10);
    assert_eq!(add_ten(-5), 5);

    // 返回 FnMut
    fn make_counter(start: i32) -> impl FnMut() -> i32 {
        let mut count = start;
        move || {
            count += 1;
            count
        }
    }

    let mut counter = make_counter(0);
    assert_eq!(counter(), 1);
    assert_eq!(counter(), 2);
    assert_eq!(counter(), 3);

    let mut counter2 = make_counter(100);
    assert_eq!(counter2(), 101);
    assert_eq!(counter2(), 102);
}

#[test]
/// 测试: 函数指针 (fn 类型/闭包转函数指针)
fn test_function_pointers() {
    // 语法: fn(i32) -> i32 是函数指针类型
    //
    // 区别:
    //   - fn 类型: 不捕获环境的函数/闭包
    //   - Fn trait: 闭包(可捕获环境)
    //   - 不捕获环境的闭包可转为 fn
    //
    // 避坑:
    //   - 函数指针不能捕获环境变量
    //   - fn 类型实现了 Fn/FnMut/FnOnce
    //

    fn add_one(x: i32) -> i32 {
        x + 1
    }

    let fn_ptr: fn(i32) -> i32 = add_one;
    assert_eq!(fn_ptr(5), 6);
    assert_eq!(fn_ptr(100), 101);

    // 不捕获环境的闭包可转为函数指针
    let closure_ptr: fn(i32) -> i32 = |x| x + 1;
    assert_eq!(closure_ptr(5), 6);

    fn apply_fn_ptr(f: fn(i32) -> i32, x: i32) -> i32 {
        f(x)
    }

    assert_eq!(apply_fn_ptr(add_one, 5), 6);
    assert_eq!(apply_fn_ptr(|x| x + 1, 5), 6);
    assert_eq!(apply_fn_ptr(|x| x * 2, 10), 20);

    // fn 指针可以存储在 Vec 中
    let ops: Vec<fn(i32, i32) -> i32> = vec![
        |a, b| a + b,
        |a, b| a - b,
        |a, b| a * b,
    ];
    assert_eq!(ops[0](10, 5), 15);
    assert_eq!(ops[1](10, 5), 5);
    assert_eq!(ops[2](10, 5), 50);
}

#[test]
/// 测试: 高阶函数 (map/filter/fold/any/all)
fn test_higher_order_functions() {
    // 语法: 闭包作为高阶函数的参数
    //
    // 常用组合:
    //   - .filter().map().collect()
    //   - .iter().fold(init, |acc, x| acc + x)
    //
    // 避坑:
    //   - 链式调用注意所有权
    //   - filter 中闭包返回 bool, map 返回转换后的值
    //

    let numbers = vec![1, 2, 3, 4, 5];

    let result: Vec<i32> = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .map(|x| x * x)
        .collect();
    assert_eq!(result, vec![4, 16]);

    // fold
    let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);
    assert_eq!(sum, 15);

    // 更复杂的 fold: 字符串拼接
    let words = vec!["Rust", "是", "一门", "系统", "编程", "语言"];
    let sentence = words.iter().fold(String::new(), |mut acc, w| {
        acc.push_str(w);
        acc
    });
    assert_eq!(sentence, "Rust是一门系统编程语言");

    // any / all
    assert!(numbers.iter().any(|&x| x > 3));
    assert!(!numbers.iter().all(|&x| x > 3));
    assert!(numbers.iter().all(|&x| x > 0));

    // find
    let found = numbers.iter().find(|&&x| x == 3);
    assert_eq!(found, Some(&3));
    let not_found = numbers.iter().find(|&&x| x == 10);
    assert_eq!(not_found, None);

    // position
    let pos = numbers.iter().position(|&x| x > 3);
    assert_eq!(pos, Some(3));
}

#[test]
/// 测试: 闭包 trait 约束 (Fn/FnMut/FnOnce 检查器)
fn test_closure_trait_bounds() {
    // 语法: 通过泛型约束测试闭包实现了哪些 trait
    //
    // 所有闭包都实现 FnOnce
    // 不修改环境的还实现 FnMut 和 Fn
    // 只修改不移动的还实现 FnMut
    //
    fn call_fn<F: Fn()>(f: F) {
        f()
    }
    fn call_fn_mut<F: FnMut()>(mut f: F) {
        f()
    }
    fn call_fn_once<F: FnOnce()>(f: F) {
        f()
    }

    let x = 10;

    // 只读捕获: 实现所有三个 trait
    let read_closure = || println!("{}", x);
    call_fn(&read_closure);
    call_fn_mut(&read_closure);
    call_fn_once(read_closure);

    let mut y = 5;
    // 可变捕获: 实现 FnMut 和 FnOnce
    let mut write_closure = || y += 1;
    call_fn_mut(&mut write_closure);
    call_fn_once(write_closure);

    let z = String::from("hello");
    // 消费捕获: 只实现 FnOnce
    let consume_closure = || drop(z);
    call_fn_once(consume_closure);

    // 验证 fn 函数指针也实现了这些 trait
    fn my_func() {}
    call_fn(my_func);
    call_fn_mut(my_func);
    call_fn_once(my_func);
}

// ============================================================================
// 新增测试
// ============================================================================

#[test]
/// 测试: 函数项类型 vs fn 指针 (ZST / 强制转换 / 唯一类型)
fn test_function_vs_fn_pointer() {
    // 语法: 函数项类型是唯一的零大小类型, fn 是指针大小
    //
    // 规则:
    //   - 每个函数定义产生唯一函数项类型 fn(Args) -> Ret {函数名}
    //   - 函数项类型大小为 0 (ZST)
    //   - fn 指针大小为 usize (通常 8 字节)
    //   - 函数项类型可自动强制转换为 fn 指针
    //
    // 避坑:
    //   - 函数项类型不能混用 (不同类型, 即使签名相同)
    //   - Vec 中存储 fn 指针而非函数项类型
    //

    fn add(x: i32, y: i32) -> i32 {
        x + y
    }
    fn multiply(x: i32, y: i32) -> i32 {
        x * y
    }

    // 函数项类型 —— 零大小类型
    let fn_item = add;
    assert_eq!(std::mem::size_of_val(&fn_item), 0);

    // fn 指针 —— 指针大小
    let fn_ptr: fn(i32, i32) -> i32 = add; // 自动强制转换
    assert_eq!(std::mem::size_of_val(&fn_ptr), std::mem::size_of::<usize>());

    // 两个不同的函数项类型不能放在同一个 Vec 中
    // 但可以转换后放入
    let ops: Vec<fn(i32, i32) -> i32> = vec![add, multiply];
    assert_eq!(ops[0](2, 3), 5);
    assert_eq!(ops[1](2, 3), 6);

    // 不捕获环境的闭包可以转为 fn 指针
    let closure_to_fn: fn(i32, i32) -> i32 = |a, b| a + b;
    assert_eq!(closure_to_fn(10, 20), 30);

    // fn 指针实现了 Fn/FnMut/FnOnce
    fn apply<F: Fn(i32, i32) -> i32>(f: F, a: i32, b: i32) -> i32 {
        f(a, b)
    }
    assert_eq!(apply(fn_ptr, 3, 4), 7);
}

#[test]
/// 测试: 发散函数 (never type ! / 永不返回)
fn test_diverging_function() {
    // 语法: 发散函数返回类型为 ! (never type)
    //
    // 规则:
    //   - ! 可以强制转换为任何类型
    //   - loop (无 break)、panic!、exit、unreachable! 返回 !
    //   - 在 match 表达式中可以用发散函数填补分支
    //
    // 避坑:
    //   - ! 类型不能实例化, 不能用 let x: ! = ...
    //   - 发散函数后的代码是死代码
    //

    fn will_panic() -> ! {
        panic!("stop");
    }

    // ! 可以强制转换为任何类型
    fn get_value_or_panic(opt: Option<i32>) -> i32 {
        match opt {
            Some(v) => v,
            None => will_panic(), // ! 强制转换为 i32
        }
    }
    assert_eq!(get_value_or_panic(Some(42)), 42);

    // loop 也是发散函数
    fn infinite_loop() -> ! {
        loop {
            // 永不返回
        }
    }

    // 在 unwrap 中的使用原理
    let x: i32 = Some(5).unwrap_or_else(|| panic!("none"));
    assert_eq!(x, 5);

    // ! 类型作为泛型参数
    fn always_err() -> Result<(), String> {
        Err("错误".into())
    }
    // 在 match 中, Err 分支的 ! 可以与 () 统一
    match always_err() {
        Ok(()) => {}
        Err(e) => assert_eq!(e, "错误"),
    }
}

#[test]
/// 测试: const fn (编译期求值)
fn test_const_fn() {
    // 语法: const fn 允许在编译期执行
    //
    // 规则:
    //   - const fn 可在编译期求值
    //   - 参数和返回值通常限制为 Copy 类型
    //   - 可用于初始化 const/static 常量
    //   - 稳定版限制: 不能使用 for、while、loop、if let 等
    //
    // 避坑:
    //   - const fn 不能使用 &mut 引用
    //   - 不能调用非 const fn
    //   - 编译期求值时不会影响运行时性能
    //

    const fn factorial(n: u64) -> u64 {
        match n {
            0 | 1 => 1,
            _ => n * factorial(n - 1),
        }
    }

    // 编译期常量
    const FACT_5: u64 = factorial(5);
    const FACT_10: u64 = factorial(10);
    assert_eq!(FACT_5, 120);
    assert_eq!(FACT_10, 3628800);

    // 运行时也可以调用 const fn
    assert_eq!(factorial(4), 24);

    const fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    const RESULT: i32 = add(10, 32);
    assert_eq!(RESULT, 42);

    // const fn 用于泛型
    const fn max(a: u32, b: u32) -> u32 {
        if a > b {
            a
        } else {
            b
        }
    }
    const MAX: u32 = max(100, 200);
    assert_eq!(MAX, 200);

    // const fn 用于数组初始化
    const fn make_array() -> [i32; 5] {
        let mut arr = [0; 5];
        arr[0] = factorial(0) as i32;
        arr[1] = factorial(1) as i32;
        arr[2] = factorial(2) as i32;
        arr[3] = factorial(3) as i32;
        arr[4] = factorial(4) as i32;
        arr
    }
    const FACT_ARRAY: [i32; 5] = make_array();
    assert_eq!(FACT_ARRAY, [1, 1, 2, 6, 24]);
}

#[test]
/// 测试: 闭包逐字段捕获 (部分捕获/独立推断)
fn test_closure_capture_refinement() {
    // 语法: 闭包对结构体的捕获精确到字段级别
    //
    // 规则:
    //   - 每个捕获变量独立推断捕获方式
    //   - 结构体字段可被分别捕获
    //   - 未捕获的字段仍可独立使用
    //   - 这使闭包捕获的粒度最小化
    //
    // 避坑:
    //   - 如果通过解构使用, 会整体借用
    //   - move 闭包会整体移动结构体
    //

    struct Point {
        x: i32,
        y: i32,
    }

    let mut p = Point { x: 0, y: 0 };

    // 闭包只捕获 p.x 的可变借用, p.y 未被捕获
    let mut c = || {
        p.x += 1;
        assert_eq!(p.x, 1);
    };
    c();

    // p.y 仍然可以独立使用
    assert_eq!(p.y, 0);
    p.y = 10;
    assert_eq!(p.y, 10);

    // 验证多个独立变量的独立推断
    let a = String::from("hello");
    let mut b = 5;
    let c_val = 10;

    let mut closure = || {
        println!("a={}", a); // a: 不可变借用 → 需要 Fn
        b += 1; // b: 可变借用 → 需要 FnMut
                // c_val 是 Copy 类型，直接复制不借用
    };
    closure();
    assert_eq!(b, 6);

    // 验证 c_val (Copy 类型) 仍可使用
    assert_eq!(c_val, 10);
}

#[test]
/// 测试: 闭包 Fn/FnMut/FnOnce 自动推导
fn test_closure_trait_auto_derive() {
    // 语法: 编译器根据闭包体自动推导实现了哪些闭包 trait
    //
    // 推导规则:
    //   1. 消费环境变量 → 仅 FnOnce
    //   2. 修改环境变量 → FnMut + FnOnce
    //   3. 只读环境变量 → Fn + FnMut + FnOnce
    //
    // 避坑:
    //   - move 不影响 trait 推导, 只影响捕获方式
    //   - FnMut 闭包调用时变量必须声明为 mut
    //

    // 场景1: 只读 → Fn + FnMut + FnOnce
    {
        let x = 42;
        let c = || x * 2; // 只读 Copy 变量

        fn expect_fn<F: Fn() -> i32>(f: F) -> i32 {
            f()
        }
        fn expect_fn_mut<F: FnMut() -> i32>(mut f: F) -> i32 {
            f()
        }
        fn expect_fn_once<F: FnOnce() -> i32>(f: F) -> i32 {
            f()
        }

        assert_eq!(expect_fn(&c), 84);
        assert_eq!(expect_fn_mut(&c), 84);
        assert_eq!(expect_fn_once(c), 84);
    }

    // 场景2: 修改 → FnMut + FnOnce
    {
        let mut counter = 0;
        let mut c = || {
            counter += 1;
            counter
        };

        fn expect_fn_mut<F: FnMut() -> i32>(mut f: F) -> i32 {
            f()
        }
        fn expect_fn_once<F: FnOnce() -> i32>(f: F) -> i32 {
            f()
        }

        assert_eq!(expect_fn_mut(&mut c), 1);
        assert_eq!(expect_fn_mut(&mut c), 2);
        assert_eq!(expect_fn_once(c), 3);
    }

    // 场景3: 消费 → 仅 FnOnce
    {
        let s = String::from("消费");
        let c = || drop(s);

        fn expect_fn_once<F: FnOnce()>(f: F) {
            f()
        }

        expect_fn_once(c); // OK
                            // expect_fn_mut 和 expect_fn 编译失败
    }

    // 场景4: move 不影响 trait 推导
    {
        let mut n = 0;
        // move 闭包，但只修改不消费 → 仍实现 FnMut
        let mut c = move || {
            n += 1;
            n
        };
        assert_eq!(c(), 1);
        assert_eq!(c(), 2); // 可多次调用
    }
}

#[test]
/// 测试: 返回闭包的多种方式 (impl Fn / Box<dyn Fn> / fn 指针)
fn test_returning_closures() {
    // 语法: 闭包的类型是匿名的, 返回时需要类型擦除
    //
    // 方式:
    //   - impl Trait: 零成本, 编译期确定具体类型
    //   - Box<dyn Fn>: 堆分配, 支持运行时异构集合
    //   - fn 指针: 仅限不捕获环境的闭包
    //
    // 避坑:
    //   - impl Trait 返回的闭包类型在编译期必须唯一
    //   - Box<dyn Fn> 支持不同分支返回不同类型
    //   - fn 指针不能捕获环境变量
    //

    // 方式1: impl Trait —— 推荐, 零成本
    fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
        move |x| x * factor
    }
    let double = make_multiplier(2);
    let triple = make_multiplier(3);
    assert_eq!(double(10), 20);
    assert_eq!(triple(10), 30);

    // 方式2: Box<dyn Fn> —— 支持条件返回不同类型
    fn make_operation(op: &str) -> Box<dyn Fn(i32, i32) -> i32> {
        match op {
            "add" => Box::new(|a, b| a + b),
            "sub" => Box::new(|a, b| a - b),
            "mul" => Box::new(|a, b| a * b),
            _ => Box::new(|a, _| a),
        }
    }
    let add_op = make_operation("add");
    let mul_op = make_operation("mul");
    assert_eq!(add_op(10, 5), 15);
    assert_eq!(mul_op(10, 5), 50);

    // 运行时选择的异构集合
    let mut ops: Vec<Box<dyn Fn(i32, i32) -> i32>> = Vec::new();
    ops.push(Box::new(|a, b| a + b));
    ops.push(Box::new(|a, b| a * b));
    assert_eq!(ops[0](3, 4), 7);
    assert_eq!(ops[1](3, 4), 12);

    // 方式3: fn 指针 —— 仅不捕获环境
    fn get_simple_op() -> fn(i32, i32) -> i32 {
        |a, b| a + b
    }
    let op = get_simple_op();
    assert_eq!(op(20, 30), 50);

    // FnMut 返回值
    fn make_counter(start: i32) -> impl FnMut() -> i32 {
        let mut count = start;
        move || {
            count += 1;
            count
        }
    }
    let mut counter = make_counter(0);
    assert_eq!(counter(), 1);
    assert_eq!(counter(), 2);
    assert_eq!(counter(), 3);
}

#[test]
/// 测试: 函数式组合子 (map/and_then/or_else/filter/filter_map/fold/reduce)
fn test_functional_combinators() {
    // 语法: 使用组合子进行声明式数据处理
    //
    // 常用组合子:
    //   - map: 对值进行转换
    //   - and_then: 链式调用 (flat_map on Option)
    //   - or_else: 为 None 提供备选
    //   - filter: 条件过滤
    //   - filter_map: 同时过滤和转换
    //   - fold / reduce: 累积计算
    //
    // 避坑:
    //   - 组合子链中的类型变化要留意
    //   - Option 和 Result 的组合子语义不同但用法相似
    //

    // map —— 对值进行转换
    let opt: Option<i32> = Some(5);
    assert_eq!(opt.map(|v| v * 2), Some(10));
    assert_eq!(None.map(|v: i32| v * 2), None);

    // map_err —— 对错误进行转换
    let res: Result<i32, &str> = Ok(5);
    assert_eq!(res.map_err(|e| format!("错误: {}", e)), Ok(5));
    let err: Result<i32, &str> = Err("失败");
    let mapped_err = err.map_err(|e| format!("错误: {}", e));
    assert!(mapped_err.is_err());

    // and_then —— 链式操作
    fn safe_div(a: i32, b: i32) -> Option<i32> {
        if b == 0 {
            None
        } else {
            Some(a / b)
        }
    }
    let result = Some(20)
        .and_then(|x| safe_div(x, 2))
        .and_then(|x| safe_div(x, 5));
    assert_eq!(result, Some(2));

    let short_circuit = Some(20)
        .and_then(|x| safe_div(x, 0)) // 短路!
        .and_then(|x| safe_div(x, 5));
    assert_eq!(short_circuit, None);

    // or_else —— 提供备选值
    assert_eq!(None.or_else(|| Some(42)), Some(42));
    assert_eq!(Some(10).or_else(|| Some(42)), Some(10));

    // filter —— 条件过滤
    assert_eq!(Some(3).filter(|&v| v > 5), None);
    assert_eq!(Some(10).filter(|&v| v > 5), Some(10));

    // filter_map —— 同时对迭代器进行过滤和转换
    let strings = vec!["1", "two", "3", "four", "5"];
    let numbers: Vec<i32> = strings
        .iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();
    assert_eq!(numbers, vec![1, 3, 5]);

    // fold —— 累积计算
    let sum = (1..=5).fold(0, |acc, x| acc + x);
    assert_eq!(sum, 15);

    let product = (1..=5).fold(1, |acc, x| acc * x);
    assert_eq!(product, 120);

    // reduce —— 使用第一个元素作为初始值
    let max_val = vec![3, 7, 2, 9, 1].into_iter().reduce(|a, b| a.max(b));
    assert_eq!(max_val, Some(9));

    let empty: Vec<i32> = vec![];
    assert_eq!(empty.into_iter().reduce(|a: i32, b| a + b), None);

    // collect 不同集合类型
    let squares: Vec<i32> = (1..=3).map(|x| x * x).collect();
    assert_eq!(squares, vec![1, 4, 9]);

    let sum_squares: i32 = (1..=3).map(|x| x * x).sum();
    assert_eq!(sum_squares, 14); // 1 + 4 + 9
}

#[test]
/// 测试: Builder 模式 (消费 self 构建复杂对象)
fn test_builder_pattern() {
    // 语法: 通过链式方法构造复杂对象
    //
    // 设计要点:
    //   - Builder 持有所有可选字段的默认值
    //   - 每个 setter 方法消费 self 并返回 Self
    //   - build() 方法消费 Builder 产生最终对象
    //
    // 避坑:
    //   - &mut self vs self: &mut self 可多次配置但不能构建
    //   - 构建后 Builder 被消费, 防止半成品状态
    //

    #[derive(Debug, PartialEq)]
    struct Connection {
        host: String,
        port: u16,
        timeout_secs: u64,
        retry_count: u32,
        use_ssl: bool,
    }

    struct ConnectionBuilder {
        host: String,
        port: u16,
        timeout_secs: u64,
        retry_count: u32,
        use_ssl: bool,
    }

    impl ConnectionBuilder {
        fn new(host: &str) -> Self {
            ConnectionBuilder {
                host: host.to_string(),
                port: 8080,
                timeout_secs: 30,
                retry_count: 3,
                use_ssl: false,
            }
        }

        fn port(mut self, port: u16) -> Self {
            self.port = port;
            self
        }

        fn timeout(mut self, secs: u64) -> Self {
            self.timeout_secs = secs;
            self
        }

        fn retry(mut self, count: u32) -> Self {
            self.retry_count = count;
            self
        }

        fn enable_ssl(mut self) -> Self {
            self.use_ssl = true;
            self
        }

        fn build(self) -> Connection {
            Connection {
                host: self.host,
                port: self.port,
                timeout_secs: self.timeout_secs,
                retry_count: self.retry_count,
                use_ssl: self.use_ssl,
            }
        }
    }

    // 使用默认配置
    let conn = ConnectionBuilder::new("localhost").build();
    assert_eq!(conn.host, "localhost");
    assert_eq!(conn.port, 8080);
    assert_eq!(conn.timeout_secs, 30);
    assert_eq!(conn.retry_count, 3);
    assert_eq!(conn.use_ssl, false);

    // 链式配置
    let conn2 = ConnectionBuilder::new("production.db")
        .port(5432)
        .timeout(60)
        .retry(5)
        .enable_ssl()
        .build();
    assert_eq!(conn2.port, 5432);
    assert_eq!(conn2.timeout_secs, 60);
    assert_eq!(conn2.retry_count, 5);
    assert_eq!(conn2.use_ssl, true);
}

#[test]
/// 测试: RAII 守卫模式 (Drop trait / 资源管理)
fn test_raii_guard_pattern() {
    // 语法: 利用 Drop trait 实现资源自动清理
    //
    // 使用场景:
    //   - 文件句柄 / 网络连接
    //   - Mutex 锁守卫
    //   - 性能计时
    //   - 临时状态切换
    //
    // 避坑:
    //   - Drop::drop 不能手动调用 (用 std::mem::drop)
    //   - 析构顺序: 先声明后析构 (栈)，与声明顺序相反
    //   - 不能有 Copy trait 的类型同时有 Drop trait
    //

    use std::cell::RefCell;
    use std::rc::Rc;

    // 自定义 RAII 守卫: 退出作用域时执行回调
    struct Guard<F: FnOnce()> {
        callback: Option<F>,
    }

    impl<F: FnOnce()> Guard<F> {
        fn new(callback: F) -> Self {
            Guard {
                callback: Some(callback),
            }
        }
    }

    impl<F: FnOnce()> Drop for Guard<F> {
        fn drop(&mut self) {
            if let Some(cb) = self.callback.take() {
                cb();
            }
        }
    }

    // 测试: Guard 在离开作用域时自动执行回调
    let flag = Rc::new(RefCell::new(false));
    let flag_clone = flag.clone();
    {
        let _guard = Guard::new(move || {
            *flag_clone.borrow_mut() = true;
        });
        assert!(!*flag.borrow()); // 守卫存活期间, 未触发
    }
    assert!(*flag.borrow()); // 守卫析构, 标志位已设置

    // 测试: 多个守卫的析构顺序
    let log = Rc::new(RefCell::new(Vec::new()));
    {
        let log1 = log.clone();
        let _g1 = Guard::new(move || log1.borrow_mut().push("守卫1"));

        let log2 = log.clone();
        let _g2 = Guard::new(move || log2.borrow_mut().push("守卫2"));
    }
    // 先声明后析构 —— 所以 g2 先析构, g1 后析构
    assert_eq!(&*log.borrow(), &vec!["守卫2", "守卫1"]);

    // 用闭包模拟 RAII 模式
    fn scoped<F: FnOnce()>(name: &str, f: F, log: &Rc<RefCell<Vec<String>>>) {
        let log_enter = log.clone();
        let enter_name = name.to_string();
        let _guard = Guard::new(move || {
            log_enter.borrow_mut().push(format!("退出: {}", enter_name));
        });
        log.borrow_mut().push(format!("进入: {}", name));
        f();
    }

    let scoped_log = Rc::new(RefCell::new(Vec::new()));
    scoped(
        "操作",
        || {
            scoped_log.borrow_mut().push("执行".into());
        },
        &scoped_log,
    );
    assert_eq!(
        &*scoped_log.borrow(),
        &vec!["进入: 操作", "执行", "退出: 操作"]
    );
}

#[test]
/// 测试: 策略模式 via 闭包 (行为参数化)
fn test_strategy_pattern_with_closures() {
    // 语法: 使用闭包替代传统的策略接口
    //
    // 优势:
    //   - 无需定义 trait 和 impl 结构体
    //   - 闭包捕获环境, 策略可携带上下文
    //   - 编译期类型检查, 零成本抽象
    //
    // 避坑:
    //   - 多个不同策略需要 Box<dyn Fn> 统一类型
    //   - 闭包的生命周期需要关注
    //

    // 使用泛型闭包的策略处理器
    struct Processor<F>
    where
        F: Fn(i32) -> i32,
    {
        strategy: F,
    }

    impl<F: Fn(i32) -> i32> Processor<F> {
        fn new(strategy: F) -> Self {
            Processor { strategy }
        }

        fn process(&self, input: i32) -> i32 {
            (self.strategy)(input)
        }
    }

    // 策略 1: 加倍
    let doubler = Processor::new(|x| x * 2);
    assert_eq!(doubler.process(5), 10);
    assert_eq!(doubler.process(0), 0);
    assert_eq!(doubler.process(-3), -6);

    // 策略 2: 平方
    let squarer = Processor::new(|x| x * x);
    assert_eq!(squarer.process(5), 25);
    assert_eq!(squarer.process(-3), 9);

    // 策略 3: 带阈值的条件策略 (闭包捕获环境)
    let threshold = 5;
    let bonus = 2;
    let conditional = Processor::new(move |x| {
        if x > threshold {
            x * bonus
        } else {
            x
        }
    });
    assert_eq!(conditional.process(3), 3);
    assert_eq!(conditional.process(7), 14);

    // 使用 Box<dyn Fn> 实现运行时策略切换
    struct DynamicProcessor {
        strategy: Box<dyn Fn(i32) -> i32>,
    }

    impl DynamicProcessor {
        fn new(strategy: Box<dyn Fn(i32) -> i32>) -> Self {
            DynamicProcessor { strategy }
        }

        fn set_strategy(&mut self, strategy: Box<dyn Fn(i32) -> i32>) {
            self.strategy = strategy;
        }

        fn process(&self, input: i32) -> i32 {
            (self.strategy)(input)
        }
    }

    let mut processor = DynamicProcessor::new(Box::new(|x| x + 10));
    assert_eq!(processor.process(5), 15);

    // 运行时切换策略
    processor.set_strategy(Box::new(|x| x * 3));
    assert_eq!(processor.process(5), 15);

    processor.set_strategy(Box::new(|x| x - 2));
    assert_eq!(processor.process(5), 3);
}
