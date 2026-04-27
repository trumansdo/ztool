//! 分层覆盖系统 (Layer + Anchor 定位)
//!
//! 为 iced 的 `stack` 布局提供锚点定位能力。
//! 在一层主内容上叠加多个浮动元素（如 Toast 通知）。
//!
//! # Rust 语法要点
//!
//! ## `struct` 结构体定义
//! ```text
//! pub struct Anchor {
//!     pub top: Option<f32>,
//!     pub right: Option<f32>,
//!     ...
//! }
//! ```
//! Rust 的 struct 字段需要显式标注类型，没有默认值。
//! `f32` 是 32 位浮点类型（IEEE 754 单精度）。
//!
//! ## `Option<f32>` —— 可空值
//! `Option<T>` 枚举有两个变体：
//! - `Some(T)` —— 有值
//! - `None` —— 无值/空
//! 使用 Option 替代 null，编译器强制处理空值情况，消除空指针错误。
//! 例如：`top: Some(8.0)` 表示距顶部 8px，`top: None` 表示不设置顶部偏移。
//!
//! ## `impl Default for Anchor` —— 自定义默认值
//! 为类型实现 `Default` trait，提供 `Anchor::default()` 方法。
//! 这里的默认值是所有方向 = None（即居中定位）。
//!
//! ## 关联函数 vs 方法
//! - `Anchor::top_left(8.0, 8.0)` —— 关联函数（类似静态方法），用 `::` 调用
//! - `self.anchor(...)` —— 实例方法，用 `.` 调用，第一个参数是 `self`
//!
//! ## 泛型 struct + 生命周期
//! `pub struct Layered<'a, M>` 有两个泛型参数：
//! - `'a`: 生命周期参数 —— 标记 Element 借用的数据需要存活多久
//! - `M`: 消息类型 —— 不同子模块有不同的消息类型
//!
//! ## 泛型方法
//! `pub fn map<N>(self, f: impl Fn(M) -> N) -> Layered<'a, N>`
//! 将 Layered<M> 转换为 Layered<N>，通过一个映射函数转换消息类型。
//! 这在 iced 中非常常见 —— 子模块的 Msg 需要被包装为顶层 Message。

use iced::widget::{container, stack};
use iced::{Alignment, Element, Length, Padding};

/// 锚点定义 —— 控制浮动元素在父容器中的位置
///
/// # 定位原理
/// 使用 iced 的 alignment + padding 机制模拟绝对定位：
/// - `align_x(End)` + `right: 8.0` → 靠右 + 右边距 8px
/// - `align_y(Start)` + `top: 8.0` → 靠上 + 上边距 8px
///
/// 四个字段都是 `Option<f32>`，只有 `Some(value)` 的方向才会生效。
/// 例如 `Anchor { top: Some(8.0), right: Some(8.0), ..default() }`
/// 表示右上角定位，距顶部 8px，距右侧 8px。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    /// 距顶部的像素距离，None 表示不限制
    pub top: Option<f32>,
    /// 距右侧的像素距离，None 表示不限制
    pub right: Option<f32>,
    /// 距底部的像素距离，None 表示不限制
    pub bottom: Option<f32>,
    /// 距左侧的像素距离，None 表示不限制
    pub left: Option<f32>,
}

/// 为 Anchor 实现 Default trait
///
/// # Rust: `impl Default for Anchor`
/// 实现标准库的 Default trait，这是 Rust 的惯用模式。
/// `Anchor::default()` 返回 `Anchor { top: None, right: None, bottom: None, left: None }`
/// 即所有方向都不限制，元素居中显示。
impl Default for Anchor {
    fn default() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
        }
    }
}

impl Anchor {
    /// 创建左上角锚点
    ///
    /// # Rust: `Self` 在 impl 块中
    /// `Self` 是当前 impl 块目标类型的别名 = `Anchor`。
    /// `..Self::default()` 是 struct update syntax —— 用 default 填充未指定的字段。
    pub fn top_left(top: f32, left: f32) -> Self {
        Self {
            top: Some(top),
            left: Some(left),
            ..Self::default()
        }
    }

    /// 创建右上角锚点
    pub fn top_right(top: f32, right: f32) -> Self {
        Self {
            top: Some(top),
            right: Some(right),
            ..Self::default()
        }
    }

    /// 创建左下角锚点
    pub fn bottom_left(bottom: f32, left: f32) -> Self {
        Self {
            bottom: Some(bottom),
            left: Some(left),
            ..Self::default()
        }
    }

    /// 创建右下角锚点
    pub fn bottom_right(bottom: f32, right: f32) -> Self {
        Self {
            bottom: Some(bottom),
            right: Some(right),
            ..Self::default()
        }
    }

    /// 创建居中锚点（四个方向均为 None，容器 alignment 决定位置）
    pub fn center() -> Self {
        Self::default()
    }
}

