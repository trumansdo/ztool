# Rust 语法特性知识体系

本目录收集了 Rust 语法特性的学习笔记，按学习曲线从入门到精通排序，共 **35 个主题**，覆盖 Rust 语言的核心概念、标准库、现代特性及高级编程范式。

> **学 Rust 就是学三样东西：所有权系统让内存安全无需 GC，类型系统让错误在编译期无处遁形，零成本抽象让你写出 C 一样快、Haskell 一样优雅的代码。**

## 学习路径

### 第一阶段：入门基础（01-09）—— 掌握 Rust 思维模型

> **所有权是 Rust 的突破性特性，它让 Rust 在完全内存安全和高效的同时，避免了垃圾回收。** — Rustonomicon

理解 Rust 的核心设计哲学——所有权、类型系统、表达式导向。变量与控制流是地基，所有权是灵魂，模式匹配和错误处理是日常武器。

| 编号 | 文件 | 主要内容 | 重要性 |
|------|------|----------|--------|
| 01 | [01-variables-types.md](01-variables-types.md) | 变量绑定、可变性、数据类型、常量、shadowing、类型推断、整数溢出 | ⭐⭐⭐⭐⭐ |
| 02 | [02-strings.md](02-strings.md) | String 与 &str、UTF-8 编码、字符串操作、字符串切片、OsString/PathBuf | ⭐⭐⭐⭐⭐ |
| 03 | [03-ownership-borrowing.md](03-ownership-borrowing.md) | 所有权规则、借用与引用、切片、内存模型、move 语义、NLL | ⭐⭐⭐⭐⭐ |
| 04 | [04-arrays-vecs.md](04-arrays-vecs.md) | 数组、切片 `&[T]`、Vec 动态数组、常用方法、容量管理、VecDeque | ⭐⭐⭐⭐⭐ |
| 05 | [05-rust模块学习笔记.md](05-rust模块学习笔记.md) | 模块系统：mod、use、pub 可见性、crate/self/super 路径、重导出 | ⭐⭐⭐⭐ |
| 06 | [06-control-flow.md](06-control-flow.md) | if/loop/while/for、循环标签、break 返回值、表达式 vs 语句、match | ⭐⭐⭐⭐⭐ |
| 07 | [07-structs-enums.md](07-structs-enums.md) | 结构体、枚举、方法 impl、Option/Result、derive、内存布局、NPO 优化 | ⭐⭐⭐⭐⭐ |
| 08 | [08-pattern-matching.md](08-pattern-matching.md) | match、if let、解构、@ 绑定、匹配守卫、穷尽性检查、let-else | ⭐⭐⭐⭐⭐ |
| 09 | [09-error-handling.md](09-error-handling.md) | Option/Result、? 操作符、panic!、thiserror/anyhow、错误链、catch_unwind | ⭐⭐⭐⭐⭐ |

### 第二阶段：核心基础（10-15）—— 构建标准库能力

> **零成本抽象意味着：你不用的，不必为之付费；你使用的，手写代码也无法做得更好。** — Bjarne Stroustrup

掌握 Rust 标准库的核心容器、抽象工具和转换能力——这是日常编程中使用频率最高的技能集合。

| 编号 | 文件 | 主要内容 | 重要性 |
|------|------|----------|--------|
| 10 | [10-iterators.md](10-iterators.md) | 迭代器 trait、适配器 map/filter/fold、消费者、自定义迭代器、惰性求值 | ⭐⭐⭐⭐⭐ |
| 11 | [11-collections.md](11-collections.md) | HashMap/BTreeMap、HashSet/BTreeSet、VecDeque、LinkedList、Entry API、性能对比 | ⭐⭐⭐⭐ |
| 12 | [12-type-system.md](12-type-system.md) | Trait 定义与实现、dyn Trait、对象安全、关联类型、孤儿规则、ZST、PhantomData | ⭐⭐⭐⭐⭐ |
| 13 | [13-generics.md](13-generics.md) | 泛型函数/结构体/枚举、trait bound、const 泛型、单态化、类型状态模式 | ⭐⭐⭐⭐⭐ |
| 14 | [14-lifetimes.md](14-lifetimes.md) | 生命周期标注、消除规则、'static、HRTB、RPIT、子类型关系、变型 | ⭐⭐⭐⭐⭐ |
| 15 | [15-smart-pointers.md](15-smart-pointers.md) | Box/Rc/Arc、Deref/Drop、Cell/RefCell、内部可变性、Cow、Weak、Pin 概念 | ⭐⭐⭐⭐⭐ |

### 第三阶段：函数式与元编程（16-19）

> **泛型和 trait 是 Rust 零成本抽象的支柱——运行时开销为零，抽象能力无限。**

闭包是 Rust 的函数式利器，宏让你在语法层面消除重复。这一阶段完成 Rust 表达能力的关键拼图。

