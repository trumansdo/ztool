//! 应用状态管理和左右两栏布局渲染
//!
//! `App` 结构体是 iced Elm Architecture 中的"Model"，持有整个应用的状态。
//!
//! # Rust 语法要点
//!
//! ## `struct` 结构体
//! ```text
//! pub struct App {
//!     pub selected_tab: Tab,     // 公开字段，外部可读可写
//!     expanded: HashSet<String>, // 私有字段，仅 App 的方法可访问
//! }
//! ```
//! - `pub` 字段：可从模块外访问
//! - 无 `pub` 字段：私有，仅 impl App 中的方法可访问
//! - Rust 没有类 (class)，用 struct + impl 代替
//!
//! ## `#[derive(Default)]` — 自动生成默认值
//! 为结构体自动实现 `Default` trait，生成 `App::default()` 方法。
//! 等同于 `App { selected_tab: Tab::default(), json_formatter: JsonFormatter::default(), ... }`。
//! 前提：所有字段类型都实现了 `Default`。
//!
//! ## `HashSet<String>` — 集合类型
//! `HashSet<T>` 是哈希集合（类似 Python set 或 Java HashSet）：
//! - 元素无序、不重复
//! - 基于哈希表，`insert` / `contains` / `remove` 均为 O(1) 均摊
//! - 需要元素类型实现 `Hash` + `Eq` trait（String 已实现）
//! 这里用它记录哪些侧边栏分类目已展开。
//!
//! ## `std::collections::HashSet`
//! 使用完全限定路径引用标准库的 HashSet，避免与自定义类型同名。
//! 也可写 `use std::collections::HashSet;` 然后在字段类型处直接写 `HashSet<String>`。

use crate::features::json_fmt::JsonFormatter;
use crate::features::net_capture::PacketCapture;
use crate::features::net_port_scan::NetScanner;
use crate::features::ui_libs::UiLibs;
use crate::ui::widgets::tree_menu::{TreeItem, render_tree_item};
use crate::ui::widgets::{Layered, layer};
use iced::widget::{container, column, row};
use iced::{Element, Task, Length, Color};

use super::{Message, Tab};

/// 应用状态结构体 —— TEA 架构中的 Model
///
/// # 状态持有
/// App 持有所有子模块的状态，实现了"单一状态树" (single source of truth)。
/// 所有功能模块的状态作为 App 的字段，便于序列化/恢复、跨模块共享。
#[derive(Default)]
pub struct App {
    /// 当前选中的标签页
    pub selected_tab: Tab,
    /// JSON 格式化工具的状态
    pub json_formatter: JsonFormatter,
    /// 端口扫描工具的状态
    pub net_port_scan: NetScanner,
    /// 网络抓包工具的状态
    pub packet_capture: PacketCapture,
    /// UI 组件库的状态
    pub ui_libs: UiLibs,
    /// 侧边栏中已展开的分类 id 集合
    pub expanded: std::collections::HashSet<String>,
}

impl App {
    /// 创建应用初始状态
    ///
    /// # Rust: `Self` 别名
    /// 在 `impl App` 块中，`Self` 是 `App` 的类型别名。
    /// `Self::default()` 等价于 `App::default()`，调用 `#[derive(Default)]` 生成的默认构造。
    ///
    /// # 初始化策略
    /// 默认展开所有分类（"net"、"data"、"ui"），方便用户看到全部功能。
    /// `HashSet::insert` 插入元素并返回 `bool`（true=新插入，false=已存在）。
    pub fn new() -> Self {
        let mut expanded = std::collections::HashSet::new();
        expanded.insert("net".to_string());
        expanded.insert("data".to_string());
        expanded.insert("ui".to_string());
        Self {
            expanded,
            ..Self::default()
        }
    }