/// 带锚点定位的子元素
///
/// 将任意 Element 与一个 Anchor 绑定，用于在 stack 布局中定位。
///
/// # 泛型参数
/// - `'a`: 生命周期，确保 Element 中引用的数据在 Layered 存活期间有效
/// - `M`: 消息类型，元素发出的消息类型
///
/// # Rust: Phantom Data?
/// 虽然 `M` 不直接存储，但因为 `Element<'a, M>` 中包含 M，
/// Rust 编译器不需要 PhantomData 也能正确追踪类型参数。
pub struct Layered<'a, M> {
    /// 实际要显示的 UI 元素
    content: Element<'a, M>,
    /// 该元素在父容器中的锚点定位
    anchor: Anchor,
}

/// `impl<'a, M: 'a>` 约束说明：
/// - `'a` 是生命周期参数
/// - `M: 'a` 是生命周期约束 —— M 类型必须至少存活 'a 这么久
impl<'a, M: 'a> Layered<'a, M> {
    /// 创建新的 Layered 元素，默认使用居中锚点
    pub fn new(content: Element<'a, M>) -> Self {
        Self {
            content,
            anchor: Anchor::default(),
        }
    }

    /// 设置锚点位置（Builder 模式 —— 返回 Self 方便链式调用）
    ///
    /// # Rust: Builder 模式
    /// `fn anchor(mut self, anchor: Anchor) -> Self` 接收所有权 (owned self)，
    /// 修改后返回 self。这种模式允许链式调用：
    /// `Layered::new(element).anchor(Anchor::top_right(8.0, 8.0))`
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// 转换消息类型 —— 将 Layered<M> 映射为 Layered<N>
    ///
    /// # Rust: 高阶函数 `Fn(M) -> N`
    /// `impl Fn(M) -> N` 接受一个闭包或函数指针。
    /// `content.map(f)` 将 Element 中的消息通过 `f` 转换。
    /// 这在 iced 中是标准模式：子页面的 Msg 需要映射为顶层 Message。
    pub fn map<N: 'a>(self, f: impl Fn(M) -> N + 'a) -> Layered<'a, N> {
        Layered {
            content: self.content.map(f),
            anchor: self.anchor,
        }
    }

    /// 将 Layered 转换为实际的 iced Element
    ///
    /// # 定位实现原理
    /// 用 `container` 包裹内容，设置 `Length::Fill` 填满父 stack 空间，
    /// 然后通过 `align_x` / `align_y` 将内容推到指定位置。
    ///
    /// 示例：Anchor { top: Some(8.0), right: Some(8.0) }
    /// - `has_right = true` → `align_x = Alignment::End` → 内容推到右侧
    /// - `has_bottom = false` → `align_y = Alignment::Start` → 内容置顶
    /// - `padding` { top: 8, right: 8 } → 距右上角 8px
    fn into_element(self) -> Element<'a, M> {
        // `Option::is_some()` 返回 true 如果值是 Some
        let has_right = self.anchor.right.is_some();
        let has_bottom = self.anchor.bottom.is_some();

        // Rust: if 是表达式，可以用于赋值
        // 等价于三元运算符: ax = has_right ? Alignment::End : Alignment::Start
        let ax = if has_right { Alignment::End } else { Alignment::Start };
        let ay = if has_bottom { Alignment::End } else { Alignment::Start };

        // `unwrap_or(default)` 从 Option 中取值，None 时返回 default
        let px = self.anchor.left.unwrap_or(0.0);
        let py = self.anchor.top.unwrap_or(0.0);
        let pr = self.anchor.right.unwrap_or(0.0);
        let pb = self.anchor.bottom.unwrap_or(0.0);

        // 用 container 包裹内容，设置 Fill 尺寸和 alignment
        // Fill 是必要的 —— 如果容器是 Shrink（仅为内容大小），
        // alignment 没有可用空间去对齐
        container(self.content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(ax)
            .align_y(ay)
            .padding(Padding { top: py, right: pr, bottom: pb, left: px })
            .into()
    }
}

/// 将多个 Layered 叠加为一层
///
/// # iced stack 布局
/// `stack(elements)` 将所有子元素叠放在一起（类似 CSS position: absolute）。
/// - 第 0 个元素在最底层
/// - 后续元素依次叠加在上方
/// - 每个元素使用自己的 alignment + padding 定位
///
/// # 空列表处理
/// 当没有叠加层时（如端口扫描页面），返回一个不可见的占位元素，
/// 避免 stack 尺寸为 0 影响整体布局。
pub fn layer<'a, M: 'a>(children: Vec<Layered<'a, M>>) -> Element<'a, M> {
    if children.is_empty() {
        return iced::widget::text("").width(Length::Shrink).height(Length::Shrink).into();
    }
    // `into_iter().map(|c| c.into_element()).collect()` - 典型的 Rust 迭代器链
    // into_iter(): 消耗 Vec，获取所有权迭代器
    // map(): 将每个 Layered 转换为 Element
    // collect(): 收集为 Vec<Element>（编译器从 stack 的参数类型推断目标集合类型）
    let elements: Vec<Element<'a, M>> = children.into_iter().map(|c| c.into_element()).collect();
    stack(elements).into()
}
