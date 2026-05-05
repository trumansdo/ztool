// ---------------------------------------------------------------------------
// 4.3 生命周期与 RPIT (Edition 2024) - 深度详解
// ---------------------------------------------------------------------------
//
// 概述: 本文件覆盖生命周期标注、消除规则、变型、子类型、HRTB、
//       'static 三种含义、RPIT 捕获规则等 Rust 核心生命周期知识。
//
// 学习路径: 生命周期标注 → 消除规则 → 结构体/方法中的生命周期 → 变型/子类型
//           → HRTB → 'static → 泛型约束 → RPIT 捕获

use std::fmt::Debug;
use std::fmt::Display;

// ===========================================================================
// 一、生命周期基础标注
// ===========================================================================

#[test]
/// 测试: 生命周期基础 — 手动标注、输入/输出生命周期关系
fn test_lifetime_basics() {
    // 语法: 'a 标记引用的有效范围, 编译器确保引用不悬垂
    // 避坑: 生命周期不是运行时概念, 只是编译期检查; 标注不改变引用的实际存活时间
    // 金句: 'a 是编译期的"标签"——标记引用之间如何关联, 不改变程序行为

    // 手动标注: 两个输入参数共享同一个 'a
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }

    // 独立生命周期: 返回值只依赖第一个参数
    fn select<'a, 'b>(first: &'a str, _second: &'b str) -> &'a str {
        first
    }

    // 输入与输出使用不同生命周期参数组合
    fn pair<'a>(x: &'a str, y: &'a str) -> (&'a str, &'a str) {
        (x, y)
    }

    let string1 = String::from("long string");
    let string2 = String::from("xyz");
    let result = longest(string1.as_str(), string2.as_str());
    assert_eq!(result, "long string");

    let r = select("hello", "world");
    assert_eq!(r, "hello");

    let (a, b) = pair("alpha", "beta");
    assert_eq!(a, "alpha");
    assert_eq!(b, "beta");
}

// ===========================================================================
// 二、生命周期消除规则详解
// ===========================================================================

#[test]
/// 测试: 3 条生命周期消除规则 — 规则1/2/3 的逐一验证
fn test_lifetime_elision_rules() {
    // 语法: 编译器通过三条消除规则自动推导生命周期, 不需要手动标注
    // 避坑: 当三条规则都走不通时编译器会报错, 此时才需要显式标注
    // 金句: 省略规则是编译器的"常识"——信任规则, 只在编译器要求时才出手

    // ── 规则1: 每个引用参数独立获得一个生命周期 ──
    // fn elision_rule1(x: &str, y: &str) 被编译器视为
    // fn elision_rule1<'a, 'b>(x: &'a str, y: &'b str)
    fn describe_item(item: &str, prefix: &str) -> String {
        format!("{}: {}", prefix, item)
    }
    assert_eq!(describe_item("Rust", "语言"), "语言: Rust");

    // ── 规则2: 只有一个输入生命周期时, 赋给所有输出 ──
    // fn elision_rule2(s: &str) -> &str 被编译器视为
    // fn elision_rule2<'a>(s: &'a str) -> &'a str
    fn first_word(s: &str) -> &str {
        s.split_whitespace().next().unwrap_or("")
    }
    assert_eq!(first_word("hello world"), "hello");

    // 规则2 也适用于返回多个引用
    fn split_once(s: &str) -> (&str, &str) {
        let mid = s.len() / 2;
        (&s[..mid], &s[mid..])
    }
    let (left, right) = split_once("abc123");
    assert_eq!(left, "abc");
    assert_eq!(right, "123");

    // ── 规则3: &self / &mut self 的生命周期赋给所有输出 ──
    struct Text(String);
    impl Text {
        fn first_word(&self) -> &str {
            self.0.split_whitespace().next().unwrap_or("")
        }

        fn announce_and_return(&self, announcement: &str) -> &str {
            // announcement 有独立生命周期, 但返回值使用 &self 的生命周期
            let _ = announcement;
            self.first_word()
        }
    }
    let t = Text(String::from("hello world"));
    assert_eq!(t.first_word(), "hello");
    assert_eq!(t.announce_and_return("attention"), "hello");

    // ── 需要手动标注: 多个输入, 规则2不适用 ──
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }
    assert_eq!(longest("short", "loooooong"), "loooooong");
}

