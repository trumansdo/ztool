// ---------------------------------------------------------------------------
// 2.1 模式匹配增强
// ---------------------------------------------------------------------------

#[test]
/// 测试: 字面量/通配符/标识符模式
fn test_literal_and_wildcard_patterns() {
    // 语法: Rust 模式分类:
    //   - 字面量模式: 0, 'a', true, "hello"  精确匹配
    //   - 通配符模式: _                    匹配任意值, 忽略绑定
    //   - 标识符模式: x, ref x, mut x       绑定到变量
    //   - 范围模式:   a..=b, a..b           匹配区间内的值
    //
    // 避坑:
    //   - 范围模式只能用于整数和 char, 不支持浮点数
    //   - ..= 包含两端, .. 不包含右端
    //   - 字面量模式要求类型一致, 不会隐式转换
    //   - _ 不绑定值, x 绑定值(即使不用也会警告)

    // 字面量匹配
    let x = 5;
    match x {
        0 => panic!("zero"),
        5 => assert!(true),
        _ => panic!("other"),
    }

    // 通配符忽略不需要的值
    let pair = (1, 2);
    match pair {
        (_, 2) => assert!(true),
        _ => panic!(),
    }

    // 范围模式
    let c = 'M';
    match c {
        'A'..='Z' => assert!(true),
        _ => panic!(),
    }

    // 浮点数不能用范围模式 (编译错误)
    // let f = 3.14;
    // match f { 0.0..=1.0 => {} } // 编译错误!
}

#[test]
/// 测试: 解构元组/元组结构体
fn test_destructure_tuple() {
    // 语法: 按位置解构元组, 支持嵌套解构和忽略字段
    // 避坑: 解构字段数必须与元组长度一致; 用 _ 忽略不需要的字段

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
    let ((a, b), (c, d)) = nested;
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
    // 语法: 按字段名解构, 支持简写语法和 .. 忽略剩余字段
    // 避坑: 字段名必须匹配; 非穷举解构用 ..; 模式中的字段顺序无关

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

    // 简写 + 重命名混合
    let User { name, age, .. } = &user;
    // name 绑定到 user.name, age 绑定到 user.age

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
    // 语法: 枚举有三种变体: 单元变体、元组变体、结构体变体, 各有不同解构方式
    // 避坑: match 必须穷举所有变体; 结构体变体用 { field } 语法; 元组变体用 () 语法

    enum Message {
        Quit,                       // 单元变体
        Move { x: i32, y: i32 },    // 结构体变体
        Write(String),              // 元组变体
        ChangeColor(i32, i32, i32), // 元组变体(多字段)
    }

    // 单元变体
    let msg = Message::Quit;
    match msg {
        Message::Quit => assert!(true),
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
    // 语法: 0 | 42 或模式, 匹配任意一个即成功 (Rust 1.53+)
    // 避坑: 或模式中各分支绑定的变量类型必须一致; 不能一部分绑定变量另一部分不绑定

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
        v @ (Status::Active | Status::Pending) => assert!(true),
        Status::Inactive => panic!(),
    }
}

#[test]
/// 测试: 切片模式匹配 (slice patterns / @ 绑定剩余部分)
fn test_slice_patterns() {
    // 语法: [first, rest @ ..] 解构数组/切片, @ 绑定剩余部分
    // 避坑: 模式必须覆盖所有长度, 否则会编译失败; 用 .. 或 rest@.. 匹配剩余元素

    // 固定长度解构
    let arr = [1, 2, 3, 4];
    if let [first, rest @ ..] = arr {
        assert_eq!(first, 1);
        assert_eq!(rest, [2, 3, 4]);
    }

    // 首尾解构
    let arr = [1, 2, 3, 4, 5];
    if let [first, middle @ .., last] = arr {
        assert_eq!(first, 1);
        assert_eq!(middle, [2, 3, 4]);
        assert_eq!(last, 5);
    }

    // 精确匹配
    let arr = [0, 0, 0];
    match arr {
        [0, 0, 0] => assert!(true),
        _ => panic!(),
    }

    // 任意长度 (.. 匹配零个或多个)
    let arr: [i32; 0] = [];
    match arr {
        [] => assert!(true),
        _ => panic!(),
    }

    // 切片上的模式匹配
    let slice: &[i32] = &[10, 20, 30];
    match slice {
        [first, ..] => assert_eq!(*first, 10),
        [] => panic!(),
    }

    // 守卫 + 切片模式
    let arr = [1, 2, 3];
    match arr {
        [x, y, z] if x + y == z => assert!(true),
        _ => panic!(),
    }
}

