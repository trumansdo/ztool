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
    'outer: for _i in 1..=3 {
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

// ---------------------------------------------------------------------------
// 新增测试：if 表达式类型深入
// ---------------------------------------------------------------------------

#[test]
/// 测试: if/else if/else 链的类型统一规则及 return 分支的 never 类型推导
fn test_if_expression_types() {
    // 语法: 多分支 if/else if/else 中, 所有分支返回值类型必须一致
    // 避坑: 分支包含 return/panic! 时类型为 !, 不影响统一; 无 else 时返回 ()
    //
    // 多分支链类型统一
    let n = 42;
    let desc: &str = if n < 0 {
        "负"
    } else if n == 0 {
        "零"
    } else if n % 2 == 0 {
        "正偶数"
    } else {
        "正奇数"
    };
    assert_eq!(desc, "正偶数");

    // 复杂类型统一 (String)
    let code = 200;
    let msg = if code == 200 {
        String::from("OK")
    } else {
        format!("错误: {code}")
    };
    assert_eq!(msg, "OK");

    // return 分支不影响类型统一 (never 类型 !)
    fn classify_early(n: i32) -> &'static str {
        let desc: &str = if n < 0 {
            "负"
        } else if n > 0 {
            "正"
        } else {
            return "这是零, 提前返回"; // ! 类型, 不影响分支类型统一
        };
        desc
    }
    assert_eq!(classify_early(0), "这是零, 提前返回");
    assert_eq!(classify_early(10), "正");

    // panic! 分支也不影响类型统一
    let _v: i32 = if true {
        42
    } else if false {
        24
    } else {
        panic!("不可能到达");
    };
}

#[test]
/// 测试: if let 与 match 的等价性验证
fn test_if_let_vs_match() {
    // 语法: if let PAT = EXPR { A } else { B } 完全等价于 match EXPR { PAT => A, _ => B }
    // 避坑: if let 的单分支绑定变量作用域仅在 if 块内; 守卫中绑定变量的作用域延续到整个条件

    let opt = Some(42);

    // 等价验证1: if let vs match (有值的情况)
    let r1 = if let Some(x) = opt { x } else { 0 };
    let r2 = match opt {
        Some(x) => x,
        None => 0,
    };
    assert_eq!(r1, r2);
    assert_eq!(r1, 42);

    // 等价验证2: if let vs match (无值的情况)
    let opt_none: Option<i32> = None;
    let r1 = if let Some(x) = opt_none { x } else { -1 };
    let r2 = match opt_none {
        Some(x) => x,
        None => -1,
    };
    assert_eq!(r1, r2);
    assert_eq!(r1, -1);

    // 等价验证3: if let 守卫 — 守卫中的变量在条件范围内可见 (Rust 1.65+)
    let opt = Some(100);
    let r1 = if let Some(x) = opt && x > 50 { x * 2 } else { 0 };
    let r2 = match opt {
        Some(x) if x > 50 => x * 2,
        _ => 0,
    };
    assert_eq!(r1, r2);
    assert_eq!(r1, 200);

    // 链式 if let-else if let
    let opt1: Option<i32> = None;
    let opt2: Option<i32> = Some(99);
    let result = if let Some(x) = opt1 {
        x
    } else if let Some(y) = opt2 {
        y
    } else {
        0
    };
    assert_eq!(result, 99);
}

#[test]
/// 测试: loop 作为表达式返回值的多种场景
fn test_loop_as_expression() {
    // 语法: loop 通过 break value 返回值, 可返回任意类型
    // 避坑: break 无值则 loop 返回 (); break 值类型必须与上下文期望一致

    // 场景1: loop 返回简单值
    let result = loop {
        break 42;
    };
    assert_eq!(result, 42);

    // 场景2: loop 返回 Option — 搜索型用法
    let items = vec![1, 3, 5, 7, 9, 12];
    let found = loop {
        let mut iter = items.iter();
        let mut result = None;
        loop {
            match iter.next() {
                Some(&x) if x % 2 == 0 => {
                    result = Some(x);
                    break;
                }
                Some(_) => continue,
                None => break,
            }
        }
        break result;
    };
    assert_eq!(found, Some(12));

    // 场景3: loop 返回元组 — 计算型用法
    let (count, sum) = loop {
        let mut i = 0;
        let mut s = 0;
        let result = loop {
            i += 1;
            s += i;
            if i >= 10 {
                break (i, s);
            }
        };
        break result;
    };
    assert_eq!(count, 10);
    assert_eq!(sum, 55); // 1+2+...+10 = 55

    // 场景4: loop 返回 String — 复杂类型
    let msg = loop {
        let s = String::from("hello");
        break s;
    };
    assert_eq!(msg, "hello");

    // 场景5: 无 break 的 loop 类型验证 (通过静态方式)
    // loop {} 类型为 !, 可以被赋给任意类型
    fn never_returns() -> i32 {
        loop {} // ! 强制转为 i32 — 实际上永不返回
    }
    // 无法在测试中真正调用 never_returns(), 但类型签名证明了 ! → i32 的强制转换
    let _: fn() -> i32 = never_returns;
}

