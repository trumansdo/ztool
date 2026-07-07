//! Toast 消息通知组件
//!
//! 在屏幕上显示临时浮动通知，支持 4 种级别 × 2 个位置（右下/右上）的组合。
//! 基于 iced 原生 Widget + Overlay 实现，定时器融入 iced 更新循环，不依赖外部 tokio。
//!
//! ## 架构总览
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  Manager (Widget 透明包装层)         │
//! │  ├─ content: Element (宿主 UI)       │
//! │  ├─ toasts: Vec<Element> (Toast列表) │
//! │  └─ positions: Vec<ToastPosition>   │
//! │                                      │
//! │  overlay() → overlay::Group          │
//! │    ├─ content.overlay() (宿主覆盖层) │
//! │    └─ Overlay (Toast 浮动层)         │
//! │         ├─ layout: 两路堆叠位置计算  │
//! │         ├─ update: 定时器 + 局部Shell │
//! │         └─ draw:   渲染每个 Toast    │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## 核心设计
//!
//! - **透明委托**：Manager 的 layout/draw/update 全部透传给 content，自身仅作 wrapper
//! - **Overlay 浮动**：Toast 脱离文档流，不受宿主布局约束
//! - **内置定时器**：通过 `RedrawRequested` 事件检查超时，`request_redraw_at` 精确调度
//! - **局部 Shell**：每个 toast 子 widget 使用独立 local_shell 捕获事件，检测关闭点击

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::mouse;
use iced::time::{self, Duration, Instant};
use iced::widget::{button, container, row, text};
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size,
    Theme, Vector,
};
use iced::window;

pub const DEFAULT_TIMEOUT: u64 = 3;

/// 4 种消息级别，每种映射一个不同的边框颜色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastLevel {
    /// 蓝色调（默认）
    #[default]
    Info,
    /// 绿色调
    Success,
    /// 橙色调
    Warning,
    /// 红色调
    Error,
}

impl ToastLevel {
    /// 每种级别对应的主题色（用于左边框标识）
    fn border_color(&self) -> Color {
        // iced: Color::from_rgb() 用 0.0~1.0 浮点值创建 RGB 颜色
        match self {
            ToastLevel::Info => Color::from_rgb(0.35, 0.55, 0.85),
            ToastLevel::Success => Color::from_rgb(0.25, 0.70, 0.35),
            ToastLevel::Warning => Color::from_rgb(0.80, 0.65, 0.15),
            ToastLevel::Error => Color::from_rgb(0.80, 0.25, 0.25),
        }
    }
}

/// 2 种弹窗显示位置，默认为右下角。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    /// 右下角（默认），从下往上堆叠
    #[default]
    BottomRight,
    /// 右上角，从上往下堆叠
    TopRight,
}

/// 单条 Toast 消息数据模型。
///
/// 由外面传入 `Manager::new()`，`Manager` 内部将其转换为 `Element` 进行渲染。
#[derive(Debug, Clone)]
pub struct Toast {
    pub level: ToastLevel,
    pub text: String,
    pub position: ToastPosition,
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            level: ToastLevel::default(),
            text: String::new(),
            position: ToastPosition::default(),
        }
    }
}