#[test]
/// 测试: 单一输入生命周期的消除 — 规则1+2 的经典组合
fn test_lifetime_elision() {
    // 规则3: &self 方法自动推导
    struct Text(String);
    impl Text {
        fn first_word(&self) -> &str {
            self.0.split_whitespace().next().unwrap_or("")
        }
    }

    let t = Text(String::from("hello world"));
    assert_eq!(t.first_word(), "hello");
}

// ===========================================================================
// 三、结构体中的生命周期
// ===========================================================================

#[test]
/// 测试: 结构体中的生命周期标注 — 多生命周期参数与嵌套使用
fn test_lifetime_in_structs() {
    // 语法: struct Foo<'a> { field: &'a T } 结构体可以包含引用
    // 避坑: 结构体的生命周期参数必须标注; 所有方法 impl 都要重复声明
    // 金句: 持有引用的结构体是一个"租客"——声明它依赖房东的生命周期

    // 基本: 单一引用字段
    struct Excerpt<'a> {
        part: &'a str,
    }

    // 多生命周期参数: 不同字段有不同的生命周期来源
    struct SplitView<'a, 'b> {
        prefix: &'a str,
        suffix: &'b str,
    }

    impl<'a> Excerpt<'a> {
        fn level(&self) -> i32 { 3 }

        fn announce_and_return_part(&self, announcement: &str) -> &str {
            // announcement 有独立生命周期, 返回值用 &self 的生命周期
            println!("Attention: {}", announcement);
            self.part
        }
    }

    impl<'a, 'b> SplitView<'a, 'b> {
        fn combined(&self) -> String {
            format!("{}{}", self.prefix, self.suffix)
        }
    }

    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();
    let excerpt = Excerpt { part: first_sentence };
    assert_eq!(excerpt.part, "Call me Ishmael");
    assert_eq!(excerpt.level(), 3);

    let prefix = "Hello, ";
    let suffix = "world!";
    let view = SplitView { prefix, suffix };
    assert_eq!(view.combined(), "Hello, world!");
}

// ===========================================================================
// 四、方法 impl 块中的生命周期
// ===========================================================================

#[test]
/// 测试: impl 块中的生命周期 — &self 省略规则与方法独立生命周期
fn test_lifetime_in_methods() {
    // 语法: impl<'a> 声明生命周期范围; &self 的方法中返回值自动用 &self 的生命周期
    // 避坑: forget 写 impl<'a> 会导致 "use of undeclared lifetime name" 编译错误
    // 金句: impl<'a> 声明了"实现适用的生命周期范围", 与方法签名形成完整的约束链

    struct Container<'a> {
        data: &'a str,
    }

    impl<'a> Container<'a> {
        // 方法1: 返回值由规则3自动推导 (来自 &self)
        fn get(&self) -> &str {
            self.data
        }

        // 方法2: 返回独立引用的方法（不依赖 self）
        fn identity(x: &str) -> &str {
            x
        }

        // 方法3: 比较长度 (返回 bool, 无生命周期问题)
        fn longer_than(&self, other: &str) -> bool {
            self.data.len() > other.len()
        }

        // 方法4: 引入独立的生命周期参数 'b
        fn find_prefix<'b>(&self, other: &'b str) -> Option<&'b str> {
            if other.starts_with(self.data) { Some(other) } else { None }
        }
    }

    let s = String::from("hello world");
    let c = Container { data: &s };

    assert_eq!(c.get(), "hello world");
    assert_eq!(Container::identity("direct"), "direct");
    assert_eq!(c.longer_than("hi"), true);
    assert_eq!(c.longer_than("this is a very long test"), false);
    assert_eq!(c.find_prefix("hello world!!!"), Some("hello world!!!"));
    assert_eq!(c.find_prefix("goodbye"), None);
}

