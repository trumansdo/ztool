//! 菜单栏组件
//!
//! 提供顶部下拉菜单的构建和渲染。当前代码在 GUI 中未实际使用
//! （菜单栏的构建和渲染函数未被调用），但保留了完整的实现供将来启用。
//!
//! # Rust 语法要点
//!
//! ## 泛型结构体
//! `pub struct MenuItem<M>` —— 菜单项可携带任意消息类型 M。
//! 这使得同一个菜单组件可以被不同应用复用。
//!
//! ## 枚举的泛型参数
//! `pub enum MenuEntry<M>` 枚举携带泛型 M：
//! - `SubMenu(MenuItem<M>)` —— 子菜单
//! - `Divider` —— 分隔线（无数据）
//! - `Item { label, on_press: M }` —— 可点击菜单项
//!
//! ## 返回 `impl Trait` 类型
//! `fn bar_btn_style(...) -> impl Fn(&Theme, Status) -> BtnStyle`
//! 返回"某个实现了 Fn trait 的类型"，具体类型由编译器推断。
//! 调用者不需要知道具体类型，只要知道它实现了 Fn 即可。
//! 这是 Rust 的"返回位置 impl Trait" (RPIT) 特性。
//!
//! ## `Clone + 'static` 约束
//! `M: Clone + 'static` 是 iced 的常见要求：
//! - `Clone`: 消息在事件循环中可能需要被复制
//! - `'static`: 消息不能借用临时数据，必须拥有所有权或静态数据

use iced::{
    widget::{button, column, container, text},
    Background, Border, Color, Element, Length, Padding, Shadow, Theme,
};
use iced::border::Radius;
use iced::widget::button::Style as BtnStyle;
use iced::widget::container::Style as ContainerStyle;
use iced::widget::button::Status;

use super::overlay::{Anchor, Layered};

/// 菜单项定义
///
/// 一个菜单项可以有子菜单项（下拉菜单）。
/// 当前所有方法均未被调用（dead code），保留了完整的实现。
#[derive(Debug, Clone)]
pub struct MenuItem<M> {
    label: &'static str,
    items: Vec<MenuEntry<M>>,
}

impl<M> MenuItem<M> {
    /// 创建新的菜单项
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            items: Vec::new(),
        }
    }

    /// 添加子菜单项
    pub fn item(mut self, label: &'static str, on_press: M) -> Self {
        self.items.push(MenuEntry::Item { label, on_press });
        self
    }

    /// 获取子菜单条目列表（用于下拉菜单渲染）
    pub fn entries(&self) -> &[MenuEntry<M>] {
        &self.items
    }
}

/// 菜单条目枚举
///
/// 下拉菜单中的每一项可以是：
/// - 普通菜单项（可点击，携带消息）
/// - 分隔线
#[derive(Debug, Clone)]
pub enum MenuEntry<M> {
    /// 可点击的菜单项
    Item {
        label: &'static str,
        on_press: M,
    },
    /// 分隔线
    Divider,
}

