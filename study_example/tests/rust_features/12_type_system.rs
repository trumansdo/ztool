// ---------------------------------------------------------------------------
// 3.1 类型系统 - impl Trait / dyn Trait / 孤儿规则 / 超trait / 完全限定语法等
// ---------------------------------------------------------------------------

#[test]
/// 测试: impl Trait 返回隐藏具体类型 (静态分发)
fn test_impl_trait() {
    // 语法: impl Trait 作为返回值, 隐藏具体类型(静态分发, 零开销)
    // 避坑: 函数所有返回路径必须返回同一具体类型; 调用方无法知道具体类型
    fn returns_impl_trait() -> impl std::fmt::Display {
        42
    }
    assert_eq!(format!("{}", returns_impl_trait()), "42");
}

#[test]
/// 测试: impl Trait 作为函数参数 (隐式泛型)
fn test_impl_trait_as_param() {
    // 语法: impl Trait 作为参数是隐式泛型参数的语法糖
    // 避坑: 每个 impl Trait 参数是独立的泛型参数, 不能假设两个 impl Trait 是同一类型
    fn print_it(x: impl std::fmt::Display) {
        let _ = format!("{}", x);
    }
    print_it(42);
    print_it("hello");

    // 两个 impl Trait 是不同类型
    fn add_display(a: impl std::fmt::Display, b: impl std::fmt::Display) {
        // a 和 b 可以是不同类型
        let _ = format!("{} {}", a, b);
    }
    add_display(42, "hello");
}

#[test]
/// 测试: impl Trait 多 trait 约束 (impl Trait1 + Trait2)
fn test_impl_trait_multiple() {
    // 语法: impl Trait1 + Trait2 要求类型同时实现多个 trait
    // 避坑: 只能有一个主 trait, 其余必须是 auto trait (Send, Sync, Unpin 等)
    //       或使用 where 子句约束
    fn process<T: std::fmt::Display + std::fmt::Debug>(x: T) -> String {
        format!("{}, {:?}", x, x)
    }
    let result = process(42i32);
    assert!(result.contains("42"));
}

#[test]
/// 测试: impl Trait 在闭包中的使用
fn test_impl_trait_in_closure() {
    // 语法: impl Trait 可用于闭包参数和返回值 (Rust 1.78+)
    // 避坑: 闭包的 impl Trait 参数也是隐式泛型
    fn apply(f: impl Fn(i32) -> i32, x: i32) -> i32 {
        f(x)
    }
    assert_eq!(apply(|x| x * 2, 5), 10);
}

#[test]
/// 测试: dyn Trait 动态分发 (trait 对象/虚表)
fn test_dyn_trait() {
    // 语法: dyn Trait 是 trait 对象(动态分发), 通过虚表调用方法
    // 避坑: trait 必须对象安全(不能有泛型方法/返回 Self); &dyn Trait 是胖指针(数据指针+虚表指针)
    trait Foo {
        fn bar(&self) -> i32;
    }

    struct Baz;
    impl Foo for Baz {
        fn bar(&self) -> i32 {
            42
        }
    }

    let obj: &dyn Foo = &Baz;
    assert_eq!(obj.bar(), 42);
}

#[test]
/// 测试: dyn Trait 作为函数参数和返回值
fn test_dyn_trait_as_param_and_return() {
    // 语法: dyn Trait 可作为参数(接受任何实现该 trait 的类型)和返回值
    // 避坑: dyn Trait 返回值必须是 Box<dyn Trait> 或 &dyn Trait, 不能直接返回
    trait Shape {
        fn area(&self) -> f64;
    }

    struct Circle {
        radius: f64,
    }
    impl Shape for Circle {
        fn area(&self) -> f64 {
            std::f64::consts::PI * self.radius * self.radius
        }
    }

    struct Rectangle {
        width: f64,
        height: f64,
    }
    impl Shape for Rectangle {
        fn area(&self) -> f64 {
            self.width * self.height
        }
    }

    fn print_area(s: &dyn Shape) -> f64 {
        s.area()
    }

    let c = Circle { radius: 1.0 };
    let r = Rectangle {
        width: 2.0,
        height: 3.0,
    };
    assert!((print_area(&c) - std::f64::consts::PI).abs() < 0.001);
    assert_eq!(print_area(&r), 6.0);

    // 返回 Box<dyn Trait>
    fn create_shape(is_circle: bool) -> Box<dyn Shape> {
        if is_circle {
            Box::new(Circle { radius: 1.0 })
        } else {
            Box::new(Rectangle {
                width: 1.0,
                height: 1.0,
            })
        }
    }

    let s = create_shape(true);
    assert!((s.area() - std::f64::consts::PI).abs() < 0.001);
}