| 编号 | 文件 | 主要内容 | 重要性 |
|------|------|----------|--------|
| 16 | [16-functions-closures.md](16-functions-closures.md) | 函数定义、闭包捕获模式、Fn/FnMut/FnOnce、高阶函数、fn 指针 | ⭐⭐⭐⭐ |
| 17 | [17-macros.md](17-macros.md) | macro_rules! 声明宏、重复模式、卫生性、递归宏、内置宏 | ⭐⭐⭐⭐ |
| 18 | [18-macro-fragments.md](18-macro-fragments.md) | 宏片段说明符 expr/stmt/ty/ident/tt、重复修饰符、Edition 2024 变更 | ⭐⭐⭐ |
| 19 | [19-tuple-iterator.md](19-tuple-iterator.md) | unzip/partition、reduce/fold、元组收集、try_fold、多元素元组 | ⭐⭐⭐ |

### 第四阶段：并发与异步（20-22）—— 无畏并发

> **无畏并发——允许你编写没有细微 bug 的并发代码，并且可以放心重构而不引入新问题。**

Send 允许跨线程转移所有权，Sync 允许跨线程共享引用。Rust 的异步不是后台运行的魔法——Future 本身就是计算，由你驱动轮询。

| 编号 | 文件 | 主要内容 | 重要性 |
|------|------|----------|--------|
| 20 | [20-async-basics.md](20-async-basics.md) | async/await、Future trait、状态机、Pin 与异步、tokio、join!/select!、Stream | ⭐⭐⭐⭐⭐ |
| 21 | [21-async-closures.md](21-async-closures.md) | AsyncFn/AsyncFnMut/AsyncFnOnce、async move 闭包、async 闭包捕获与组合 | ⭐⭐⭐ |
| 22 | [22-concurrency.md](22-concurrency.md) | 线程 spawn/join/scope、Send/Sync、Mutex/RwLock、channel、Atomic、Barrier/Condvar | ⭐⭐⭐⭐⭐ |

### 第五阶段：系统编程（23-27）—— 触碰底层

> **Unsafe Rust 不是让你抛弃安全，而是让你在编译器无法验证的边界上，用契约承担起程序员的责任。**

当安全 Rust 的抽象不足以表达你的需求时，unsafe 给你打开了一扇通往底层世界的大门。

| 编号 | 文件 | 主要内容 | 重要性 |
|------|------|----------|--------|
| 23 | [23-unsafe-programming.md](23-unsafe-programming.md) | 裸指针、MaybeUninit、transmute、union、NonNull、unsafe trait/impl、FFI 包装 | ⭐⭐⭐⭐ |
| 24 | [24-ffi.md](24-ffi.md) | extern "C"、#[no_mangle]、repr(C)、CStr/CString、回调函数、ABI 稳定性 | ⭐⭐⭐ |
| 25 | [25-testing.md](25-testing.md) | 单元测试、集成测试、#[cfg(test)]、should_panic、文档测试、测试组织 | ⭐⭐⭐⭐ |
| 26 | [26-file-io.md](26-file-io.md) | 文件读写、BufReader/BufWriter、OpenOptions、Seek、Cursor、Path/PathBuf、serde | ⭐⭐⭐⭐ |
| 27 | [27-conditional-compilation.md](27-conditional-compilation.md) | cfg 属性、cfg! 宏、cfg_attr、target 条件、feature 标志、平台代码组织 | ⭐⭐⭐ |

### 第六阶段：现代特性与版本前瞻（28-35）

> **Rust 每六周一次发布，不断进化——保持对新特性的关注，就是保持技术竞争力。**

Rust 语言持续演进，Edition 2024 带来众多改进。了解前沿特性，不仅能写出更优雅的代码，还能为未来的 Rust 版本做好准备。

| 编号 | 文件 | 主要内容 | 重要性 |
|------|------|----------|--------|
| 28 | [28-let-chains.md](28-let-chains.md) | Let Chains、if let 链式组合、短路求值、while let 链式 | ⭐⭐⭐ |
| 29 | [29-if-let-scope.md](29-if-let-scope.md) | if let 临时作用域 (Edition 2024 变更)、Drop 顺序、MutexGuard 持有时间 | ⭐⭐⭐ |
| 30 | [30-diagnostic-attributes.md](30-diagnostic-attributes.md) | diagnostic::do_not_recommend、must_use、deprecated、track_caller、cold | ⭐⭐⭐ |
| 31 | [31-precise-capturing.md](31-precise-capturing.md) | 精确捕获 use<> 语法、RPIT 生命周期控制、impl Trait 改进 | ⭐⭐⭐ |
| 32 | [32-reserved-keywords.md](32-reserved-keywords.md) | 保留关键字、raw identifier r# 语法、gen/in 关键字、Edition 迁移 | ⭐⭐⭐ |
| 33 | [33-edition-2024-safety.md](33-edition-2024-safety.md) | unsafe extern、repr 对齐变更、Never Type !、unsafe 块语义、安全层次 | ⭐⭐⭐ |
| 34 | [34-latest-features.md](34-latest-features.md) | const fn 进阶、const 泛型、关联常量默认值、trait upcasting、空白et impl | ⭐⭐⭐ |
| 35 | [35-comprehensive.md](35-comprehensive.md) | 综合测试与集成：多特性组合、异步+集合、类型状态、RAII、unsafe 包装 | ⭐⭐ |

