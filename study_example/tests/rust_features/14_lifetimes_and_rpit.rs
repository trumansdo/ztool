// ---------------------------------------------------------------------------
// 4.3 生命周期与 RPIT (Edition 2024)
// ---------------------------------------------------------------------------

#[test]
/// 测试: Edition 2024 RPIT 生命周期捕获
fn test_rpit_capture() {
    // 语法: Edition 2024 中 impl Trait 返回值默认捕获所有生命周期; 可用 use<> 精确控制
    // 避坑: 旧版 Edition 不捕获生命周期, 跨版本迁移需注意; use<> 中省略的参数不捕获
    fn capture_lifetime<'a>(x: &'a str) -> impl std::fmt::Debug {
        let _ = x;
        42usize
    }

    fn capture_lifetime_all<'a>(x: &'a str) -> &'a str {
        x
    }

    let s = String::from("test");
    assert_eq!(format!("{:?}", capture_lifetime(&s)), "42");
    assert_eq!(capture_lifetime_all(&s), "test");
}

#[test]
/// 测试: 显式生命周期语法 (Iter<'_, T>)
fn test_explicit_lifetime_syntax() {
    // 语法: 返回类型中显式写 '_ 表明借用关系, 提高可读性 (mismatched_lifetime_syntaxes lint)
    // 避坑: 省略 '_ 在 2024 Edition 会触发 lint 警告; 建议始终显式标注
    fn get_iter(items: &[u8]) -> std::slice::Iter<'_, u8> {
        items.iter()
    }
    let data = vec![1, 2, 3];
    let mut iter = get_iter(&data);
    assert_eq!(iter.next(), Some(&1));
}

#[test]
/// 测试: 生命周期基础 (借用检查规则)
fn test_lifetime_basics() {
    // 语法: 生命周期 'a 标记引用的有效范围, 编译器确保引用不悬垂
    // 避坑: 生命周期不是运行时概念, 只是编译期检查; 不需要手动标注, 编译器能推导大部分
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() {
            x
        } else {
            y
        }
    }

    let string1 = String::from("long string");
    let string2 = String::from("xyz");
    // Both strings alive, result gets the shorter lifetime
    let result = longest(string1.as_str(), string2.as_str());
    assert_eq!(result, "long string");
}

#[test]
/// 测试: 生命周期消除规则 (Lifetime Elision)
fn test_lifetime_elision() {
    // 语法: 编译器有三条消除规则, 自动推导大部分生命周期
    //   1. 每个引用参数获得独立生命周期
    //   2. 只有一个输入生命周期时, 赋给所有输出
    //   3. 有 &self 时, 输出生命周期 = self 的生命周期
    // 避坑: 不满足消除规则时必须显式标注

    // 规则3: &self 方法自动推导
    struct Text(String);
    impl Text {
        fn first_word(&self) -> &str {
            self.0
                .split_whitespace()
                .next()
                .unwrap_or("")
        }
    }

    let t = Text(String::from("hello world"));
    assert_eq!(t.first_word(), "hello");
}

#[test]
/// 测试: 结构体中的生命周期
fn test_lifetime_in_structs() {
    // 语法: struct Foo<'a> { field: &'a T } 结构体可以包含引用
    // 避坑: 结构体的生命周期参数必须标注; 所有方法 impl 都要重复声明
    struct ImportantExcerpt<'a> {
        part: &'a str,
    }

    impl<'a> ImportantExcerpt<'a> {
        fn level(&self) -> i32 {
            3
        }

        fn announce_and_return_part(&self, announcement: &str) -> &str {
            println!("Attention: {}", announcement);
            self.part
        }
    }

    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();
    let excerpt = ImportantExcerpt { part: first_sentence };
    assert_eq!(excerpt.part, "Call me Ishmael");
    assert_eq!(excerpt.level(), 3);
}

#[test]
/// 测试: 'static 生命周期
fn test_static_lifetime() {
    // 语法: 'static 表示数据存活于整个程序运行期
    // 避坑: 字符串字面量是 &'static str; 不要滥用 'static, 它限制了借用灵活性
    let s: &'static str = "I have a static lifetime.";
    assert_eq!(s, "I have a static lifetime.");

    // String 转为 'static
    let owned: &'static str = String::from("hello").leak();
    assert_eq!(owned, "hello");
}

#[test]
/// 测试: 生命周期边界 (T: 'a)
fn test_lifetime_bounds() {
    // 语法: T: 'a 表示类型 T 中的所有引用至少存活 'a
    // 避坑: 通常不需要显式写 T: 'a, 编译器自动推断; 用于泛型结构体包含引用时
    use std::fmt::Display;

    fn print_ref<T: Display>(t: &T) {
        println!("{}", t);
    }

    print_ref(&42);
    print_ref(&String::from("hello"));
}

#[test]
/// 测试: HRTB (Higher-Ranked Trait Bounds)
fn test_hrtb() {
    // 语法: for<'a> Fn(&'a T) -> R 表示对所有生命周期 'a 都满足
    // 避坑: HRTB 用于闭包/函数指针参数; 编译器通常能自动推断, 不需要手动写
    fn call_on_ref<F>(f: F, val: i32) -> i32
    where
        F: for<'a> Fn(&'a i32) -> i32,
    {
        f(&val)
    }

    assert_eq!(call_on_ref(|x| *x * 2, 21), 42);
}

#[test]
/// 测试: 匿名生命周期 '_
fn test_anonymous_lifetime() {
    // 语法: '_ 表示"我不关心具体生命周期, 让编译器推导"
    // 避坑: '_ 只能用于类型位置; 不能用于函数签名中的参数
    fn first_chars<'a>(strings: &[&'a str]) -> Vec<&'a str> {
        strings
            .iter()
            .map(|s| &s[..1])
            .collect()
    }

    let strs = vec!["hello", "world", "rust"];
    let result = first_chars(&strs);
    assert_eq!(result, vec!["h", "w", "r"]);
}