#[test]
/// 测试: 对象安全性 (Object Safety)
fn test_object_safety() {
    // 语法: trait 要作为 dyn Trait 使用, 必须对象安全:
    //   - 不能有泛型方法
    //   - 不能有返回 Self 的方法 (Self: Sized 除外)
    //   - 不能有 Self 作为参数的方法
    //   - 不能要求 Self: Sized
    //
    // 避坑:
    //   - 可以用 where Self: Sized 标记不对象安全的方法, 其余方法仍可用于 dyn Trait
    //   - 泛型方法可以用, 但不能通过 dyn Trait 调用

    trait MyTrait {
        fn regular_method(&self) -> i32;

        // 标记为 Self: Sized 的方法不能通过 dyn Trait 调用
        fn generic_method<T>(&self, _x: T) -> i32
        where
            Self: Sized,
        {
            0
        }
    }

    struct MyType;
    impl MyTrait for MyType {
        fn regular_method(&self) -> i32 {
            42
        }
    }

    // 可以通过 dyn Trait 调用 regular_method
    let obj: &dyn MyTrait = &MyType;
    assert_eq!(obj.regular_method(), 42);
    assert_eq!(obj.regular_method(), 42);

    // generic_method 有 Self: Sized 约束，必须用具体类型调用
    let concrete = MyType;
    assert_eq!(concrete.generic_method(42), 0);
    // 或者显式指定类型参数
    assert_eq!(concrete.generic_method::<i32>(42), 0);
}

#[test]
/// 测试: trait upcasting 子trait向上转型为超trait (1.86+)
fn test_trait_upcasting() {
    // 语法: 子 trait 对象可自动向上转型为超 trait 对象 (1.86+)
    // 避坑: 必须有明确的超 trait 关系(Sub: Super); 向上转型后只能调用超 trait 的方法
    trait Super {
        fn super_method(&self) -> i32;
    }

    trait Sub: Super {
        fn sub_method(&self) -> i32;
    }

    struct MyStruct;
    impl Super for MyStruct {
        fn super_method(&self) -> i32 {
            10
        }
    }
    impl Sub for MyStruct {
        fn sub_method(&self) -> i32 {
            20
        }
    }

    fn upcast(x: &dyn Sub) -> &dyn Super {
        x
    }

    let obj = MyStruct;
    let sub: &dyn Sub = &obj;
    assert_eq!(sub.sub_method(), 20);

    // Rust 中 trait upcasting 是隐式的，不需要强制转换语法
    // &dyn Sub 可以直接赋值给 &dyn Super
    let super_ref: &dyn Super = sub;
    assert_eq!(super_ref.super_method(), 10);

    // 也可以通过函数参数隐式转换
    assert_eq!(upcast(sub).super_method(), 10);

    // 显式转换 (Rust 1.86+)
    let explicit_super = sub as &dyn Super;
    assert_eq!(explicit_super.super_method(), 10);
}

#[test]
/// 测试: Any upcasting 和 downcast_ref 类型向下转型
fn test_any_upcast() {
    // 语法: 自定义 trait 继承 Any 后, 可通过 upcast 实现 downcast_ref
    // 避坑: 必须在 trait 对象上实现 downcast_ref, 利用 as &dyn Any 向上转型
    use std::any::Any;

    trait MyAny: Any {}
    impl dyn MyAny {
        fn downcast_ref<T: Any>(&self) -> Option<&T> {
            (self as &dyn Any).downcast_ref()
        }
    }
    impl MyAny for i32 {}

    let val: i32 = 42;
    let my_any: &dyn MyAny = &val;
    assert_eq!(my_any.downcast_ref::<i32>(), Some(&42));
}

#[test]
/// 测试: dyn Any 直接向下转型
fn test_dyn_any_downcast() {
    // 语法: std::any::Any 提供类型运行时反射, 可 downcast/downcast_ref
    // 避坑: Any 只适用于 'static 生命周期(不含借用引用); downcast 消费值, downcast_ref 不消费
    use std::any::Any;

    let value: String = String::from("hello");
    let any: &dyn Any = &value;

    // downcast_ref (不消费)
    assert_eq!(any.downcast_ref::<String>(), Some(&String::from("hello")));
    assert_eq!(any.downcast_ref::<i32>(), None);

    // downcast (消费, 需要 owned 值)
    let value2: Box<dyn Any> = Box::new(42i32);
    let unboxed = value2
        .downcast::<i32>()
        .unwrap();
    assert_eq!(*unboxed, 42);
}

#[test]
/// 测试: 返回位置 impl Trait 隐藏具体类型
fn test_impl_trait_return() {
    // 语法: impl Trait 作为返回值隐藏具体类型, 调用方不知道实现细节
    // 避坑: 函数所有返回路径必须返回同一具体类型; 不同于 type alias impl trait (unstable)
    fn create_iter() -> impl Iterator<Item = i32> {
        (0..5).into_iter()
    }

    let result: Vec<i32> = create_iter().collect();
    assert_eq!(result, vec![0, 1, 2, 3, 4]);
}

#[test]
/// 测试: 返回位置协变 (Return Position Impl Trait in Trait)
fn test_rpitit() {
    // 语法: trait 定义中可以使用 impl Trait 作为返回类型 (1.75+)
    // 避坑: 实现必须返回的具体类型在编译期确定; 不同实现可返回不同类型
    trait Generator {
        fn generate(&self) -> impl Iterator<Item = i32>;
    }

    struct RangeGen;
    impl Generator for RangeGen {
        fn generate(&self) -> impl Iterator<Item = i32> {
            0..3
        }
    }

    struct VecGen;
    impl Generator for VecGen {
        fn generate(&self) -> impl Iterator<Item = i32> {
            vec![10, 20, 30].into_iter()
        }
    }

    let range: Vec<i32> = RangeGen.generate().collect();
    assert_eq!(range, vec![0, 1, 2]);

    let vec_result: Vec<i32> = VecGen.generate().collect();
    assert_eq!(vec_result, vec![10, 20, 30]);
}

