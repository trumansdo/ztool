//! # 菜单树数据模型层
//!
//! 定义菜单的静态拓扑结构，对应 libcosmic 菜单系统架构中的**数据模型层**。
//!
//! ## 架构定位
//!
//! ```text
//! 数据模型层 (本文件)         → 编译期确定的菜单树结构
//! 状态控制层 (menu_inner.rs)  → 运行时激活路径与交互状态
//! 渲染表现层 (menu_inner.rs)  → 布局计算、绘制、事件拦截
//! ```
//!
//! ## 设计要点
//!
//! - **递归引用**：`Folder.children` 通过 `Vec<MenuTree>` 实现树形结构的深度递归
//! - **静态定义**：菜单结构在视图构建时确定，运行时仅做只读遍历，保证数据稳定性
//! - **消息直通**：`Item.action` 直接持有 `Message`（本项目中为 `UiLibs::Msg`），
//!   点击菜单项时由 Overlay 层直接 `shell.publish(action)`，无需中间映射
//!
//! # Rust 语法要点
//!
//! ## 泛型枚举 (Generic Enum)
//! ```text
//! enum MenuTree<'a, Message> { Item(Item<'a, Message>), ... }
//! ```
//! - `Message`: 泛型消息类型，与 iced 的 `Element<Message>` 泛型一致
//! - `'a`: 生命周期参数，用于 `icon: Option<&'a str>` 借用图标字符字面量
//!   本项目中菜单数据整体以 `'static` 生命周期使用（标签/图标均为字面量）
//!
//! ## 枚举携带数据 (ADT)
//! `Item(Item<'a, Message>)` 是"元组变体"，携带一个结构体；
//! `Separator` 是"单元变体"，不携带数据 —— 与 C 语言枚举不同，
//! Rust 枚举的每个变体可以携带不同类型的数据。

/// 菜单树节点枚举，支持递归定义
///
/// 三个变体对应菜单的三种拓扑角色：
/// - [`Item`]：叶子节点，可执行动作
/// - [`Folder`]：分支节点，包含子菜单（递归）
/// - [`Separator`]：分隔符，仅做视觉分隔
#[derive(Debug, Clone)]
pub enum MenuTree<'a, Message> {
    /// 叶子节点：可执行动作
    Item(Item<'a, Message>),
    /// 分支节点：包含子菜单（递归持有 `Vec<MenuTree>` 实现无限层级）
    Folder(Folder<'a, Message>),
    /// 分隔符
    Separator,
}

/// 动作项（叶子节点）定义
#[derive(Debug, Clone)]
pub struct Item<'a, Message> {
    /// 菜单项显示文本
    pub label: String,
    /// 点击后发送的消息；`None` 表示点击无动作
    pub action: Option<Message>,
    /// 图标字符（如 "📄"、"❓"），`None` 表示无图标
    pub icon: Option<&'a str>,
    /// 是否可用；`false` 时灰显且不可点击
    pub enabled: bool,
}

/// 文件夹（子菜单）定义
#[derive(Debug, Clone)]
pub struct Folder<'a, Message> {
    /// 文件夹显示文本
    pub label: String,
    /// 图标字符，`None` 表示无图标
    pub icon: Option<&'a str>,
    /// 递归持有子节点，实现无限层级
    pub children: Vec<MenuTree<'a, Message>>,
}

impl<'a, Message> MenuTree<'a, Message> {
    /// 创建叶子节点（动作项）
    ///
    /// # Rust: `impl Into<String>` 参数
    /// 接受任何可转换为 `String` 的类型（`&str`、`String` 等），
    /// 调用 `label.into()` 统一转为 `String` 存储。
    pub fn item(label: impl Into<String>, action: Message) -> Self {
        Self::Item(Item {
            label: label.into(),
            action: Some(action),
            icon: None,
            enabled: true,
        })
    }

    /// 创建分支节点（子菜单）
    ///
    /// # Rust: `Vec<MenuTree<'a, Message>>` 所有权
    /// 子节点列表以值传递（move），`Folder` 拥有其所有子节点，
    /// 无需引用计数（`Rc`）即可表达树形所有权结构。
    pub fn folder(label: impl Into<String>, children: Vec<MenuTree<'a, Message>>) -> Self {
        Self::Folder(Folder {
            label: label.into(),
            icon: None,
            children,
        })
    }

    /// 创建分隔符
    pub fn separator() -> Self {
        Self::Separator
    }

    /// 获取节点的显示文本；分隔符无文本，返回 `None`
    pub fn label(&self) -> Option<&str> {
        match self {
            MenuTree::Item(item) => Some(&item.label),
            MenuTree::Folder(folder) => Some(&folder.label),
            MenuTree::Separator => None,
        }
    }

    /// 是否为分支节点（有子菜单）
    pub fn is_folder(&self) -> bool {
        matches!(self, MenuTree::Folder(_))
    }

    /// 是否为分隔符
    pub fn is_separator(&self) -> bool {
        matches!(self, MenuTree::Separator)
    }

    /// 设置图标字符（链式构建器，委托到内部 Item/Folder）
    ///
    /// 示例：`MenuTree::item("新建", msg).icon("📄")`
    pub fn icon(mut self, icon: &'a str) -> Self {
        match &mut self {
            MenuTree::Item(item) => item.icon = Some(icon),
            MenuTree::Folder(folder) => folder.icon = Some(icon),
            MenuTree::Separator => {}
        }
        self
    }

    /// 设置是否可用（链式构建器，委托到内部 Item）
    ///
    /// 示例：`MenuTree::item("保存", msg).enabled(false)` 灰显不可点击。
    /// 仅对 Item 生效；Folder/Separator 忽略。
    pub fn enabled(mut self, enabled: bool) -> Self {
        if let MenuTree::Item(item) = &mut self {
            item.enabled = enabled;
        }
        self
    }
}

impl<'a, Message> Item<'a, Message> {
    /// 设置图标字符（链式构建器模式）
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 设置是否可用（链式构建器模式）
    ///
    /// 用于示例：`MenuTree::item("保存", msg).enabled(false)` 会渲染为灰显。
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<'a, Message> Folder<'a, Message> {
    /// 设置图标字符（链式构建器模式）
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
}