/// Toast 管理器 —— **透明的包装层**，将 Toast 列表以 Overlay 形式浮在宿主 UI 之上。
///
/// ## 架构角色
/// - 包装一个 `content`（宿主 UI 元素），自身不参与布局渲染
/// - `layout()` / `draw()` / `update()` 等方法全部**委托**给 `content`
/// - 真正的 Toast 渲染在 `overlay()` 返回的 `Overlay` 层中完成
///
/// ## 状态管理
/// - 通过 `Tree.state` 维护 `Vec<Option<Instant>>`，每个 toast 对应一个创建时刻
/// - `diff()` 负责同步 toast 生命周期的增减
pub struct Manager<'a, Message> {
    /// 宿主 UI 元素（所有非 toast 的界面内容）
    // iced: Element<'a, Message> 是类型擦除的 Widget 包装，携带生命周期 'a 和消息类型 Message
    content: Element<'a, Message>,
    /// 预构建的 Toast Element 列表（每个包含 container + button + text）
    toasts: Vec<Element<'a, Message>>,
    /// 每个 toast 对应的位置类型
    positions: Vec<ToastPosition>,
    /// 超时秒数（默认 DEFAULT_TIMEOUT = 3）
    timeout_secs: u64,
    /// 关闭回调，参数为 toast 索引
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Manager<'a, Message>
where
    Message: 'a + Clone,
{
    /// 构造 Manager，同时将 `Toast` 数据预转为 `Element`。
    ///
    /// 预转换在 `new()` 中一次完成，避免每帧重复构建，每个 toast 包含：
    /// - 左侧文本（`text`），宽度 Fill 填充剩余空间
    /// - 右侧关闭按钮（`button("×")`），点击触发 `on_close(index)`
    /// - 深色背景 + 级别对应边框色的 `container`
    pub fn new(
        // iced: impl Into<Element> 允许任何可转换为 Element 的类型作为宿主内容
        content: impl Into<Element<'a, Message>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        // 提取每个 toast 的位置类型
        let positions: Vec<ToastPosition> = toasts.iter().map(|t| t.position).collect();

        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                let border_color = toast.level.border_color();

                // 关闭按钮：× 符号，无默认边框，hover/pressed 有深色背景
                // iced: button() 创建按钮 widget
                // iced: text() 创建文本 widget，.size() 设置字号
                let close_btn: Element<'a, Message> = button(text("×").size(9))
                    // iced: Button::on_press() 设置点击时发送的消息
                    .on_press((on_close)(index))
                    // iced: Button::padding() 设置内边距
                    .padding([0, 3])
                    // iced: Button::style() 根据 Theme 和 Status 动态设置样式
                    .style(|_: &Theme, status: button::Status| {
                        // iced: button::Status 枚举 Hovered/Pressed/Active
                        let bg = match status {
                            button::Status::Hovered => Color::from_rgb(0.3, 0.3, 0.35),
                            button::Status::Pressed => Color::from_rgb(0.4, 0.4, 0.45),
                            _ => Color::TRANSPARENT,  // iced: Color::TRANSPARENT 全透明颜色
                        };
                        // iced: button::Style 按钮的完整样式定义
                        button::Style {
                            // iced: Background::Color() 纯色背景
                            background: Some(Background::Color(bg)),
                            // iced: Border 定义边线颜色/宽度/圆角
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                // iced: .into() 转换 f32 到 Radius
                                radius: 3.0.into(),
                            },
                            // iced: Color::from_rgb() 从 RGB 浮点值创建颜色
                            text_color: Color::from_rgb(0.55, 0.55, 0.6),
                            ..Default::default()
                        }
                    })
                    .into();  // iced: .into() 将 Button 转换为 Element

                // Toast 容器：深色底 + 级别色左边框，固定 220px 宽度
                // iced: container() 创建容器 widget
                container(
                    // iced: row![] 宏创建水平布局行
                    row![
                        // iced: text() 文本 widget，.size(12) 字号，.width(Length::Fill) 填充剩余宽度
                        text(toast.text.as_str()).size(12).width(Length::Fill),
                        close_btn,
                    ]
                    // iced: Row::spacing() 设置子元素间距
                    .spacing(4)
                    // iced: Row::align_y() 垂直对齐方式
                    .align_y(Alignment::Center),
                )
                // iced: Container::padding() 设置内边距 [上下, 左右]
                .padding([6, 8])
                // iced: Container::width(Length::Fixed()) 固定宽度 220px
                .width(Length::Fixed(220.0))
                // iced: Container::style() 设置容器样式
                .style(move |_: &Theme| container::Style {
                    // iced: container::Style 容器样式定义
                    background: Some(Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()  // iced: .into() 将 Container 转换为 Element
            })
            .collect();

        Self {
            // iced: .into() 将宿主内容转换为 Element
            content: content.into(),
            toasts,
            positions,
            timeout_secs: DEFAULT_TIMEOUT,
            // iced: Box::new() 将闭包装箱，存入 trait object
            on_close: Box::new(on_close),
        }
    }

    pub fn timeout(self, seconds: u64) -> Self {
        Self {
            timeout_secs: seconds,
            ..self
        }
    }
}