#[test]
/// 测试: Sized trait 和 ?Sized
fn test_sized_trait() {
    // 语法: Sized 是自动实现的标记 trait, 表示类型大小编译期已知
    //   - 所有泛型参数默认隐含 T: Sized
    //   - ?Sized 放宽此限制, 允许动态大小类型 (dyn Trait, str, [T])
    //
    // 避坑:
    //   - fn foo<T>(x: T) 等价于 fn foo<T: Sized>(x: T)
    //   - fn foo<T: ?Sized>(x: &T) 允许传入 &str, &dyn Trait 等
    //   - ?Sized 只能用于 trait 约束, 不能用于具体类型

    // 默认 T: Sized, 不能传入 str
    fn takes_sized<T: std::fmt::Display>(x: T) -> String {
        format!("{}", x)
    }
    assert_eq!(takes_sized(42), "42");

    // T: ?Sized, 可以传入 &str
    fn takes_maybe_unsized<T: std::fmt::Display + ?Sized>(x: &T) -> String {
        format!("{}", x)
    }
    assert_eq!(takes_maybe_unsized("hello"), "hello");
    assert_eq!(takes_maybe_unsized(&42), "42");

    // 验证 Sized 类型的 size_of 都大于 0
    assert!(std::mem::size_of::<i32>() > 0);
    assert!(std::mem::size_of::<String>() > 0);

    // dyn Trait 是 DST，编译期大小未知
    // &dyn Display 是胖指针 (Sized), 不是 DST 本身
    assert!(std::mem::size_of::<&dyn std::fmt::Display>() > 0);

    // 标记 trait 自身不占空间
    fn takes_ref_trait(x: &dyn std::fmt::Display) -> usize {
        std::mem::size_of_val(x) // 返回 x 指向的实际值的大小 (动态)
    }
    // 注意: size_of_val 对 DST 返回实际大小
    let val: &dyn std::fmt::Display = &42i32;
    assert_eq!(std::mem::size_of_val(val), std::mem::size_of::<i32>());
}

#[test]
/// 测试: 零大小类型 (ZST - Zero Sized Types)
fn test_zero_sized_types() {
    // 语法: ZST 是不占内存的类型, sizeof 为 0
    //   - () 单元类型
    //   - 空结构体 struct Empty;
    //   - 标记类型 (phantom types)
    //   - 函数指针类型 fn()
    //
    // 避坑:
    //   - ZST 不分配内存, 但 Vec<ZST> 的 len 仍然有效
    //   - 不能用 &ZST 做指针运算
    //   - 编译器会优化掉 ZST 的所有运行时操作

    struct Marker;
    assert_eq!(std::mem::size_of::<Marker>(), 0);
    assert_eq!(std::mem::size_of::<()>(), 0);
    assert_eq!(std::mem::size_of::<std::convert::Infallible>(), 0);

    // Vec<ZST> 仍然可以追踪长度
    let v = vec![Marker, Marker, Marker];
    assert_eq!(v.len(), 3);

    // Option<()> 通常被 niche 优化为 0 大小，但取决于编译器版本
    // 某些 Rust 版本下 Option<ZST> 为 1 字节, 不影响语义

    // Infallible 是空枚举 (编译器保证 size_of = 0)
    assert_eq!(std::mem::size_of::<std::convert::Infallible>(), 0);

    // PhantomData 也是 ZST
    use std::marker::PhantomData;
    assert_eq!(std::mem::size_of::<PhantomData<i32>>(), 0);
    assert_eq!(std::mem::size_of::<PhantomData<String>>(), 0);
}

// ===========================================================================
// 新增测试 —— 孤儿规则 / 超trait / 完全限定语法 / 虚表 / impl Trait 对比 /
//         关联类型&泛型 / derive 全部 / Sized深入 / 标记trait / ZST&PhantomData
// ===========================================================================

#[test]
/// 测试: 孤儿规则与 newtype 模式绕行
fn test_orphan_rule() {
    // 语法: 孤儿规则 —— 不能为外部类型实现外部 trait
    //   ✅ 可以为本地类型实现外部 trait
    //   ✅ 可以为外部类型实现本地 trait
    //   ❌ 不能为外部类型实现外部 trait
    //
    // 绕行: newtype模式 —— 用元组结构体包装外部类型, 再为新类型实现外部 trait

    // ===== 场景1: 本地 trait + 外部类型 (✅ 允许) =====
    trait LocalTrait {
        fn describe(&self) -> String;
    }
    // Vec<i32> 是外部类型, LocalTrait 是本地 trait, 可以实现
    impl LocalTrait for Vec<i32> {
        fn describe(&self) -> String {
            format!("Vec 包含 {} 个元素", self.len())
        }
    }
    let v = vec![1, 2, 3];
    assert!(v.describe().contains("3"));

    // ===== 场景2: 外部 trait + 本地类型 (✅ 允许) =====
    // Display 是外部 trait, LocalType 是本地类型, 可以实现
    struct LocalType(i32);
    impl std::fmt::Display for LocalType {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "LocalType({})", self.0)
        }
    }
    assert_eq!(format!("{}", LocalType(10)), "LocalType(10)");

    // ===== 场景3: newtype 绕行 (❌ → ✅) =====
    // 目标: 为 Vec<i32> 实现 Display (原本不可能)
    // 解决: 用 newtype 包装
    struct MyVec(Vec<i32>);

    impl std::fmt::Display for MyVec {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            let items: Vec<String> = self.0.iter().map(|x| x.to_string()).collect();
            write!(f, "[{}]", items.join(", "))
        }
    }
    let mv = MyVec(vec![1, 2, 3]);
    assert_eq!(format!("{}", mv), "[1, 2, 3]");

    // newtype 也可以实现 Deref 以复用内部类型的方法
    use std::ops::Deref;
    impl Deref for MyVec {
        type Target = Vec<i32>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    // 现在可以用 Vec 的方法
    assert_eq!(mv.len(), 3);
    assert_eq!(mv[0], 1);
}