---

## 学习建议

### 初识 Rust（第一阶段：01-09）

**核心任务**：理解所有权机制，能用 Rust 写出正确的程序。

- 变量和控制流是基础中的基础，务必熟练掌握表达式导向思维——`if` 和 `loop` 有返回值
- 字符串看似简单实则复杂：String 在堆上，&str 是借用，UTF-8 编码意味着不能随意索引
- 所有权是 Rust 最独特的概念——理解它你就理解了 Rust 的设计灵魂。反复练习 &T vs &mut T vs T 的区别
- 模块系统（05）尽早学习，了解如何组织代码——不要等到代码混乱了再回头学
- 用 match 和 if let 替代你过去习惯的 if-else 链——习惯后回不去了
- 尽早习惯 Result 和 ? 操作符，它们是 Rust 错误处理的门面

### 核心突破（第二阶段：10-15）

**核心任务**：掌握标准库的核心工具，让代码简洁且高性能。

- 迭代器是 Rust 的循环——`for` 实际上解糖为迭代器调用。学会用 map/filter/fold 链式表达逻辑
- 集合选型有讲究：HashMap 查找 O(1)，BTreeMap 有序 O(log n)，VecDeque 双端 O(1)
- trait 是 Rust 的接口，但比 OOP 的接口强大得多：关联类型、默认实现、blanket implementation
- 泛型通过单态化实现零成本——编译后没有泛型参数，只有具体类型的代码
- "生命周期不能解决的问题，都是内存结构表现力不足的外在反映"
- Box 用于堆分配，Rc/Arc 用于共享所有权，Cell/RefCell 实现内部可变性

### 进阶抽象（第三阶段：16-19）

**核心任务**：用闭包和宏提升代码表达力，完成 Rust 核心技能拼图。

- 闭包捕获有三种模式：借用的 Fn、可变借用的 FnMut、获取所有权的 FnOnce
- 声明宏（macro_rules!）适合消除重复的样板代码——理解片段说明符和重复模式
- 元组迭代器方法（unzip、partition）让你用声明式风格处理多路数据流

### 并发世界（第四阶段：20-22）

**核心任务**：安全地驾驭多线程和异步编程。

- Rust 的类型系统天然防止数据竞争——Send 和 Sync 是编译期就确定的并发安全契约
- Mutex 保护共享数据，channel 用于线程间通信——"不要通过共享内存通信，而要通过通信共享内存"
- 异步编程的核心是 Future trait——它是一种惰性状态机，由运行时轮询驱动
- Pin 是自引用结构和异步编程的基础——保证值在创建后不会在内存中移动

### 系统底层（第五阶段：23-27）

**核心任务**：在安全抽象的边界上，用 unsafe 触及操作系统和硬件的底层能力。

- unsafe 代码不等于不安全代码——它只是把安全验证的责任从编译器转移到了程序员
- FFI 让你复用数十年的 C 语言生态——Rust 是 C 的超级替代品
- 测试不是事后补课——Rust 的测试框架内置且简洁，养成写测试的习惯
- Cargo 是 Rust 的杀手级工具：依赖管理、构建、测试、发布一条龙

### 前沿探索（第六阶段：28-35）

**核心任务**：跟上 Rust 社区的演进节奏，了解 Edition 2024 和 nightly 特性。

- Rust 的版本管理保证了向后兼容——一个 Rust 1.0 的 crate 仍能在最新编译器上编译
- Let Chains 让你写出更紧凑的条件逻辑，是 if let 的自然进化
- 精确捕获（use<>）让 impl Trait 返回类型的生命周期控制更精细化
- 不需要记住所有新特性，但要知道"有这个能力"，需要时能回来查阅

---

## 推荐学习资源

- **入门必读**：[The Rust Programming Language](https://doc.rust-lang.org/book/)（Rust 圣经，官方入门教程）
- **实例驱动**：[Rust By Example](https://doc.rust-lang.org/rust-by-example/)（通过可运行的示例学习每个特性）
- **体系课程**：[Google Comprehensive Rust](https://google.github.io/comprehensive-rust/)（Google 内部 Rust 培训材料）
- **进阶必看**：[Rustonomicon](https://doc.rust-lang.org/nomicon/)（unsafe 编程指南，系统编程必读）
- **实战练习**：[Rustlings](https://github.com/rust-lang/rustlings/)（通过修复编译错误学习 Rust）
- **参考路线**：[Rust 学习路线图 roadmap.sh](https://roadmap.sh/rust)（社区维护的技能树）
- **异步指南**：[Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)（官方异步编程手册）
- **设计模式**：[Rust Design Patterns](https://rust-unofficial.github.io/patterns/)（社区 Rust 模式目录）
- **标准库文档**：[std API 文档](https://doc.rust-lang.org/std/)（权威参考，遇到类型/方法及时查阅）
- **小技巧集合**：[Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/)（常见任务的 Rust 惯用解法）
- **Rust 周刊**：[This Week in Rust](https://this-week-in-rust.org/)（社区动态、新 crate、技术文章每周汇总）
