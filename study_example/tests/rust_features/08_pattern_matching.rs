// ---------------------------------------------------------------------------
// 2.1 模式匹配 —— 穷举性 / 全套模式语法 / 可反驳模式 / let-else / matches!
// ---------------------------------------------------------------------------

#[test]
/// 测试: 穷举性检查的各类边界 —— 编译器如何知道你覆盖了所有情况
fn test_exhaustiveness() {
    // 穷举性是 Rust 模式匹配的核心安全保证: 编译器在类型层面验证所有可能路径
    // 原理: 编译器利用 ADT(代数数据类型)的 "和类型" / "积类型" 分解，
    //       逐层展开变体, 直到确认每个可能的构造形式都被覆盖。

    // —— 边界1: 枚举必须覆盖所有变体 ——
    enum TrafficLight4 {
        Red,
        Yellow,
        Green,
        Blue, // 新增变体 → 编译立刻报错 (除非有通配符 _)
    }
    let light = TrafficLight4::Red;
    match light {
        TrafficLight4::Red => {}
        TrafficLight4::Yellow => {}
        TrafficLight4::Green => {}
        TrafficLight4::Blue => {} // 不写 _ 也能通过: 四个变体全覆盖
    }

    // —— 边界2: 整数范围无法穷举, 必须靠 _ ——
    let num = 42_i32;
    match num {
        0 => {}
        1..=100 => {}
        _ => {} // i32 范围无限, 必须有通配符
    }

    // —— 边界3: bool 类型只有 true/false, 两个分支即穷举 ——
    let flag = true;
    match flag {
        true => {}
        false => {} // 不需要 _
    }

    // —— 边界4: Option<bool> 有 3 个形态 (None, Some(true), Some(false)) ——
    let opt = Some(true);
    match opt {
        None => {}
        Some(true) => {}
        Some(false) => {} // 穷举! 不需要 _
    }

    // —— 边界5: 嵌套枚举穷举 ——
    let nested: Option<Option<i32>> = Some(None);
    match nested {
        None => {}
        Some(None) => {}
        Some(Some(_)) => {}
    }

    // —— 边界6: 结构体 + 枚举组合 ——
    enum Shape {
        Circle { radius: f64 },
        Rect { width: f64, height: f64 },
    }
    let s = Shape::Circle { radius: 5.0 };
    match &s {
        Shape::Circle { radius } if *radius > 0.0 => {}
        Shape::Circle { radius: _ } => {}
        Shape::Rect { .. } => {}
    }

    // —— 边界7: 或模式减少重复同时保持穷举 ——
    match light {
        TrafficLight4::Red | TrafficLight4::Yellow => {}
        TrafficLight4::Green | TrafficLight4::Blue => {}
    }
}

#[test]
/// 测试: 全套模式语法 —— 字面量 / 变量绑定 / 通配符 / 守卫 / 或模式 / 范围 / 解构 / ref / ref mut / @
fn test_pattern_syntax_comprehensive() {
    // ===== 字面量模式 =====
    let x = 42;
    match x {
        0 => panic!(),
        42 => {} // 字面量精确匹配
        _ => panic!(),
    }

    // ===== 变量绑定模式 =====
    let x = 99;
    match x {
        value => assert_eq!(value, 99), // value 绑定到 x
    }

    // ===== 通配符 _ =====
    let pair = (1, 2);
    match pair {
        (1, _) => {} // _ 匹配任意但不绑定
        _ => panic!(),
    }

    // ===== 范围模式 ..= (包含) 和 .. (独占) =====
    let c = 'k';
    match c {
        'a'..='z' => {} // 包含两端
        _ => panic!(),
    }

    // ===== 或模式 | =====
    let num = 3;
    match num {
        1 | 3 | 5 | 7 => {}
        _ => panic!(),
    }

    // ===== 解构模式 (元组/结构体) =====
    struct Pos {
        x: i32,
        y: i32,
    }
    let pos = Pos { x: 10, y: 20 };
    match pos {
        Pos { x, y } => {
            assert_eq!(x, 10);
            assert_eq!(y, 20);
        }
    }

    // ===== 引用模式 ref / ref mut =====
    let name = String::from("hello");
    match name {
        ref n => {
            assert_eq!(n, "hello");
            // name 未被移动, n 是引用
        }
    }

    let mut val = 100;
    match val {
        ref mut v => {
            *v = 200;
        }
    }
    assert_eq!(val, 200);

    // ===== @ 绑定 =====
    let num = 50;
    match num {
        n @ 40..=60 => assert_eq!(n, 50),
        _ => panic!(),
    }

    // ===== 守卫 if =====
    let opt = Some(7);
    match opt {
        Some(n) if n > 5 && n < 10 => {}
        _ => panic!(),
    }

    // ===== 解构 + 守卫组合 =====
    enum Color {
        Rgb(u8, u8, u8),
        Hsl(u16, u8, u8),
    }
    let c = Color::Rgb(255, 128, 0);
    match c {
        Color::Rgb(r, g, _) if r > g => {} // 红分量大于绿分量
        Color::Hsl(..) => panic!(),
        _ => {}
    }
}