#[test]
/// 测试: 超 trait (Supertrait) 的用法
fn test_supertrait() {
    // 语法: trait Sub: Super 表示实现 Sub 之前必须先实现 Super
    // 避坑: 子 trait 可以在默认方法中调用超 trait 的方法
    //       超 trait 关系支持向上转型 (upcasting, Rust 1.86+)

    // ===== 基本超 trait =====
    trait Named {
        fn name(&self) -> &str;
    }

    // Greeter 要求实现者同时实现 Named
    trait Greeter: Named {
        fn greet(&self) -> String {
            // 可以在 Greeter 的默认方法中使用 Named 的方法
            format!("你好, 我叫{}", self.name())
        }
    }

    struct Person {
        name: String,
    }

    // 先实现 Named
    impl Named for Person {
        fn name(&self) -> &str {
            &self.name
        }
    }

    // 再实现 Greeter (Named 已实现)
    impl Greeter for Person {}

    let p = Person {
        name: String::from("张三"),
    };
    assert_eq!(p.greet(), "你好, 我叫张三");
    assert_eq!(p.name(), "张三");

    // ===== 多超 trait =====
    trait A {
        fn a(&self) -> i32;
    }
    trait B {
        fn b(&self) -> i32;
    }
    trait C: A + B {
        fn sum(&self) -> i32 {
            self.a() + self.b()
        }
    }

    struct Data {
        a: i32,
        b: i32,
    }
    impl A for Data {
        fn a(&self) -> i32 {
            self.a
        }
    }
    impl B for Data {
        fn b(&self) -> i32 {
            self.b
        }
    }
    impl C for Data {}

    let d = Data { a: 10, b: 20 };
    assert_eq!(d.sum(), 30);

    // ===== 超 trait 与 upcasting =====
    trait Super {
        fn super_val(&self) -> i32;
    }
    trait SubTrait: Super {
        fn sub_val(&self) -> i32;
    }

    struct Value(i32);
    impl Super for Value {
        fn super_val(&self) -> i32 {
            self.0
        }
    }
    impl SubTrait for Value {
        fn sub_val(&self) -> i32 {
            self.0 * 2
        }
    }

    let sub: &dyn SubTrait = &Value(5);
    // 隐式 upcast (1.86+)
    let sup: &dyn Super = sub;
    assert_eq!(sup.super_val(), 5);
}

#[test]
/// 测试: 完全限定语法 (Fully Qualified Syntax)
fn test_fully_qualified_syntax() {
    // 语法: 当多个 trait 有同名方法时, 用完全限定语法消除歧义
    //   三种调用方式:
    //   1. instance.method()  —— 调用自身方法 (优先级最高)
    //   2. Trait::method(&instance)  —— 调用指定 trait 的方法
    //   3. <Type as Trait>::method(&instance)  —— 完全限定语法

    trait Pilot {
        fn fly(&self) -> &str;
    }

    trait Wizard {
        fn fly(&self) -> &str;
    }

    struct Human;

    impl Pilot for Human {
        fn fly(&self) -> &str {
            "飞行员: 起飞!"
        }
    }

    impl Wizard for Human {
        fn fly(&self) -> &str {
            "法师: 飞行咒!"
        }
    }

    impl Human {
        fn fly(&self) -> &str {
            "人类: 挥手"
        }
    }

    let person = Human;

    // 方式1: 默认调用自身方法 (优先级最高)
    assert_eq!(person.fly(), "人类: 挥手");

    // 方式2: Trait::method 语法
    assert_eq!(Pilot::fly(&person), "飞行员: 起飞!");
    assert_eq!(Wizard::fly(&person), "法师: 飞行咒!");

    // 方式3: 完全限定语法 (最显式)
    assert_eq!(<Human as Pilot>::fly(&person), "飞行员: 起飞!");
    assert_eq!(<Human as Wizard>::fly(&person), "法师: 飞行咒!");

    // ===== 无 self 的关联函数也需要完全限定 =====
    trait Animal {
        fn baby_name() -> String;
    }

    struct Dog;

    impl Dog {
        fn baby_name() -> String {
            String::from("点点")
        }
    }

    impl Animal for Dog {
        fn baby_name() -> String {
            String::from("小狗")
        }
    }

    // 自身关联函数
    assert_eq!(Dog::baby_name(), "点点");

    // 完全限定语法调用 trait 的关联函数
    assert_eq!(<Dog as Animal>::baby_name(), "小狗");

    // ===== 泛型场景中的完全限定 =====
    trait Identity {
        fn id(&self) -> i32;
    }

    struct Wrapper<T>(T);

    impl Identity for Wrapper<i32> {
        fn id(&self) -> i32 {
            self.0
        }
    }

    let w = Wrapper(42i32);
    assert_eq!(<Wrapper<i32> as Identity>::id(&w), 42);
}

