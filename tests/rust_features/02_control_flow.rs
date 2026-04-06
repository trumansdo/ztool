// ---------------------------------------------------------------------------
// 1.3 控制流 - if/for/while/loop/match
// ---------------------------------------------------------------------------

#[test]
/// 测试: if 条件表达式 (无括号/表达式返回值/无三元运算符)
fn test_if_expression() {
    // 语法: if 条件不需要括号; if 是表达式, 有返回值; Rust 没有三元运算符 ? :
    //
    // 基本用法:
    //   - if cond { ... }                    基本 if
    //   - if cond { ... } else { ... }       if-else
    //   - if c1 { ... } else if c2 { ... }   链式 if-else if
    //   - let x = if cond { a } else { b };  作为表达式赋值
    //
    // 避坑:
    //   - 条件必须是 bool 类型, 不支持隐式转换 (if x 而非 if x != 0)
    //   - if 不需要括号: if x > 0 而非 if (x > 0)
    //   - if/else 各分支返回值类型必须一致
    //   - 没有 else 时 if 表达式返回 ()
    //   - Rust 没有三元运算符, 用 if-else 表达式替代
    //
    let x: i32 = 5;

    // 基本 if (无括号)
    if x > 0 {
        assert!(x.is_positive());
    }

    // if-else 表达式
    let sign = if x > 0 {
        "positive"
    } else if x < 0 {
        "negative"
    } else {
        "zero"
    };
    assert_eq!(sign, "positive");

    // 替代三元运算符
    let abs = if x < 0 { -x } else { x };
    assert_eq!(abs, 5);

    // 分支类型必须一致
    let val = if true { 1 } else { 2 };
    assert_eq!(val, 1);
}

#[test]
/// 测试: if let 简化模式匹配
fn test_if_let() {
    // 语法: if let 简化单一模式匹配, 避免写完整的 match
    // 避坑: if let 没有 else 时, 不匹配的情况被静默忽略; 绑定变量作用域仅在 if 块内
    let value = Some(42);
    if let Some(x) = value {
        assert_eq!(x, 42);
    }

    // if let else
    let opt: Option<i32> = None;
    let result = if let Some(x) = opt { x } else { 0 };
    assert_eq!(result, 0);
}

#[test]
/// 测试: match 穷举匹配和范围模式
fn test_match() {
    // 语法: match 必须穷举所有可能, _ 通配符匹配剩余情况; 0..=3 是包含两端的范围模式
    // 避坑: match 是表达式, 每个分支返回值类型必须一致; 忘记穷举会编译失败
    let value = 5;
    match value {
        0..=3 => panic!("too small"),
        4..=10 => assert!(true),
        _ => panic!("too large"),
    }

    // match 作为表达式
    let day = 3;
    let name = match day {
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 | 7 => "Weekend", // 或模式
        _ => "Invalid",
    };
    assert_eq!(name, "Wednesday");
}

#[test]
/// 测试: match 高级用法 (守卫/解构/@绑定)
fn test_match_advanced() {
    // 语法: match 支持守卫(if guard)、解构、@ 绑定、匹配范围
    // 避坑: 守卫中的条件不能移动值; @ 绑定的变量和模式同时可用

    // match 守卫 (match guard)
    let num = Some(7);
    match num {
        Some(n) if n % 2 == 0 => assert!(false, "should not reach here"),
        Some(n) => assert!(n == 7),
        None => assert!(false),
    }

    // @ 绑定
    let x = 10;
    match x {
        n @ 1..=10 => assert_eq!(n, 10),
        _ => assert!(false),
    }

    // 解构元组
    let pair = (0, -2);
    match pair {
        (0, y) => assert_eq!(y, -2),
        (x, 0) => assert_eq!(x, 0),
        _ => assert!(false),
    }

    // 解构结构体
    struct Point {
        x: i32,
        y: i32,
    }
    let p = Point { x: 0, y: 7 };
    match p {
        Point { x: 0, y } => assert_eq!(y, 7),
        Point { x, y: 0 } => assert_eq!(x, 0),
        _ => assert!(false),
    }
}

