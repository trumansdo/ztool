// ---------------------------------------------------------------------------
// 1.0 变量与数据类型
// ---------------------------------------------------------------------------

#[test]
/// 测试: 变量绑定与可变性 (let/mut/shadowing/命名规范)
fn test_variable_binding() {
    // 语法: let 绑定变量, 默认不可变; mut 声明可变; 同名变量可遮蔽(shadow)
    //
    // 变量绑定:
    //   - let x = 5;                    不可变绑定
    //   - let mut y = 10;              可变绑定
    //   - let x: i32 = 5;              显式类型注解
    //
    // Shadowing 特性:
    //   - 可重复声明同名变量, 遮蔽前一个绑定
    //   - 可以改变类型! mut 做不到这点
    //   - 原绑定被丢弃, 创建新绑定
    //
    // 命名规范:
    //   - _ 前缀消除未使用变量警告
    //   - __ 双下划线通常用于宏生成代码中, 表示"意图性忽略"
    //   - _ 单独使用时完全不绑定值(立即丢弃)
    //
    // 避坑:
    //   - 不可变变量不能重新赋值
    //   - shadowing 和 mut 是不同的: mut 修改同一内存, shadowing 创建新绑定
    //   - 未使用的变量会有警告, 用 _ 前缀消除: let _x = 5;
    //
    let x = 5;
    assert_eq!(x, 5);

    let mut y = 10;
    y += 5;
    assert_eq!(y, 15);

    // shadowing
    let z = "hello";
    let z = z.len();
    assert_eq!(z, 5);

    // _ 前缀消除未使用警告
    let _unused = 42;

    // _ 完全不绑定值
    let _ = 99;

    // __ 双下划线同样消除警告, 语义更强
    let __macro_generated = "from macro";
    assert_eq!(__macro_generated, "from macro");
}

#[test]
/// 测试: 整型 (各种类型/字面量写法/类型后缀/范围/溢出方法族)
fn test_integer_types() {
    // 语法: Rust 提供 i8/i16/i32/i64/i128/isize 和 u8/u16/u32/u64/u128/usize
    //
    // 字面量:
    //   - 98_222            下划线分隔(可读性)
    //   - 0xff              十六进制
    //   - 0o77              八进制
    //   - 0b1111_0000       二进制
    //   - b'A'              字节字面量(仅 u8)
    //
    // 类型后缀:
    //   - 5u8 / 5i16 / 0xffu32 / 100usize   字面量后直接跟类型名
    //
    // 整数范围:
    //   - i8: -128 ~ 127
    //   - u8: 0 ~ 255
    //   - i32: -2147483648 ~ 2147483647
    //
    // 溢出方法族:
    //   - wrapping_add      环绕(模运算)
    //   - checked_add       返回 Option, 溢出时 None
    //   - saturating_add    饱和运算, 钳制到边界
    //   - overflowing_add   返回 (结果, 是否溢出)
    //
    // 避坑:
    //   - 默认类型推断为 i32
    //   - debug 模式下溢出会 panic, release 模式下会环绕(wrapping)
    //   - usize/isize 取决于平台(32位/64位)
    //
    let decimal = 98_222;
    assert_eq!(decimal, 98222);

    let hex = 0xff;
    assert_eq!(hex, 255);

    let octal = 0o77;
    assert_eq!(octal, 63);

    let binary = 0b1111_0000;
    assert_eq!(binary, 240);

    let byte: u8 = b'A';
    assert_eq!(byte, 65);

    // 类型推导
    let default_int: i32 = 42;
    assert_eq!(default_int, 42);

    // 类型后缀
    let suffix_u8 = 200u8;
    assert_eq!(suffix_u8, 200u8);

    let suffix_i16 = -100i16;
    assert_eq!(suffix_i16, -100);

    let suffix_usize = 1024usize;
    assert_eq!(suffix_usize, 1024);

    let suffix_hex = 0xffu32;
    assert_eq!(suffix_hex, 255u32);

    // 整数范围
    assert_eq!(u8::MIN, 0);
    assert_eq!(u8::MAX, 255);
    assert_eq!(i8::MIN, -128);
    assert_eq!(i8::MAX, 127);
    assert_eq!(i32::MIN, -2147483648i32);
    assert_eq!(i32::MAX, 2147483647i32);

    // 溢出方法族 —— wrapping_add (环绕)
    let wrap_result = 200u8.wrapping_add(100);
    assert_eq!(wrap_result, 44u8); // (200 + 100) % 256 = 44

    // checked_add —— 溢出返回 None
    assert_eq!(200u8.checked_add(30), Some(230));
    assert_eq!(200u8.checked_add(100), None);

    // saturating_add —— 饱和到边界
    assert_eq!(200u8.saturating_add(30), 230);
    assert_eq!(200u8.saturating_add(100), 255); // 钳制到 u8::MAX

    // overflowing_add —— 返回 (结果, 是否溢出)
    let (ov_result, did_overflow) = 200u8.overflowing_add(100);
    assert_eq!(ov_result, 44);
    assert!(did_overflow);

    let (ov_result2, did_overflow2) = 200u8.overflowing_add(30);
    assert_eq!(ov_result2, 230);
    assert!(!did_overflow2);
}