/// 菜单栏按钮样式
///
/// 下拉菜单关闭时呈暗色，展开时高亮指示当前激活的菜单。
///
/// # Rust: 返回 `impl Trait`
/// `impl Fn(&Theme, Status) -> BtnStyle` 作为返回类型，
/// 调用者只关心返回的是一个可以调用的函数，不关心具体 closure 类型。
fn bar_btn_style(is_open: bool) -> impl Fn(&Theme, Status) -> BtnStyle {
    move |_theme: &Theme, status: Status| {
        let (bg, border_color) = if is_open {
            // 展开状态：更亮的背景 + 底部高亮边框
            (Color::from_rgb(0.25, 0.25, 0.3), Color::from_rgb(0.4, 0.5, 0.6))
        } else {
            // 收起状态：透明背景
            match status {
                Status::Hovered => (Color::from_rgb(0.22, 0.22, 0.28), Color::TRANSPARENT),
                _ => (Color::TRANSPARENT, Color::TRANSPARENT),
            }
        };
        BtnStyle {
            background: Some(Background::Color(bg)),
            text_color: Color::from_rgb(0.85, 0.85, 0.85),
            border: Border {
                color: border_color,
                width: 2.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            snap: Default::default(),
        }
    }
}

/// 下拉菜单项样式
///
/// hover 时高亮显示，提供触摸/鼠标反馈。
fn item_btn_style() -> impl Fn(&Theme, Status) -> BtnStyle {
    |_theme: &Theme, status: Status| {
        let bg = match status {
            Status::Hovered => Color::from_rgb(0.25, 0.25, 0.3),
            _ => Color::TRANSPARENT,
        };
        BtnStyle {
            background: Some(Background::Color(bg)),
            text_color: Color::from_rgb(0.85, 0.85, 0.85),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
            snap: Default::default(),
        }
    }
}

/// 渲染顶部菜单栏
///
/// 水平排列所有菜单项按钮，点击后展开对应的下拉菜单。
///
/// # Rust: 迭代器链
/// `.iter().enumerate().map(|(i, item)| { ... }).collect()`
/// - `.iter()`: 不可变借用迭代器
/// - `.enumerate()`: 为每个元素添加索引 `(usize, &T)`
/// - `.map(...)`: 转换每个元素
/// - `.collect()`: 收集到目标集合（这里是 Row）
pub fn menu_bar<M: Clone + 'static>(
    items: &[MenuItem<M>],
    open_index: Option<usize>,
    on_toggle: impl Fn(Option<usize>) -> M + Clone + 'static,
) -> Element<'static, M> {
    let buttons: Vec<Element<'_, M>> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_open = open_index == Some(i);
            let on_toggle = on_toggle.clone();
            button(text(item.label))
                .on_press(on_toggle(Some(i)))
                .padding([4, 12])
                .style(bar_btn_style(is_open))
                .into()
        })
        .collect();

    container(iced::widget::row(buttons))
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.12, 0.12, 0.16))),
            border: Border {
                color: Color::from_rgb(0.2, 0.2, 0.25),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

/// 渲染下拉菜单
///
/// 将菜单项渲染为一个垂直列表，浮动在菜单栏下方。
/// 返回 Layered 叠加元素，通过锚点定位到菜单栏下方。
///
/// # Rust: pattern matching in match
/// 匹配 MenuEntry 的不同变体：
/// - `MenuEntry::Item { label, on_press }` 解构出 label 和 on_press
/// - `MenuEntry::Divider` 简单匹配，无数据提取
pub fn dropdown<M: Clone + 'static>(
    items: &[MenuEntry<M>],
    _index: usize,
    left: f32,
) -> Layered<'static, M> {
    let dropdown_items: Vec<Element<'_, M>> = items
        .iter()
        .map(|entry| match entry {
            MenuEntry::Item { label, on_press } => {
                button(text(*label))
                    .on_press(on_press.clone())
                    .padding([6, 16])
                    .style(item_btn_style())
                    .width(Length::Fill)
                    .into()
            }
            MenuEntry::Divider => {
                container(iced::widget::rule::horizontal(1))
                    .padding(Padding::from([4, 8]))
                    .into()
            }
        })
        .collect();

    // 渲染下拉菜单容器
    let body = container(column(dropdown_items))
        .width(Length::Fixed(160.0))
        .padding(Padding::from([4, 0]))
        .style(|_theme: &Theme| ContainerStyle {
            background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.2))),
            border: Border {
                color: Color::from_rgb(0.3, 0.3, 0.35),
                width: 1.0,
                radius: Radius::from(4.0),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: [0.0, 4.0].into(),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .into();

    // 用 Anchor 定位：顶部 30px（菜单栏高度）+ 左侧偏移
    Layered::new(body).anchor(Anchor {
        top: Some(30.0),
        left: Some(left),
        ..Anchor::default()
    })
}