#[test]
/// 测试: 嵌套循环标签的高级用法 (三维/模式组合)
fn test_loop_labels() {
    // 语法: 'label: loop/for/while { break 'label value; } 跳出指定层级
    // 避坑: 标签仅能跳出或跳过循环, 不能跳入; 标签绑定到最内层的循环

    // 场景1: 三维搜索 — 一次性跳出所有层级
    let matrix = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]];
    let result = 'outermost: loop {
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    if matrix[i][j][k] == 6 {
                        break 'outermost (i, j, k);
                    }
                }
            }
        }
        break (99, 99, 99); // 兜底
    };
    assert_eq!(result, (1, 0, 1)); // matrix[1][0][1] = 6

    // 场景2: 标签 + continue 组合 — 跳过外循环的当前迭代
    let mut count = 0;
    'row: for row in 0..4 {
        for col in 0..4 {
            if col > row {
                continue 'row; // 跳过整行
            }
            count += 1;
        }
    }
    // 只处理下三角: row0 1次, row1 2次, row2 3次, row3 4次 = 10
    assert_eq!(count, 10);

    // 场景3: loop 内部嵌套 for, 标签跳出
    let target = 25;
    let result = 'find: loop {
        for x in 1..=10 {
            for y in 1..=10 {
                if x * y == target {
                    break 'find Some((x, y));
                }
            }
        }
        break None;
    };
    assert_eq!(result, Some((5, 5)));

    // 场景4: 标签命名可包含字母和数字
    let mut found = false;
    'l1: for _a in 0..1 {
        for _b in 0..1 {
            found = true;
            break 'l1;
        }
    }
    assert!(found);
}

#[test]
/// 测试: while let 模式匹配循环的多种场景
fn test_while_let() {
    // 语法: while let PAT = EXPR { ... } 在模式匹配成功时执行, 失败时退出
    // 避坑: 退出循环后, 循环内部绑定的变量不可见; while let 的匹配在每轮迭代开始时重新评估

    // 场景1: while let 消费 Vec (栈操作)
    let mut stack = vec![1, 2, 3, 4];
    let mut popped = Vec::new();
    while let Some(top) = stack.pop() {
        popped.push(top);
    }
    assert_eq!(popped, vec![4, 3, 2, 1]);
    assert!(stack.is_empty());

    // 场景2: while let 消费迭代器 (等价 for 的底层实现)
    let vals = vec![10, 20, 30];
    let mut iter = vals.into_iter();
    let mut sum = 0;
    while let Some(x) = iter.next() {
        sum += x;
    }
    assert_eq!(sum, 60);

    // 场景3: while let 处理 Result 迭代 — 遇到 Err 自动终止
    let results = vec![Ok(10), Ok(20), Err("失败"), Ok(30)];
    let mut iter = results.into_iter();
    let mut sum = 0;
    while let Some(Ok(val)) = iter.next() {
        sum += val;
    }
    assert_eq!(sum, 30); // 只累加了前两个, 遇到 Err 后退出

    // 场景4: while let 带守卫条件
    let nums = vec![1, 5, 10, 15, 20, 30];
    let mut iter = nums.into_iter();
    let mut count = 0;
    // while let 守卫需要 nightly: 改用内部 if
    while let Some(x) = iter.next() {
        if x > 10 {
            count += 1;
        }
    }
    assert_eq!(count, 3); // 15, 20, 30

    // 场景5: while let 连续消费两个源
    let mut s1 = vec![1, 2];
    let mut s2 = vec![3, 4];
    let mut interleaved = Vec::new();
    while let Some(x) = s1.pop() {
        interleaved.push(x);
        if let Some(y) = s2.pop() {
            interleaved.push(y);
        }
    }
    assert_eq!(interleaved, vec![2, 4, 1, 3]);
}