#[test]
/// 测试: | 模式与守卫结合 —— 守卫作用于整个 | 分支
fn test_or_pattern_with_guard() {
    // match 臂的格式:  pattern1 | pattern2 if guard => body
    // 守卫作用于整个 | 分支 (所有子模式)

    // —— 基本组合 ——
    let x = Some(3);
    match x {
        Some(1) | Some(2) | Some(3) if true => {}
        _ => panic!(),
    }

    // —— 守卫共享 ——
    let num = 5;
    match num {
        1 | 3 | 5 if num % 2 == 1 => {} // 所有子模式共享守卫
        _ => panic!(),
    }

    // —— 守卫阻止匹配 ——
    let num = 4;
    match num {
        n @ (2 | 4 | 6) if n > 5 => panic!("守卫应阻止"),
        n @ (2 | 4 | 6) => assert_eq!(n, 4), // 守卫失败后继续匹配
        _ => panic!(),
    }

    // —— 同一个 match 中多次匹配 ——
    let x = 7;
    match x {
        n if n < 5 => panic!(),
        n @ (5 | 6 | 7) if n % 2 == 0 => panic!(),
        n @ (5 | 6 | 7) => assert_eq!(n, 7),
        _ => panic!(),
    }

    // —— 枚举变体 + 或模式 + 守卫 ——
    #[derive(Debug, PartialEq)]
    enum Event {
        Key(char),
        Click { x: i32, y: i32 },
        Resize(u32, u32),
    }
    let ev = Event::Key('a');
    match ev {
        Event::Key(c @ ('a' | 'A')) => assert_eq!(c, 'a'), // 或模式绑定
        Event::Resize(w, _h) if w > 100 => panic!(),
        Event::Click { x, y } if x > 0 && y > 0 => {}
        _ => {}
    }
}

#[test]
/// 测试: 不可反驳模式 vs 可反驳模式 —— 概念与使用场景
fn test_refutable_vs_irrefutable() {
    // 不可反驳模式 (irrefutable): 永远匹配成功
    //   适用: let, for, 函数参数
    //   示例: x, (x, y), Point { x, y }, _
    //
    // 可反驳模式 (refutable): 可能不匹配
    //   适用: if let, while let, match arm
    //   示例: Some(x), Ok(x), [1, 2, 3], 0..=10

    // ===== 不可反驳模式在 let 中 =====
    let (a, _b) = (1, 2);
    assert_eq!(a, 1);

    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }
    let Point { x, y: _ } = Point { x: 3, y: 4 };
    assert_eq!(x, 3);

    // ===== 不可反驳模式在函数参数中 =====
    fn print_pair(&(x, y): &(i32, i32)) {
        assert!(x >= 0);
        assert!(y >= 0);
    }
    print_pair(&(1, 2));

    // ===== 不可反驳模式在 for 循环中 =====
    let pairs = [(1, 2), (3, 4)];
    for &(x, y) in &pairs {
        assert!(x < y);
    }

    // ===== 可反驳模式在 if let 中 =====
    let opt: Option<i32> = Some(42);
    if let Some(x) = opt {
        assert_eq!(x, 42);
    } else {
        panic!();
    }

    // ===== 可反驳模式在 while let 中 =====
    let mut stack = vec![1, 2, 3];
    let mut count = 0;
    while let Some(_top) = stack.pop() {
        count += 1;
    }
    assert_eq!(count, 3);

    // ===== 复合场景: match 中可以混合 =====
    let val: Result<i32, &str> = Ok(5);
    match val {
        Ok(n) if n > 0 => {}
        Ok(_) => {}
        Err(e) => assert_eq!(e, ""), // 不会到这
    }
}