#[test]
/// 测试: 浮点类型 (f32/f64/精度/特殊值/分类/epsilon比较)
fn test_float_types() {
    // 语法: f32(单精度), f64(双精度, 默认)
    //
    // 特殊值:
    //   - f64::NAN              非数值
    //   - f64::INFINITY         正无穷
    //   - f64::NEG_INFINITY     负无穷
    //   - f32::MIN_POSITIVE     最小正正常浮点数
    //
    // 状态判断:
    //   - is_finite()          既不是 NaN 也不是无穷
    //   - is_normal()          常规浮点数(非零、非subnormal、非无穷、非NaN)
    //   - is_subnormal()       次正规数(极小值, 精度降低)
    //   - classify()           返回 FpCategory 枚举
    //
    // 避坑:
    //   - 浮点没有实现 Eq, 使用 assert_eq! 需注意
    //   - NaN 不等于自身: f.is_nan() 判断
    //   - 浮点比较用 (a - b).abs() < epsilon
    //   - subnormal 数存在但运算极慢, 可能被硬件刷为零
    //
    let f1: f32 = 3.14;
    assert!((f1 - 3.14f32).abs() < 0.001);

    let f2: f64 = 3.14159265358979;
    assert!((f2 - std::f64::consts::PI).abs() < 0.001);

    // NaN
    let nan = f64::NAN;
    assert!(nan.is_nan());
    assert_ne!(nan, nan);

    // 无穷
    let inf = f64::INFINITY;
    assert!(inf.is_infinite());
    assert!(inf.is_sign_positive());

    let neg_inf = f64::NEG_INFINITY;
    assert!(neg_inf.is_infinite());
    assert!(neg_inf.is_sign_negative());

    // 带符号零
    assert!((-0.0f64).is_sign_negative());

    // is_finite —— NaN 和无穷都不是有限值
    assert!(1.0f64.is_finite());
    assert!(!f64::NAN.is_finite());
    assert!(!f64::INFINITY.is_finite());
    assert!(!f64::NEG_INFINITY.is_finite());

    // is_normal —— 常规浮点数
    assert!(1.0f64.is_normal());
    assert!(!0.0f64.is_normal());      // 零
    assert!(!f64::NAN.is_normal());    // NaN
    assert!(!f64::INFINITY.is_normal()); // 无穷

    // f32::MIN_POSITIVE —— 最小正正常浮点数
    assert!(f32::MIN_POSITIVE > 0.0);
    assert!((f32::MIN_POSITIVE - 1.1754944e-38f32).abs() < 1e-45);

    // is_subnormal —— 次正规数 (比 MIN_POSITIVE 更小的正数)
    // 使用 from_bits 构建一个极小的 subnormal 数
    let tiny_f64 = f64::from_bits(1); // 最小的正 subnormal 数
    assert!(tiny_f64 > 0.0);
    assert!(tiny_f64 < f64::MIN_POSITIVE);
    assert!(tiny_f64.is_subnormal());
    assert!(!tiny_f64.is_normal());

    // 浮点分类 —— classify()
    use std::num::FpCategory;
    assert_eq!(1.0f64.classify(), FpCategory::Normal);
    assert_eq!(0.0f64.classify(), FpCategory::Zero);
    assert_eq!(tiny_f64.classify(), FpCategory::Subnormal);
    assert_eq!(f64::INFINITY.classify(), FpCategory::Infinite);
    assert_eq!(f64::NAN.classify(), FpCategory::Nan);

    // epsilon 比较 —— 浮点不能用 ==
    let a: f64 = 0.1 + 0.2;
    let b: f64 = 0.3;
    assert_ne!(a, b); // 浮点精度导致不相等!
    let epsilon = 1e-10;
    assert!((a - b).abs() < epsilon);

    // f64::EPSILON —— 机器精度
    assert!((a - b).abs() < f64::EPSILON * 10.0);
}