#[test]
/// 测试: trait 对象的虚表 (vtable) 行为
fn test_trait_object_vtable() {
    // 语法: &dyn Trait 是胖指针 (数据指针 + 虚表指针), 共 16 字节
    //   - 相同具体类型通过同一 trait 生成的对象, 共享同一个虚表
    //   - 不同具体类型各自有独立的虚表
    //
    // 避坑: 虚表查找有微小运行时开销; 相同类型的多个 trait 对象仍共享虚表

    trait Animal {
        fn speak(&self) -> &str;
        fn legs(&self) -> u32;
    }

    struct Dog;
    impl Animal for Dog {
        fn speak(&self) -> &str {
            "汪汪"
        }
        fn legs(&self) -> u32 {
            4
        }
    }

    struct Cat;
    impl Animal for Cat {
        fn speak(&self) -> &str {
            "喵喵"
        }
        fn legs(&self) -> u32 {
            4
        }
    }

    // ===== 胖指针大小验证 =====
    // &dyn Trait 包含 data ptr (8) + vtable ptr (8) = 16
    assert_eq!(
        std::mem::size_of::<&dyn Animal>(),
        2 * std::mem::size_of::<usize>()
    );
    // &Dog 只是普通引用 (8 字节)
    assert_eq!(
        std::mem::size_of::<&Dog>(),
        std::mem::size_of::<usize>()
    );

    // ===== 虚表共享验证 =====
    let dog = Dog;
    let cat = Cat;

    let d1: &dyn Animal = &dog;
    let d2: &dyn Animal = &dog; // 同一Dog实例的另一个胖指针
    let c1: &dyn Animal = &cat;

    // 胖指针的结构是 (data_ptr, vtable_ptr)
    // 用 transmute_copy 提取 vtable 指针进行比较
    type FatPtrRepr = (*const u8, *const u8);

    let d1_repr: FatPtrRepr = unsafe { std::mem::transmute_copy(&d1) };
    let d2_repr: FatPtrRepr = unsafe { std::mem::transmute_copy(&d2) };
    let c1_repr: FatPtrRepr = unsafe { std::mem::transmute_copy(&c1) };

    // 同类型 (Dog) 的不同胖指针共享同一个虚表
    assert_eq!(d1_repr.1, d2_repr.1);

    // 不同类型 (Dog vs Cat) 有不同虚表
    assert_ne!(d1_repr.1, c1_repr.1);

    // ===== 方法调用验证 =====
    assert_eq!(d1.speak(), "汪汪");
    assert_eq!(d2.speak(), "汪汪");
    assert_eq!(c1.speak(), "喵喵");
    assert_eq!(d1.legs(), 4);
    assert_eq!(c1.legs(), 4);

    // ===== 多个 trait 的 vtable =====
    trait Runnable {
        fn run(&self) -> &str;
    }
    impl Runnable for Dog {
        fn run(&self) -> &str {
            "奔跑"
        }
    }

    // 同一个 Dog 通过不同 trait 有不同的虚表
    let a: &dyn Animal = &dog;
    let r: &dyn Runnable = &dog;
    let a_repr: FatPtrRepr = unsafe { std::mem::transmute_copy(&a) };
    let r_repr: FatPtrRepr = unsafe { std::mem::transmute_copy(&r) };
    // 不同 trait 的虚表不同 (虚表内容不同 —— 方法集不同)
    assert_ne!(a_repr.1, r_repr.1);
}

#[test]
/// 测试: impl Trait 在参数位置和返回位置的对比
fn test_impl_trait_in_both_positions() {
    // ===== 参数位置: impl Trait 是泛型的语法糖 =====
    // 以下两种写法等价:
    fn with_impl(x: impl std::fmt::Display) -> String {
        format!("{}", x)
    }

    fn with_generic<T: std::fmt::Display>(x: T) -> String {
        format!("{}", x)
    }

    assert_eq!(with_impl(42), "42");
    assert_eq!(with_generic(42), "42");
    assert_eq!(with_impl("hello"), "hello");

    // 注意: 每个 impl Trait 参数是独立的泛型参数
    fn two_different(a: impl std::fmt::Display, b: impl std::fmt::Display) -> String {
        format!("{} {}", a, b)
    }
    assert_eq!(two_different(42, "hello"), "42 hello");

    // ===== 返回位置: impl Trait 隐藏具体类型 (opaque type) =====
    // 调用方不知道返回的具体类型, 只能按 trait 使用
    fn make_range() -> impl Iterator<Item = i32> {
        0..3
    }

    fn make_vec_iter() -> impl Iterator<Item = i32> {
        vec![1, 2, 3].into_iter()
    }

    // 返回类型不同, 但都满足 Iterator<Item = i32>
    let r: Vec<i32> = make_range().collect();
    assert_eq!(r, vec![0, 1, 2]);

    let v: Vec<i32> = make_vec_iter().collect();
    assert_eq!(v, vec![1, 2, 3]);

    // ===== RPIT 的类型一致性约束 =====
    // ✅ 所有路径返回同一具体类型
    fn always_range(flag: bool) -> impl Iterator<Item = i32> {
        if flag {
            (0..3).into_iter() // Range<i32>
        } else {
            (0..3).into_iter() // 相同类型 Range<i32>
        }
    }

    let result: Vec<i32> = always_range(true).collect();
    assert_eq!(result, vec![0, 1, 2]);

    // ===== 多 trait 约束的 impl Trait 返回 =====
    fn returns_debug_and_display() -> impl std::fmt::Display + std::fmt::Debug {
        42i32
    }

    let val = returns_debug_and_display();
    assert_eq!(format!("{}", val), "42");
    assert_eq!(format!("{:?}", val), "42");
}