#[test]
/// 测试: let-else 语句 —— 模式不匹配时走 else 发散分支 (Rust 1.65+)
fn test_let_else_statement() {
    // let-else 语法: let PATTERN = expr else { diverging_block };
    // else 块必须发散: return / break / continue / panic! / todo! / unreachable!

    // —— 基本场景: 从 Option 提取值 ——
    fn double_if_some(val: Option<i32>) -> i32 {
        let Some(x) = val else {
            return 0;
        };
        x * 2
    }
    assert_eq!(double_if_some(Some(5)), 10);
    assert_eq!(double_if_some(None), 0);

    // —— 解构 Result ——
    fn get_value(res: Result<i32, &str>) -> i32 {
        let Ok(v) = res else {
            return -1;
        };
        v
    }
    assert_eq!(get_value(Ok(42)), 42);
    assert_eq!(get_value(Err("失败")), -1);

    // —— 结构体解构 + let-else ——
    struct Config {
        debug: bool,
        port: u16,
    }
    fn get_config(opt: Option<Config>) -> u16 {
        let Some(Config { port, .. }) = opt else {
            return 8080; // 默认端口
        };
        port
    }
    assert_eq!(
        get_config(Some(Config {
            debug: true,
            port: 3000
        })),
        3000
    );
    assert_eq!(get_config(None), 8080);

    // —— 嵌套解构 + let-else ——
    fn get_inner(opt: Option<Option<i32>>) -> i32 {
        let Some(Some(x)) = opt else {
            return -1;
        };
        x
    }
    assert_eq!(get_inner(Some(Some(10))), 10);
    assert_eq!(get_inner(Some(None)), -1);
    assert_eq!(get_inner(None), -1);

    // —— 或模式 + let-else ——
    fn is_valid(status: Option<u32>) -> bool {
        let Some(200 | 201 | 204) = status else {
            return false;
        };
        true
    }
    assert!(is_valid(Some(200)));
    assert!(!is_valid(Some(404)));
    assert!(!is_valid(None));

    // —— 守卫在 let-else 中? ——
    // let-else 不支持守卫 (不支持 if), 需要守卫时用 match
    // 错误示例: let Some(x) if x > 0 = opt else { ... }

    // —— break 发散 ——
    let mut found = None;
    for i in 0..5 {
        let Some(val) = (if i == 3 { Some(i) } else { None }) else {
            continue;
        };
        found = Some(val);
        break;
    }
    assert_eq!(found, Some(3));
}

#[test]
/// 测试: matches! 宏 —— 单行布尔模式匹配 (Rust 1.0+)
fn test_matches_macro() {
    // matches! 返回 bool, 等价于 if let 的简写
    // 语法: matches!(expr, pattern) → bool

    // —— 基本用法 ——
    let opt = Some(42);
    assert!(matches!(opt, Some(_)));
    assert!(matches!(opt, Some(42)));
    assert!(!matches!(opt, None));

    // —— 范围模式 ——
    let num = 50;
    assert!(matches!(num, 1..=100));
    assert!(!matches!(num, 1..=49));

    // —— 或模式 ——
    let ch = 'c';
    assert!(matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'c'));
    assert!(!matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u'));

    // —— 解构 + matches! ——
    enum Message {
        Text(String),
        Number(i32),
        Empty,
    }
    let msg = Message::Text(String::from("你好"));
    assert!(matches!(msg, Message::Text(_)));
    assert!(!matches!(msg, Message::Number(_)));

    // —— 守卫 ——
    let val = Some(15);
    assert!(matches!(val, Some(x) if x > 10));
    assert!(!matches!(val, Some(x) if x > 20));

    // —— 在 filter/map 中使用 ——
    let nums = vec![Some(1), None, Some(2), Some(3), None];
    let count = nums
        .iter()
        .filter(|x| matches!(x, Some(v) if v % 2 == 1))
        .count();
    assert_eq!(count, 2); // Some(1) 和 Some(3)

    // —— 匹配 &str ——
    let s = "hello";
    assert!(matches!(s, "hello"));
    assert!(!matches!(s, "world"));

    // —— 切片模式 + matches! ——
    let arr = [1, 2, 3];
    assert!(matches!(arr, [1, ..]));
    assert!(matches!(arr, [_, _, 3]));
    assert!(matches!(arr, [_, _, _])); // 精确 3 元素匹配
    assert!(!matches!(arr, [4, _, _])); // 首元素不匹配

    // —— 结构体模式 ——
    struct Point2 {
        x: i32,
        y: i32,
    }
    let p = Point2 { x: 3, y: 4 };
    assert!(matches!(p, Point2 { x: 0..=5, .. }));
    assert!(!matches!(p, Point2 { x: 10..=20, .. }));
}

