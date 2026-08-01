//! # 菜单栏核心组件层
//!
//! 对应 libcosmic 菜单系统架构中的**核心组件层**，实现 `iced::Widget` trait，
//! 作为菜单系统的宿主组件。
//!
//! ## 架构职责
//!
//! ```text
//! MenuBar (Widget trait)
//!   ├── 拥有静态菜单数据 Vec<MenuTree>          ← 数据模型层
//!   ├── 通过 tag()/state() 挂载 MenuBarState     ← 状态控制层
//!   ├── layout/draw/update 处理根菜单项          ← 渲染表现层（根层）
//!   └── overlay() 返回 Menu Overlay              ← 渲染表现层（弹出层）
//! ```
//!
//! ## Overlay 构造机制
//!
//! `overlay()` 方法实现视图分离：
//! 1. 查询 `tree.state` 中的 `MenuBarState`，未打开则返回 `None`
//! 2. 通过 `layout.children()` 正向推导各根菜单项边界（零全局查询开销）
//! 3. 构造 `Menu` Overlay（`depth = 0`），传递父级 Layout 信息作为定位基准
//!
//! # Rust 语法要点
//!
//! ## 泛型消息 `Message`
//! `MenuBar<Message>` 的消息类型直接使用应用消息（本项目为 `UiLibs::Msg`），
//! 菜单项点击时由 Overlay 直接 `shell.publish(action)` 发出，无需消息映射层。
//!
//! ## `Widget::overlay` 生命周期
//! ```text
//! fn overlay<'b>(&'b mut self, tree: &'b mut Tree, ...)
//!     -> Option<overlay::Element<'b, Message, Theme, Renderer>>
//! ```
//! 所有借用共享 `'b`：overlay 与 widget 同生命周期，保证状态（`tree.state`）
//! 与数据（`self.menu_roots`）在 Overlay 存活期间均有效。

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::renderer::Quad;
// 引入渲染器 trait：`fill_quad`/`fill_text` 均为 trait 方法
use iced::advanced::renderer::Renderer as _;
use iced::advanced::text::Renderer as _;
use iced::advanced::widget::{self, Tree};
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

use super::menu_inner::{Menu, MenuBarState, ITEM_HEIGHT};
use super::menu_tree::MenuTree;

/// 根菜单项水平间距
const ROOT_SPACING: f32 = 4.0;
/// 根菜单项内边距
const ROOT_PADDING_X: f32 = 10.0;
/// 根菜单项字号
const ROOT_FONT_SIZE: f32 = 14.0;
/// 根菜单项 hover 高亮色（与弹出菜单一致）
const ROOT_HOVER: Color = Color::from_rgb8(40, 80, 120);

/// 菜单栏组件
///
/// 拥有静态菜单数据（`Vec<MenuTree>`），通过 `iced::Widget` trait
/// 嵌入 UI 树，点击根菜单项时经 `overlay()` 弹出下拉菜单。
pub struct MenuBar<Message> {
    /// 静态菜单数据（拥有所有权，避免借用生命周期问题）
    menu_roots: Vec<MenuTree<'static, Message>>,
    /// 根菜单项高度
    item_height: f32,
    /// 根菜单项边界缓存（overlay() 中填充，供 Menu 定位）
    root_bounds: Vec<Rectangle>,
}

impl<Message> MenuBar<Message> {
    /// 创建菜单栏
    ///
    /// # 参数
    /// - `menu_roots`: 根级菜单树列表（`MenuTree::folder` 构建子菜单）
    ///
    /// # 所有权设计
    /// `MenuBar` 拥有菜单数据（而非借用），这使得菜单数据可整体以 `'static`
    /// 生命周期构建，返回的 `Element<'static>` 可安全地嵌入任意视图。
    pub fn new(menu_roots: Vec<MenuTree<'static, Message>>) -> Self {
        Self {
            menu_roots,
            item_height: ITEM_HEIGHT,
            root_bounds: Vec::new(),
        }
    }