#[test]
/// 测试: 关联类型与泛型参数的对比和选择原则
fn test_associated_type_vs_generic() {
    // 选择原则:
    //   - 关联类型: "一个类型 → 一个关联类型" (如 Iterator::Item)
    //   - 泛型参数: "一个类型 → 多种类型参数" (如 From<T>)
    //
    // 避坑:
    //   - 如果需要对同一类型多次实现 trait (不同泛型参数), 用泛型
    //   - 如果每个类型只有一种自然的关联类型, 用关联类型

    // ===== 关联类型示例: 一个类型只有一个 Item =====
    trait Container {
        type Item;
        fn get(&self) -> Option<&Self::Item>;
    }

    struct IntBox(i32);
    impl Container for IntBox {
        type Item = i32;
        fn get(&self) -> Option<&Self::Item> {
            Some(&self.0)
        }
    }

    let ib = IntBox(42);
    assert_eq!(ib.get(), Some(&42));

    // ===== 泛型参数示例: 一个类型可以 From 多种类型 =====
    struct MyNumber(i32);

    // 可以为多种来源类型实现 From
    impl From<i32> for MyNumber {
        fn from(v: i32) -> Self {
            MyNumber(v)
        }
    }
    impl From<&str> for MyNumber {
        fn from(v: &str) -> Self {
            MyNumber(v.parse().unwrap_or(0))
        }
    }

    let n1: MyNumber = 42i32.into();
    let n2: MyNumber = "99".into();
    assert_eq!(n1.0, 42);
    assert_eq!(n2.0, 99);

    // ===== 关联类型 + 泛型的组合使用 =====
    trait Converter<T> {
        type Output;
        fn convert(&self, input: T) -> Self::Output;
    }

    struct Stringifier;
    // 一个 Converter 可以为不同 T 实现不同 Output
    impl Converter<i32> for Stringifier {
        type Output = String;
        fn convert(&self, input: i32) -> Self::Output {
            input.to_string()
        }
    }
    impl Converter<f64> for Stringifier {
        type Output = String;
        fn convert(&self, input: f64) -> Self::Output {
            format!("{:.2}", input)
        }
    }

    let s = Stringifier;
    assert_eq!(s.convert(42), "42");
    assert_eq!(s.convert(3.14159), "3.14");

    // ===== 使用 Iterator trait (关联类型的经典用例) =====
    struct Counter {
        count: i32,
        max: i32,
    }

    impl Iterator for Counter {
        type Item = i32;
        fn next(&mut self) -> Option<Self::Item> {
            if self.count < self.max {
                self.count += 1;
                Some(self.count)
            } else {
                None
            }
        }
    }

    let c = Counter { count: 0, max: 3 };
    let result: Vec<i32> = c.collect();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
/// 测试: 全部 derive trait 的完整测试
fn test_derive_traits_comprehensive() {
    // 语法: #[derive(...)] 自动为类型实现标准 trait
    // 可用 derive 的 trait:
    //   Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default
    //
    // 避坑:
    //   - Copy 是 Clone 的超 trait, derive Copy 需要同时 derive Clone
    //   - Eq 是 PartialEq 的超 trait
    //   - Ord 是 PartialOrd + Eq 的超 trait
    //   - 包含 f32/f64 的类型不能 derive Eq/Ord (NaN 不是全序)

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    // ===== Debug =====
    assert_eq!(format!("{:?}", Point { x: 1, y: 2 }), "Point { x: 1, y: 2 }");

    // ===== Clone =====
    let p = Point { x: 10, y: 20 };
    let cloned = p.clone();
    assert_eq!(p, cloned);
    // 确认是深拷贝 (对于 Copy 类型, Clone 就是位拷贝)
    assert_eq!(cloned.x, 10);
    assert_eq!(cloned.y, 20);

    // ===== Copy: 赋值不移动原值 =====
    let p1 = Point { x: 5, y: 6 };
    let p2 = p1; // Copy, 不是 move
    assert_eq!(p1, p2); // p1 仍然可用
    assert_eq!(p2.x, 5);

    // ===== PartialEq: == 比较 =====
    let a = Point { x: 1, y: 2 };
    let b = Point { x: 1, y: 2 };
    let c = Point { x: 3, y: 4 };
    assert!(a == b);
    assert!(a != c);

    // ===== Eq: 自反性 (a == a 永远为真, 无 NaN 问题) =====
    assert_eq!(a, a);

    // ===== PartialOrd: < > <= >= 比较 =====
    let small = Point { x: 1, y: 1 };
    let large = Point { x: 10, y: 10 };
    assert!(small < large);
    assert!(large > small);

    // ===== Ord: 全序 (对每对值都可比) =====
    let mut points = vec![
        Point { x: 3, y: 1 },
        Point { x: 1, y: 3 },
        Point { x: 2, y: 2 },
    ];
    points.sort();
    assert_eq!(points[0], Point { x: 1, y: 3 });
    assert_eq!(points[1], Point { x: 2, y: 2 });
    assert_eq!(points[2], Point { x: 3, y: 1 });

    // ===== Hash =====
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Point { x: 1, y: 2 });
    set.insert(Point { x: 1, y: 2 }); // 重复, 不会插入
    assert_eq!(set.len(), 1);
    assert!(set.contains(&Point { x: 1, y: 2 }));

    // ===== Default =====
    // 对于 i32, Default 值是 0
    let default_p = Point::default();
    assert_eq!(default_p, Point { x: 0, y: 0 });

    // ===== Copy + Clone 的关系验证 =====
    // Copy 类型必须同时实现 Clone
    fn assert_copy_clone<T: Copy + Clone>(_: T) {}
    assert_copy_clone(Point { x: 1, y: 2 });

    // ===== f32 不能 derive Eq/Ord (NaN 导致不满足全序) =====
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
    struct FloatPoint {
        x: f32,
        y: f32,
    }
    // FloatPoint 能正常使用 PartialEq 和 PartialOrd
    let fp = FloatPoint { x: 1.0, y: 2.0 };
    assert_eq!(fp, fp);
    assert!(fp < FloatPoint { x: 3.0, y: 4.0 });
    // 但不能 derive Eq/Ord, 需要手动实现或确认无 NaN
}