#[test]
/// 测试: 完整切片模式语法 —— [first, .., last] / [.., last] / [first, ..] / rest @ ..
fn test_slice_patterns() {
    // ===== 固定长度精确匹配 =====
    let arr = [0, 0, 0];
    match arr {
        [0, 0, 0] => {}
        _ => panic!(),
    }

    // ===== 解构: first + 剩余 =====
    let arr = [1, 2, 3, 4];
    let [first, rest @ ..] = arr;
    assert_eq!(first, 1);
    assert_eq!(rest, [2, 3, 4]);

    // ===== 解构: first + middle + last =====
    let arr = [1, 2, 3, 4, 5];
    let [first, middle @ .., last] = arr;
    assert_eq!(first, 1);
    assert_eq!(middle, [2, 3, 4]);
    assert_eq!(last, 5);

    // ===== 只匹配尾部: 前部任意 + 特定结尾 =====
    let arr = [10, 20, 30, 40];
    let [.., last] = arr;
    assert_eq!(last, 40);
    let [.., penultimate, _] = arr;
    assert_eq!(penultimate, 30);

    // ===== 只匹配首部: 特定开头 + 尾部任意 =====
    let [first, ..] = arr;
    assert_eq!(first, 10);

    // ===== 首尾都指定 + 中间任意 =====
    let [start, .., end] = arr;
    assert_eq!(start, 10);
    assert_eq!(end, 40);

    // ===== 空切片 =====
    let empty: [i32; 0] = [];
    match empty {
        [] => {}
    }

    // ===== 切片引用匹配 =====
    let slice: &[i32] = &[10, 20, 30];
    match slice {
        [first, ..] => assert_eq!(*first, 10),
        [] => panic!(),
    }

    // ===== 守卫 + 切片模式 =====
    let arr = [1, 2, 3];
    match arr {
        [x, y, z] if x + y == z => {} // 1 + 2 == 3
        _ => panic!(),
    }

    // ===== 嵌套切片解构 =====
    let nested = [1, 2, 3, 4, 5];
    let [a, b @ .., c, d] = nested;
    assert_eq!(a, 1);
    assert_eq!(b, [2, 3]);
    assert_eq!(c, 4);
    assert_eq!(d, 5);

    // ===== Vec 解构 (nightly: #![feature(slice_patterns)] =====
    // let v = vec![1, 2, 3];
    // let [a, ..] = v.as_slice();
}

#[test]
/// 测试: 范围模式的各种形式 —— 包含/独占/char/整数/守卫组合
fn test_range_patterns() {
    // ===== 包含范围 ..= (闭区间) =====
    let x = 5;
    match x {
        1..=5 => {} // 包含 1 和 5
        _ => panic!(),
    }

    // ===== 独占范围 .. (左闭右开) —— 仅用于模式(Pattern)时需 #![feature(exclusive_range_pattern)] =====
    // 稳定 Rust 中 .. 只能在切片模式中用于前缀/后缀, 不能用于值范围匹配

    // ===== 边界值测试 =====
    let x = 1;
    match x {
        1..=100 => {} // 1 在范围内
        _ => panic!(),
    }
    let x = 100;
    match x {
        1..=100 => {} // 100 在范围内
        _ => panic!(),
    }

    // ===== char 范围 =====
    let c = 'K';
    match c {
        'A'..='Z' => {} // 大写字母
        'a'..='z' => panic!(),
        _ => panic!(),
    }

    // ===== 数字字符范围 =====
    let d = '5';
    match d {
        '0'..='9' => {} // 数字字符
        _ => panic!(),
    }

    // ===== 范围 + 守卫 =====
    let num = 50;
    match num {
        n @ 1..=100 if n % 10 == 0 => assert_eq!(n, 50),
        _ => panic!(),
    }

    // ===== 或模式中的范围 =====
    let x = 15;
    match x {
        1..=10 | 20..=30 => panic!(),
        11..=19 => {}
        _ => panic!(),
    }

    // ===== 范围模式 + 解构 (结构体字段) =====
    struct Age {
        value: u32,
    }
    let age = Age { value: 25 };
    match age {
        Age { value: 0..=17 } => panic!(),
        Age { value: 18..=65 } => {}
        Age { value: _ } => {}
    }

    // ===== 浮点数不能用范围模式 (编译错误) =====
    // 因为浮点数不满足 Eq/Ord 的严格要求, 不支持范围匹配
    // let f = 3.14;
    // match f { 0.0..=1.0 => {} } // 编译错误!
}