#[test]
/// 测试: 布尔和字符类型 (Unicode转义/char方法)
fn test_bool_and_char() {
    // 语法: bool 类型有 true/false; char 是 4 字节 Unicode 标量值
    //
    // Unicode 转义:
    //   - '\u{03C0}'         π (码点 U+03C0)
    //   - '\u{4E2D}'         中 (码点 U+4E2D)
    //   - '\n' '\t' '\\' '\'' '\"' '\0' '\r'  常见转义序列
    //
    // char 方法:
    //   - is_alphabetic()    是否为字母
    //   - is_digit(10)       是否为十进制数字
    //   - is_uppercase()     是否为大写
    //   - to_uppercase()     转大写(返回迭代器!)
    //   - len_utf8()         UTF-8 编码字节数
    //
    // 避坑:
    //   - char 不是 1 字节, 是 4 字节!
    //   - bool 不能转为整数 (if x 而不是 if x != 0)
    //   - '' 是字符, "" 是字符串
    //   - to_uppercase() 返回迭代器, 因为 Unicode 可能一对多映射
    //
    let t: bool = true;
    let f: bool = false;
    assert!(t);
    assert!(!f);

    // 逻辑运算
    assert_eq!(t && f, false);
    assert_eq!(t || f, true);
    assert_eq!(!t, false);

    // char 基本测试
    let c: char = 'z';
    assert_eq!(c, 'z');

    let emoji = '😻';
    assert_eq!(emoji.len_utf8(), 4);

    let heart = '❤';
    assert_eq!(heart.len_utf8(), 3);

    // Unicode 转义字符
    let pi = '\u{03C0}';
    assert_eq!(pi, 'π');
    assert_eq!(pi.len_utf8(), 2);

    let chinese = '\u{4E2D}';
    assert_eq!(chinese, '中');
    assert_eq!(chinese.len_utf8(), 3);

    // 常见转义序列
    assert_eq!('\n' as u32, 10);       // 换行 LF
    assert_eq!('\t' as u32, 9);        // 制表符 TAB
    assert_eq!('\\' as u32, 92);       // 反斜杠
    assert_eq!('\'' as u32, 39);       // 单引号
    assert_eq!('\"' as u32, 34);       // 双引号
    assert_eq!('\0' as u32, 0);        // 空字符
    assert_eq!('\r' as u32, 13);       // 回车 CR

    // char 方法 —— is_alphabetic (字母)
    assert!('A'.is_alphabetic());
    assert!('中'.is_alphabetic());
    assert!('π'.is_alphabetic());
    assert!(!'1'.is_alphabetic());

    // char 方法 —— is_digit (数字)
    assert!('0'.is_digit(10));
    assert!('9'.is_digit(10));
    assert!(!'A'.is_digit(10));
    assert!(!'a'.is_digit(10));
    // is_digit(radix) 支持多进制
    assert!('A'.is_digit(16));  // A 在十六进制中是数字
    assert!('f'.is_digit(16));  // f 在十六进制中是数字

    // char 方法 —— is_uppercase / is_lowercase
    assert!('A'.is_uppercase());
    assert!(!'a'.is_uppercase());
    assert!('a'.is_lowercase());
    assert!(!'A'.is_lowercase());

    // char 方法 —— to_uppercase / to_lowercase (返回迭代器!)
    let upper_a: String = 'a'.to_uppercase().collect();
    assert_eq!(upper_a, "A");
    let lower_a: String = 'A'.to_lowercase().collect();
    assert_eq!(lower_a, "a");

    // Unicode 特殊映射: ß 转大写变成 SS (一对多)
    let eszett_upper: String = 'ß'.to_uppercase().collect();
    assert_eq!(eszett_upper, "SS");

    // char 方法 —— is_whitespace / is_alphanumeric
    assert!(' '.is_whitespace());
    assert!('\t'.is_whitespace());
    assert!('A'.is_alphanumeric());
    assert!('1'.is_alphanumeric());
    assert!(!'!'.is_alphanumeric());
}