#[test]
/// 测试: Sized 和 ?Sized 深入的边界场景
fn test_sized_vs_unsized() {
    // 语法: T: Sized 是默认约束, T: ?Sized 放宽此约束
    // 避坑:
    //   - ?Sized 只能用于泛型/trait 约束, 不能用于具体变量声明
    //   - dyn Trait 是 !Sized, 但 &dyn Trait 是 Sized (胖指针)
    //   - str 是 !Sized, 但 &str 是 Sized

    // ===== 默认 Sized 约束 =====
    fn only_sized<T>(_x: &T) -> String
    where
        T: std::fmt::Display,
    {
        String::from("需要 Sized")
    }
    // T 默认 Sized, 所以可以传递 &T (T 是 Sized, &T 也是 Sized)
    assert_eq!(only_sized(&42), "需要 Sized");

    // ===== ?Sized 放宽后可以接收 DST 的引用 =====
    fn maybe_unsized<T: std::fmt::Display + ?Sized>(_x: &T) -> String {
        String::from("允许 ?Sized")
    }
    assert_eq!(maybe_unsized("hello str"), "允许 ?Sized");
    assert_eq!(maybe_unsized(&42), "允许 ?Sized");

    // ===== dyn Trait 引用 vs 值 =====
    // &dyn Display: Sized (胖指针本身大小已知)
    let obj: &dyn std::fmt::Display = &42;
    assert!(std::mem::size_of_val(&obj) > 0);

    // ===== ?Sized 在 trait 定义中的应用 =====
    trait MaybeSizedTrait {
        fn describe(&self) -> String;

        // 这个方法要求 Self: Sized, 不能通过 dyn 调用
        fn from_value(val: i32) -> Self
        where
            Self: Sized;
    }

    #[derive(Debug, PartialEq)]
    struct Concrete {
        val: i32,
    }

    impl MaybeSizedTrait for Concrete {
        fn describe(&self) -> String {
            format!("Concrete({})", self.val)
        }

        fn from_value(val: i32) -> Self
        where
            Self: Sized,
        {
            Concrete { val }
        }
    }

    let c = Concrete { val: 99 };
    let obj_dyn: &dyn MaybeSizedTrait = &c;
    assert_eq!(obj_dyn.describe(), "Concrete(99)");
    // obj_dyn.from_value(10); // ❌ 不能通过 dyn 调用 (Self: Sized)

    // 具体类型可以调用
    assert_eq!(Concrete::from_value(10), Concrete { val: 10 });

    // ===== Box<dyn Trait> 是 Sized =====
    let boxed: Box<dyn std::fmt::Display> = Box::new(42);
    assert!(std::mem::size_of_val(&boxed) > 0);
    assert_eq!(format!("{}", boxed), "42");

    // ===== 验证 str 和 [T] 是 !Sized =====
    // 通过引用使用
    let s: &str = "hello";
    assert_eq!(s.len(), 5);
    let arr: &[i32] = &[1, 2, 3];
    assert_eq!(arr.len(), 3);
}

#[test]
/// 测试: 标记 trait (Send/Sync/Copy/Sized/Unpin) 综述
fn test_marker_traits() {
    // 语法: 标记 trait 不定义方法, 用类型系统标记安全属性
    //   核心标记 trait: Send, Sync, Copy, Sized, Unpin
    //
    // 避坑:
    //   - Rc<T>: !Send, !Sync
    //   - RefCell<T>: Send (条件), !Sync
    //   - 裸指针: !Send, !Sync
    //   - Cell<T> / RefCell<T> 有内部可变性, 与 Sync 互斥

    // ===== Send: 所有权可在线程间安全转移 =====
    fn assert_send<T: Send>() {}
    assert_send::<i32>();
    assert_send::<String>();
    assert_send::<Vec<i32>>();
    assert_send::<Box<i32>>();
    assert_send::<&i32>(); // &T: Send (如果 T: Sync)

    // ===== Sync: 引用可在线程间安全共享 =====
    fn assert_sync<T: Sync>() {}
    assert_sync::<i32>();
    assert_sync::<&i32>();
    // assert_sync::<Cell<i32>>(); // ❌ Cell !Sync (有内部可变性)

    // ===== Copy: 位拷贝而非移动 =====
    fn assert_copy<T: Copy>() {}
    assert_copy::<i32>();
    assert_copy::<f64>();
    assert_copy::<bool>();
    assert_copy::<char>();
    assert_copy::<&i32>(); // 共享引用总是 Copy
    assert_copy::<*const i32>(); // 裸指针是 Copy

    // ===== Sized: 编译期已知大小 (默认约束) =====
    fn assert_sized<T: Sized>() {}
    assert_sized::<i32>();
    assert_sized::<String>();
    assert_sized::<&dyn std::fmt::Display>(); // 胖指针是 Sized
    // assert_sized::<dyn std::fmt::Display>(); // ❌ dyn Trait 不是 Sized
    // assert_sized::<str>(); // ❌ str 不是 Sized

    // ===== 共享引用 &T 的性质 =====
    // &T: Copy  (始终)
    // &T: Send  (当 T: Sync)
    // &T: Sync  (当 T: Sync)

    // ===== 验证标记 trait 不占空间 =====
    // 这些 trait 都是编译期概念, 运行时零开销
    assert!(std::mem::size_of::<i32>() == 4);
    // i32 同时是 Send + Sync + Copy + Sized + Unpin
    // 并不因此增加任何内存开销
}