/// ## Widget 实现 —— 透明委托模式
///
/// Manager 实现了 `Widget<Message, Theme, Renderer>` trait，但**自身不负责布局和绘制**。
/// 其 `layout()` / `draw()` / `update()` 等方法均直接**委托给 `self.content`（宿主 UI）**，
/// 通过 `self.content.as_widget_mut()` 调用。Manager 的角色是一个**透明的包装层**，
/// 真正的功能在 `overlay()` 返回值的 Toast Overlay 层中实现。
// iced: Widget<Message, Theme, Renderer> 是 iced 的自定义 widget trait，需实现 layout/draw/update/overlay 等方法
impl<Message> Widget<Message, Theme, Renderer> for Manager<'_, Message>
where
    Message: Clone,
{
    // iced: Widget::size() 返回 widget 的首选尺寸提示（用于 layout 阶段）
    fn size(&self) -> Size<Length> {
        // iced: Element::as_widget() 获取其内部的 Widget trait 引用
        // iced: Widget::size() 委托给宿主 content
        self.content.as_widget().size()
    }

    // iced: Widget::layout() 执行布局计算，接收 Tree/Renderer/Limits，返回 Node
    fn layout(
        &mut self,
        tree: &mut Tree,
        // iced: Renderer 是图形渲染后端的抽象
        renderer: &Renderer,
        // iced: layout::Limits 定义布局约束（最小/最大尺寸）
        limits: &layout::Limits,
    // iced: layout::Node 是布局结果节点，包含位置/尺寸/子节点
    ) -> layout::Node {
        // 委托给宿主 content
        // iced: Element::as_widget_mut() 获取其内部 Widget 的可变引用
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    // iced: Widget::tag() 返回 widget::tree::Tag 类型标签，供 iced 树 diff 时识别 widget 类型
    fn tag(&self) -> widget::tree::Tag {
        struct Marker;
        // iced: widget::tree::Tag::of::<T>() 创建 T 类型的唯一标签
        widget::tree::Tag::of::<Marker>()
    }

    // iced: Widget::state() 分配 widget 的私有状态存储
    fn state(&self) -> widget::tree::State {
        // iced: widget::tree::State::new() 用任意类型创建初始状态
        // 此处存储 Vec<Option<Instant>>，每个 toast 对应一个创建时刻
        widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    // iced: Widget::children() 返回子树列表，iced 据此构建状态树
    fn children(&self) -> Vec<Tree> {
        // iced: Tree::new(&element) 从 Element 创建对应的状态树节点
        // 子树结构: [content_tree, toast0_tree, toast1_tree, ...]
        std::iter::once(Tree::new(&self.content))
            .chain(self.toasts.iter().map(Tree::new))
            .collect()
    }

    // iced: Widget::diff() 在每次 UI 更新时同步 Tree 状态
    fn diff(&self, tree: &mut Tree) {
        // iced: Tree::state.downcast_mut::<T>() 将 state 下转型为具体类型 T 的可变引用
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();
        // 移除已过期/已关闭的 toast 状态
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            (old, new) if old > new => {
                // toast 被删除了，截断状态列表
                instants.truncate(new);
            }
            (old, new) if old < new => {
                // 新 toast 加入，记录当前时刻
                instants.extend(std::iter::repeat_n(Some(Instant::now()), new - old));
            }
            _ => {}
        }

        // 同步子树结构
        // iced: Tree::diff_children() 对比新旧子树列表，增删对应的 Tree 节点
        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toasts.iter())
                .collect::<Vec<_>>(),
        );
    }

    // iced: Widget::operate() 遍历访问子 widget（用于焦点导航/可访问性）
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        // iced: Renderer 是图形渲染后端的抽象
        renderer: &Renderer,
        // iced: Operation 是操作访问器 trait
        operation: &mut dyn Operation,
    ) {
        // 委托给宿主 content
        // iced: Operation::container() 标记当前容器边界
        operation.container(None, layout.bounds());
        // iced: Operation::traverse() 遍历所有子 widget
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    // iced: Widget::update() 处理事件（键盘/鼠标/窗口等）
    fn update(
        &mut self,
        tree: &mut Tree,
        // iced: Event 是 iced 的事件枚举
        event: &Event,
        // iced: Layout 布局节点引用
        layout: Layout<'_>,
        // iced: Cursor 鼠标位置信息
        cursor: mouse::Cursor,
        // iced: Renderer 渲染后端
        renderer: &Renderer,
        // iced: Clipboard 系统剪贴板接口
        _clipboard: &mut dyn Clipboard,
        // iced: Shell 消息/动作总线
        shell: &mut Shell<'_, Message>,
        // iced: Rectangle 视口矩形
        viewport: &Rectangle,
    ) {
        // 委托给宿主 content
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            _clipboard,
            shell,
            viewport,
        );
    }

    // iced: Widget::draw() 执行绘制
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        // iced: renderer::Style 包含文本颜色等渲染参数
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        // 委托给宿主 content
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    // iced: Widget::mouse_interaction() 返回鼠标交互类型（决定光标形状）
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    // iced: mouse::Interaction 枚举（Idle/Pointing/Text/Resizing 等）
    ) -> mouse::Interaction {
        // 委托给宿主 content
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    // iced: Widget::overlay() 返回 overlay::Element，使 widget 可以在边界外渲染浮动内容
    fn overlay<'b>(
        &'b mut self,
        // iced: Tree 是 widget 的状态树
        tree: &'b mut Tree,
        // iced: Layout 布局节点引用
        layout: Layout<'b>,
        // iced: Renderer 渲染后端
        renderer: &Renderer,
        // iced: Rectangle 视口边界
        viewport: &Rectangle,
        // iced: Vector 是 2D 偏移向量
        translation: Vector,
    // iced: overlay::Element 是 overlay 的不透明包装
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // iced: Tree::state.downcast_mut() 从 Tree 状态中取出 Vec<Option<Instant>>
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

        // iced: Tree::children.split_at_mut() 将子树分为 [content] 和 [toasts...]
        let (content_state, toasts_state) = tree.children.split_at_mut(1);

        // 1. 宿主自身的 overlay（如果有）
        // iced: Element::as_widget_mut().overlay() 获取宿主自身的 overlay
        let content = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        // 2. Toast Overlay：toasts 非空时才创建
        let toasts = (!self.toasts.is_empty()).then(|| {
            // iced: overlay::Element::new(Box::new(overlay)) 将 Overlay trait 对象包装为 Element
            overlay::Element::new(Box::new(Overlay {
                // iced: Layout::bounds() 返回节点边界矩形
                // iced: Rectangle::position() 返回左上角坐标
                position: layout.bounds().position() + translation,
                viewport: *viewport,
                toasts: &mut self.toasts,
                positions: &self.positions,
                trees: toasts_state,
                instants,
                on_close: &self.on_close,
                timeout_secs: self.timeout_secs,
            }))
        });

        // 合并两个 overlay 层
        let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

        // iced: overlay::Group::with_children() 创建组合 overlay
        // iced: overlay::Group::overlay() 转换为 overlay::Element
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

/// Overlay 层：真正的 Toast 浮动渲染层。
///
/// 持有 Manager 中 toast 相关数据的 &mut 引用，在 overlay 层级完成：
/// - 位置计算（两路堆叠）
/// - 定时器倒计时（融入 iced 事件循环）
/// - 用户交互（关闭按钮点击检测）
///
/// ## 生命周期关系
/// Manager 通过 `overlay()` 方法创建 Overlay，Overlay 的 &mut 引用直接指向
/// Manager 中的 toasts/trees/instants，因此两者共享同一份状态。
// iced: overlay::Overlay<Message, Theme, Renderer> 是实现浮动层的 trait（此处未标注，在 impl 行）
struct Overlay<'a, 'b, Message> {
    /// 宿主布局左上角 + translation 偏移后的锚点
    // iced: Point 是 2D 坐标点 (x, y)
    position: Point,
    /// 视口矩形，用于约束 toast 的 x 坐标
    // iced: Rectangle 是矩形区域 (x, y, width, height)
    viewport: Rectangle,
    /// Toast Element 列表（直接从 Manager 透传）
    // iced: Element 是类型擦除的 Widget 包装，携带生命周期 'a 和消息类型 Message
    toasts: &'b mut [Element<'a, Message>],
    /// 每个 toast 的位置类型（TopRight / BottomRight）
    positions: &'b [ToastPosition],
    /// 每个 toast 对应的子树状态
    // iced: Tree 是 widget 的递归状态树，与 Element 列表一一对应
    trees: &'b mut [Tree],
    /// 每个 toast 的创建时刻，None 表示已过期或已关闭
    // iced: Instant 是单调时钟上的时间点，用于计算经过时间
    instants: &'b mut [Option<Instant>],
    /// 关闭回调，参数为 toast 在列表中的索引
    on_close: &'b dyn Fn(usize) -> Message,
    /// 超时秒数（默认 3s）
    timeout_secs: u64,
}

