//! # 菜单状态控制层 + Overlay 渲染层
//!
//! 对应 libcosmic 菜单系统架构中的**状态控制层**与**渲染表现层**，分两部分：
//!
//! ## 1. 状态控制层 —— `MenuBarState`（有限状态机 FSM）
//!
//! ```text
//! MenuBarState
//!   ├── open: bool                    全局开关
//!   ├── root_hovered: Option<usize>   根菜单项悬停索引（MenuBar 自身绘制用）
//!   ├── active_path: Vec<usize>       激活路径栈 [0, 2] = 根第0项下的第2个子菜单
//!   └── menu_states: HashMap<Vec<usize>, MenuState>   多级菜单状态映射表（惰性初始化）
//! ```
//!
//! - **路径寻址**：`Vec<usize>` 作为唯一标识符，实现任意深度的状态索引
//! - **惰性初始化**：`menu_states` 仅在用户交互（Hover）时初始化对应层级状态
//! - **事件驱动**：事件处理函数修改状态，触发重绘请求
//!
//! ## 2. 渲染表现层 —— `Menu`（实现 `iced::Overlay` trait）
//!
//! `Menu` 每层独立处理坐标计算与事件拦截，通过 `depth` 控制递归：
//! - 根实例 (`depth: 0`) 由 `MenuBar::overlay` 创建
//! - 在 `Menu::overlay` 中检测到子菜单激活时，递归创建新 `Menu` (`depth + 1`)
//! - 定位策略：`depth=0` 定位在根菜单项下方；`depth>0` 定位在父菜单项右侧
//!
//! # Rust 语法要点
//!
//! ## 有限状态机 (FSM)
//! 菜单交互是一个典型的状态机：关闭 → 打开(根) → 展开(子) → 关闭。
//! 状态转移由事件驱动（Hover/Click/Escape），每次转移后请求重绘。
//!
//! ## HashMap 惰性初始化
//! ```text
//! self.menu_states.entry(key).or_default().hovered = value;
//! ```
//! `entry().or_default()` 在键不存在时插入 `Default` 值 —— 仅在首次交互时分配。

use std::collections::HashMap;

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::renderer::Quad;
// 引入渲染器 trait：`fill_quad`/`fill_paragraph`/`default_font` 均为 trait 方法
use iced::advanced::renderer::Renderer as _;
use iced::advanced::text::paragraph::Plain;
use iced::advanced::text::{self, Text as IcedText};
use iced::advanced::text::Renderer as _;
use iced::advanced::{Clipboard, Shell};
use iced::mouse;
use iced::{Border, Color, Event, Point, Rectangle, Renderer, Size, Theme, Vector};

use super::menu_tree::MenuTree;

// --------------- 视觉常量 ---------------

/// 菜单项高度（像素）
pub(crate) const ITEM_HEIGHT: f32 = 28.0;
/// 菜单水平内边距（文本与边框间距）
const H_PADDING: f32 = 10.0;
/// 菜单垂直内边距（首项与边框间距）
const V_PADDING: f32 = 6.0;
/// 图标预留区宽度（无图标时也预留，保持各菜单项文本对齐）
const ICON_AREA: f32 = 22.0;
/// 右侧箭头预留区宽度（Folder 指示箭头）
const ARROW_AREA: f32 = 18.0;
/// 菜单最小宽度
const MIN_WIDTH: f32 = 150.0;
/// 菜单文本字号
const FONT_SIZE: f32 = 14.0;

/// 菜单背景色（深色面板）
const BG: Color = Color::from_rgb8(31, 31, 41);
/// 菜单项悬停高亮色（与 tree_menu 选中色一致的蓝色调）
const HOVER: Color = Color::from_rgb8(40, 80, 120);
/// 菜单边框色
const BORDER: Color = Color::from_rgb8(64, 64, 84);
/// 菜单文本色
const TEXT: Color = Color::WHITE;
/// 禁用项文本色（灰显）
const DISABLED: Color = Color::from_rgb8(115, 115, 128);
/// 分隔线颜色
const SEPARATOR: Color = Color::from_rgb8(77, 77, 97);
/// Folder 箭头颜色
const ARROW_COLOR: Color = Color::from_rgb8(178, 178, 191);