#[test]
/// 测试: 元组类型 (创建/解构/索引/嵌套解构/单元类型作为返回值)
fn test_tuple_type() {
    // 语法: 元组是固定长度、可包含不同类型的集合
    //
    // 操作:
    //   - let tup: (i32, f64, u8) = (500, 6.4, 1);   创建
    //   - let (x, y, z) = tup;                        解构
    //   - tup.0                                        索引访问
    //   - let unit: () = ();                           单元类型
    //
    // 避坑:
    //   - 元组长度固定, 不能增删元素
    //   - 超过 12 个元素, Debug trait 不可用
    //   - 单元类型 () 是函数的默认返回值
    //   - 嵌套元组解构注意层级
    //
    let tup: (i32, f64, u8) = (500, 6.4, 1);

    // 解构
    let (x, y, z) = tup;
    assert_eq!(x, 500);
    assert!((y - 6.4).abs() < 0.001);
    assert_eq!(z, 1);

    // 索引访问
    assert_eq!(tup.0, 500);
    assert_eq!(tup.2, 1);

    // 单元类型
    let unit: () = ();
    assert_eq!(unit, ());

    // 单元类型作为函数返回值
    fn do_nothing() -> () {
        // 没有表达式, 默认返回 ()
    }
    assert_eq!(do_nothing(), ());

    // 显式返回单元类型
    fn returns_unit() {
        // 省略 -> () 也是合法的
    }
    assert_eq!(returns_unit(), ());

    // 嵌套元组
    let nested = (1, (2, 3), 4);
    assert_eq!((nested.1).0, 2);
    assert_eq!((nested.1).1, 3);

    // 更深层嵌套元组与解构
    let deep = ((1, 2), (3, 4), 5);
    assert_eq!((deep.0).0, 1);
    assert_eq!((deep.1).1, 4);

    // 嵌套解构
    let ((a, b), (c, d), e) = deep;
    assert_eq!(a, 1);
    assert_eq!(b, 2);
    assert_eq!(c, 3);
    assert_eq!(d, 4);
    assert_eq!(e, 5);

    // 部分忽略的嵌套解构
    let (first, _, last) = (1, 2, 3);
    assert_eq!(first, 1);
    assert_eq!(last, 3);
}

#[test]
/// 测试: 数组类型 (固定长度/索引/越界/多维数组/数组方法)
fn test_array_type() {
    // 语法: [T; N] 固定长度、同类型, 栈上分配
    //
    // 创建:
    //   - let a = [1, 2, 3];       自动推断 [i32; 3]
    //   - let a: [i32; 5] = [1, 2, 3, 4, 5];
    //   - let a = [0; 10];         初始化为 0 的数组
    //
    // 多维数组:
    //   - let m: [[i32; 2]; 3] = [[1, 2], [3, 4], [5, 6]];
    //
    // 数组方法:
    //   - .iter()          返回切片迭代器
    //   - .map(|x| ...)    对每个元素应用闭包, 返回新数组
    //   - .as_slice()      转为切片引用 &[T]
    //
    // 避坑:
    //   - 编译时大小固定, 不能 push/pop
    //   - 越界访问在运行时 panic (checked at runtime)
    //   - 数组在栈上分配, Vec 在堆上
    //
    let arr: [i32; 5] = [1, 2, 3, 4, 5];
    assert_eq!(arr[0], 1);
    assert_eq!(arr[4], 5);

    // 初始化语法
    let zeros = [0; 10];
    assert_eq!(zeros.len(), 10);
    assert_eq!(zeros[5], 0);

    // 越界检查 (编译通过, 运行 panic)
    // arr[10]; // panic: index out of bounds

    // 多维数组
    let matrix: [[i32; 2]; 3] = [[1, 2], [3, 4], [5, 6]];
    assert_eq!(matrix[0][0], 1);
    assert_eq!(matrix[0][1], 2);
    assert_eq!(matrix[1][0], 3);
    assert_eq!(matrix[2][1], 6);

    // 三维数组
    let cube: [[[i32; 2]; 2]; 2] = [
        [[1, 2], [3, 4]],
        [[5, 6], [7, 8]],
    ];
    assert_eq!(cube[0][0][0], 1);
    assert_eq!(cube[1][1][1], 8);

    // 数组方法 —— iter()
    let nums = [10, 20, 30, 40, 50];
    let sum: i32 = nums.iter().sum();
    assert_eq!(sum, 150);

    // 数组方法 —— iter() + 适配器
    let doubled: Vec<i32> = nums.iter().map(|x| x * 2).collect();
    assert_eq!(doubled, vec![20, 40, 60, 80, 100]);

    // 数组方法 —— map() (返回新数组)
    let squared = nums.map(|x| x * x);
    assert_eq!(squared, [100, 400, 900, 1600, 2500]);

    // 数组方法 —— as_slice()
    let slice: &[i32] = nums.as_slice();
    assert_eq!(slice.len(), 5);
    assert_eq!(slice[0], 10);

    // as_slice() 返回的切片可以用切片方法
    assert!(slice.contains(&30));
    assert!(!slice.contains(&99));
}