// ===========================================================================
// 五、'static 生命周期的三种含义
// ===========================================================================

#[test]
/// 测试: 'static 的三种含义 — 程序生命周期、字符串字面量、T: 'static 约束
fn test_static_lifetime_meanings() {
    // 语法: 'static 有三种不同但一致的用法:
    //   含义1: 引用存活于整个程序运行期
    //   含义2: 字符串字面量天然是 &'static str (编译时嵌入二进制)
    //   含义3: T: 'static 约束表示 T 不包含非静态引用 (T 是一个"干净的"自有类型)
    // 避坑: 'static 不是"永远存在的引用", 是"可以存活整个程序周期"
    // 金句: 'static 是"三位一体"概念——不同上下文有截然不同但逻辑一致的含义

    // ── 含义1: 整个程序生命周期 ──
    let s: &'static str = "I have a static lifetime.";
    assert_eq!(s, "I have a static lifetime.");

    // static 变量也是 'static
    fn get_static() -> &'static str {
        "always here"
    }
    assert_eq!(get_static(), "always here");

    // ── 含义2: 字符串字面量 ──
    fn accept_static(s: &'static str) -> &'static str {
        s
    }
    assert_eq!(accept_static("literal"), "literal");

    // String::leak() 产生 &'static str (可控内存泄漏)
    let owned: &'static str = String::from("leaked").leak();
    assert_eq!(owned, "leaked");

    // ── 含义3: T: 'static — T 不包含非 'static 引用 ──
    // 整数、String、Vec 等自有类型都满足 T: 'static
    fn print_static<T: Debug + 'static>(t: T) -> T {
        println!("{:?}", t);
        t
    }

    let num = print_static(42);
    assert_eq!(num, 42);

    let name = print_static(String::from("Rust"));
    assert_eq!(name, "Rust");

    // Vec 元素也是自有类型
    let v = print_static(vec![1, 2, 3]);
    assert_eq!(v, vec![1, 2, 3]);

    // 注意: &T 不满足 T: 'static (除非 T 本身是 'static)
    // let ref_val = print_static(&42); // ❌ 编译错误: &i32 不满足 'static
}

#[test]
/// 测试: 'static 基础 — 字符串字面量和 leak()
fn test_static_lifetime() {
    let s: &'static str = "I have a static lifetime.";
    assert_eq!(s, "I have a static lifetime.");

    let owned: &'static str = String::from("hello").leak();
    assert_eq!(owned, "hello");
}

// ===========================================================================
// 六、高阶生命周期 (HRTB)
// ===========================================================================

#[test]
/// 测试: HRTB 基础 — for<'a> 语法与 Fn trait 的关系
fn test_hrtb_basics() {
    // 语法: for<'a> 表示"对所有可能的生命周期都成立", 而非绑定到某一个具体生命周期
    // 避坑: HRTB 主要用于闭包/函数指针参数; Fn(&T) 实际上是 for<'a> Fn(&'a T) 的语法糖
    // 金句: for<'a> 是"无限量化"——闭包不绑定到特定生命周期, 每次调用的输入生命周期可以不同

    // ── 基本 HRTB: for<'a> 约束 ──
    fn call_on_ref<F>(f: F, val: i32) -> i32
    where
        F: for<'a> Fn(&'a i32) -> i32,
    {
        f(&val)
    }
    assert_eq!(call_on_ref(|x| *x * 2, 21), 42);

    // ── 等价写法: Fn(&i32) 是 HRTB 的语法糖 ──
    fn call_on_ref_sugar<F>(f: F, val: i32) -> i32
    where
        F: Fn(&i32) -> i32,
    {
        f(&val)
    }
    assert_eq!(call_on_ref_sugar(|x| *x + 1, 41), 42);

    // ── 字符串引用场景: 闭包操作不同生命周期的字符串 ──
    fn transform<F>(f: F) -> String
    where
        F: for<'a> Fn(&'a str) -> &'a str,
    {
        let s1 = String::from("hello");
        let s2 = String::from("world");
        let r = f(&s1);
        format!("{}-{}", r, f(&s2))
    }
    assert_eq!(transform(|s| s), "hello-world");
}