// --------------- 状态控制层 ---------------

/// 单层菜单状态
///
/// 由 [`MenuBarState::menu_states`] 持有，以 `Vec<usize>` 路径为键。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MenuState {
    /// 当前层级悬停索引
    pub hovered: Option<usize>,
}

/// 菜单栏状态机 (FSM)
///
/// 管理运行时菜单的激活状态与交互路径。核心状态：
/// - `active_path`: 从根到叶的索引序列（如 `[0, 2]` 表示根菜单第0项下的第2个子菜单激活）
/// - `menu_states`: 每层菜单的 hover 状态映射（惰性初始化）
#[derive(Debug, Clone, Default)]
pub struct MenuBarState {
    /// 全局开关状态
    pub open: bool,
    /// 根菜单项悬停索引（由 MenuBar 自身管理，用于根项高亮）
    pub root_hovered: Option<usize>,
    /// 激活路径栈
    pub active_path: Vec<usize>,
    /// 多级菜单的状态映射表
    pub menu_states: HashMap<Vec<usize>, MenuState>,
}

impl MenuBarState {
    /// 菜单是否打开
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// 打开菜单
    pub fn open(&mut self) {
        self.open = true;
    }

    /// 关闭菜单（清空路径与根悬停）
    pub fn close(&mut self) {
        self.open = false;
        self.active_path.clear();
        self.root_hovered = None;
    }

    /// 切换开关状态
    ///
    /// 保留为 FSM 完整 API（当前 MenuBar 使用 open/close 语义更精确），
    /// 由单元测试覆盖。
    #[allow(dead_code)]
    pub fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open = true;
        }
    }

    /// 计算指定层级的路径键（`active_path[..=depth]`）
    ///
    /// depth=0 的键是 `[根索引]`，depth=1 的键是 `[根索引, 子索引]`。
    /// 该键同时作为 `menu_states` 的 HashMap 键。
    fn path_key(&self, depth: usize) -> Vec<usize> {
        self.active_path.iter().take(depth + 1).copied().collect()
    }

    /// 查询指定层级的悬停索引
    pub fn hovered_at(&self, depth: usize) -> Option<usize> {
        self.menu_states
            .get(&self.path_key(depth))
            .and_then(|state| state.hovered)
    }

    /// 设置指定层级的悬停索引（惰性初始化对应 MenuState）
    pub fn set_hovered(&mut self, depth: usize, hovered: Option<usize>) {
        self.menu_states
            .entry(self.path_key(depth))
            .or_default()
            .hovered = hovered;
    }
}

// --------------- 渲染表现层 ---------------

/// 单层菜单 Overlay
///
/// 每个 [`Menu`] 实例负责一层下拉菜单的定位、绘制与事件处理。
/// 子菜单通过 [`Overlay::overlay`] 嵌套递归创建，`depth` 标识层级深度。
pub(crate) struct Menu<'a, Message> {
    /// 菜单栏状态机（各层共享同一实例）
    pub(crate) state: &'a mut MenuBarState,
    /// 当前层渲染的菜单项列表
    pub(crate) items: &'a [MenuTree<'static, Message>],
    /// 层级深度（0 = 根菜单）
    pub(crate) depth: usize,
    /// 菜单栏整体边界（备用定位基准）
    pub(crate) bar_bounds: Rectangle,
    /// 菜单栏各根项边界（depth=0 定位基准）
    pub(crate) root_bounds_list: &'a [Rectangle],
    /// 视图平移偏移（滚动等场景）
    pub(crate) translation: Vector,
    /// 父菜单项边界（depth>0 时用于右侧定位）
    pub(crate) parent_bounds: Option<Rectangle>,
}