#[test]
/// 测试: const 常量与 static 变量
fn test_const_and_static() {
    // 语法:
    //   - const: 编译期常量, 内联到使用处
    //   - static: 固定内存地址, 生命周期 'static
    //
    // 避坑:
    //   - const 必须标注类型
    //   - static mut 需要 unsafe 块才能读写
    //   - const 不使用需加 #[allow(dead_code)]
    //
    const MAX_POINTS: u32 = 100_000;
    assert_eq!(MAX_POINTS, 100_000);

    static APP_NAME: &str = "MyApp";
    assert_eq!(APP_NAME, "MyApp");
}

#[test]
/// 测试: 类型别名与类型注解
fn test_type_annotations() {
    // 语法: type 关键字创建类型别名; 编译器可推断大多数类型
    //
    // 避坑:
    //   - 别名不创建新类型, 只是同义替换
    //   - parse() 等方法必须标注类型
    //   - 集合类型需要类型注解
    //
    type Kilometres = i32;
    let distance: Kilometres = 100;
    let _standard: i32 = distance; // 等价类型
    assert_eq!(distance, 100);

    // 类型注解
    let x: i32 = 5;
    let _y = 5u64; // 字面量后缀
    let _z: u32 = "42".parse().expect("not a number");
    assert_eq!(x, 5);

    // turbo-fish 语法指定类型
    let parsed = "99".parse::<i32>().unwrap();
    assert_eq!(parsed, 99);
}

#[test]
/// 测试: 表达式与语句 (块表达式/分号作用)
fn test_expressions_vs_statements() {
    // 语法: 表达式有返回值, 语句没有
    //
    // - 块表达式: { let x = 3; x + 1 } 返回 4
    // - 分号将表达式转为语句
    // - if/loop/match 都是表达式
    //
    // 避坑:
    //   - 忘记加分号导致类型不匹配
    //   - 在表达式中使用 let 需要加分号
    //   - 函数最后没有表达式时返回 ()
    //
    let y = {
        let x = 3;
        x + 1
    };
    assert_eq!(y, 4);

    // if 表达式
    let z = if true { 5 } else { 6 };
    assert_eq!(z, 5);

    // 块表达式
    let result = {
        let a = 2;
        let b = 3;
        a + b
    };
    assert_eq!(result, 5);

    // 分号将表达式转为语句 —— 返回 ()
    let result2 = {
        let a = 2;
        let b = 3;
        let _ = a + b; // 分号! 使表达式变成语句
    };
    assert_eq!(result2, ());

    // match 也是表达式
    let grade = match 85 {
        0..=59 => 'F',
        60..=79 => 'C',
        80..=100 => 'A',
        _ => '?',
    };
    assert_eq!(grade, 'A');
}

// ---------------------------------------------------------------------------
// 2.0 整数溢出深度测试
// ---------------------------------------------------------------------------