#[test]
/// 测试: HRTB 基础 — 函数指针与闭包
fn test_hrtb() {
    fn call_on_ref<F>(f: F, val: i32) -> i32
    where
        F: for<'a> Fn(&'a i32) -> i32,
    {
        f(&val)
    }

    assert_eq!(call_on_ref(|x| *x * 2, 21), 42);
}

// ===========================================================================
// 七、生命周期变型 (Variance)
// ===========================================================================

#[test]
/// 测试: 生命周期变型 — &T 协变 vs &mut T 不变
fn test_lifetime_variance() {
    // 语法: 变型描述"当泛型参数替换为子类型时, 整体类型如何变化"
    //   协变: 子类型可替代父类型 (&T 对 'a 和 T 都协变)
    //   不变: 必须完全相同 (&mut T 对 T 是不变的)
    // 避坑: &mut T 对 T 不变——这意味着可变引用不能被"偷换"类型
    // 金句: &T 让你"看世界"（协变）; &mut T 让你"改变世界"（不变）

    // ── 协变演示: 'static 引用可"缩短"为更短生命周期 ──
    fn take_str<'a>(_x: &'a str) {}

    let static_str: &'static str = "hello";
    // 这里创建一个更短的作用域, 'static 引用可安全传入
    {
        take_str(static_str); // ✅ 'static → 任意 'a (协变)
    }

    // ── 协变: Box<T> 对 T 协变 ──
    fn take_box(_b: Box<&str>) {}
    let boxed: Box<&'static str> = Box::new("hello");
    take_box(boxed); // ✅ Box<&'static str> → Box<&str> (协变)

    // ── 不变: &mut T 对 T 必须相同 ──
    // 以下代码在注释中展示不变性的含义:
    // 如果 &mut T 对 T 协变, 则可以把 &mut &'static str 当 &mut &'short str 用,
    // 从而写入短命引用, 导致悬垂引用——因此 &mut T 对 T 是不变的。

    // 能编译的 &mut 操作 (类型完全相同)
    let mut slot: &str = "";
    let r: &mut &str = &mut slot;
    *r = "new value"; // ✅ 类型完全匹配
    assert_eq!(slot, "new value");

    // 协变实用: 获取更短生命周期引用传给函数
    fn identity<T>(t: T) -> T { t }

    let short_lived = String::from("temp");
    let borrowed: &str = &short_lived;
    // identity 的 &str 可以接收任何生命周期的 &str (协变)
    assert_eq!(identity(borrowed), "temp");
}

// ===========================================================================
// 八、生命周期子类型: 'long 是 'short 的子类型
// ===========================================================================