impl<'a, Message> Menu<'a, Message> {
    /// 计算菜单内容尺寸（宽度取文本测量最大值，高度按项数累加）
    ///
    /// `available`: 可用布局区域（窗口大小），用作文本测量边界（有限值，避免换行异常）
    fn measure(&self, renderer: &Renderer, available: Size) -> Size {
        let mut width = MIN_WIDTH;
        for item in self.items {
            if let Some(label) = item.label() {
                let text_width = measure_text_width(renderer, label, FONT_SIZE, available);
                let total = text_width + ICON_AREA + ARROW_AREA + 2.0 * H_PADDING;
                width = width.max(total);
            }
        }
        let height = self.items.len() as f32 * ITEM_HEIGHT + 2.0 * V_PADDING;
        Size::new(width, height)
    }
}

impl<'a, Message> Menu<'a, Message>
where
    Message: Clone,
{
    /// 命中测试：返回光标所在菜单项的索引
    fn hit_test(&self, layout: Layout<'_>, position: Point) -> Option<usize> {
        layout
            .children()
            .enumerate()
            .find(|(_, child)| child.bounds().contains(position))
            .map(|(i, _)| i)
    }
}

/// `Overlay` trait 实现 —— 菜单浮动层
///
/// iced 渲染管线对每个 overlay 依次调用：
/// 1. `layout()` —— 计算定位与尺寸
/// 2. `update()` —— 处理事件（鼠标/键盘）
/// 3. `draw()` —— 绘制
/// 4. `overlay()` —— 返回嵌套子 overlay（若有）
impl<'a, Message> overlay::Overlay<Message, Theme, Renderer> for Menu<'a, Message>
where
    Message: Clone,
{
    /// 计算 Overlay 布局
    ///
    /// ## 定位策略（Layout 正向传递，零全局查询）
    /// - `depth = 0`：定位在激活根菜单项的下方（`root_bounds_list[i].below()`）
    /// - `depth > 0`：定位在父菜单项的右侧（`parent_bounds.right_side()`）
    ///
    /// ## 返回结构
    /// 根节点 bounds 为整个菜单矩形，子节点为每个菜单项（纵向排列，绝对坐标）。
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let size = self.measure(renderer, bounds);

        // 定位计算
        let position = if self.depth == 0 {
            // 根菜单：定位在激活根菜单项下方
            let root_index = self.state.active_path.first().copied().unwrap_or(0);
            let root_bounds = self
                .root_bounds_list
                .get(root_index)
                .copied()
                .unwrap_or(self.bar_bounds);
            Point::new(
                root_bounds.x + self.translation.x,
                root_bounds.y + root_bounds.height + self.translation.y,
            )
        } else {
            // 子菜单：定位在父菜单项右侧
            let parent = self.parent_bounds.unwrap_or(self.bar_bounds);
            Point::new(parent.x + parent.width, parent.y)
        };

        // 子节点：每个菜单项一个纵向排列的布局节点
        // 注意：children 使用【相对坐标】((0, y))，iced 的 Layout::children()
        // 会自动累加父节点 position —— 若此处用绝对坐标会导致双重偏移
        let mut children = Vec::with_capacity(self.items.len());
        let mut y = V_PADDING;
        for _ in self.items {
            children.push(
                layout::Node::new(Size::new(size.width, ITEM_HEIGHT))
                    .move_to(Point::new(0.0, y)),
            );
            y += ITEM_HEIGHT;
        }

        // 根节点 move_to(position) 定位整个菜单；children 相对坐标由 Layout 累加
        layout::Node::with_children(size, children).move_to(position)
    }

    /// 绘制菜单    ///
    /// 渲染顺序：背景层 → 逐项绘制（图标、文本、箭头、hover 高亮、分隔线）
    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let hovered = self.state.hovered_at(self.depth);

        // 1. 背景层（圆角深色面板 + 边框）
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    color: BORDER,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            },
            BG,
        );

        // 2. 逐项绘制
        for (i, (item, child_layout)) in self.items.iter().zip(layout.children()).enumerate() {
            let item_bounds = child_layout.bounds();

            if let MenuTree::Separator = item {
                // 分隔线：绘制在菜单项区域的垂直中线
                let line = Rectangle {
                    x: bounds.x + H_PADDING,
                    y: item_bounds.center_y(),
                    width: bounds.width - 2.0 * H_PADDING,
                    height: 1.0,
                };
                renderer.fill_quad(Quad { bounds: line, ..Default::default() }, SEPARATOR);
                continue;
            }

            // 提取通用属性（label / icon / enabled）
            let (label, icon, enabled) = match item {
                MenuTree::Item(menu_item) => (&menu_item.label, menu_item.icon, menu_item.enabled),
                MenuTree::Folder(folder) => (&folder.label, folder.icon, true),
                MenuTree::Separator => unreachable!("已在上面处理"),
            };

            // hover 高亮（内缩 2px 的圆角矩形）
            if hovered == Some(i) {
                let highlight = Rectangle {
                    x: item_bounds.x + 2.0,
                    y: item_bounds.y + 2.0,
                    width: item_bounds.width - 4.0,
                    height: item_bounds.height - 4.0,
                };
                renderer.fill_quad(
                    Quad {
                        bounds: highlight,
                        border: Border {
                            radius: 3.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    HOVER,
                );
            }

            // 文本垂直居中：y + (项高 - 字号) / 2
            let text_y = item_bounds.y + (ITEM_HEIGHT - FONT_SIZE) / 2.0;
            let text_color = if enabled { TEXT } else { DISABLED };

            // 图标（预留 ICON_AREA 保证文本对齐）
            if let Some(icon) = icon {
                renderer.fill_text(
                    make_text(
                        renderer,
                        icon.to_string(),
                        FONT_SIZE - 2.0,
                        Size::new(item_bounds.width, ITEM_HEIGHT),
                    ),
                    Point::new(item_bounds.x + H_PADDING, text_y),
                    text_color,
                    bounds,
                );
            }

            // 文本
            renderer.fill_text(
                make_text(
                    renderer,
                    label.clone(),
                    FONT_SIZE,
                    Size::new(item_bounds.width, ITEM_HEIGHT),
                ),
                Point::new(item_bounds.x + H_PADDING + ICON_AREA, text_y),
                text_color,
                bounds,
            );

            // Folder 右侧箭头
            if item.is_folder() {
                renderer.fill_text(
                    make_text(
                        renderer,
                        "›".to_string(),
                        FONT_SIZE,
                        Size::new(item_bounds.width, ITEM_HEIGHT),
                    ),
                    Point::new(
                        item_bounds.x + item_bounds.width - H_PADDING - ARROW_AREA + 4.0,
                        text_y,
                    ),
                    ARROW_COLOR,
                    bounds,
                );
            }
        }
    }

    /// 处理 Overlay 事件
    ///
    /// ## 事件语义
    /// - `Escape`：任意层级关闭整个菜单
    /// - `CursorMoved`：命中测试更新 hover；hover 到 Folder 展开子菜单，否则收起
    /// - `ButtonPressed(Left)`：**仅最深层**处理 ——
    ///   点击 Item 发送消息并关闭；点击外部区域关闭整个菜单。
    ///   非最深层不处理点击，避免多级菜单竞争关闭。
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        // ESC 关闭（任意层级）
        if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            ..
        }) = event
        {
            self.state.close();
            shell.request_redraw();
            return;
        }

        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let hovered = self.hit_test(layout, *position);

                // 仅当 hover 变化时更新状态（避免无意义重绘）
                if self.state.hovered_at(self.depth) != hovered {
                    self.state.set_hovered(self.depth, hovered);

                    // 仅当鼠标位于本层某项上时更新展开路径：
                    // - hover 到 Folder → 展开子菜单（active_path.push(i)）
                    // - hover 到 Item → 收起子菜单（truncate 到当前层）
                    // - 鼠标移出本层（可能进入子菜单区域）→ 保持现有路径，
                    //   避免子菜单在鼠标穿越层级边界时闪烁关闭
                    if let Some(i) = hovered {
                        self.state.active_path.truncate(self.depth + 1);
                        if self.items.get(i).is_some_and(|item| item.is_folder()) {
                            self.state.active_path.push(i);
                        }
                    }
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                // 仅最深层 Menu 处理点击（active_path 长度 == depth + 1）
                if self.state.active_path.len() != self.depth + 1 {
                    return;
                }

                let Some(pos) = cursor.position() else {
                    return;
                };

                if !layout.bounds().contains(pos) {
                    // 点击外部 → 关闭整个菜单；
                    // 但点击落在菜单栏根项区域（bar_bounds）时交给 MenuBar 处理
                    // （打开/切换/关闭根菜单），避免本层误关。
                    if !self.bar_bounds.contains(pos) {
                        self.state.close();
                        shell.request_redraw();
                    }
                } else if let Some(index) = self.hit_test(layout, pos) {
                    // 点击 Item → 发送消息并关闭（disabled 项不响应）
                    if let MenuTree::Item(menu_item) = &self.items[index]
                        && menu_item.enabled
                    {
                        if let Some(action) = menu_item.action.clone() {
                            shell.publish(action);
                        }
                        self.state.close();
                        shell.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    /// 返回嵌套子菜单 overlay（递归机制）
    ///
    /// 当 `active_path` 深度超过当前层（`len() > depth + 1`）时，
    /// 沿路径下钻创建下一层 `Menu`，`depth + 1`。
    /// 子菜单的定位基准（父菜单项边界）从本层 layout 的子节点中直接获取。
    fn overlay<'b>(
        &'b mut self,
        layout: Layout<'b>,
        _renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // 子菜单展开条件：路径比当前层深
        if self.state.active_path.len() <= self.depth + 1 {
            return None;
        }

        // 父菜单项索引 = 本层激活的子项（active_path[depth + 1]）
        // 注意：active_path[depth] 是“本层来自哪一层”的索引（根项），
        // 而展开子菜单需要的是“本层 hover 到哪一项”的索引（当前层的下一级）。
        let parent_index = self.state.active_path[self.depth + 1];
        // 父菜单项边界（Layout 正向传递，零查询开销）
        let parent_bounds = layout
            .children()
            .nth(parent_index)
            .map(|node| node.bounds())?;

        // 下钻子菜单项
        let child_items = match self.items.get(parent_index) {
            Some(MenuTree::Folder(folder)) => &folder.children,
            _ => return None,
        };

        Some(overlay::Element::new(Box::new(Menu {
            state: self.state,
            items: child_items,
            depth: self.depth + 1,
            bar_bounds: self.bar_bounds,
            root_bounds_list: self.root_bounds_list,
            translation: self.translation,
            parent_bounds: Some(parent_bounds),
        })))
    }

    /// 鼠标交互类型：悬停在可用 Item 上时显示手型指针
    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if let Some(i) = self.state.hovered_at(self.depth)
            && matches!(
                self.items.get(i),
                Some(MenuTree::Item(item)) if item.enabled
            )
        {
            return mouse::Interaction::Pointer;
        }
        mouse::Interaction::Idle
    }

    /// Overlay 层级索引：子菜单应显示在上层菜单之上
    fn index(&self) -> f32 {
        1.0 + self.depth as f32 * 0.01
    }
}

