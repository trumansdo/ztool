//! Toast 消息通知组件
//!
//! ## 实现原理
//!
//! 本组件实现了一个轻量级的浮动 Toast 通知系统，完全基于 iced 原生 Widget + Overlay 机制，
//! 不依赖外部异步运行时（如 tokio）。核心思路如下：
//!
//! ### 架构层次
//!
//! ```
//! Manager (Widget trait)               ← 作为普通 Widget 嵌入 UI 树
//!   ├── content: 底层主内容（透传的子 Widget，如整个应用程序页面）
//!   └── overlay() → Overlay (Overlay trait) ← 浮动层，绘制 Toast 并在 RedrawRequested 事件中管理倒计时
//!         ├── 每个 Toast = 一个 container(row[text + close_btn])
//!         ├── 位置计算：根据 ToastPosition 沿右边界从顶部/底部堆叠
//!         └── 定时关闭：利用 iced 的 RedrawRequested 事件 + Instant 做倒计时
//! ```
//!
//! ### 定时器机制（核心设计点）
//!
//! 不依赖 tokio::time，而是**融入 iced 原生事件循环**：
//!
//! 1. 每个 Toast 维护一个 `Option<Instant>`（记录创建时刻）
//! 2. Overlay::update() 中监听 `Event::Window(RedrawRequested(now))` 事件
//! 3. 遍历所有 Instant，计算 `remaining = timeout - elapsed`
//! 4. 若 remaining == 0 → 触发 `on_close(index)` 消息通知外部移除该 Toast
//! 5. 若 remaining > 0 → 调用 `shell.request_redraw_at(now + remaining)` 预约下一次重绘
//!
//! 这样形成了一个精密的倒计时链：每次 RedrawRequested 到来时检查是否到期，未到期则预约下一次，
//! 最终在到期那一刻触发 close 消息。整个过程无需任何外部定时器或异步任务。
//!
//! ### toast 列表管理
//!
//! toast 列表由**外部状态持有**（不在 Manager 内部），Manager 只负责渲染当前快照。
//! 外部在 update() 中对 Toast 列表执行 push/retain 操作，Manager 通过构造函数接收引用。
//! 当 close 消息触发后，外部调用 remove(index) 即可从列表中删除对应 Toast。
//!
//! ### Widget::diff() 机制
//!
//! iced 的 diff 机制在每次视图重建时被调用，用于增量同步 Widget Tree 与状态。
//! 本组件利用 diff 做两件事：
//! 1. 同步 Instant 数组长度与 toast 数组长度（手动管理对齐）
//! 2. 调用 tree.diff_children() 将子 Widget 树差分同步到底层渲染树
//!
//! 基于 iced 原生 Widget + Overlay 实现，定时器融入 iced 更新循环，不依赖外部 tokio。

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::mouse;
use iced::time::{self, Duration, Instant};
use iced::widget::{button, container, row, text};
use iced::window;
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};

// 默认超时时间：3 秒
// ponytail: 硬编码常量，若需每个 toast 独立超时可通过 Toast struct 传入
pub const DEFAULT_TIMEOUT: u64 = 3;

// --------------- Toast 级别枚举 ---------------

/// Toast 通知的级别，控制边框颜色用于视觉区分。
///
/// 实现了 `Default` trait，默认值为 `Info`。
/// 实现了 `Copy`，值传递零开销。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastLevel {
    /// 普通信息通知（默认）
    #[default]
    Info,
    /// 成功通知
    Success,
    /// 警告通知
    Warning,
    /// 错误通知
    Error,
}

impl ToastLevel {
    /// 根据级别返回对应的边框颜色，用于视觉区分。
    fn border_color(&self) -> Color {
        match self {
            // Color::from_rgb(r, g, b) — 每个通道取值 0.0..=1.0（f32），不是 0..255
            ToastLevel::Info => Color::from_rgb(0.35, 0.55, 0.85), // 蓝色调
            ToastLevel::Success => Color::from_rgb(0.25, 0.70, 0.35), // 绿色调
            ToastLevel::Warning => Color::from_rgb(0.80, 0.65, 0.15), // 黄色调
            ToastLevel::Error => Color::from_rgb(0.80, 0.25, 0.25), // 红色调
        }
    }
}