#[test]
/// 测试: 嵌套结构体/枚举的深度模式匹配 —— 多层解构
fn test_deep_pattern_matching() {
    // —— 场景: 网络协议消息 (3层嵌套) ——
    enum NetworkMessage {
        Request { id: u32, payload: RequestPayload },
        Response { id: u32, status: u16, body: ResponseBody },
    }

    enum RequestPayload {
        Get { path: String },
        Post { path: String, body: String },
        Delete { path: String },
    }

    enum ResponseBody {
        Json(String),
        Binary(Vec<u8>),
        Empty,
    }

    // —— 深度匹配: 三层解构 ——
    let msg = NetworkMessage::Request {
        id: 1,
        payload: RequestPayload::Post {
            path: "/api/users".to_string(),
            body: r#"{"name":"Alice"}"#.to_string(),
        },
    };

    match msg {
        NetworkMessage::Request {
            id,
            payload: RequestPayload::Post { path, body },
        } => {
            assert_eq!(id, 1);
            assert_eq!(path, "/api/users");
            assert!(body.contains("Alice"));
        }
        NetworkMessage::Request {
            id,
            payload: RequestPayload::Get { path },
        } => {
            let _ = (id, path);
        }
        NetworkMessage::Request {
            payload: RequestPayload::Delete { .. },
            ..
        } => {}
        NetworkMessage::Response { id, status, .. } => {
            let _ = (id, status);
        }
    }

    // —— 嵌套 Option 深度解构 ——
    let deeply_nested = Some(Some(Some(42)));
    match deeply_nested {
        Some(Some(Some(x))) => assert_eq!(x, 42),
        _ => panic!(),
    }

    // —— 嵌套结构体 (结构体套结构体) ——
    struct Address {
        city: String,
        country: String,
    }
    struct Person {
        name: String,
        address: Address,
    }

    let alice = Person {
        name: "Alice".to_string(),
        address: Address {
            city: "Beijing".to_string(),
            country: "China".to_string(),
        },
    };

    match alice {
        Person {
            name,
            address: Address { city, country },
        } => {
            assert_eq!(name, "Alice");
            assert_eq!(city, "Beijing");
            assert_eq!(country, "China");
        }
    }

    // —— 部分深度解构 (只取深层字段) ——
    let resp = NetworkMessage::Response {
        id: 2,
        status: 200,
        body: ResponseBody::Json(r#"{"ok":true}"#.to_string()),
    };
    match resp {
        NetworkMessage::Response {
            status: 200,
            body: ResponseBody::Json(ref data),
            ..
        } => {
            assert!(data.contains("ok"));
        }
        _ => panic!(),
    }

    // —— 嵌套元组 + 枚举 ——
    let data: (Result<i32, &str>, Option<bool>) = (Ok(5), Some(true));
    match data {
        (Ok(n), Some(b)) if n > 0 && b => {}
        _ => panic!(),
    }
}

#[test]
/// 测试: @ 绑定的各种场景 —— 同时绑定值并匹配内部结构
fn test_at_bindings() {
    // —— 基本: @ 绑定数值范围 ——
    let num = 42;
    match num {
        x @ 40..=50 => assert_eq!(x, 42),
        _ => panic!(),
    }

    // —— 结构体 + @: 保留完整值同时解构字段 ——
    struct Point3 {
        x: i32,
        y: i32,
    }
    let p = Point3 { x: 3, y: 4 };
    match p {
        whole @ Point3 { x: 0..=5, y: 0..=5 } => {
            assert_eq!(whole.x, 3);
            assert_eq!(whole.y, 4);
        }
        _ => panic!(),
    }

    // —— 枚举 + @: 保留完整变体值 ——
    let opt = Some(10);
    match opt {
        s @ Some(x) if x > 5 => {
            assert_eq!(s, Some(10)); // s 是完整的 Some(10)
            assert_eq!(x, 10); // x 是解构出的 10
        }
        _ => panic!(),
    }

    // —— @ + 或模式 ——
    let num = 3;
    match num {
        n @ (1 | 3 | 5) => assert_eq!(n, 3),
        _ => panic!(),
    }

    // —— @ + 切片模式 ——
    let arr = [1, 2, 3, 4];
    let all @ [first, ref rest @ ..] = arr;
    assert_eq!(first, 1);
    assert_eq!(*rest, [2, 3, 4]);
    assert_eq!(all, [1, 2, 3, 4]);

    // —— @ + 嵌套结构体 ——
    struct Container {
        inner: Point3,
    }
    let c = Container {
        inner: Point3 { x: 7, y: 8 },
    };
    match c {
        Container {
            inner: p @ Point3 { x: 5..=10, .. },
        } => {
            assert_eq!(p.x, 7);
            assert_eq!(p.y, 8);
        }
        _ => panic!(),
    }

    // —— @ 绑定多个分支检查 ——
    let tuple_pair = (1, 2);
    match tuple_pair {
        whole @ (x, y) if x < y => {
            assert_eq!(whole, (1, 2));
            assert_eq!(x, 1);
            assert_eq!(y, 2);
        }
        _ => panic!(),
    }
}

// ======================== 以下为原有测试 (增强+修复警告) ========================

#[test]
/// 测试: 字面量/通配符/标识符模式
fn test_literal_and_wildcard_patterns() {
    // 字面量匹配
    let x = 5;
    match x {
        0 => panic!("zero"),
        5 => {}
        _ => panic!("other"),
    }

    // 通配符忽略不需要的值
    let pair = (1, 2);
    match pair {
        (_, 2) => {}
        _ => panic!(),
    }

    // 范围模式 (char)
    let c = 'M';
    match c {
        'A'..='Z' => {}
        _ => panic!(),
    }

    // 浮点数不能用范围模式 (编译错误)
    // let f = 3.14;
    // match f { 0.0..=1.0 => {} } // 编译错误!
}

#[test]
/// 测试: 解构元组/元组结构体
fn test_destructure_tuple() {
    // 基本解构
    let tuple = (1, "hello", true);
    let (num, text, flag) = tuple;
    assert_eq!(num, 1);
    assert_eq!(text, "hello");
    assert_eq!(flag, true);

    // 部分解构 (忽略某些字段)
    let tuple = (1, "hello", true);
    let (_, text, _) = tuple;
    assert_eq!(text, "hello");

    // 嵌套解构
    let nested = ((1, 2), (3, 4));
    let ((a, _b), (_c, d)) = nested;
    assert_eq!(a, 1);
    assert_eq!(d, 4);

    // 元组结构体解构
    struct Color(u8, u8, u8);
    let red = Color(255, 0, 0);
    let Color(r, g, b) = red;
    assert_eq!(r, 255);
    assert_eq!(g, 0);
    assert_eq!(b, 0);

    // match 中解构
    let point = (3, 4);
    match point {
        (0, y) => assert_eq!(y, 4),
        (x, 0) => assert_eq!(x, 3),
        (x, y) => {
            assert_eq!(x, 3);
            assert_eq!(y, 4);
        }
    }
}

#[test]
/// 测试: 解构结构体 (字段名/简写/..忽略)
fn test_destructure_struct() {
    struct User {
        name: String,
        age: u32,
        email: String,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };

    // 完整解构
    let User { name, age, email } = &user;
    assert_eq!(name, "Alice");
    assert_eq!(*age, 30);
    assert_eq!(email, "alice@example.com");

    // 部分解构 (.. 忽略剩余字段)
    let User { name, .. } = &user;
    assert_eq!(name, "Alice");

    // 字段重命名
    let User { name: n, age: a, .. } = &user;
    assert_eq!(n, "Alice");
    assert_eq!(*a, 30);

    // match 中解构
    match &user {
        User { age: 0..=17, .. } => panic!("minor"),
        User { name, age, .. } => {
            assert_eq!(name, "Alice");
            assert_eq!(*age, 30);
        }
    }
}

#[test]
/// 测试: 解构枚举 (单元/元组/结构体枚举)
fn test_destructure_enum() {
    enum Message {
        Quit,                       // 单元变体
        Move { x: i32, y: i32 },    // 结构体变体
        Write(String),              // 元组变体
        ChangeColor(i32, i32, i32), // 元组变体(多字段)
    }

    // 单元变体
    let msg = Message::Quit;
    match msg {
        Message::Quit => {}
        _ => panic!(),
    }

    // 结构体变体解构
    let msg = Message::Move { x: 10, y: 20 };
    match msg {
        Message::Move { x, y } => {
            assert_eq!(x, 10);
            assert_eq!(y, 20);
        }
        _ => panic!(),
    }

    // 元组变体解构
    let msg = Message::Write(String::from("hello"));
    match msg {
        Message::Write(text) => assert_eq!(text, "hello"),
        _ => panic!(),
    }

    // 多字段元组变体
    let msg = Message::ChangeColor(255, 0, 0);
    match msg {
        Message::ChangeColor(r, g, b) => {
            assert_eq!(r, 255);
            assert_eq!(g, 0);
            assert_eq!(b, 0);
        }
        _ => panic!(),
    }

    // 部分解构枚举
    let msg = Message::Move { x: 10, y: 20 };
    if let Message::Move { x, .. } = msg {
        assert_eq!(x, 10);
    }
}

#[test]
/// 测试: 或模式 (or patterns, Rust 1.53+)
fn test_or_patterns() {
    // 基本或模式
    let value = Some(42);
    if let Some(0 | 42) = value {
        assert!(true);
    }

    // 嵌套或模式
    let value = Some(Some(5));
    if let Some(Some(1 | 2 | 5)) = value {
        assert!(true);
    }

    // 或模式 + 绑定 (所有分支必须绑定同名同类型变量)
    let value = Some(10);
    if let Some(x @ (1 | 5 | 10)) = value {
        assert_eq!(x, 10);
    }

    // 枚举变体或模式
    #[derive(PartialEq)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }
    let status = Status::Active;
    match status {
        Status::Active | Status::Pending => {}
        Status::Inactive => panic!(),
    }
}

#[test]
/// 测试: match guard 条件守卫
fn test_match_guards() {
    let value = Some(5);
    match value {
        Some(x) if x > 3 => {}
        _ => panic!("Match guard failed"),
    }

    // 多个守卫
    let num = 15;
    match num {
        n if n < 10 => panic!("too small"),
        n if n > 20 => panic!("too large"),
        n => assert_eq!(n, 15),
    }

    // 守卫中使用外部变量
    let threshold = 10;
    let value = Some(15);
    match value {
        Some(x) if x > threshold => {}
        _ => panic!(),
    }

    // 守卫 + 或模式
    let value = Some(7);
    match value {
        Some(x) if x % 2 == 0 => panic!("even"),
        Some(x @ (1 | 3 | 5 | 7 | 9)) => assert_eq!(x, 7),
        _ => panic!(),
    }
}

#[test]
/// 测试: 匹配自动解引用 (match ergonomics, Rust 1.26+)
fn test_binding_modes() {
    // 旧版需要写 ref, 新版自动处理
    let s = String::from("hello");
    let r = &s;
    match r {
        s => assert_eq!(s, "hello"), // s 自动为 &String
    }

    // Option 的引用
    let option = &Some(String::from("hello"));
    if let Some(s) = option {
        assert_eq!(s, "hello"); // s 自动为 &String
    }

    // 可变引用
    let mut x = 42;
    let r = &mut x;
    match r {
        val => {
            *val = 100;
            assert_eq!(*val, 100);
        }
    }
}

#[test]
/// 测试: ref / ref mut 显式引用绑定
fn test_ref_patterns() {
    let name = String::from("Alice");

    // ref 绑定 (不移动 name)
    match name {
        ref n => {
            assert_eq!(n, "Alice"); // n 是 &String
        }
    }
    assert_eq!(name, "Alice"); // name 没被移动

    // ref mut 绑定
    let mut num = 42;
    match num {
        ref mut n => {
            *n = 100;
            assert_eq!(*n, 100);
        }
    }
    assert_eq!(num, 100);

    // 嵌套 ref
    let pair = (String::from("a"), String::from("b"));
    match pair {
        (ref first, ref second) => {
            assert_eq!(first, "a");
            assert_eq!(second, "b");
        }
    }
    // pair 仍可用
    assert_eq!(pair.0, "a");
}
