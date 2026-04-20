use iced::widget::{container, Column};
use iced::Element;

use super::{Message, Tab};
use super::widgets::{render_tree_item, TreeItem};

/// 渲染左侧菜单面板
pub fn view_menu_panel(
    expanded_categories: &std::collections::HashSet<String>,
    selected: Tab,
) -> Element<'static, Message> {
    // 定义菜单树结构
    let items = [
        TreeItem::new("net_tools", "网络工具")
            .child(TreeItem::new("net_port_scan", "端口扫描"))
            .child(TreeItem::new("net_capture", "网络抓包")),
        TreeItem::new("data_tools", "数据工具").child(
            TreeItem::new("json_fmt", "JSON格式化"),
        ),
        TreeItem::new("ui_libs", "组件库").child(
            TreeItem::new("ui_libs_page", "组件示例"),
        ),
    ];

    // 获取当前选中的ID
    let selected_id: String = selected.into();

    // 垂直排列所有菜单项
    let mut col = Column::new().spacing(0);
    for item in &items {
        col = col.push(render_tree_item(item, 0, expanded_categories, &selected_id));
    }

    // 包装成固定宽度容器
    container(col)
        .width(iced::Length::Fixed(150.0))
        .into()
}