    /// 处理消息，分发到对应子模块
    ///
    /// # TEA 架构: update 模式
    /// 顶层 update 作为"消息路由器"：
    /// 1. match 消息变体，判断属于哪个子系统
    /// 2. 调用子系统的 update 函数，传入可变引用
    /// 3. 将子系统返回的 Task 映射为顶层 Message（重新包装）
    ///
    /// ## Rust: `Task<Message>` 异步任务
    /// Task 是 iced 的异步抽象。`Task::none()` 表示没有后续操作。
    /// `task.map(Message::JsonFmt)` 将子模块的 Msg 包装回顶层的 Message 变体，
    /// 这样 iced 运行时知道该把后续消息发送到顶层 update。
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Rust: `match` 穷尽性检查
        // 编译器保证所有 Message 变体都被处理，
        // 遗漏任何变体会导致编译错误。
        match message {
            Message::ToggleCategory(id) => {
                // HashSet::contains 检查元素是否存在
                if self.expanded.contains(&id) {
                    self.expanded.remove(&id);
                } else {
                    self.expanded.insert(id);
                }
                Task::none()
            }
            Message::TabSelected(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
            // `m` 绑定匹配到的 json_fmt::Msg
            Message::JsonFmt(m) => {
                // 调用子模块 update，用 .map() 包装返回的 Task
                crate::features::json_fmt::update(&mut self.json_formatter, m)
                    .map(Message::JsonFmt)
            }
            Message::NetPortScan(m) => {
                crate::features::net_port_scan::update(&mut self.net_port_scan, m)
                    .map(Message::NetPortScan)
            }
            Message::NetCapture(m) => {
                crate::features::net_capture::update(&mut self.packet_capture, m)
                    .map(Message::NetCapture)
            }
            Message::UiLibs(m) => {
                crate::features::ui_libs::update(&mut self.ui_libs, m)
                    .map(Message::UiLibs)
            }
            // `_ =>` 通配符匹配所有剩余变体（ToggleMenu 等暂未使用的变体）
            _ => Task::none(),
        }
    }

    /// 渲染应用界面 —— 左右两栏布局
    ///
    /// # 布局结构
    /// ```text
    /// ┌──────────────────────────────────────────────────────┐
    /// │  左侧面板 (160px 固定宽度)  │  右侧内容区 (Fill 填充)  │
    /// │  ┌─────────────────────┐   │  ┌─────────────────────┐│
    /// │  │  网络工具  [-]      │   │  │  当前标签页内容      ││
    /// │  │   端口扫描          │   │  │  + Toast 叠加层     ││
    /// │  │   网络抓包          │   │  │                     ││
    /// │  │  数据工具  [-]      │   │  │                     ││
    /// │  │   JSON格式化        │   │  │                     ││
    /// │  │  组件库   [-]       │   │  │                     ││
    /// │  │   组件示例          │   │  │                     ││
    /// │  └─────────────────────┘   │  └─────────────────────┘│
    /// └──────────────────────────────────────────────────────┘
    /// ```
    ///
    /// # Rust: 泛型生命周期 `'a`
    /// `Element<'_, Message>` 中的 `'_` 是匿名生命周期：
    /// Element 的生命周期取决于其引用的数据（app state），
    /// Rust 的借用检查器自动推断正确的生命周期关系。
    /// 简单地理解为：Element 引用了 App 的数据，Element 不能比 App 活得长。
    pub fn view(&self) -> Element<'_, Message> {
        // 构建菜单树 —— 定义左侧导航的结构
        //
        // TreeItem::new(id, label) 创建树节点
        // .child(item) 添加子节点（返回 Self 方便链式调用）
        //
        // 只有叶子节点的点击才触发 TabSelected，
        // 有子节点的节点点击触发 ToggleCategory（展开/折叠）
        let menu_tree = vec![
            TreeItem::new("net", "网络工具")
                .child(TreeItem::new("net_port_scan", "端口扫描"))
                .child(TreeItem::new("net_capture", "网络抓包")),
            TreeItem::new("data", "数据工具")
                .child(TreeItem::new("json_fmt", "JSON格式化")),
            TreeItem::new("ui", "组件库")
                .child(TreeItem::new("ui_libs", "组件示例")),
        ];

