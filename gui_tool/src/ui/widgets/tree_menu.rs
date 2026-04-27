//! 树形导航菜单组件
//!
//! 递归渲染左侧边栏的层级菜单，支持：
//! - 分类节点：点击切换展开/折叠，多个分类可同时展开
//! - 叶子节点：点击切换到对应功能页面，高亮显示当前选中项
//!
//! # Rust 语法要点
//!
//! ## `derive` 宏对比
//! ```text
//! #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//! pub struct TreeItem { ... }
//! ```
//! 每个 trait 的作用：
//! - `Debug`: 生成 `"{:?}"` 格式化输出，用于调试
//! - `Clone`: 生成 `.clone()` 方法，深拷贝（String 的 clone 会复制整个字符串）
//! - `PartialEq` + `Eq`: 全等比较，`item1 == item2` 可用于比较/断言
//! - `Hash`: 可作 HashMap/HashSet 的键（但这里 TreeItem 不会被放进 HashSet）
//!
//! ## 构造函数模式
//! ```text
//! TreeItem::new("id", "标签").child(子节点1).child(子节点2)
//! ```
//! - `new()` 是静态工厂方法（关联函数）
//! - `.child()` 返回 Self，支持链式调用（Builder 模式）
//!
//! ## `impl Into<String>` vs `impl Into<String> + 'static`
//! `new(id: impl Into<String>, label: impl Into<String>)`
//! - `impl Into<String>`: 接受任何可转为 String 的类型（&str, String, ...)
//! - 调用 `id.into()` 执行转换
//!
//! ## 递归函数
//! `render_tree_item()` 在渲染有子节点的展开节点时递归调用自身。
//! Rust 支持递归，但需要注意栈溢出的风险（这里树深度不超过 3，安全）。
//!
//! ## `'static` 生命周期
//! `Element<'static, Message>` 中的 `'static` 表示该元素不借用任何临时数据，
//! 可以存活到程序结束。菜单上的文本是编译时确定的 &'static str。

use iced::{
    widget::{container, mouse_area, text, Column, Row, container::Style},
    Color, Element, Length,
};

use crate::ui::{Message, Tab};

/// 树节点定义
///
/// # 数据结构选择
/// 用 `Vec<TreeItem>` 而非递归的引用计数树（Rc<T> 等）：
/// - `TreeItem` 拥有其子节点（move 语义），生命周期简单
/// - 树结构小（约 10 个节点），clone 开销可忽略
/// - 不需要共享所有权或内部可变性
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TreeItem {
    /// 节点唯一标识符，对应 Tab 的 id（如 "json_fmt", "net_port_scan"）
    pub id: String,
    /// 节点显示文本（中文标签）
    pub label: String,
    /// 子节点列表
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    /// 创建新的树节点
    ///
    /// # Rust: 泛型参数 `impl Into<String>`
    /// 这不叫 "泛型函数" —— 而是 "impl Trait 语法糖"。
    /// 编译器会为每个不同的调用类型生成单态化代码。
    /// `id.into()` 调用 `Into<String>::into()` 将参数转为 String。
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
        }
    }

    /// 添加子节点（返回 Self 以支持链式调用）
    pub fn child(mut self, item: TreeItem) -> Self {
        self.children.push(item);
        self
    }

    /// 是否有子节点
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// 递归渲染树节点
///
/// # 参数
/// - `item`: 当前渲染的节点
/// - `level`: 缩进层级（0 = 顶层，1 = 子项...）
/// - `expanded`: 已展开分类的 id 集合
/// - `selected`: 当前选中叶子节点的 id
///
/// # 递归逻辑
/// 1. 渲染当前节点（缩进 + 图标 + 文本 + 背景色）
/// 2. 如果当前节点有子节点且已展开，递归渲染每个子节点
/// 3. 返回 Column 包含自身和所有子项
///
/// # Rust: 函数不返回值时的返回类型
/// 看起来没有显式写 `->`，但 iced 的 Column 实现了 `Into<Element>`。
/// `col.into()` 将 Column 转为 Element 并作为最后表达式返回。
pub fn render_tree_item(
    item: &TreeItem,
    level: usize,
    expanded: &std::collections::HashSet<String>,
    selected: &str,
) -> Element<'static, Message> {
    use iced::Alignment;

    // `HashSet::contains()` 查询集合中是否包含某个值
    let is_expanded = expanded.contains(&item.id);
    // 字符串切片可以直接用 `==` 比较（实现了 PartialEq）
    let is_selected = selected == &item.id;
    let has_children = item.has_children();

    // 根据选中状态设置背景色
    // Rust: if 是表达式，可以直接赋值
    // `Color::from_rgb8(r, g, b)` 接收 0~255 范围的整数
    let bg = if is_selected {
        Color::from_rgb8(40, 80, 120)  // 蓝色高亮
    } else {
        Color::from_rgb8(35, 35, 50)    // 暗色底色
    };

    // 缩进计算：每层缩进 10px，用空格字符串模拟
    let indent = level * 10;
    // `" ".repeat(n)` 生成 n 个空格的 String
    let indent_str = " ".repeat(indent / 4);

    // 图标：有子节点的显示 +/-，叶子节点显示空格（无图标）
    let icon = if has_children {
        if is_expanded { "-" } else { "+" }
    } else {
        " "
    };

    // 菜单项的行内容：缩进 | 图标 | 文本
    //
    // Rust: `item.label.clone()` — String 的 clone 会复制堆上的字符数据
    let label = item.label.clone();
    let icon_col = container(text(icon).size(14.0))
        .width(Length::Fixed(16.0))
        .align_x(Alignment::Center);

    let label_col = container(text(label).size(14.0))
        .width(Length::Fill)
        .align_x(Alignment::Start);

    let row = Row::with_children(vec![
        text(indent_str).size(14.0).into(),
        icon_col.into(),
        label_col.into(),
    ])
    .spacing(0)
    .align_y(Alignment::Center);

    // 根据是否有子节点，确定点击行为
    //
    // Rust: `if` 作为表达式
    // `item.id.clone()` — String 的 clone 创建新的堆分配字符串
    let msg = if has_children {
        Message::ToggleCategory(item.id.clone())
    } else {
        // `Tab::from_id()` 返回 Option<Tab>
        // `.unwrap_or(Tab::JsonFmt)` — 提取 Some 值，None 时间退到默认
        Message::TabSelected(Tab::from_id(&item.id).unwrap_or(Tab::JsonFmt))
    };

    // `mouse_area(container)` 创建一个可点击区域
    // `.on_press(msg)` 点击时发送消息
    // `.into()` 将 mouse_area 转换为 Element
    let item_el: Element<'static, Message> = mouse_area(
        container(row)
            .width(Length::Fill)
            .height(Length::Fixed(24.0))
            .padding(2)
            // `move |_|` 闭包 —— move 关键字让闭包获取 bg 的所有权
            // 这对于 'static 生命周期是必要的
            .style(move |_| Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            })
    )
    .on_press(msg)
    .into();

    // 如果有子节点且已展开，递归渲染子节点
    if has_children && is_expanded {
        let mut col = Column::new().spacing(0);
        col = col.push(item_el);
        for child in &item.children {
            // 递归调用 —— 每个子节点 level + 1
            col = col.push(render_tree_item(child, level + 1, expanded, selected));
        }
        col.into()
    } else {
        item_el
    }
}