#[test]
/// 测试: 长生命周期是短生命周期的子类型 — 'long: 'short
fn test_long_lifetime_is_subtype() {
    // 语法: 'long: 'short 读作 "'long 活得比 'short 久" 或 "'long 是 'short 的子类型"
    // 避坑: 在生命周期中, "活得久"的反而是"子类型"——因为长生命周期
    //       可以安全地用于任何期望短生命周期的地方
    // 金句: 'long 是 'short 的子类型——活得越久, 约束越强, 能替代的场景越多

    // ── 核心演示: 'static 引用传递给接受 'a 的函数 ──
    fn accept_short<'a>(x: &'a str) -> &'a str {
        x
    }
    let static_str: &'static str = "I live forever";
    // 'static 是任意 'a 的子类型, 可以安全传入
    let result = accept_short(static_str);
    assert_eq!(result, "I live forever");

    // ── 不同生命周期层级的引用传递 ──
    fn assign<'a>(target: &mut &'a str, source: &'a str) {
        *target = source;
    }

    let long_lived = String::from("long");
    {
        let short_lived = String::from("short");
        let mut slot: &str = "";
        // slot(短生命周期) 可以接收 long_lived 的引用 (长→短, 协变)
        {
            assign(&mut slot, &long_lived);
            assert_eq!(slot, "long");
        }
        // slot 的生命周期是短的, 但 long_lived 的数据足够长
        let _ = short_lived; // 保持 short_lived 存活到此处
    }

    // ── 结构体字段的多生命周期: 长短混合 ──
    struct Borrowed<'a, 'b> {
        short_ref: &'a str,
        long_ref: &'b str,
    }

    let long = String::from("long lived");
    let short = String::from("short");
    let b = Borrowed {
        short_ref: &short, // 'a = short 的生命周期
        long_ref: &long,   // 'b = long 的生命周期, 'b 长于 'a
    };
    assert_eq!(b.short_ref, "short");
    assert_eq!(b.long_ref, "long lived");
    // b.long_ref 可以用于任何需要 &'a str 的地方 (长是短的子类型)
    fn expects_short<'a>(_x: &'a str) {}
    expects_short(b.long_ref); // ✅ 'b: 'a, 长生命周期满足短生命周期要求
}

// ===========================================================================
// 九、多个生命周期参数
// ===========================================================================

#[test]
/// 测试: 多个生命周期参数 — 结构体/函数/方法中的 'a, 'b, 'c
fn test_multiple_lifetime_params() {
    // 语法: 当不同引用来自不同数据源时, 每个引用可以有独立的生命周期
    // 避坑: 不要把不同来源的引用都绑定到同一个 'a, 这会施加不必要的约束
    // 金句: 生命周期参数越细分, 类型表达能力越强——不要让不必要的约束限制你的 API

    // ── 多参数结构体 ──
    struct Pair<'a, 'b> {
        key: &'a str,
        value: &'b str,
    }

    impl<'a, 'b> Pair<'a, 'b> {
        fn key(&self) -> &str { self.key }
        fn value(&self) -> &str { self.value }
        fn to_string(&self) -> String {
            format!("{}={}", self.key, self.value)
        }
    }

    let key = String::from("name");
    let value = String::from("Rust");
    let pair = Pair { key: &key, value: &value };
    assert_eq!(pair.key(), "name");
    assert_eq!(pair.value(), "Rust");
    assert_eq!(pair.to_string(), "name=Rust");

    // ── 多参数函数: 三个独立的生命周期 ──
    fn combine<'a, 'b, 'c>(a: &'a str, b: &'b str, sep: &'c str) -> String {
        format!("{}{}{}", a, sep, b)
    }
    let a = String::from("left");
    let b = String::from("right");
    let sep = String::from(" | ");
    assert_eq!(combine(&a, &b, &sep), "left | right");

    // ── 方法中引入新的生命周期参数 ──
    struct Wrapper<'a> {
        inner: &'a str,
    }
    impl<'a> Wrapper<'a> {
        fn prefix_with<'b>(&self, prefix: &'b str) -> String {
            format!("{}{}", prefix, self.inner)
        }
    }
    let w = Wrapper { inner: "world" };
    assert_eq!(w.prefix_with("hello "), "hello world");
}

// ===========================================================================
// 十、泛型中的生命周期约束
// ===========================================================================

