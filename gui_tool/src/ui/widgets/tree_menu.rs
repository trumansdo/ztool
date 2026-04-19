use iced::{
    widget::{container, mouse_area, text, Column, Row, container::Style},
    Color, Element, Length,
};

use crate::ui::{App, Message, Tab};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TreeItem {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, item: TreeItem) -> Self {
        self.children.push(item);
        self
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

pub fn render_tree_item(
    item: &TreeItem,
    level: usize,
    expanded: &std::collections::HashSet<String>,
    selected: &str,
) -> Element<'static, Message> {
    use iced::Alignment;

    let is_expanded = expanded.contains(&item.id);
    let is_selected = selected == &item.id;
    let has_children = item.has_children();

    let bg = if is_selected {
        Color::from_rgb8(40, 80, 120)
    } else {
        Color::from_rgb8(35, 35, 50)
    };

    let indent = level * 10;
    let indent_str = " ".repeat(indent / 4);

    let icon = if has_children {
        if is_expanded { "-" } else { "+" }
    } else {
        " "
    };
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
    .on_press(if has_children {
        super::Message::ToggleCategory(item.id.clone())
    } else {
        super::Message::TabSelected(match item.id.as_str() {
            "json_fmt" => super::Tab::JsonFmt,
            "net_scan" => super::Tab::NetScan,
            "net_capture" => super::Tab::NetCapture,
            "ui_libs_page" => super::Tab::UiLibs,
            _ => super::Tab::JsonFmt,
        })
    })
    .into();

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