// --------------- 工具函数 ---------------

/// 构造用于绘制/测量的 [`IcedText`]
///
/// # Rust: 结构体字面量
/// `Text` 的所有字段为 `pub`，可直接用结构体字面量构造。
/// `bounds` 设为无穷大，避免文本在测量时被换行截断。
pub(crate) fn make_text(
    renderer: &Renderer,
    content: String,
    size: f32,
    bounds: Size,
) -> IcedText<String, <Renderer as iced::advanced::text::Renderer>::Font> {
    IcedText {
        content,
        // 使用调用方提供的有限布局边界（对齐 iced 官方 `limits.max()` 范式），
        // 避免 INFINITY 导致 cosmic-text 布局异常
        bounds,
        size: iced::Pixels(size),
        // LineHeight 无 Default，显式指定 Relative(1.0)
        line_height: text::LineHeight::Relative(1.0),
        font: renderer.default_font(),
        align_x: text::Alignment::Left,
        align_y: iced::alignment::Vertical::Top,
        // Shaping 无 Default，显式指定 Auto
        shaping: text::Shaping::Auto,
        // 菜单文本单行绘制/测量，禁用换行
        wrapping: text::Wrapping::None,
    }
}

/// 测量文本宽度（用于菜单自适应宽度）
///
/// 使用 `Plain` 段落（cosmic-text 后端）计算文本最小宽度，
/// `with_text` 创建时即完成布局，可直接查询 `min_width()`。
pub(crate) fn measure_text_width(renderer: &Renderer, label: &str, size: f32, bounds: Size) -> f32 {
    let paragraph = Plain::<<Renderer as iced::advanced::text::Renderer>::Paragraph>::new(
        make_text(renderer, label.to_string(), size, bounds),
    );
    paragraph.min_width()
}

