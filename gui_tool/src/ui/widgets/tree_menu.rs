use iced::{
    widget::{container, mouse_area, text, Column, Row, container::Style},
    Color, Element, Length,
};

use crate::ui::{Message, Tab};

/// 树形菜单项数据结构
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TreeItem {
    pub id: String,                // 唯一标识
    pub label: String,              // 显示文本
    pub children: Vec<TreeItem>,  // 子项列表
}

impl TreeItem {
    /// 创建新的树形项
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
        }
    }

    /// 添加子项
    pub fn child(mut self, item: TreeItem) -> Self {
        self.children.push(item);
        self
    }

    /// 判断是否有子项
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// 渲染单个树形菜单项（递归实现）
pub fn render_tree_item(
    item: &TreeItem,
    level: usize,
    expanded: &std::collections::HashSet<String>,
    selected: &str,
) -> Element<'static, Message> {
    use iced::Alignment;

    // 判断展开/选中状态
    let is_expanded = expanded.contains(&item.id);
    let is_selected = selected == &item.id;
    let has_children = item.has_children();

    // 根据选中状态设置背景色
    let bg = if is_selected {
        Color::from_rgb8(40, 80, 120)
    } else {
        Color::from_rgb8(35, 35, 50)
    };

    // 计算缩进
    let indent = level * 10;
    let indent_str = " ".repeat(indent / 4);

    // 展开/收起图标：有子项显示+/-，无子项显示空格
    let icon = if has_children {
        if is_expanded { "-" } else { "+" }
    } else {
        " "
    };
    let label = item.label.clone();

    // 图标列
    let icon_col = container(text(icon).size(14.0))
        .width(Length::Fixed(16.0))
        .align_x(Alignment::Center);

    // 标签列
    let label_col = container(text(label).size(14.0))
        .width(Length::Fill)
        .align_x(Alignment::Start);

    // 水平排列：缩进+图标+标签
    let row = Row::with_children(vec![
        text(indent_str).size(14.0).into(),
        icon_col.into(),
        label_col.into(),
    ])
    .spacing(0)
    .align_y(Alignment::Center);

    // 确定点击消息：有子项→展开/收起，无子项→切换Tab
    let msg = if has_children {
        Message::ToggleCategory(item.id.clone())
    } else {
        Message::TabSelected(match item.id.as_str() {
            "json_fmt" => Tab::JsonFmt,
            "net_port_scan" => Tab::NetPortScan,
            "net_capture" => Tab::NetCapture,
            "ui_libs_page" => Tab::UiLibs,
            _ => Tab::JsonFmt,
        })
    };

    // 创建可点击的菜单项元素
    let item_el: Element<'static, Message> = mouse_area(
        container(row)
            .width(Length::Fill)
            .height(Length::Fixed(24.0))
            .padding(2)
            .style(move |_| Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            })
    )
    .on_press(msg)
    .into();

    // 如果有子项且已展开，递归渲染子项
    if has_children && is_expanded {
        let mut col = Column::new().spacing(0);
        col = col.push(item_el);
        for child in &item.children {
            col = col.push(render_tree_item(child, level + 1, expanded, selected));
        }
        col.into()
    } else {
        item_el
    }
}