use iced::Task;

#[derive(Debug, Clone)]
pub enum Msg {
    ButtonPressed,
}

pub struct UiLibs {
    pub click_count: u32,
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
}

pub fn update(libs: &mut UiLibs, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::ButtonPressed => {
            libs.click_count += 1;
        }
    }
    Task::none()
}