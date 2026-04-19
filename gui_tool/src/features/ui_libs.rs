use iced::widget::{button, row, Column, Text};
use iced::Element;

#[derive(Debug, Clone)]
pub enum Msg {
    ButtonPressed,
}

pub struct UiLibs {
    click_count: u32,
}

impl Default for UiLibs {
    fn default() -> Self {
        Self::new()
    }
}

impl UiLibs {
    pub fn new() -> Self {
        Self { click_count: 0 }
    }

    pub fn update(&mut self, message: Msg) {
        match message {
            Msg::ButtonPressed => {
                self.click_count += 1;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Msg> {
        let col = Column::new()
            .push(Text::new("iced UI 示例").size(24))
            .push(Text::new(""))
            .push(Text::new("点击次数:").size(18))
            .push(Text::new(format!("{}", self.click_count)).size(32))
            .push(Text::new(""))
            .push(button("点击增加").on_press(Msg::ButtonPressed))
            .spacing(10)
            .padding(20);

        col.into()
    }
}