impl<Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, '_, Message> {
    /// 两路堆叠布局：TopRight 从上往下排，BottomRight 从下往上排。
    ///
    /// ## 位置计算
    /// - **右对齐**：`x = viewport.right - width - 12px` 边距
    /// - **TopRight 堆叠**：y 从锚点向下累加 `Σ(prev_height + 8px间隙)`
    /// - **BottomRight 堆叠**：y 从视口底部向上累减 `bounds.height - Σ(prev_height + 8px) - 自身高度`
    ///
    /// 两路分别收集到 `top_nodes` / `bottom_nodes`，最后合并为一个 Node。
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        // iced: Size 是二维宽高结构体
        // iced: layout::Limits 定义布局的最小/最大尺寸约束
        // iced: layout::Limits::new(min, max) 创建指定范围的布局约束
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let mut top_nodes = Vec::new();
        let mut bottom_nodes = Vec::new();

        for (i, toast) in self.toasts.iter_mut().enumerate() {
            // iced: Widget::layout() 对子 widget 执行布局计算，返回 layout::Node（含尺寸和子节点）
            let node = toast.as_widget_mut().layout(&mut self.trees[i], renderer, &limits);
            // iced: Node::bounds() 返回节点的 Rectangle 边界矩形
            // iced: Rectangle::size() 提取宽高
            let size = node.bounds().size();

            // 所有 toast 右对齐，距右边界 12px
            let x = self.viewport.x + self.viewport.width - size.width - 12.0;
            // 根据位置类型计算 y 偏移（堆叠）
            let y_offset = match self.positions.get(i).copied().unwrap_or_default() {
                ToastPosition::TopRight => {
                    let prev_height: f32 = top_nodes.iter().map(|n: &layout::Node| n.bounds().height + 8.0).sum::<f32>();
                    self.position.y + 8.0 + prev_height
                }
                ToastPosition::BottomRight => {
                    let prev_height: f32 = bottom_nodes.iter().map(|n: &layout::Node| n.bounds().height + 8.0).sum::<f32>();
                    self.position.y + bounds.height - size.height - 8.0 - prev_height
                }
            };

            // iced: Node::move_to() 将布局节点移动到指定位置（修改其 bounds）
            let positioned = node.move_to(Point::new(x, y_offset));
            match self.positions.get(i).copied().unwrap_or_default() {
                ToastPosition::TopRight => top_nodes.push(positioned),
                ToastPosition::BottomRight => bottom_nodes.push(positioned),
            }
        }

        let mut children = top_nodes;
        children.extend(bottom_nodes);
        // iced: layout::Node::with_children() 用边界矩形和子节点列表创建父布局节点
        layout::Node::with_children(bounds, children)
    }

    /// ### 定时器机制（上半部分）
    ///
    /// 不依赖 `tokio` 或 `setTimeout`，完全融入 iced 帧事件循环：
    /// 1. 监听 `RedrawRequested(now)` 事件 → 遍历每个 toast 的创建时刻
    /// 2. `remaining = timeout - elapsed`，若归零则 `publish(on_close)` 并标记 `None`
    /// 3. 否则 `request_redraw_at(now + remaining)` 精确调度下一帧
    ///
    /// ### 交互处理（下半部分）
    ///
    /// 对每个 toast element 创建**局部 Shell** 捕获子 widget 事件：
    /// - 若关闭按钮被点击 → `local_shell` 非空 → `instant.take()` 取消定时器
    /// - 然后 `shell.merge(local_shell)` 将消息向上传播到 Manager
    fn update(
        &mut self,
        // iced: Event 是 iced 的事件枚举（键盘/鼠标/窗口/触摸/定时器等）
        event: &Event,
        // iced: Layout 是布局节点的引用
        layout: Layout<'_>,
        // iced: Cursor 封装鼠标位置信息
        cursor: mouse::Cursor,
        // iced: Renderer 是图形渲染后端的抽象
        renderer: &Renderer,
        // iced: Clipboard 是系统剪贴板访问接口
        _clipboard: &mut dyn Clipboard,
        // iced: Shell 是消息/动作总线，publish() 发消息，merge() 合并子 Shell，request_redraw_at() 调度重绘
        shell: &mut Shell<'_, Message>,
    ) {
        // --- 定时器：RedrawRequested 事件驱动超时检查 ---
        // iced: RedrawRequested(now) 每帧渲染前触发，now 为当前 Instant，是 iced 内置定时器的基础
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            self.instants
                .iter_mut()
                .enumerate()
                .for_each(|(index, maybe_instant)| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        // iced: time::seconds() 将 u64 转为 Duration
                        // iced: Instant::elapsed() 返回从该时刻到现在的 Duration
                        let remaining =
                            time::seconds(self.timeout_secs)
                                .saturating_sub(instant.elapsed());

                        // iced: Duration::ZERO 是零时长常量
                        if remaining == Duration::ZERO {
                            // 超时：标记为已过期，发送关闭消息
                            maybe_instant.take();
                            // iced: Shell::publish() 向 widget 树发送一条消息
                            shell.publish((self.on_close)(index));
                        } else {
                            // 未超时：精确调度下一次检查
                            // iced: Shell::request_redraw_at() 在指定 Instant 触发 RedrawRequested
                            shell.request_redraw_at(*now + remaining);
                        }
                    }
                });
        }

        // --- 交互：局部 Shell 捕获 toast 内部事件 ---
        // iced: Layout::bounds() 返回节点的 Rectangle 边界
        let viewport = layout.bounds();

        for (((child, state), child_layout), instant) in self
            .toasts
            .iter_mut()
            .zip(self.trees.iter_mut())
            // iced: Layout::children() 遍历所有子布局节点
            .zip(layout.children())
            .zip(self.instants.iter_mut())
        {
            let mut local_messages = vec![];
            // iced: Shell::new() 创建局部 Shell，捕获子 widget 产生的消息但不立即执行
            let mut local_shell = Shell::new(&mut local_messages);
            // iced: clipboard::Null 是空剪贴板实现，防止子 widget 访问系统剪贴板
            let mut local_clipboard = iced::advanced::clipboard::Null;

            // 将事件转发给子 widget（button/container）处理
            child.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                &mut local_clipboard,
                &mut local_shell,
                &viewport,
            );

            // 若子 widget 产生消息（关闭按钮被点击），取消该 toast 的定时器
            if !local_shell.is_empty() {
                instant.take();
            }

            // 将局部消息合并到外层 Shell，驱动 UI 更新
            // iced: Shell::merge() 将子 Shell 的消息合并到父 Shell
            shell.merge(local_shell, std::convert::identity);
        }
    }

    fn draw(
        &self,
        // iced: Renderer 是图形渲染后端，提供 draw_xxx 绘制原语
        renderer: &mut Renderer,
        // iced: Theme 是 iced 的主题系统，提供颜色/圆角/间距等设计 tokens
        theme: &Theme,
        // iced: renderer::Style 包含文本颜色等渲染参数
        style: &renderer::Style,
        // iced: Layout 布局节点引用，包含绝对位置和尺寸
        layout: Layout<'_>,
        // iced: Cursor 封装鼠标位置信息
        cursor: mouse::Cursor,
    ) {
        // iced: Layout::bounds() 返回节点的 Rectangle 边界
        let viewport = layout.bounds();

        for ((child, tree), layout) in self
            .toasts
            .iter()
            .zip(self.trees.iter())
            // iced: Layout::children() 迭代所有子布局节点
            .zip(layout.children())
        {
            child
                .as_widget()
                // iced: Widget::draw() 接收 Tree + Renderer + Theme + Style + Layout + Cursor 进行绘制
                .draw(tree, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    fn operate(
        &mut self,
        // iced: Layout 是布局节点的引用
        layout: Layout<'_>,
        // iced: Renderer 是图形渲染后端的抽象
        renderer: &Renderer,
        // iced: Operation 是用于焦点导航/可访问性遍历的访问器模式 trait
        operation: &mut dyn widget::Operation,
    ) {
        // iced: Operation::container() 标记当前容器在操作树中的边界
        operation.container(None, layout.bounds());
        // iced: Operation::traverse() 对容器内所有子 widget 执行遍历访问
        operation.traverse(&mut |operation| {
            self.toasts
                .iter_mut()
                .zip(self.trees.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child.as_widget_mut().operate(state, layout, renderer, operation);
                });
        });
    }

    fn mouse_interaction(
        &self,
        // iced: Layout 是布局节点的引用，包含位置和尺寸
        layout: Layout<'_>,
        // iced: Cursor 封装鼠标位置信息，提供 is_over() 命中检测
        cursor: mouse::Cursor,
        // iced: Renderer 是图形渲染后端的抽象
        renderer: &Renderer,
    // iced: mouse::Interaction 枚举（None/Idle/Pointing/Text/Grabbing/Resizing 等）
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.trees.iter())
            // iced: Layout::children() 遍历所有子布局节点
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, &self.viewport, renderer)
                    // iced: .max() 取两个 Interaction 中优先级更高的（Pointing > Idle > None）
                    .max(if cursor.is_over(layout.bounds()) {
                        // iced: Cursor::is_over() 判断鼠标是否在给定矩形区域内
                        mouse::Interaction::Idle
                    } else {
                        // iced: Default::default() 返回 mouse::Interaction 默认值（None）
                        Default::default()
                    })
            })
            .max()
            .unwrap_or_default()
    }
}

/// 将 Manager 转换为 Element，使其可作为 iced 组件树的一部分使用。
impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(manager: Manager<'a, Message>) -> Self {
        // iced: Element::new() 将自定义 Widget 包装为不透明 Element，可插入组件树
        Element::new(manager)
    }
}