#[test]
/// 测试: for 循环与 IntoIterator trait 的关系及迭代器方法
fn test_for_iteration_patterns() {
    // 语法: for x in expr 自动调用 IntoIterator::into_iter(expr)
    // 避坑: 直接 for x in vec 会消费所有权; 三种遍历 (into_iter/iter/iter_mut) 对应三种借用

    // 验证 IntoIterator 的存在性
    let v = vec![1, 2, 3];
    let mut iter = v.into_iter(); // 显式调用
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);

    // iter() 不消费 — 保留所有权
    let v = vec![1, 2, 3];
    let mut squared = Vec::new();
    for x in v.iter() {
        squared.push(x * x);
    }
    assert_eq!(v, vec![1, 2, 3]); // v 仍可用
    assert_eq!(squared, vec![1, 4, 9]);

    // cloned() / copied() — 复制值
    let v = vec![1, 2, 3];
    let doubled: Vec<i32> = v.iter().copied().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![2, 4, 6]);

    // zip — 并行迭代
    let names = vec!["Alice", "Bob", "Carol"];
    let scores = vec![85, 92, 78];
    let mut result = Vec::new();
    for (name, score) in names.iter().zip(scores.iter()) {
        result.push((*name, *score));
    }
    assert_eq!(result, vec![("Alice", 85), ("Bob", 92), ("Carol", 78)]);

    // 迭代器链式调用
    let result: Vec<i32> = (0..20)
        .filter(|x| x % 2 == 1)  // 奇数
        .map(|x| x * x)           // 平方
        .take(3)                  // 只取前3个
        .collect();
    assert_eq!(result, vec![1, 9, 25]); // 1^2, 3^2, 5^2
}

#[test]
/// 测试: Range 类型的多种用法 (.. / ..= / step_by / rev)
fn test_range_types() {
    // 语法: a..b 左闭右开, a..=b 闭区间, step_by(n) 步进, rev() 反向
    // 避坑: 正向 Range 中 start > end 为空迭代; Range 实现了 IntoIterator

    // 半开区间 ..
    let range: Vec<i32> = (0..5).collect();
    assert_eq!(range, vec![0, 1, 2, 3, 4]);
    // 空区间
    let empty: Vec<i32> = (5..0).collect();
    assert!(empty.is_empty());

    // 闭区间 ..=
    let inclusive: Vec<i32> = (0..=5).collect();
    assert_eq!(inclusive, vec![0, 1, 2, 3, 4, 5]);

    // step_by — 步进范围
    let evens: Vec<i32> = (0..10).step_by(2).collect();
    assert_eq!(evens, vec![0, 2, 4, 6, 8]);
    let odds: Vec<i32> = (1..10).step_by(2).collect();
    assert_eq!(odds, vec![1, 3, 5, 7, 9]);

    // 步进 + 闭区间
    let step3: Vec<i32> = (0..=9).step_by(3).collect();
    assert_eq!(step3, vec![0, 3, 6, 9]);

    // rev — 反向迭代
    let descending: Vec<i32> = (0..5).rev().collect();
    assert_eq!(descending, vec![4, 3, 2, 1, 0]);
    let desc_inclusive: Vec<i32> = (1..=5).rev().collect();
    assert_eq!(desc_inclusive, vec![5, 4, 3, 2, 1]);

    // 组合：步进 + 反向 (避免 RangeInclusive 不支持 ExactSizeIterator)
    let rev_step: Vec<i32> = (0..12).step_by(3).rev().collect();
    // 正向 step_by(3): 0, 3, 6, 9  →  反向: 9, 6, 3, 0
    assert_eq!(rev_step, vec![9, 6, 3, 0]);

    // Range 可作为值传递
    let r = 0..5;
    let v: Vec<i32> = r.collect();
    assert_eq!(v, vec![0, 1, 2, 3, 4]);

    // Range 实现了 DoubleEndedIterator (双向迭代器)
    let mut r = 0..5;
    assert_eq!(r.next(), Some(0));
    assert_eq!(r.next_back(), Some(4));
    assert_eq!(r.next(), Some(1));
    assert_eq!(r.next_back(), Some(3));
}

