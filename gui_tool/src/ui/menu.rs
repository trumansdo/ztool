use crate::ui::widgets::menu_bar::{MenuItem, menu_bar as custom_menu_bar, dropdown};
use crate::ui::widgets::Layered;
use iced::Element;

use super::{Message, Tab};

pub fn menu_items() -> Vec<MenuItem<Message>> {
    vec![
        MenuItem::new("网络工具")
            .item("端口扫描", Message::TabSelected(Tab::NetPortScan))
            .item("网络抓包", Message::TabSelected(Tab::NetCapture)),
        MenuItem::new("数据工具")
            .item("JSON格式化", Message::TabSelected(Tab::JsonFmt)),
        MenuItem::new("组件库")
            .item("组件示例", Message::TabSelected(Tab::UiLibs)),
    ]
}

pub fn menu_bar(open_index: Option<usize>) -> Element<'static, Message> {
    custom_menu_bar(&menu_items(), open_index, Message::ToggleMenu)
}

pub fn view_dropdown(index: usize) -> Option<Layered<'static, Message>> {
    let items = menu_items();
    if index >= items.len() {
        return None;
    }
    let left = match index {
        0 => 10.0,
        1 => 85.0,
        2 => 165.0,
        _ => return None,
    };
    Some(dropdown(items[index].entries(), index, left))
}