#[test]
/// 测试: 泛型中的生命周期约束 — T: 'a 的含义与自动推导
fn test_lifetime_bounds_in_generics() {
    // 语法: T: 'a 表示类型 T 中的所有引用至少存活 'a 久
    // 避坑: 编译器通常自动推导 T: 'a 约束; 显式写通常是多余的
    //   但当泛型由多个层级嵌套时, 显式约束能让错误信息更友好
    // 金句: T: 'a 是说"T 里面没有比 'a 更短命的借用"——不是 T 本身是 'a

    // ── 基本约束: RefHolder 存储对 T 的引用 ──
    // T: 'a 约束被编译器自动推导, 但显式写出可增加可读性
    struct RefHolder<'a, T> {
        value: &'a T,
    }

    impl<'a, T: Debug> RefHolder<'a, T> {
        fn display(&self) -> String {
            format!("{:?}", self.value)
        }
    }

    let num = 42_usize;
    let holder = RefHolder { value: &num };
    assert_eq!(holder.display(), "42");

    // ── T: 'static 约束: 类型不含任何短命引用 ──
    fn static_only<T: Debug + 'static>(t: T) -> T {
        println!("{:?}", t);
        t
    }
    assert_eq!(static_only(100), 100);
    assert_eq!(static_only("literal"), "literal");

    // ── 结构体隐含约束: 编译器自动添加 T: 'a ──
    struct Owned<T> {
        value: T, // Owned 拥有 T, 不包含引用, T: 'static 不是必需的
    }
    let _o = Owned { value: 42 };

    // ── 多约束组合: &'a T 隐含 T: 'a ──
    fn debug_ref<'a, T: Debug>(r: &'a T) -> String {
        format!("{:?}", r)
    }
    assert_eq!(debug_ref(&true), "true");

    // ── Vec 持有应用: T: 'a 自动推导 ──
    fn first_element<'a, T>(slice: &'a [T]) -> Option<&'a T> {
        slice.first()
    }
    let arr = [1, 2, 3];
    assert_eq!(first_element(&arr), Some(&1));
    assert_eq!(first_element::<i32>(&[]), None);
}

#[test]
/// 测试: 生命周期边界 (T: 'a) — 基本用法
fn test_lifetime_bounds() {
    fn print_ref<T: Display>(t: &T) {
        println!("{}", t);
    }
    print_ref(&42);
    print_ref(&String::from("hello"));
}

// ===========================================================================
// 十一、匿名生命周期 '_
// ===========================================================================

#[test]
/// 测试: 匿名生命周期 '_ — 让编译器推导
fn test_anonymous_lifetime() {
    // 语法: '_ 表示"我不关心具体生命周期, 让编译器推导"
    // 避坑: '_ 只能用于类型位置 (如 std::slice::Iter<'_, u8>);
    //       不能用于函数签名中的参数位置
    fn first_chars<'a>(strings: &[&'a str]) -> Vec<&'a str> {
        strings.iter().map(|s| &s[..1]).collect()
    }

    let strs = vec!["hello", "world", "rust"];
    let result = first_chars(&strs);
    assert_eq!(result, vec!["h", "w", "r"]);

    // 匿名生命周期在类型位置中的使用
    fn get_iter(items: &[u8]) -> std::slice::Iter<'_, u8> {
        // '_ 让编译器自动推导 Iter 内部的引用生命周期
        items.iter()
    }
    let data = vec![1, 2, 3];
    let mut iter = get_iter(&data);
    assert_eq!(iter.next(), Some(&1));
}

// ===========================================================================
// 十二、RPIT 捕获规则
// ===========================================================================