// --------------- 单元测试 ---------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 打开/关闭/切换 基本开关状态
    #[test]
    fn open_close_toggle() {
        let mut state = MenuBarState::default();
        assert!(!state.is_open());

        state.open();
        assert!(state.is_open());

        state.close();
        assert!(!state.is_open());
        assert!(state.active_path.is_empty());

        state.toggle();
        assert!(state.is_open());
        state.toggle();
        assert!(!state.is_open());
    }

    /// 路径展开与收起：hover Folder 展开，hover Item 收起
    #[test]
    fn path_expand_and_collapse() {
        let mut state = MenuBarState::default();
        state.open();
        state.active_path = vec![0];

        // 模拟 depth=0 层 hover 到 Folder(index=2) → 展开子菜单
        state.active_path.truncate(1); // depth + 1 = 1
        state.active_path.push(2);
        assert_eq!(state.active_path, vec![0, 2]);

        // 模拟 depth=0 层 hover 到 Item(index=1) → 收起子菜单
        state.active_path.truncate(1);
        assert_eq!(state.active_path, vec![0]);
    }

    /// hover 状态惰性初始化与查询
    #[test]
    fn hovered_state_lazy_init() {
        let mut state = MenuBarState::default();
        state.open();
        state.active_path = vec![0];

        // 未初始化时查询为 None
        assert_eq!(state.hovered_at(0), None);

        // 设置后查询生效
        state.set_hovered(0, Some(3));
        assert_eq!(state.hovered_at(0), Some(3));

        // 不同路径互不影响
        state.active_path = vec![1];
        assert_eq!(state.hovered_at(0), None);
        state.set_hovered(0, Some(1));
        assert_eq!(state.hovered_at(0), Some(1));
    }

    /// 关闭时清空全部状态
    #[test]
    fn close_resets_all() {
        let mut state = MenuBarState::default();
        state.open();
        state.active_path = vec![0, 2];
        state.root_hovered = Some(1);
        state.set_hovered(0, Some(3));

        state.close();
        assert!(!state.is_open());
        assert!(state.active_path.is_empty());
        assert_eq!(state.root_hovered, None);
        // menu_states 残留无害（惰性清理），但路径已清空
        assert!(state.menu_states.contains_key(&vec![0]));
    }
}