#[test]
/// 测试: ZST (零大小类型) 和 PhantomData 的综合使用
fn test_zst_and_phantomdata() {
    use std::marker::PhantomData;

    // ===== ZST 基础 =====
    struct UnitLike;
    assert_eq!(std::mem::size_of::<UnitLike>(), 0);
    assert_eq!(std::mem::align_of::<UnitLike>(), 1);

    // ===== PhantomData 作为 ZST =====
    assert_eq!(std::mem::size_of::<PhantomData<i32>>(), 0);
    assert_eq!(std::mem::size_of::<PhantomData<*const i32>>(), 0);

    // ===== 场景1: PhantomData 标记所有权 =====
    // 自定义类似 Box 的类型, 用 PhantomData 告诉 drop checker "我拥有 T"
    struct MyBox<T> {
        ptr: *mut T,
        _marker: PhantomData<T>, // 标记: 拥有 T 的所有权
    }

    impl<T> MyBox<T> {
        fn new(value: T) -> Self {
            let boxed = Box::new(value);
            MyBox {
                ptr: Box::into_raw(boxed),
                _marker: PhantomData,
            }
        }

        fn as_ref(&self) -> &T {
            unsafe { &*self.ptr }
        }
    }

    impl<T> Drop for MyBox<T> {
        fn drop(&mut self) {
            // PhantomData 告诉编译器: 我们拥有 T, 释放时需要 drop T
            unsafe {
                drop(Box::from_raw(self.ptr));
            }
        }
    }

    let mb = MyBox::new(42i32);
    assert_eq!(*mb.as_ref(), 42);

    // ===== 场景2: PhantomData 标记生命周期关系 =====
    // 类型不包含引用字段, 但逻辑上借用数据
    struct BorrowedSlice<'a, T> {
        ptr: *const T,
        len: usize,
        _marker: PhantomData<&'a T>, // 告诉编译器: 此类型借用了 T, 生命周期为 'a
    }

    let data = vec![1, 2, 3];
    let bs = BorrowedSlice {
        ptr: data.as_ptr(),
        len: data.len(),
        _marker: PhantomData,
    };
    unsafe {
        let slice = std::slice::from_raw_parts(bs.ptr, bs.len);
        assert_eq!(slice, &[1, 2, 3]);
    }

    // ===== 场景3: PhantomData 标记 Send/Sync 排除 =====
    struct NotSend {
        _marker: PhantomData<*const ()>, // *const () 不是 Send, 所以 NotSend 也不是 Send
    }

    // 验证 NotSend 确实 !Send
    // 编译期检查: 如果 NotSend 是 Send, 此行会编译失败
    // fn require_send<T: Send>() {}
    // require_send::<NotSend>(); // 如果执行: ❌ NotSend 不是 Send

    // ===== 场景4: PhantomData 控制类型变体 =====
    // 协变 (默认): PhantomData<T>
    struct Covariant<T> {
        _marker: PhantomData<T>,
    }

    // 逆变: PhantomData<fn(T)> - T 在函数参数位置
    struct Contravariant<T> {
        _marker: PhantomData<fn(T)>,
    }

    // 不变: PhantomData<*mut T> - T 通过裸指针使用
    struct Invariant<T> {
        _marker: PhantomData<*mut T>,
    }

    // 都是 ZST
    assert_eq!(std::mem::size_of::<Covariant<i32>>(), 0);
    assert_eq!(std::mem::size_of::<Contravariant<i32>>(), 0);
    assert_eq!(std::mem::size_of::<Invariant<i32>>(), 0);

    // ===== 场景5: PhantomData 在不同变体场景中的使用 =====
    use std::any::TypeId;

    // 不同类型的 TypeId 不同
    assert_ne!(TypeId::of::<PhantomData<i32>>(), TypeId::of::<PhantomData<String>>());

    // PhantomData 可用于异构集合
    struct TypedSlot<T> {
        _marker: PhantomData<T>,
        id: TypeId,
    }

    impl<T: 'static> TypedSlot<T> {
        fn new() -> Self {
            TypedSlot {
                _marker: PhantomData,
                id: TypeId::of::<T>(),
            }
        }
    }

    let slot_i32 = TypedSlot::<i32>::new();
    let slot_str = TypedSlot::<String>::new();
    assert_eq!(slot_i32.id, TypeId::of::<i32>());
    assert_ne!(slot_i32.id, slot_str.id);
}