#[test]
/// 测试: enumerate 获取索引及与 zip 的组合
fn test_enumerate_and_index() {
    // 语法: iter.enumerate() 将 T -> (usize, T), 索引从 0 开始
    // 避坑: enumerate 的索引始终从 0 开始, 不能自定义起始值; 使用 zip 可合并多个序列

    // 基本 enumerate
    let items = ["苹果", "香蕉", "樱桃"];
    let mut result = Vec::new();
    for (i, item) in items.iter().enumerate() {
        result.push((i, *item));
    }
    assert_eq!(result, vec![(0, "苹果"), (1, "香蕉"), (2, "樱桃")]);

    // enumerate 与 Range 组合
    let mut pairs = Vec::new();
    for (i, val) in (100..105).enumerate() {
        pairs.push((i, val));
    }
    assert_eq!(pairs, vec![(0, 100), (1, 101), (2, 102), (3, 103), (4, 104)]);

    // enumerate + zip 多维组合
    let names = ["Alice", "Bob", "Carol"];
    let scores = [85, 92, 78];
    let mut report = Vec::new();
    for ((i, name), score) in names.iter().enumerate().zip(scores.iter()) {
        report.push((i, *name, *score));
    }
    assert_eq!(
        report,
        vec![(0, "Alice", 85), (1, "Bob", 92), (2, "Carol", 78)]
    );

    // enumerate 在消费迭代中
    let v = vec!["x", "y", "z"];
    let mut indices = Vec::new();
    for (i, _) in v.into_iter().enumerate() {
        indices.push(i);
    }
    assert_eq!(indices, vec![0, 1, 2]);

    // enumerate 与 filter 结合定位元素
    let nums = vec![0, 3, 7, 9, 12, 15];
    let mut positions = Vec::new();
    for (i, &val) in nums.iter().enumerate() {
        if val % 3 == 0 && val > 5 {
            positions.push(i);
        }
    }
    assert_eq!(positions, vec![3, 4, 5]); // 9(索引3), 12(索引4), 15(索引5)
}

#[test]
/// 测试: 循环中的借用模式 (所有权/不可变借用/可变借用)
fn test_borrowing_in_loops() {
    // 语法: for x in &vec (不可变借用), for x in &mut vec (可变借用), for x in vec (消费)
    // 避坑: 消费遍历后原集合不可再用; 可变借用遍历期间不能同时读原集合

    // 不可变借用 — 集合遍历后仍可用
    let v = vec![String::from("hello"), String::from("world")];
    let mut lengths = Vec::new();
    for s in &v {
        lengths.push(s.len());
    }
    assert_eq!(v.len(), 2); // v 仍可用
    assert_eq!(lengths, vec![5, 5]);

    // 可变借用 — 修改元素
    let mut v = vec![String::from("he"), String::from("wo")];
    for s in &mut v {
        s.push_str("!");
    }
    assert_eq!(v, vec!["he!", "wo!"]);

    // 可变借用 — 替换元素值
    let mut nums = vec![1, 2, 3];
    for n in &mut nums {
        *n *= 2;
    }
    assert_eq!(nums, vec![2, 4, 6]);

    // 消费遍历 — 取得每个元素的所有权
    let v = vec![String::from("a"), String::from("b")];
    let mut owned = Vec::new();
    for s in v {
        // s: String, 所有权在循环体内
        owned.push(s.to_uppercase());
    }
    // v 已被消费, 无法再使用
    assert_eq!(owned, vec!["A", "B"]);

    // 循环中借用规则：不可变引用可与不可变引用共存
    let v = vec![1, 2, 3];
    let ref1 = &v;
    let ref2 = &v;
    let mut sum = 0;
    for x in ref1 {
        sum += x;
    }
    assert_eq!(*ref2, vec![1, 2, 3]); // ref2 仍有效
    assert_eq!(sum, 6);

    // 使用 iter_mut() 显式获取可变迭代器
    let mut v = vec![10, 20, 30];
    for item in v.iter_mut() {
        *item += 5;
    }
    assert_eq!(v, vec![15, 25, 35]);
}