#[test]
/// 测试: for 循环遍历 (into_iter/iter/iter_mut/范围循环)
fn test_for_loop() {
    // 语法: for x in iter { ... } 遍历迭代器, 自动调用 IntoIterator
    //
    // 遍历方式:
    //   - for x in vec         消费 Vec, 取得所有权
    //   - for x in &vec        借用遍历, x 是 &T
    //   - for x in &mut vec    可变借用遍历, x 是 &mut T
    //   - for x in a..b        范围遍历 [a, b)
    //   - for x in a..=b       包含范围遍历 [a, b]
    //   - for x in (0..10).step_by(2)  步长遍历
    //   - for (i, x) in iter.enumerate()  带索引遍历
    //   - for x in iter.rev()  反向遍历
    //
    // 避坑:
    //   - for 循环自动调用 into_iter(), 遍历后 Vec 被消费
    //   - 需要保留 Vec 时用 &vec 或 vec.iter()
    //   - for 循环不能手动 break 到指定标签(除非用 labeled loop)
    //   - 范围 a..b 中 a > b 时为空迭代, 不会 panic
    //
    // 消费遍历
    let vec = vec![1, 2, 3];
    let mut sum = 0;
    for x in vec {
        sum += x;
    }
    assert_eq!(sum, 6);
    // vec 已被消费, 不能再使用

    // 借用遍历
    let vec = vec![1, 2, 3];
    for x in &vec {
        assert!(*x > 0);
    }
    assert_eq!(vec.len(), 3); // vec 仍可用

    // 可变遍历
    let mut vec = vec![1, 2, 3];
    for x in &mut vec {
        *x *= 2;
    }
    assert_eq!(vec, vec![2, 4, 6]);

    // 范围遍历
    let mut sum = 0;
    for i in 1..=5 {
        sum += i;
    }
    assert_eq!(sum, 15);

    // 步长遍历
    let evens: Vec<i32> = (0..10).step_by(2).collect();
    assert_eq!(evens, vec![0, 2, 4, 6, 8]);

    // 带索引遍历
    let vec = vec!["a", "b", "c"];
    for (i, item) in vec.iter().enumerate() {
        assert_eq!(i, item.bytes().next().unwrap() as usize - b'a' as usize);
    }

    // 反向遍历
    let reversed: Vec<i32> = (1..=5).rev().collect();
    assert_eq!(reversed, vec![5, 4, 3, 2, 1]);
}

#[test]
/// 测试: while 循环和 while let 模式循环
fn test_while_loop() {
    // 语法: while cond { ... } 条件循环; while let pat = expr { ... } 模式循环
    // 避坑: while 条件必须是 bool; while let 不匹配时自动退出循环

    // 基本 while
    let mut count = 0;
    while count < 5 {
        count += 1;
    }
    assert_eq!(count, 5);

    // while let (模式循环, 常用于 Option/Result/迭代器)
    let mut stack = vec![1, 2, 3];
    let mut result = Vec::new();
    while let Some(top) = stack.pop() {
        result.push(top);
    }
    assert_eq!(result, vec![3, 2, 1]);

    // while let 处理迭代器
    let mut iter = vec![10, 20, 30].into_iter();
    let mut sum = 0;
    while let Some(x) = iter.next() {
        sum += x;
    }
    assert_eq!(sum, 60);
}

#[test]
/// 测试: loop 无限循环和 break/continue
fn test_loop() {
    // 语法: loop { ... } 无限循环, 必须用 break 退出
    //
    // loop 特性:
    //   - loop 保证至少执行一次, 编译器可做更激进优化
    //   - break 可返回值: break value;
    //   - continue 跳过本次循环
    //   - 可嵌套使用, 用标签指定跳出哪层
    //
    // 避坑:
    //   - loop 必须有退出条件, 否则死循环
    //   - break 返回值类型必须与 loop 表达式期望类型一致
    //   - 没有 break 的 loop 返回 ! (never 类型)
    //   - while true 和 loop 不等价: loop 保证执行, while 可能不执行
    //
    // 基本 loop + break
    let mut i = 0;
    loop {
        i += 1;
        if i >= 5 {
            break;
        }
    }
    assert_eq!(i, 5);

    // loop 返回值
    let result = loop {
        let x = 42;
        if x > 0 {
            break x;
        }
    };
    assert_eq!(result, 42);

    // continue
    let mut sum = 0;
    let mut i = 0;
    loop {
        i += 1;
        if i > 10 {
            break;
        }
        if i % 2 == 0 {
            continue;
        }
        sum += i;
    }
    assert_eq!(sum, 1 + 3 + 5 + 7 + 9); // 奇数和
}