        // 当前选中的 tab id，用于菜单高亮
        let selected_id = match self.selected_tab {
            Tab::JsonFmt => "json_fmt",
            Tab::NetPortScan => "net_port_scan",
            Tab::NetCapture => "net_capture",
            Tab::UiLibs => "ui_libs",
        };

        // 递归渲染菜单树
        // `column![]` 是 iced 的宏，用于创建垂直布局
        // `.spacing(0)` 设置子元素间距为 0
        let mut menu_col = column![].spacing(0);
        for item in &menu_tree {
            menu_col = menu_col.push(render_tree_item(item, 0, &self.expanded, selected_id));
        }

        // 左侧面板：固定宽度 160px，高度填充，深色背景
        //
        // iced widget 样式系统：`.style(|_| container::Style { ... })`
        // 闭包接收 `&Theme` 参数，返回样式结构体。
        // `_` 忽略 theme 参数因为我们使用自定义硬编码颜色。
        let menu_panel = container(menu_col)
            .width(Length::Fixed(160.0))
            .height(Length::Fill)
            .style(|_| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(25, 25, 40))),
                ..Default::default()
            });

        // 右侧内容：根据 selected_tab 渲染对应的功能视图
        //
        // 每个 view 函数返回 (Element, Vec<Layered>)：
        // - Element: 主内容
        // - Vec<Layered>: 叠加层（如 Toast 通知），通过锚点定位
        //
        // `(c.map(Message::X), o.into_iter().map(|e| e.map(Message::X)).collect())`
        // .map() 将子模块的 Msg 映射为顶层 Message，因为 iced 的消息类型必须一致。
        // `into_iter().map().collect()` 将 Vec<Layered<子Msg>> 转换为 Vec<Layered<Message>>。
        let (content, overlays): (Element<'_, Message>, Vec<Layered<'_, Message>>) = match self.selected_tab {
            Tab::JsonFmt => {
                let (c, o) = crate::features::json_fmt::view(&self.json_formatter);
                (c.map(Message::JsonFmt), o.into_iter().map(|e| e.map(Message::JsonFmt)).collect())
            }
            Tab::NetPortScan => {
                let (c, o) = crate::features::net_port_scan::view(&self.net_port_scan);
                (c.map(Message::NetPortScan), o.into_iter().map(|e| e.map(Message::NetPortScan)).collect())
            }
            Tab::NetCapture => {
                let (c, o) = crate::features::net_capture::view(&self.packet_capture);
                (c.map(Message::NetCapture), o.into_iter().map(|e| e.map(Message::NetCapture)).collect())
            }
            Tab::UiLibs => {
                let (c, o) = crate::features::ui_libs::view(&self.ui_libs);
                (c.map(Message::UiLibs), o.into_iter().map(|e| e.map(Message::UiLibs)).collect())
            }
        };

        // 用 Layer 叠加系统组合主内容和浮动层
        // 第 0 层: 主内容面板 (Fill)
        // 第 1+ 层: Toast 等浮动元素，通过 Anchor 定位到特定位置
        let content_panel = container(content)
            .width(Length::Fill)
            .height(Length::Fill);

        let main = Layered::new(content_panel.into());
        let mut all_layers = vec![main];
        all_layers.extend(overlays);
        let content_with_overlays: Element<'_, Message> = layer(all_layers);

        // 用 `row!` 宏拼接左右两栏
        // `row!` 是 iced 提供的声明式布局宏，
        // 相当于 `Row::with_children(vec![...])`
        row![menu_panel, content_with_overlays]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// 窗口标题
    ///
    /// iced 在窗口创建和窗口标题需要变化时调用此函数。
    pub fn title() -> String {
        "综合工具".to_string()
    }

    /// 应用主题
    ///
    /// `iced::Theme::Dark` 是内置暗色主题，包含预设的颜色方案。
    /// 返回 Theme::Dark 意味着所有 widget 默认使用暗色风格。
    pub fn theme() -> iced::Theme {
        iced::Theme::Dark
    }
}