#[test]
/// 测试: 控制流在错误处理中的应用 (? 操作符 / if let Ok/Err / let-else)
fn test_control_flow_in_error_handling() {
    // 语法: ? 传播错误; if let Ok/Err 分别处理成功/失败; let-else 解构或退出
    // 避坑: ? 需要在返回 Result/Option 的函数中使用; 不要忽略 Result

    // 辅助函数: 展示 ? 操作符的传播机制
    fn parse_and_double(s: &str) -> Result<i32, std::num::ParseIntError> {
        let n: i32 = s.parse()?; // 成功则解包, 失败则提前 return Err
        Ok(n * 2)
    }
    assert_eq!(parse_and_double("21").unwrap(), 42);
    assert!(parse_and_double("abc").is_err());

    // 辅助函数: 多个 ? 串联
    fn process(input: &str) -> Result<i32, Box<dyn std::error::Error>> {
        let trimmed = input.trim();
        let n: i32 = trimmed.parse()?;
        if n == 0 {
            return Err("值为零".into());
        }
        Ok(100 / n)
    }
    assert_eq!(process("10").unwrap(), 10);
    assert!(process("0").is_err());
    assert!(process("   ").is_err());

    // if let Ok — 只处理成功分支
    let result: Result<i32, &str> = Ok(42);
    if let Ok(val) = result {
        assert_eq!(val, 42);
    }

    // if let Err — 只处理错误分支
    let result: Result<i32, &str> = Err("失败");
    if let Err(e) = result {
        assert_eq!(e, "失败");
    }

    // 循环中过滤 Result
    let items = vec![Ok(1), Err("x"), Ok(3), Err("y")];
    let mut successes = Vec::new();
    for item in &items {
        if let Ok(v) = item {
            successes.push(*v);
        }
    }
    assert_eq!(successes, vec![1, 3]);

    // let-else 模式 (Rust 1.65+): 成功解包, 失败则退出
    fn use_let_else(opt: Option<i32>) -> Option<i32> {
        let Some(val) = opt else {
            return None;
        };
        Some(val * 10)
    }
    assert_eq!(use_let_else(Some(7)), Some(70));
    assert_eq!(use_let_else(None), None);

    // Option 的 ? 传播
    fn maybe_add(a: Option<i32>, b: Option<i32>) -> Option<i32> {
        let x = a?;
        let y = b?;
        Some(x + y)
    }
    assert_eq!(maybe_add(Some(10), Some(20)), Some(30));
    assert_eq!(maybe_add(None, Some(20)), None);
}

#[test]
/// 测试: match 作为控制流表达式的基本用法 (赋值/穷举/守卫/多模式)
fn test_match_as_control_flow() {
    // 语法: match 是穷举的模式匹配表达式, 每个分支返回相同类型, _ 匹配剩余
    // 避坑: match 必须穷举; 每个分支类型必须一致; 分支内部可使用 return/panic! 提前退出

    // 场景1: match 作为表达式赋值
    let weather = "多云";
    let activity = match weather {
        "晴天" => "远足",
        "雨天" => "读书",
        "雪天" => "滑雪",
        "多云" => "骑行",
        _ => "休息",
    };
    assert_eq!(activity, "骑行");

    // 场景2: 穷举性 — 自定义 enum
    enum TrafficLight {
        Red,
        Yellow,
        Green,
    }
    let light = TrafficLight::Yellow;
    let action = match light {
        TrafficLight::Red => "停止",
        TrafficLight::Yellow => "注意",
        TrafficLight::Green => "通行",
    };
    assert_eq!(action, "注意");

    // 场景3: match 守卫 — 条件细化
    let num = Some(15);
    let category = match num {
        None => "无值",
        Some(n) if n < 0 => "负数",
        Some(n) if n == 0 => "零",
        Some(n) if n % 2 == 0 => "偶数",
        Some(_) => "奇数",
    };
    assert_eq!(category, "奇数");

    // 场景4: 多模式匹配 (| 运算符)
    let code = 404;
    let status = match code {
        200 | 201 | 204 => "成功",
        301 | 302 => "重定向",
        401 | 403 | 404 => "客户端错误",
        500..=599 => "服务器错误",
        _ => "未知",
    };
    assert_eq!(status, "客户端错误");

    // 场景5: match 分支内 return 不影响类型统一
    fn classify_fruit(fruit: &str) -> &'static str {
        match fruit {
            "苹果" | "香蕉" => "常见",
            "榴莲" => return "特殊水果: 有气味",
            "火龙果" => "异域",
            _ => "其他",
        }
    }
    assert_eq!(classify_fruit("苹果"), "常见");
    assert_eq!(classify_fruit("榴莲"), "特殊水果: 有气味");

    // 场景6: match 解构 Option 并计算
    let opt = Some(7);
    let doubled = match opt {
        Some(n) => n * 2,
        None => 0,
    };
    assert_eq!(doubled, 14);
}