#[test]
/// 测试: RPIT 生命周期捕获 — Edition 2024 默认行为
fn test_rpit_capture_rules() {
    // 语法: Edition 2024 中 impl Trait 返回值默认捕获所有生命周期参数
    // 避坑: 旧版 (2021) 不自动捕获, 跨版本迁移需注意; use<> 可精确控制
    // 金句: RPIT 的"捕获"决定了返回的匿名类型携带哪些生命周期——捕获太多约束过紧, 捕获太少类型不完整

    // ── Edition 2024: 默认捕获所有生命周期 ──
    fn capture_single<'a>(x: &'a str) -> impl Debug {
        let _ = x; // 即使只在函数体内使用, 'a 仍被捕获
        42_usize
    }
    let s = String::from("test");
    assert_eq!(format!("{:?}", capture_single(&s)), "42");

    // ── 多个生命周期: 全部被捕获 ──
    fn capture_multi<'a, 'b>(a: &'a str, b: &'b str) -> impl Debug {
        // 'a 和 'b 都被捕获到返回类型中
        let _ = a;
        let _ = b;
        42_usize
    }
    let s1 = String::from("a");
    let s2 = String::from("b");
    assert_eq!(format!("{:?}", capture_multi(&s1, &s2)), "42");

    // ── use<> 精确捕获: 只捕获指定参数 ──
    fn capture_precise<'a, 'b>(a: &'a str, _b: &'b str) -> impl Debug + use<'a> {
        // 只捕获 'a, 不捕获 'b
        let _ = a;
        42_usize
    }
    let s3 = String::from("precise");
    let s4 = String::from("ignored");
    // s4 的生命周期 'b 不被返回类型约束
    assert_eq!(format!("{:?}", capture_precise(&s3, &s4)), "42");

    // ── 无生命周期参数的 RPIT ──
    fn no_lifetime() -> impl Debug {
        42_usize
    }
    assert_eq!(format!("{:?}", no_lifetime()), "42");
}

#[test]
/// 测试: Edition 2024 RPIT 生命周期捕获 — 旧版兼容
fn test_rpit_capture() {
    fn capture_lifetime<'a>(x: &'a str) -> impl Debug {
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

// ===========================================================================
// 十三、显式生命周期语法
// ===========================================================================

#[test]
/// 测试: 显式生命周期语法 — Iter<'_, T>
fn test_explicit_lifetime_syntax() {
    // 语法: 返回类型中显式写 '_ 表明借用关系, 提高可读性
    // 避坑: 省略 '_ 在 Edition 2024 会触发 lint 警告; 建议始终显式标注
    fn get_iter(items: &[u8]) -> std::slice::Iter<'_, u8> {
        items.iter()
    }
    let data = vec![1, 2, 3];
    let mut iter = get_iter(&data);
    assert_eq!(iter.next(), Some(&1));
}

// ===========================================================================
// 十四、常见生命周期错误模式 (编译演示)
// ===========================================================================

#[test]
/// 测试: 常见生命周期正确模式 — 对比错误模式的正确写法
fn test_common_lifetime_patterns() {
    // 金句: 编译器的生命周期报错不是噩梦——它是理解代码所有权关系的最精确指南

    // ── 模式1: 返回最长的引用 (正是编译器无法自动推导的场景) ──
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() { x } else { y }
    }
    assert_eq!(longest("ab", "abc"), "abc");

    // ── 模式2: 切片方法的生命周期 ──
    fn first_n<'a>(s: &'a str, n: usize) -> &'a str {
        &s[..n]
    }
    assert_eq!(first_n("hello", 2), "he");

    // ── 模式3: 返回自有值 (不需要生命周期) ──
    fn make_string() -> String {
        String::from("owned")
    }
    assert_eq!(make_string(), "owned");

    // ── 模式4: 借用链: 从 Vec 到 Iterator ──
    let data = vec![10, 20, 30];
    let iter = data.iter();
    let first: Option<&i32> = iter.clone().next();
    assert_eq!(first, Some(&10));

    // ── 模式5: 嵌套引用 ──
    fn find_in_slices<'a>(needle: &str, haystacks: &[&'a str]) -> Option<&'a str> {
        haystacks.iter().find(|&&s| s.contains(needle)).copied()
    }
    let words = vec!["hello world", "foo bar", "rust lang"];
    let found = find_in_slices("rust", &words);
    assert_eq!(found, Some("rust lang"));
}
