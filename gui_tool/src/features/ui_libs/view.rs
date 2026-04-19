use iced::widget::{button, Column, Text};
use iced::Element;

use super::{Msg, UiLibs};

pub fn view(libs: &UiLibs) -> Element<'_, Msg> {
    let col = Column::new()
        .push(Text::new("iced UI 示例").size(24))
        .push(Text::new(""))
        .push(Text::new("点击次数:").size(18))
        .push(Text::new(format!("{}", libs.click_count)).size(32))
        .push(Text::new(""))
        .push(button("点击增加").on_press(Msg::ButtonPressed))
        .spacing(10)
        .padding(20);

    col.into()
}