#[test]
/// 测试: 循环标签 (labeled loops) 和嵌套 break/continue
fn test_labeled_loops() {
    // 语法: 'label: loop/for/while { ... } 给循环加标签, break/continue 可指定标签
    // 避坑: 标签名以 ' 开头; break label 必须用于 loop, 不能用于 if/else

    // 嵌套循环, 跳出外层
    let mut found = None;
    'outer: for i in 1..=5 {
        for j in 1..=5 {
            if i * j == 12 {
                found = Some((i, j));
                break 'outer;
            }
        }
    }
    assert_eq!(found, Some((3, 4)));

    // continue 外层循环
    let mut count = 0;
    'outer: for i in 1..=3 {
        for j in 1..=3 {
            if j == 2 {
                continue 'outer; // 跳过 i 的下一轮
            }
            count += 1;
        }
    }
    assert_eq!(count, 3); // 每轮 i 只计数一次 (j=1 时)

    // loop 嵌套, 指定 break 层级
    let result = 'a: loop {
        let mut inner = 0;
        loop {
            inner += 1;
            if inner >= 3 {
                break 'a inner; // 跳出外层并返回值
            }
        }
    };
    assert_eq!(result, 3);
}

#[test]
/// 测试: 控制流中的类型系统 (never 类型/发散表达式)
fn test_control_flow_types() {
    // 语法: ! (never 类型) 表示永不返回; panic!/return/break/continue 都是 ! 类型
    //
    // never 类型特性:
    //   - ! 可以强制转换为任何类型 (coerce to any type)
    //   - panic!() 的类型是 !, 所以可放在任何分支
    //   - 空 match 表达式返回 ! 类型
    //   - 函数返回 ! 标记为发散函数 (diverging function)
    //
    // 避坑:
    //   - ! 类型不能创建值, 只能通过发散表达式获得
    //   - match 的所有分支类型必须一致, ! 可自动转换匹配
    //
    // panic! 在任何位置都类型兼容
    let x: i32 = if true {
        42
    } else {
        panic!("impossible"); // ! 类型, 可转为 i32
    };
    assert_eq!(x, 42);

    // 发散函数
    fn never_returns() -> ! {
        panic!("always panics");
    }

    // 返回值推导
    let val: i32 = match Some(10) {
        Some(n) => n,
        None => return, // () 类型, 但在 -> i32 上下文中... 这里用 panic
    };
    assert_eq!(val, 10);
}

#[test]
/// 测试: Rust 特有控制流模式 (守卫/范围/解构组合)
fn test_rust_specific_patterns() {
    // 语法: Rust 独有的控制流特性, 其他语言少见

    // 1. if let + else 链
    let value: Result<i32, &str> = Ok(42);
    let result = if let Ok(n) = value {
        n * 2
    } else if let Err(e) = value {
        panic!("error: {}", e);
    } else {
        unreachable!()
    };
    assert_eq!(result, 84);

    // 2. match 中匹配引用
    let opt = &Some(String::from("hello"));
    match opt {
        Some(s) => assert_eq!(s, "hello"),
        None => assert!(false),
    }

    // 3. 范围模式匹配
    let c = 'M';
    let category = match c {
        'a'..='z' => "lowercase",
        'A'..='Z' => "uppercase",
        '0'..='9' => "digit",
        _ => "other",
    };
    assert_eq!(category, "uppercase");

    // 4. 匹配时忽略 (_ 忽略部分字段)
    struct Config {
        host: String,
        port: u16,
        timeout: u32,
    }
    let cfg = Config {
        host: "localhost".to_string(),
        port: 8080,
        timeout: 30,
    };
    match cfg {
        Config { port: 8080, .. } => assert!(true),
        _ => assert!(false),
    }
}