    /// 设置根菜单项高度（链式构建器）
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height;
        self
    }

    /// 测量根菜单项文本宽度（bounds 为可用布局区域，有限值）
    fn root_label_width(renderer: &Renderer, label: &str, bounds: Size) -> f32 {
        // 复用 menu_inner 的文本测量工具（保持字号一致）
        super::menu_inner::measure_text_width(renderer, label, ROOT_FONT_SIZE, bounds)
    }
}

impl<Message> MenuBar<Message>
where
    Message: Clone,
{
    /// 构建根菜单项布局：每个根项宽度自适应文本，横向排列
    fn layout_roots(&self, renderer: &Renderer, limits: &layout::Limits) -> (layout::Node, Vec<f32>) {
        let mut children = Vec::with_capacity(self.menu_roots.len());
        let mut widths = Vec::with_capacity(self.menu_roots.len());
        let mut x = 0.0;
        // 文本测量边界：当前可用空间（对齐 iced 官方 limits.max() 范式）
        let measure_bounds = limits.max();

        for item in &self.menu_roots {
            let label = item.label().unwrap_or("");
            let width = Self::root_label_width(renderer, label, measure_bounds) + 2.0 * ROOT_PADDING_X;
            children.push(
                layout::Node::new(Size::new(width, self.item_height))
                    .move_to(Point::new(x, 0.0)),
            );
            widths.push(width);
            x += width + ROOT_SPACING;
        }

        let total_width = if children.is_empty() {
            0.0
        } else {
            x - ROOT_SPACING
        };
        let node = layout::Node::with_children(Size::new(total_width, self.item_height), children);
        (node, widths)
    }

    /// 命中测试：返回光标所在根菜单项索引
    fn hit_test_roots(layout: Layout<'_>, position: Point) -> Option<usize> {
        layout
            .children()
            .enumerate()
            .find(|(_, child)| child.bounds().contains(position))
            .map(|(i, _)| i)
    }
}