#[test]
/// 测试: match guard 条件守卫
fn test_match_guards() {
    // 语法: if 条件附加在模式后, 进一步过滤匹配
    // 避坑: guard 中的表达式不能移动值; guard 失败后继续尝试下一个分支

    let value = Some(5);
    match value {
        Some(x) if x > 3 => assert!(true),
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
        Some(x) if x > threshold => assert!(true),
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
    // 语法: 匹配自动适配引用, 不需要写 ref/ref mut (match ergonomics, Rust 1.26+)
    // 避坑: 对 &Option<String> 匹配时, 绑定变量自动解引用为 &String

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
    // 语法: ref 创建引用绑定而非移动; ref mut 创建可变引用绑定
    // 避坑: match ergonomics 已自动处理大部分情况, ref 主要用于精确控制

    let name = String::from("Alice");

    // ref 绑定 (不移动 name)
    match name {
        ref n => {
            assert_eq!(n, "Alice"); // n 是 &String
            // name 仍可用
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

#[test]
/// 测试: @ 绑定 (同时绑定值和匹配模式)
fn test_at_bindings() {
    // 语法: var @ pattern 同时绑定整个值到 var 并匹配 pattern
    // 避坑: @ 绑定的变量和模式中的变量可同时使用; 不能嵌套 @

    let num = 42;
    match num {
        x @ 40..=50 => {
            assert_eq!(x, 42); // x 绑定到 42
        }
        _ => panic!(),
    }

    // 嵌套结构体 + @
    struct Point {
        x: i32,
        y: i32,
    }
    let p = Point { x: 3, y: 4 };
    match p {
        p @ Point { x: 0..=5, y: 0..=5 } => {
            assert_eq!(p.x, 3); // 使用 @ 绑定的完整值
        }
        _ => panic!(),
    }

    // 枚举 + @
    let opt = Some(10);
    match opt {
        s @ Some(x) if x > 5 => {
            assert_eq!(s, Some(10)); // s 是 Some(10)
            assert_eq!(x, 10); // x 是 10
        }
        _ => panic!(),
    }
}

#[test]
/// 测试: 不可反驳模式 vs 可反驳模式
fn test_irrefutable_vs_refutable() {
    //   - 可反驳模式: 可能不匹配 (if let, while let, match arm 可用可反驳模式)
    //
    // 不可反驳: x, (x, y), Point { x, y }, _
    // 可反驳:   Some(x), Ok(x), [1, 2, 3], 0..=10
    //
    // 避坑:
    //   - let 语句不能用可反驳模式: let Some(x) = opt; 编译错误!
    //   - if let 只能用可反驳模式: if let x = 5; 无意义(必定匹配)
    //   - match 可以混合使用, 但必须穷举

    // 不可反驳模式 (let 语句)
    let (a, b) = (1, 2);
    assert_eq!(a, 1);

    let Point { x, y } = Point { x: 3, y: 4 };
    assert_eq!(x, 3);

    // 可反驳模式 (if let)
    let opt: Option<i32> = Some(42);
    if let Some(x) = opt {
        assert_eq!(x, 42);
    }

    // 可反驳模式 (while let)
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        assert!(top >= 1);
    }

    struct Point {
        x: i32,
        y: i32,
    }
}

#[test]
/// 测试: 嵌套模式 (深层解构)
fn test_nested_patterns() {
    // 语法: 模式可以任意嵌套, 解构复杂数据结构
    // 避坑: 嵌套层次深时注意可读性; 用 .. 忽略不需要的深层字段

    enum NetworkMessage {
        Request { id: u32, payload: RequestPayload },
        Response { id: u32, data: String },
    }

    enum RequestPayload {
        Get { path: String },
        Post { path: String, body: String },
    }

    let msg = NetworkMessage::Request {
        id: 1,
        payload: RequestPayload::Post {
            path: "/api".to_string(),
            body: "{}".to_string(),
        },
    };

    // 深层嵌套解构
    match msg {
        NetworkMessage::Request {
            id,
            payload: RequestPayload::Post { path, body },
        } => {
            assert_eq!(id, 1);
            assert_eq!(path, "/api");
            assert_eq!(body, "{}");
        }
        _ => panic!(),
    }

    // 用 .. 忽略不需要的部分
    let msg = NetworkMessage::Response {
        id: 2,
        data: "ok".to_string(),
    };
    match msg {
        NetworkMessage::Response { id, .. } => assert_eq!(id, 2),
        _ => panic!(),
    }
}
