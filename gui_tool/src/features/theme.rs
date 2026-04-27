//! 主题配置模块
//!
//! 提供全局统一的字体大小、组件尺寸和内边距计算。
//! 所有 UI 组件应使用此模块的函数确保视觉一致性。
//!
//! # 设计模式: 主题集中管理
//! 将尺寸计算集中在一个模块中，而非在各视图文件里硬编码数值。
//! 这样做的好处：
//! - **单一修改点**: 需要调整整体大小时，只需修改 `FONT_SIZE` / `COMPONENT_SIZE`
//! - **视觉一致性**: 所有组件通过倍数引用，比例关系自动保持
//! - **语义化**: `font(1.5)` 比写死 `21.0` 更能表达"这是标题字体"
//!
//! # Rust 语法要点
//!
//! ## `const` vs `static`
//! - `pub const FONT_SIZE: f32 = 14.0;` —— 编译时常量，值在编译时内联到使用处。
//!   const 没有固定内存地址，每次使用时会复制值。
//! - `static` —— 运行时常量，有固定内存地址，适合需要引用的场景。
//!
//! ## 函数签名 `fn font(multiplier: f32) -> f32`
//! Rust 的函数参数必须显式标注类型，没有类型推断。
//! `->` 箭头后是返回类型，无返回值时省略（即返回 `()` unit type）。
//!
//! ## `Pixels` 新类型模式 (Newtype Pattern)
//! `iced::Pixels` 是一个新类型包装：`struct Pixels(pub f32)`。
//! 它包装 `f32` 但语义上是独立的类型，不能直接与 `f32` 混用。
//! 这利用了 Rust 的类型系统防止混淆 —— `Pixels` 不能直接传给需要 `f32` 的函数。
//! 访问内部值用 `.0` 语法（tuple struct 字段访问）。

use iced::Pixels;

/// 基础字体大小 (像素)
///
/// # Rust: `pub const`
/// `pub const` 定义编译期公共常量。const 的值必须在编译时确定。
/// `f32` 是 Rust 的 32 位浮点类型，对应 C 的 `float`。
/// `14.0` 是 f32 字面量 —— 带小数点的数字默认是 f64，但赋给 f32 变量时编译器自动转换。
pub const FONT_SIZE: f32 = 14.0;

/// 基础组件大小 (像素)
pub const COMPONENT_SIZE: f32 = 14.0;

/// 计算字体大小
///
/// # Rust: 文档注释 `///`
/// 支持 Markdown 格式，`cargo doc` 会生成 HTML 文档。
/// 代码块示例会自动测试（运行 `cargo test` 时），除非标注 `no_run` 或 `ignore`。
///
/// # Rust: 函数定义语法
/// ```text
/// pub fn 函数名(参数名: 类型) -> 返回类型 { 函数体 }
/// ```
/// - `pub` 表示公开可见（可从模块外调用）
/// - 最后一行表达式（不加分号）作为返回值 —— 这是 Rust 的"表达式语言"特性
///
/// # 参数
/// - `multiplier`: 倍数，如 1.0 = 基础大小，1.2 = 放大 20%
///
/// # 返回值
/// 计算后的字体像素大小（f32 类型）
pub fn font(multiplier: f32) -> f32 {
    // `*` 是乘法运算符，f32 之间的乘法返回 f32
    FONT_SIZE * multiplier
}

/// 计算组件尺寸
///
/// # 参数
/// - `multiplier`: 倍数，如 `size(0.5)` = 7 像素
///
/// # 返回值
/// `iced::Pixels` 类型的包装值，可直接用于 widget 的 `.width()` / `.height()` 方法。
///
/// # Rust: 结构的构造语法
/// `Pixels(value)` 是 "tuple struct" 构造语法。
/// `Pixels(COMPONENT_SIZE * multiplier)` 创建一个 Pixels 实例，
/// 其内部 f32 字段值为计算结果。
pub fn size(multiplier: f32) -> Pixels {
    Pixels(COMPONENT_SIZE * multiplier)
}

/// 计算统一内边距（上下左右相同）
///
/// # Rust: `iced::Padding` 类型
/// `Padding` 可以多种方式构造：
/// - `Padding::from([top_bottom, left_right])` —— 来自长度为 2 的数组
/// - `Padding::from(u16)` —— 四边相同
/// - `Padding { top, right, bottom, left }` —— 结构体字面量
pub fn padding(multiplier: f32) -> iced::Padding {
    let p = size(multiplier).0;
    // `size(multiplier).0` 访问 Pixels 的内部 f32 值
    // `.0` 是 tuple struct 的字段访问语法（字段用索引而非名称）
    iced::Padding::from([p, p])
}

/// 计算非统一内边距（垂直和水平方向不同）
///
/// # 参数
/// - `v`: 垂直方向（上下）的倍数
/// - `h`: 水平方向（左右）的倍数
pub fn padding2(v: f32, h: f32) -> iced::Padding {
    // 分别计算垂直和水平像素值，再构造 Padding
    iced::Padding::from([size(v).0, size(h).0])
}