// --------------- Toast 显示位置枚举 ---------------

/// Toast 通知在屏幕上的显示位置。
///
/// 实现了 `Default`，默认为 `BottomRight`（右下角）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    /// 右下角（默认）
    #[default]
    BottomRight,
    /// 右上角
    TopRight,
}

// --------------- Toast 数据结构 ---------------

/// 单个 Toast 消息的配置数据，由外部调用方创建。
///
/// 实现 `Clone` 是因为 Manager::new() 中需要遍历它；实现 `Debug` 用于调试。
#[derive(Debug, Clone)]
pub struct Toast {
    /// 通知级别，决定边框颜色
    pub level: ToastLevel,
    /// 通知文本内容
    pub text: String,
    /// 显示位置（右下 / 右上）
    pub position: ToastPosition,
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            level: ToastLevel::default(), // → Info
            text: String::new(),
            position: ToastPosition::default(), // → BottomRight
        }
    }
}

// --------------- Manager Widget ---------------

/// Toast Manager 是核心 Widget，实现 `Widget` trait，作为**透传容器**嵌入 UI 树。
///
/// ## 职责
/// - 透明向下传递 content（底层页面内容的布局与事件）
/// - 通过 `overlay()` 方法返回 `Overlay` 在内容上方绘制 Toast 列表
/// - 管理每个 Toast 的 `Instant` 定时器状态
///
/// ## 泛型参数
/// - `'a`: 生命周期，content 及 toast Element 的引用生存期
/// - `Message`: 应用的消息类型，必须 `Clone`（iced 内部需要克隆消息副本）
///
/// ## Toast 为何是 `Vec<Element<'a, Message>>`
/// 不是 `Vec<Toast>` — `new()` 中已将 Toast 数据转换为完全构建的 `Element`（含关闭按钮），
/// 这样 overlay 层直接布局/绘制 Element，无需在 overlay 中再做构建逻辑。
pub struct Manager<'a, Message> {
    /// 底层页面内容，直接透传布局/事件
    content: Element<'a, Message>,
    /// 已构建完成的 Toast Element 列表（含样式与关闭按钮）
    toasts: Vec<Element<'a, Message>>,
    /// 每个 Toast 对应的位置信息，长度与 toasts 一致
    positions: Vec<ToastPosition>,
    /// 每个 Toast 的超时秒数（当前统一，可通过 timeout() 方法修改）
    timeout_secs: u64,
    /// 关闭回调：接收 toast 的 index，返回对应的 Message 以通知外部删除该 toast
    ///
    /// 使用 `Box<dyn Fn>` 而非泛型 F 以简化 Manager 的类型签名（避免泛型参数膨胀）
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Manager<'a, Message>
where
    // Message 必须 Clone，因为 iced 内部可能克隆消息发给多个 Shell
    Message: 'a + Clone,
{
    /// 构造 Manager Widget。
    ///
    /// ## 参数
    /// - `content`: 底层页面内容，会被包裹在 Manager 内（用户看不到 Manager 本身）
    /// - `toasts`: 当前所有 Toast 的引用切片，Manager 不会持有所有权
    /// - `on_close`: 当某个 Toast 超时到期或被点击关闭按钮时，调用此闭包返回 Message
    ///
    /// ## 实现细节
    /// 在此方法中完成 Toast 数据 → Element 的转换：
    /// - 每个 Toast 被构建为一个 `container(row[text + close_btn])`
    /// - 关闭按钮样式为透明背景的 `×` 字符，hover/pressed 时显示灰色背景
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        // 提前收集位置数组，方便 overlay 中快速索引
        let positions: Vec<ToastPosition> = toasts
            .iter()
            .map(|t| t.position)
            .collect();

        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                let border_color = toast.level.border_color();

                // ---------- 关闭按钮 ----------
                //
                // button(text("×").size(9)) — 创建一个按钮，内容为缩小到 9px 的 "×" 字
                //   .on_press((on_close)(index)) — 点击时触发 on_close，传入 toast 的索引
                //   .padding([0, 3]) — 上下 0，左右 3px
                //   .style(...) — 自定义按钮样式
                //
                // button::Status 枚举：
                //   - Active   — 正常状态
                //   - Hovered  — 鼠标悬停
                //   - Pressed  — 鼠标按下
                //   - Disabled — 禁用
                //
                // button::Style 字段：
                //   - background: Option<Background> — 背景（None = 透明）
                //   - border: Border — 边框样式
                //   - text_color: Color — 文字颜色
                //   - shadow: Shadow — 阴影
                //   - icon_color: Option<Color> — 图标颜色
                let close_btn: Element<'a, Message> = button(text("×").size(9))
                    .on_press((on_close)(index))
                    .padding([0, 3])
                    .style(|_: &Theme, status: button::Status| {
                        let bg = match status {
                            button::Status::Hovered => Color::from_rgb(0.3, 0.3, 0.35),
                            button::Status::Pressed => Color::from_rgb(0.4, 0.4, 0.45),
                            _ => Color::TRANSPARENT,
                        };
                        button::Style {
                            // Background::Color(c) — 纯色背景
                            background: Some(Background::Color(bg)),
                            // Border 字段：
                            //   color: Color — 边框颜色
                            //   width: f32 — 边框宽度（0.0 = 无边框）
                            //   radius: Radius — 圆角半径，.into() 将 f32 转为 Radius
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 3.0.into(),
                            },
                            text_color: Color::from_rgb(0.55, 0.55, 0.6),
                            ..Default::default()
                        }
                    })
                    .into();

                // ---------- Toast 卡片 = container(row[text + close_btn]) ----------
                //
                // row![] 宏：创建水平排列的 Row Widget
                //   .spacing(4) — 子元素间距 4px
                //   .align_y(Alignment::Center) — 垂直居中对齐
                //
                // container(...) 包裹 row：
                //   .padding([6, 8]) — 上下 6px，左右 8px（[vertical, horizontal]）
                //   .width(Length::Fixed(220.0)) — 固定宽度 220px
                //   .style(...) — 自定义容器样式（深色背景 + 彩色左边框）
                container(
                    row![
                        // Length::Fill — 填充所有可用空间（将关闭按钮推到最右）
                        text(toast.text.as_str())
                            .size(12)
                            .width(Length::Fill),
                        close_btn,
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .padding([6, 8])
                .width(Length::Fixed(220.0))
                .style(move |_: &Theme| container::Style {
                    // 深色半透明背景（暗色主题风格）
                    background: Some(Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                    border: Border {
                        color: border_color, // 左边框颜色 = 级别对应的颜色
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        Self {
            content: content.into(),
            toasts,
            positions,
            timeout_secs: DEFAULT_TIMEOUT,
            on_close: Box::new(on_close),
        }
    }

    /// 设置总超时时间（秒），以构建器模式返回新 Self。
    ///
    /// ```ignore
    /// Manager::new(content, toasts, on_close).timeout(5)
    /// ```
    pub fn timeout(self, seconds: u64) -> Self {
        Self {
            timeout_secs: seconds,
            ..self
        }
    }
}

// --------------- Widget trait 实现 ---------------
//
// Widget trait 是 iced 自定义 Widget 的核心接口，Manager 实现它来作为可嵌入 UI 树的 Widget。
//
// iced 渲染管线在每个帧周期依次调用：
//   1. tag() / state() — 初始化 Widget 状态（首次）
//   2. children() — 声明子 Widget 树结构
//   3. diff() — 增量同步 Widget 树变更
//   4. size() / layout() — 计算布局
//   5. update() — 处理事件（鼠标/键盘/窗口等）
//   6. draw() — 绘制
//   7. overlay() — 返回浮动 Overlay（如果有）

impl<Message> Widget<Message, Theme, Renderer> for Manager<'_, Message>
where
    Message: Clone,
{
    /// 返回 Widget 的尺寸定义，直接委托给 content 子 Widget。
    ///
    /// `as_widget()` 将 `Element` 解包为 `&dyn Widget`，再调用其 `size()` 方法。
    ///
    /// ## 返回值
    /// `Size<Length>` — Size { width: Length, height: Length }
    /// 其中 Length 可以是 Fixed/Fill/FillPortion/Shrink
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    /// 计算 Widget 布局，直接委托给 content 子 Widget。
    ///
    /// ## 参数
    /// - `tree`: iced 维护的 Widget 状态树，tree.children[0] 即 content 的子 Tree
    /// - `renderer`: 渲染器，某些布局需要它来计算文字尺寸等
    /// - `limits`: layout::Limits — 布局约束边界
    ///   - `limits.min()`: Size — 最小允许尺寸
    ///   - `limits.max()`: Size — 最大允许尺寸
    ///   - `limits.fill()`: Limits — 填充到 max 的新约束
    ///   - `limits.shrink(width/height)`: Limits — 收缩指定维度
    ///
    /// ## 返回值
    /// `layout::Node` — 布局节点树
    ///   - `node.bounds()`: Rectangle — 节点边界
    ///   - `node.children()`: 子节点迭代器
    ///   - `node.move_to(point)`: Node — 移动到指定位置
    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    /// 返回该 Widget 的类型标签，用于 iced 内部状态类型检查。
    ///
    /// 使用零大小 Struct `Marker` 的 `TypeId` 作为此 Widget 的唯一标识。
    /// 每个自定义 Widget 类型都需要唯一 tag，否则 `downcast_mut()` 会 panic。
    fn tag(&self) -> widget::tree::Tag {
        struct Marker; // 定义零大小的唯一标记类型
        widget::tree::Tag::of::<Marker>() // 将 Marker 的 TypeId 包装为 Tag
    }

    /// 初始化 Widget 的运行时状态。
    ///
    /// 返回 `Vec<Option<Instant>>`（初始为空），每个 Option<Instant> 记录一个 Toast 的创建时刻。
    /// `None` 表示该 toast 已被关闭/计时取消（close 按钮被点击时设为 None）。
    ///
    /// ## 参数
    /// `widget::tree::State::new(value)` — 从任意值创建 State，内部存储为 `Box<dyn Any>`
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    /// 声明子 Widget 的 Tree 列表。
    ///
    /// 子 Tree 的顺序严格对应 layout/update/draw 中 `tree.children[i]` 的索引。
    ///
    /// ## 子节点索引约定
    /// - `tree.children[0]` → content（主内容）
    /// - `tree.children[1 + i]` → toasts[i]（第 i 个 toast）
    ///
    /// `std::iter::once(x).chain(iter)` 将 content 作为第一个，然后链接 toast 列表。
    ///
    /// `Tree::new(element)` — 从 Element 创建空状态 Tree，后续 layout 时会填充子 Node。
    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(
                self.toasts
                    .iter()
                    .map(Tree::new),
            )
            .collect()
    }

    /// 差分同步 Widget Tree，在 iced 每次视图重建时调用。
    ///
    /// ## 本方法做了什么
    ///
    /// ### 1. 同步 Instant 数组
    /// iced 的状态系统只做类型检查，不自动修正数组长度。因此需要手动对齐：
    /// - 先 `retain` 过滤掉 None（保留仍然活跃的计时器）
    /// - 若 toast 数量减少（关闭了）→ truncate 截断 Instant 数组
    /// - 若 toast 数量增加（新增了）→ push `Instant::now()` 作为新 toast 的起始时间
    ///
    /// `std::iter::repeat_n(value, count)` — 创建重复 count 次的迭代器（nightly API），
    /// 此处用于批量追加 count 个 `Some(Instant::now())`
    ///
    /// ### 2. 调用 tree.diff_children()
    /// 将 self 的当前子 Element 列表与 tree 的现有子 Tree 列表进行差分比较，
    /// iced 内部会执行最小化 DOM 更新（重用未变的 Tree 节点，替换变更的）。
    fn diff(&self, tree: &mut Tree) {
        // State::downcast_mut<T>() — 从 Box<dyn Any> 中取出 &mut T
        // 类型必须与 state() 中传入的类型完全一致，否则 panic
        let instants = tree
            .state
            .downcast_mut::<Vec<Option<Instant>>>();
        // 过滤保留所有 Some 值（活跃的计时器）
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            // toast 减少了 → 截断到当前长度
            (old, new) if old > new => {
                instants.truncate(new);
            }
            // toast 增加了 → 追加新计时器，每个 Timer 的起始时间 = 当前时刻
            (old, new) if old < new => {
                // Instant::now() — 获取当前时间点（单调时钟，不受系统时间调整影响）
                instants.extend(std::iter::repeat_n(Some(Instant::now()), new - old));
            }
            // 长度相同 → 无操作
            _ => {}
        }

        // diff_children 将当前 Element 引用列表与已存在的 Tree 列表做差分
        // iced 内部通过对比 Element 指针/类型决定复用还是重建 Tree 节点
        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toasts.iter())
                .collect::<Vec<_>>(),
        );
    }

    /// 参与 iced 的操作系统（operation system），允许外部 Operation 遍历 Widget 树。
    ///
    /// 例如 `operation::focus(id)` 需要遍历树找到指定 id 的 Widget 并请求焦点。
    /// 本方法声明了一个容器层级并递归遍历 content 子树。
    ///
    /// ## 参数
    /// - `operation`: &mut dyn Operation — 操作对象
    ///   - `operation.container(id, bounds)` — 标记进入一个容器节点
    ///   - `operation.traverse(|op| ...)` — 遍历子节点
    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        // 标记进入一个容器（id=None, bounds=当前布局边界），使 operation 知道进入了新层级
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        });
    }

    /// 处理运行时事件（鼠标点击、键盘输入、窗口事件等）。
    ///
    /// 直接委托给 content 子 Widget 处理。Toast 的交互事件在 overlay 中处理。
    ///
    /// ## 参数
    /// - `event`: &Event — 事件枚举
    ///   - `Event::Mouse(mouse::Event)`, `Event::Keyboard(keyboard::Event)`, `Event::Window(window::Event)`, ...
    /// - `cursor`: mouse::Cursor — 鼠标光标状态
    ///   - `cursor.position()`: Option<Point> — 当前鼠标位置（逻辑坐标）
    ///   - `cursor.is_over(rect)`: bool — 判断鼠标是否在矩形内
    /// - `_clipboard`: &mut dyn Clipboard — 剪贴板接口
    /// - `shell`: &mut Shell<Message> — 消息发布出口
    ///   - `shell.publish(msg)` — 发布一条消息到应用消息循环
    ///   - `shell.request_redraw()` — 请求下一帧重绘
    /// - `viewport`: &Rectangle — 当前可视区域
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(&mut tree.children[0], event, layout, cursor, renderer, _clipboard, shell, viewport);
    }

    /// 绘制 Widget，直接委托给 content 子 Widget。
    ///
    /// ## 参数
    /// - `renderer`: &mut Renderer — iced 的渲染器（内部使用 tiny-skia 后端）
    /// - `theme`: &Theme — 当前主题配色
    /// - `style`: &renderer::Style — 渲染样式（背景色、文字色等默认值）
    /// - `cursor`: mouse::Cursor — 鼠标状态
    /// - `viewport`: &Rectangle — 当前可视区域边界
    ///
    /// Toast 自身的绘制在 overlay 的 draw() 中完成（这里是 content 的绘制入口）。
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(&tree.children[0], renderer, theme, style, layout, cursor, viewport);
    }

    /// 返回 Widget 与鼠标的交互类型（箭头、手型、文本选择等）。
    ///
    /// 委托给 content，Toast 不会在这里干预鼠标样式（在 overlay 中处理）。
    ///
    /// ## 返回值
    /// `mouse::Interaction` 枚举：
    ///   - `Idle` — 默认（箭头）
    ///   - `Pointer` — 可点击（手型）
    ///   - `Text` — 文本选择（I 型光标）
    ///   - `Grab` / `Grabbing` — 拖拽
    ///   - `ResizingHorizontally / Vertically` — 调整大小
    ///   - `NotAllowed` — 禁止操作
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    /// 返回浮动层 Overlay，用于在 content 之上绘制 Toast 列表。
    ///
    /// 这是 Toast 系统的**核心调度点**：
    /// 1. 取出 Instant 数组用于定时器状态管理
    /// 2. 分离 content 与 toasts 的子 Tree：`tree.children.split_at_mut(1)`
    ///    - content_state = children[0]（content 的 Tree 节点）
    ///    - toasts_state = children[1..]（所有 toast 的 Tree 节点）
    /// 3. 如果存在 toast → 创建 `Overlay` 结构并包装为 `overlay::Element`
    /// 4. 将 content 自身的 overlay（如果有）与 toast overlay 合并为 `overlay::Group`
    ///
    /// ## 返回值
    /// `Option<overlay::Element>` — None 表示没有浮动层，Some 返回浮动层 Element
    ///
    /// ## translation 参数
    /// `Vector` = { x: f32, y: f32 }，滚动/平移偏移量，用来调整 overlay 位置的基准。
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let instants = tree
            .state
            .downcast_mut::<Vec<Option<Instant>>>();

        // split_at_mut(1) — 分割可变切片，索引 1 之前和之后
        // content_state = [0]，toasts_state = [1..]
        let (content_state, toasts_state) = tree.children.split_at_mut(1);

        // 先获取 content 自身的 overlay（如果有），这允许 content 内部也能有自己的浮动层
        let content = self
            .content
            .as_widget_mut()
            .overlay(&mut content_state[0], layout, renderer, viewport, translation);

        // 仅当存在 toast 时才创建 overlay
        let toasts = (!self.toasts.is_empty()).then(|| {
            // overlay::Element::new(Box<dyn Overlay>) — 从 Overlay trait 对象创建 Element
            overlay::Element::new(Box::new(Overlay {
                // layout.bounds().position() 获取 Manager 在窗口中的起始坐标
                // + translation 加上滚动/平移偏移，确保 overlay 位置正确
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

        // 将 content overlay 和 toast overlay 合并（两者可能同时存在）
        let overlays = content
            .into_iter()
            .chain(toasts)
            .collect::<Vec<_>>();

        // overlay::Group::with_children(vec).overlay() — 将多个 overlay 合并为一个组
        // 这样 iced 渲染时会在一次 overlay pass 中绘制所有浮动层
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

// --------------- Overlay 结构 ---------------

/// Toast 的浮动层实现，负责：
/// - 布局每个 Toast 卡片的位置（根据 ToastPosition）
/// - 在 RedrawRequested 事件中做倒计时 + 触发关闭
/// - 绘制每个 Toast 卡片
///
/// Overlay 在 Widget 树之上渲染，因此 Toast 始终浮于内容上方。
struct Overlay<'a, 'b, Message> {
    /// Overlay 基准坐标（Manager Widget 在窗口中的起始位置 + translation）
    position: Point,
    /// 窗口可视区域边界
    viewport: Rectangle,
    /// 所有 Toast Element 的可变引用
    toasts: &'b mut [Element<'a, Message>],
    /// 每个 Toast 的位置枚举引用
    positions: &'b [ToastPosition],
    /// 每个 Toast 对应的 Widget Tree 节点（由 Manager 的 children() 分配）
    trees: &'b mut [Tree],
    /// 每个 Toast 的创建时间（用于倒计时），None 表示计时已取消
    instants: &'b mut [Option<Instant>],
    /// 关闭回调
    on_close: &'b dyn Fn(usize) -> Message,
    /// 超时秒数
    timeout_secs: u64,
}

// --------------- Overlay trait 实现 ---------------
//
// Overlay trait 与 Widget trait 类似，但运行在浮动层。iced 在每帧渲染时：
//   1. layout() — 计算 overlay 内部布局
//   2. update() — 处理事件（包括窗口重绘事件用于定时器）
//   3. draw() — 绘制 overlay 内容

impl<Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, '_, Message> {
    /// 计算 Overlay 内部布局：对每个 Toast 做 layout 并放置到正确位置。
    ///
    /// ## 算法
    /// 1. 对每个 Toast 调用 `as_widget_mut().layout()` 获取其自然尺寸
    /// 2. X 坐标统一：`viewport.x + viewport.width - toast_width - 12.0`（靠右，留 12px 边距）
    /// 3. Y 坐标根据位置不同：
    ///    - TopRight:    从上往下堆叠，`position.y + 8 + 前面所有 top toast 的高度之和`
    ///    - BottomRight: 从下往上堆叠，`position.y + bounds.height - toast_height - 8 - 前面 bottom toast 高度之和`
    ///
    /// 先处理所有 TopRight toast 再处理 BottomRight toast，保证同位置 toast 按顺序堆叠。
    ///
    /// ## 返回值
    /// `layout::Node::with_children(bounds, children)` — 创建带子节点的布局树
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        // Limits::new(min, max) — 创建布局约束，此处 min=ZERO 允许零尺寸，max=bounds
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let mut top_nodes = Vec::new();
        let mut bottom_nodes = Vec::new();

        for (i, toast) in self
            .toasts
            .iter_mut()
            .enumerate()
        {
            // 对每个 Toast Element 调用其内部 Widget 的 layout 方法
            let node = toast
                .as_widget_mut()
                .layout(&mut self.trees[i], renderer, &limits);
            let size = node.bounds().size();

            // X: 从右边界向内偏移 (width + 12px margin)
            let x = self.viewport.x + self.viewport.width - size.width - 12.0;

            // Y: 根据位置计算
            let y_offset = match self
                .positions
                .get(i)
                .copied()
                .unwrap_or_default()
            {
                ToastPosition::TopRight => {
                    // 已排列的 top toast 累计高度 + 每个 8px 间距
                    let prev_height: f32 = top_nodes
                        .iter()
                        .map(|n: &layout::Node| n.bounds().height + 8.0)
                        .sum::<f32>();
                    self.position.y + 8.0 + prev_height
                }
                ToastPosition::BottomRight => {
                    // 已排列的 bottom toast 累计高度（从下往上推）
                    let prev_height: f32 = bottom_nodes
                        .iter()
                        .map(|n: &layout::Node| n.bounds().height + 8.0)
                        .sum::<f32>();
                    self.position.y + bounds.height - size.height - 8.0 - prev_height
                }
            };

            // move_to(point) — 将 layout node 平移到指定坐标（保持尺寸不变）
            let positioned = node.move_to(Point::new(x, y_offset));
            match self
                .positions
                .get(i)
                .copied()
                .unwrap_or_default()
            {
                ToastPosition::TopRight => top_nodes.push(positioned),
                ToastPosition::BottomRight => bottom_nodes.push(positioned),
            }
        }

        // 合并：TopRight 节点在前，BottomRight 在后
        let mut children = top_nodes;
        children.extend(bottom_nodes);
        layout::Node::with_children(bounds, children)
    }

    /// 处理 Overlay 事件。核心逻辑：
    ///
    /// ### 1. 定时器倒计时（在 RedrawRequested 事件中）
    ///
    /// ```
    /// 对每个 instant:
    ///   remaining = timeout - instant.elapsed()
    ///   if remaining == 0:
    ///     instant.take()        → 设为 None（取消计时）
    ///     shell.publish(...)    → 触发关闭消息
    ///   else:
    ///     shell.request_redraw_at(now + remaining) → 预约到期时刻重绘
    /// ```
    ///
    /// ### 2. 事件转发给每个 Toast Element
    ///
    /// 遍历所有 toast+tree+layout+instant，为每个创建独立的 local_shell/clipboard 转发事件。
    /// 如果 local_shell 发布消息（用户点击了关闭按钮），立即将对应 instant 设为 None（取消倒计时）。
    ///
    /// ## Shell 要点
    /// - `shell.publish(msg)` — 发布消息到应用消息循环
    /// - `shell.request_redraw()` — 请求下一帧重绘
    /// - `shell.request_redraw_at(instant)` — 请求在指定时刻重绘（精确到 Instant）
    /// - `shell.merge(other, f)` — 合并另一个 Shell 的消息，f 用于消息类型转换（此处为 identity）
    /// - `shell.is_empty()` — 判断是否有待发布的消息
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        // ---------- 定时器倒计时 ----------
        //
        // iced 的 RedrawRequested 事件在窗口需要重绘时发出。
        // 这是 iced 内部的时间驱动机制：通过 request_redraw_at() 预约时间，
        // 届时系统会发出 RedrawRequested(now) 事件，我们在那时检查过期。
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            // window::Event::RedrawRequested(Instant) — 窗口请求重绘事件
            // now 是当前帧的时间点，由 iced 运行时提供
            self.instants
                .iter_mut()
                .enumerate()
                .for_each(|(index, maybe_instant)| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        // time::seconds(n) — 将 u64 秒数转为 Duration
                        // instant.elapsed() — 计算从创建到此刻经过的时间
                        // saturating_sub — 饱和减法，确保不会下溢为负
                        let remaining = time::seconds(self.timeout_secs).saturating_sub(instant.elapsed());

                        if remaining == Duration::ZERO {
                            // 时间到 → 取消该计时器，触发关闭
                            // Option::take() — 取出值并设为 None
                            maybe_instant.take();
                            shell.publish((self.on_close)(index));
                        } else {
                            // 未到期 → 预约下一次重绘（在到期时刻）
                            shell.request_redraw_at(*now + remaining);
                        }
                    }
                });
        }

        // ---------- 转发事件到每个 Toast Element ----------
        let viewport = layout.bounds();

        for (((child, state), child_layout), instant) in self
            .toasts
            .iter_mut()
            .zip(self.trees.iter_mut())
            .zip(layout.children())
            .zip(self.instants.iter_mut())
        {
            // 为每个 toast 创建隔离的消息/剪贴板环境
            let mut local_messages = vec![];
            let mut local_shell = Shell::new(&mut local_messages);
            // clipboard::Null — 空剪贴板实现，不读写系统剪贴板
            let mut local_clipboard = iced::advanced::clipboard::Null;

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

            // 如果 toast 内部的 Widget 发布了消息（例如关闭按钮被点击）
            if !local_shell.is_empty() {
                // 取消该 toast 的倒计时（不再等待 timeout）
                instant.take();
            }

            // shell.merge(other, converter) — 将 local_shell 的消息合并到父 shell
            // std::convert::identity 作为 converter = 消息类型不变（Message → Message）
            shell.merge(local_shell, std::convert::identity);
        }
    }

    /// 绘制所有 Toast 卡片。
    ///
    /// 遍历每个 toast Element，调用其内部 Widget 的 draw 方法。
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        for ((child, tree), layout) in self
            .toasts
            .iter()
            .zip(self.trees.iter())
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    /// 参与 iced 的操作系统，允许外部 Operation 遍历 overlay widget 树。
    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.toasts
                .iter_mut()
                .zip(self.trees.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    /// 返回 overlay 的鼠标交互类型。
    ///
    /// 遍历所有 toast，取最"高优先级"的交互类型。
    /// 如果光标悬停在某个 toast 上，至少返回 Idle（而不是什么都不返回）。
    ///
    /// ## 逻辑
    /// - 对每个 toast 调用其 mouse_interaction
    /// - 如果光标在 toast 边界内 → 确保至少返回 Idle（表示"我在这里"）
    /// - 所有结果取 max（Interaction 实现了 Ord，优先级 Idle < Pointer < ... < NotAllowed）
    fn mouse_interaction(&self, layout: Layout<'_>, cursor: mouse::Cursor, renderer: &Renderer) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.trees.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, &self.viewport, renderer)
                    // 确保光标悬停在 toast 上时至少返回 Idle
                    .max(if cursor.is_over(layout.bounds()) { mouse::Interaction::Idle } else { Default::default() })
            })
            .max() // 取最高优先级的交互
            .unwrap_or_default()
    }
}

// --------------- From<Manager> for Element ---------------

/// 允许 Manager 直接转换为 `Element`，方便嵌入 UI 树。
///
/// 用法：
/// ```ignore
/// let element: Element<Message> = Manager::new(content, &toasts, on_close).into();
/// ```
///
/// `Element::new(widget)` — 将任何实现了 Widget trait 的实例包装为 Element。
impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(manager: Manager<'a, Message>) -> Self {
        Element::new(manager)
    }
}
