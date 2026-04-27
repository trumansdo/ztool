//! Toast 消息通知组件
//!
//! 在屏幕上显示临时浮动通知，支持 4 种级别 × 9 个位置 = 36 种组合。
//! 纯自定义实现，未使用 `iced_toaster` 库。
//!
//! # Rust 语法要点
//!
//! ## 枚举 (enum) 的进阶用法
//! Rust 的枚举变体可以携带方法、实现 trait。
//! 本文件中的 `ToastLevel`、`ToastPosition` 都是枚举，每个都有：
//! - `#[derive(...)]` 自动生成 trait 实现
//! - `impl ToastLevel { fn border_color(&self) -> Color { ... } }` 方法定义
//!
//! ## `match` 穷尽性强制
//! `match` 必须覆盖所有可能的变体。如果新增了变体但忘记添加相应的 match 分支，
//! 编译器会报错。这是 Rust 安全性的重要来源。
//!
//! ## 闭包 (Closure)
//! `|_: &Theme, status: button::Status| { ... }` 是闭包语法：
//! - `|参数| { 函数体 }` 是闭包的字面量语法
//! - `_` 忽略第一个参数（因为不需要 theme 信息）
//! - 闭包可以捕获外部变量（这里是 `border_color`）
//!
//! ## 泛型函数
//! `fn view_toasts<'a, M>(toasts: &[ToastItem], on_dismiss: impl Fn(u64) -> M) -> Vec<...>`
//! - `'a`: 生命周期参数
//! - `M`: 消息类型参数
//! - `&[ToastItem]`: 切片引用（类似 &Vec 但更通用）
//! - `impl Fn(u64) -> M`: 接受任何实现了 Fn(u64) -> M 的类型（闭包或函数指针）
//!
//! ## `use super::...` —— 模块树引用
//! `super` 引用父模块（`widgets`），`super::overlay` 即 `widgets::overlay`。
//! 也可以写 `crate::ui::widgets::overlay`（绝对路径），但 `super` 更简洁且不依赖顶层结构。