impl<Message> widget::Widget<Message, Theme, Renderer> for MenuBar<Message>
where
    Message: Clone,
{
    /// 返回 Widget 的尺寸定义
    ///
    /// 根菜单宽度需在 `layout()` 中测量（依赖 renderer），此处返回
    /// `Shrink`（收缩到内容）与固定高度。
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Fixed(self.item_height))
    }

    /// 计算布局：根菜单项横向排列，宽度按文本自适应
    fn layout(&mut self, _tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.layout_roots(renderer, limits).0
    }

    /// 返回该 Widget 的类型标签（iced 状态系统标识）
    fn tag(&self) -> widget::tree::Tag {
        struct Marker;
        widget::tree::Tag::of::<Marker>()
    }

    /// 初始化 Widget 运行时状态：`MenuBarState`（默认关闭）
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(MenuBarState::default())
    }

    /// 无子 Widget，返回空
    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    /// 绘制根菜单项
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuBarState>();

        // 菜单打开时高亮激活根项，否则高亮悬停根项
        let highlight = if state.is_open() {
            state.active_path.first().copied()
        } else {
            state.root_hovered
        };

        for (i, (item, child)) in self.menu_roots.iter().zip(layout.children()).enumerate() {
            let bounds = child.bounds();

            // hover / 激活高亮
            if highlight == Some(i) {
                let highlight_rect = Rectangle {
                    x: bounds.x + 1.0,
                    y: bounds.y + 1.0,
                    width: bounds.width - 2.0,
                    height: bounds.height - 2.0,
                };
                renderer.fill_quad(
                    Quad {
                        bounds: highlight_rect,
                        border: Border {
                            radius: 3.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ROOT_HOVER,
                );
            }

            // 根菜单项文本（fill_text 自带文本生命周期管理，clip 用 viewport）
            if let Some(label) = item.label() {
                renderer.fill_text(
                    super::menu_inner::make_text(
                        renderer,
                        label.to_string(),
                        ROOT_FONT_SIZE,
                        Size::new(bounds.width, self.item_height),
                    ),
                    Point::new(bounds.x + ROOT_PADDING_X, bounds.y + (self.item_height - ROOT_FONT_SIZE) / 2.0),
                    Color::WHITE,
                    *viewport,
                );
            }
        }
    }

    /// 处理根菜单项事件
    ///
    /// ## 事件语义
    /// - `CursorMoved`：更新 `root_hovered`；菜单打开时 hover 其他根项 → 切换激活根项
    /// - `ButtonPressed(Left)`：
    ///   - 未打开 & 点击根项 → 打开菜单并激活该根项（**Captured**，overlay 不再处理）
    ///   - 已打开 & 点击激活根项 → 关闭（toggle）
    ///   - 已打开 & 点击其他根项 → 切换激活根项
    ///   - 未点击根项 → **Ignored**（交给 overlay 处理"点击外部关闭"）
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<MenuBarState>();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let hovered = Self::hit_test_roots(layout, *position);
                if state.root_hovered != hovered {
                    state.root_hovered = hovered;
                    shell.request_redraw();
                }

                // 菜单打开时：hover 到其他根项 → 即时切换激活根项（Windows 风格）
                if state.is_open()
                    && let Some(i) = hovered
                    && state.active_path.first() != Some(&i)
                {
                    state.active_path = vec![i];
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(pos) = cursor.position() else {
                    return;
                };
                let clicked = Self::hit_test_roots(layout, pos);

                match (state.is_open(), clicked) {
                    // 未打开 & 点击根项 → 打开
                    (false, Some(i)) => {
                        state.open();
                        state.active_path = vec![i];
                        shell.request_redraw();
                    }
                    // 已打开 & 点击激活根项 → 关闭（toggle）
                    (true, Some(i)) if state.active_path.first() == Some(&i) => {
                        state.close();
                        shell.request_redraw();
                    }
                    // 已打开 & 点击其他根项 → 切换
                    (true, Some(i)) => {
                        state.active_path = vec![i];
                        shell.request_redraw();
                    }
                    // 未点击根项 → 交给 overlay 的“点击外部关闭”逻辑处理
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// 返回浮动 Overlay —— 菜单系统的核心调度点
    ///
    /// ## 构造流程
    /// 1. 状态查询：`tree.state` 下转型为 `MenuBarState`，未打开则返回 `None`
    /// 2. 坐标正向推导：`layout.children()` 收集各根菜单项边界（`root_bounds_list`）
    /// 3. 构造 `Menu`（`depth = 0`），将父级 Layout 信息作为定位基准
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<MenuBarState>();
        if !state.is_open() {
            return None;
        }

        // 取激活根菜单项的 children 作为下拉菜单内容（数据下钻）
        let &root_index = state.active_path.first()?;
        let root_items = match self.menu_roots.get(root_index) {
            Some(MenuTree::Folder(folder)) => folder.children.as_slice(),
            // 根项为 Item/Separator 时无下拉菜单
            _ => return None,
        };

        // 收集各根菜单项边界（Layout 正向传递，零全局查询）
        // 缓存到 self.root_bounds 以解决局部变量借用生命周期问题
        self.root_bounds = layout.children().map(|child| child.bounds()).collect();

        Some(overlay::Element::new(Box::new(Menu {
            state,
            items: root_items,
            depth: 0,
            bar_bounds: layout.bounds(),
            root_bounds_list: &self.root_bounds,
            translation,
            parent_bounds: None,
        })))
    }

    /// 鼠标交互：悬停根菜单项时显示手型指针
    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position()
            && Self::hit_test_roots(layout, pos).is_some()
        {
            return mouse::Interaction::Pointer;
        }
        mouse::Interaction::Idle
    }
}

/// 允许 `MenuBar` 直接转换为 `Element`
impl<Message> From<MenuBar<Message>> for Element<'static, Message, Theme, Renderer>
where
    Message: 'static + Clone,
{
    fn from(menu_bar: MenuBar<Message>) -> Self {
        Element::new(menu_bar)
    }
}