#[test]
/// 测试: 整数溢出的各种处理方法族 (checked/saturating/wrapping/overflowing)
fn test_integer_overflow() {
    // 语法: Rust 提供四种溢出处理策略, 均以方法族形式提供
    //
    // 四种方法族:
    //   - checked_*     -> Option<T>,     溢出返回 None
    //   - saturating_*  -> T,             溢出时钳制到边界值
    //   - wrapping_*    -> T,             溢出时环绕(模运算)
    //   - overflowing_* -> (T, bool),     返回结果 + 溢出标志
    //
    // 避坑:
    //   - Debug 模式默认溢出 panic, 不要依赖 release 的 wrapping 行为
    //   - 显式使用这些方法让意图清晰
    //   - Wrapping<T> 包装类型也提供环绕语义
    //
    let max_u8 = 255u8;

    // ========== checked_* 方法族 ==========
    // 正常操作返回 Some
    assert_eq!(100u8.checked_add(50), Some(150));
    assert_eq!(100u8.checked_sub(50), Some(50));
    assert_eq!(10u8.checked_mul(10), Some(100));

    // 溢出返回 None
    assert_eq!(max_u8.checked_add(1), None);
    assert_eq!(0u8.checked_sub(1), None);
    assert_eq!(16u8.checked_mul(16), None); // 256 > 255

    // ========== saturating_* 方法族 ==========
    // 正常操作返回正确值
    assert_eq!(100u8.saturating_add(50), 150);
    assert_eq!(100u8.saturating_sub(50), 50);
    assert_eq!(10u8.saturating_mul(10), 100);

    // 溢出时钳制到边界
    assert_eq!(max_u8.saturating_add(1), 255);   // 钳制到 u8::MAX
    assert_eq!(0u8.saturating_sub(1), 0);         // 钳制到 u8::MIN
    assert_eq!(16u8.saturating_mul(20), 255);     // 钳制到 u8::MAX

    // 有符号类型
    let max_i8 = 127i8;
    let min_i8 = -128i8;
    assert_eq!(max_i8.saturating_add(1), 127);
    assert_eq!(min_i8.saturating_sub(1), -128);

    // ========== wrapping_* 方法族 ==========
    // 正常操作
    assert_eq!(100u8.wrapping_add(50), 150);
    assert_eq!(100u8.wrapping_sub(50), 50);

    // 溢出时环绕
    assert_eq!(max_u8.wrapping_add(1), 0);      // 255 + 1 ≡ 0 (mod 256)
    assert_eq!(0u8.wrapping_sub(1), 255);        // 0 - 1 ≡ 255 (mod 256)

    // wrapping_mul
    assert_eq!(100u8.wrapping_mul(3), 44);       // 300 % 256 = 44

    // wrapping_neg
    assert_eq!(1u8.wrapping_neg(), 255);          // -1 ≡ 255 (mod 256)

    // ========== overflowing_* 方法族 ==========
    // 正常操作: bool 为 false
    {
        let (result, overflowed) = 100u8.overflowing_add(50);
        assert_eq!(result, 150);
        assert!(!overflowed);
    }

    // 溢出: bool 为 true
    {
        let (result, overflowed) = max_u8.overflowing_add(1);
        assert_eq!(result, 0); // 环绕结果
        assert!(overflowed);
    }

    // overflowing_sub
    {
        let (result, overflowed) = 0u8.overflowing_sub(1);
        assert_eq!(result, 255);
        assert!(overflowed);
    }

    // overflowing_mul
    {
        let (result, overflowed) = 16u8.overflowing_mul(16);
        assert_eq!(result, 0); // 256 % 256 = 0
        assert!(overflowed);
    }

    // ========== Wrapping<T> 包装类型 ==========
    use std::num::Wrapping;

    let w1 = Wrapping(200u8);
    let w2 = Wrapping(100u8);
    let w_sum = w1 + w2;
    assert_eq!(w_sum.0, 44); // (200 + 100) % 256

    let w_neg = -Wrapping(1u8);
    assert_eq!(w_neg.0, 255);
}

// ---------------------------------------------------------------------------
// 3.0 as 类型转换
// ---------------------------------------------------------------------------

