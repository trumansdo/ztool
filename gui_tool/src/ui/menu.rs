use iced::widget::{container, Column};
use iced::Element;

use super::{Message, Tab};
use super::widgets::{render_tree_item, TreeItem};

pub fn view_menu_panel(
    expanded_categories: &std::collections::HashSet<String>,
    selected: Tab,
) -> Element<'static, Message> {
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

    let selected_id: String = selected.into();

    let mut col = Column::new().spacing(0);
    for item in &items {
        col = col.push(render_tree_item(item, 0, expanded_categories, &selected_id));
    }

    container(col)
        .width(iced::Length::Fixed(150.0))
        .into()
}