use iced::widget::{button, container, row, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use super::overlay::{Anchor, Layered};

/// Toast 级别 —— 控制提示消息的颜色和语义
///
/// # Rust: `#[derive(PartialEq, Eq)]`
/// `PartialEq` + `Eq` 允许用 `==` 比较两个 ToastLevel 是否相等。
/// `#[default]` 标记默认变体（`Info`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Toast 显示位置 —— 9 宫格定位
///
/// 通过 `anchor()` 方法映射到具体的 Anchor 值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    #[default]
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// 为 ToastPosition 实现 `anchor()` 方法
///
/// 每个位置变体转换为对应的 `Anchor` 结构体，用于叠加层定位。
impl ToastPosition {
    pub fn anchor(self) -> Anchor {
        match self {
            // Anchor::top_left(top, left) 创建带顶部+左侧锚点
            ToastPosition::TopLeft => Anchor::top_left(8.0, 8.0),
            // Anchor { top: Some(8.0), ..default() } 只设 top，水平方向居中
            ToastPosition::TopCenter => Anchor { top: Some(8.0), ..Anchor::default() },
            ToastPosition::TopRight => Anchor::top_right(8.0, 8.0),
            ToastPosition::CenterLeft => Anchor { left: Some(8.0), ..Anchor::default() },
            ToastPosition::Center => Anchor::center(),
            ToastPosition::CenterRight => Anchor { right: Some(8.0), ..Anchor::default() },
            ToastPosition::BottomLeft => Anchor::bottom_left(8.0, 8.0),
            ToastPosition::BottomCenter => Anchor { bottom: Some(8.0), ..Anchor::default() },
            ToastPosition::BottomRight => Anchor::bottom_right(8.0, 8.0),
        }
    }
}

/// 单个 Toast 的完整信息
///
/// # Rust: `#[derive(Debug, Clone)]`
/// - `Debug`: 打印调试信息 `println!("{:?}", toast)`
/// - `Clone`: 生成深拷贝 `toast.clone()`
/// 不能 derive Copy，因为包含 `String`（堆分配类型）
#[derive(Debug, Clone)]
pub struct ToastItem {
    /// 唯一标识符，用于删除特定 toast
    pub id: u64,
    /// 消息级别（影响左边框颜色）
    pub level: ToastLevel,
    /// 显示的文本内容
    pub text: String,
    /// 在屏幕上的显示位置
    pub position: ToastPosition,
}

/// 为 ToastLevel 添加获取边框颜色的方法
impl ToastLevel {
    /// 根据级别返回对应的边框颜色
    fn border_color(&self) -> Color {
        match self {
            // Color::from_rgb(r, g, b) 接收 0.0~1.0 范围的浮点值
            ToastLevel::Info => Color::from_rgb(0.35, 0.55, 0.85),
            ToastLevel::Success => Color::from_rgb(0.25, 0.70, 0.35),
            ToastLevel::Warning => Color::from_rgb(0.80, 0.65, 0.15),
            ToastLevel::Error => Color::from_rgb(0.80, 0.25, 0.25),
        }
    }
}

/// 将 Toast 列表渲染为一组 Layered 叠加元素
///
/// # Rust: 泛型函数与生命周期
/// ```text
/// pub fn view_toasts<'a, M: Clone + 'a>(
///     toasts: &'a [ToastItem],
///     on_dismiss: impl Fn(u64) -> M + Clone + 'a,
/// ) -> Vec<Layered<'a, M>>
/// ```
/// - `'a`: 生命周期参数，表示 toast 切片和 Layered 元素都存活 'a
/// - `M: Clone + 'a`: M 必须实现 Clone（闭包需要克隆），且存活至少 'a
/// - `impl Fn(u64) -> M + Clone + 'a`: 接受任何实现了 Fn 闭包 trait 且可克隆的类型
///
/// # Rust: 迭代器与 `map` + `collect`
/// `toasts.iter().map(|toast| { ... }).collect()`:
/// - `.iter()`: 创建不可变借用迭代器，产生 `&ToastItem`
/// - `.map(|toast| { ... })`: 将每个 toast 转换为 Layered 元素
/// - `.collect()`: 收集为 Vec<Layered>，编译器从函数返回类型推断目标集合
pub fn view_toasts<'a, M: Clone + 'a>(
    toasts: &'a [ToastItem],
    on_dismiss: impl Fn(u64) -> M + Clone + 'a,
) -> Vec<Layered<'a, M>> {
    toasts
        .iter()
        .map(|toast| {
            let border_color = toast.level.border_color();
            let id = toast.id;
            let on_dismiss = on_dismiss.clone();

            // 关闭按钮 ×
            //
            // Rust: 闭包捕获
            // `move` 关键字使闭包获取变量所有权（这里不需要）。
            // `.style(|_: &Theme, status: button::Status| { ... })`
            // 这个闭包在每次按钮需要重绘时被 iced 调用，
            // 接收 theme 引用和按钮状态，返回 button::Style。
            let close_btn: Element<'a, M> = button(text("×").size(9))
                .on_press(on_dismiss(id))
                .padding([0, 3])
                .style(|_: &Theme, status: button::Status| {
                    // match 表达式作为值返回
                    let bg = match status {
                        button::Status::Hovered => Color::from_rgb(0.3, 0.3, 0.35),
                        button::Status::Pressed => Color::from_rgb(0.4, 0.4, 0.45),
                        // Status::Active 等默认情况 → 透明
                        _ => Color::TRANSPARENT,
                    };
                    button::Style {
                        // `Some(Background::Color(bg))` —— Option 包装
                        background: Some(Background::Color(bg)),
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            // `.into()` 将 f32 3.0 转为 `Radius::from(3.0)`
                            // Rust 的 From/Into trait 实现自动类型转换
                            radius: 3.0.into(),
                        },
                        text_color: Color::from_rgb(0.55, 0.55, 0.6),
                        // `..Default::default()` —— 其余字段用默认值
                        ..Default::default()
                    }
                })
                .into();

            // Toast 主体：文本 + 关闭按钮
            //
            // `row![...]` 是 iced 的声明式布局宏，
            // 展开为 `Row::with_children(vec![...])`。
            let body = container(
                row![
                    // `text(&toast.text)` 借用 text 字段
                    // `.width(Length::Fill)` 让文本占据剩余空间，把按钮推到右边
                    text(&toast.text).size(12).width(Length::Fill),
                    close_btn,
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .padding([6, 8])
            .width(Length::Fixed(220.0))
            .style(move |_: &Theme| container::Style {
                // 深色背景 + 颜色边框（左侧）
                background: Some(Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into();

            // 创建 Layered 元素并设置锚点
            Layered::new(body).anchor(toast.position.anchor())
        })
        .collect()
}