#[test]
/// 测试: as 类型转换 (数值互转/截断/符号)
fn test_type_coercion() {
    // 语法: 使用 as 关键字进行显式原始类型转换
    //
    // 转换规则:
    //   - 小整型 -> 大整型:    零扩展(无符号)或符号扩展(有符号)
    //   - 大整型 -> 小整型:    截断低 N 位
    //   - 整数 -> 浮点:        可能损失精度(大整数)
    //   - 浮点 -> 整数:        向零舍入(截断), 超出范围是未定义行为
    //   - 有符号 <-> 无符号:   保留位模式, 重新解释
    //
    // 避坑:
    //   - as 不做溢出检查!
    //   - 大整数转小整型会被静默截断
    //   - 超出范围的浮点转整数是未定义行为(UB)
    //
    // 小整型 -> 大整型 (安全)
    let small: u8 = 42;
    let big: u32 = small as u32;
    assert_eq!(big, 42);

    let small_signed: i8 = -42;
    let big_signed: i32 = small_signed as i32; // 符号扩展
    assert_eq!(big_signed, -42);

    // 大整型 -> 小整型 (截断!)
    let big_val: u32 = 300;
    let truncated: u8 = big_val as u8;    // 300 % 256 = 44
    assert_eq!(truncated, 44);

    let big_s: i32 = 128;
    let truncated_s: i8 = big_s as i8;    // 128 as i8 = -128 (溢出)
    assert_eq!(truncated_s, -128);

    // 整数 -> 浮点
    let int_val: i32 = 1000;
    let float_val: f64 = int_val as f64;
    assert!((float_val - 1000.0).abs() < 0.001);

    // 浮点 -> 整数 (截断小数, 向零舍入)
    let pi: f64 = 3.14159;
    let truncated_pi: i32 = pi as i32;
    assert_eq!(truncated_pi, 3);

    let neg_pi: f64 = -3.14159;
    let neg_trunc: i32 = neg_pi as i32;
    assert_eq!(neg_trunc, -3); // 向零舍入

    // 有符号 <-> 无符号 (保留位模式)
    let neg_one: i8 = -1;           // 位模式: 1111_1111
    let as_unsigned: u8 = neg_one as u8;
    assert_eq!(as_unsigned, 255);   // 重新解释为 u8

    let pos129: u8 = 129;           // 位模式: 1000_0001
    let as_signed: i8 = pos129 as i8;
    assert_eq!(as_signed, -127);    // 重新解释为 i8 (129 - 256)

    // isize <-> usize (平台相关, 但相互转换总是安全的)
    let arch_signed: isize = -1;
    let arch_unsigned: usize = arch_signed as usize;
    assert_eq!(arch_unsigned, usize::MAX); // -1 在补码下用全1表示
}

// ---------------------------------------------------------------------------
// 4.0 never type !
// ---------------------------------------------------------------------------

#[test]
/// 测试: never type (!) 的概念与特性
fn test_never_type() {
    // 语法: ! 是 never type (无返回值类型), 表示永远不会返回的计算
    //
    // 产生 ! 的表达式 (发散表达式):
    //   - panic!("msg")    恐慌
    //   - loop { ... }     无限循环
    //   - std::process::exit()  退出进程
    //   - break / continue / return  控制流跳转
    //
    // 关键特性:
    //   - ! 可以强制转换为任何其他类型
    //   - 在 match 分支中, panic! 返回 ! 可以匹配任何类型的其他分支
    //   - ! 被视为任意类型的子类型
    //
    // 避坑:
    //   - 不能"构建"一个 ! 类型的值 (无实例)
    //   - break/continue 的类型也是 !, 但它们在 loop/while 中才合法
    //
    // panic! 在 match 分支中被强制为任意类型
    let result: i32 = match Some(42) {
        Some(x) => x,
        None => panic!("永远不会到这里"), // panic! 返回 !, 被强制转为 i32
    };
    assert_eq!(result, 42);

    // match 两个分支都可能是 !
    let another: i32 = match 5 {
        0..=9 => 100,
        _ => panic!("不可能!"), // ! 可以被强制为 i32
    };
    assert_eq!(another, 100);

    // break 表达式的类型是 !
    let mut found = false;
    let sum: i32 = loop {
        if found {
            break 42; // break 的类型是 !, loop 返回值是 i32
        }
        found = true;
    };
    assert_eq!(sum, 42);

    // continue 的类型也是 !
    let mut acc = 0;
    for i in 0..5 {
        if i == 3 {
            continue; // continue 的类型是 !
        }
        acc += i;
    }
    // 0 + 1 + 2 + 4 = 7 (跳过了 3)
    assert_eq!(acc, 7);

    // return 也是 !
    fn early_return(x: i32) -> i32 {
        if x == 0 {
            return 0; // return 的类型是 !
        }
        x
    }
    assert_eq!(early_return(0), 0);
    assert_eq!(early_return(7), 7);
}
