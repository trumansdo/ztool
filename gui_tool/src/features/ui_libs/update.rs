use iced::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComponentTab {
    #[default]
    Badge,
    Card,
    Button,
    Toggle,
}

#[derive(Debug, Clone)]
pub enum Msg {
    TabSelected(ComponentTab),
    ButtonPressed,
    ToggleChanged(bool),
}

#[derive(Default)]
pub struct UiLibs {
    pub selected_tab: ComponentTab,
    pub click_count: u32,
    pub toggle_value: bool,
}

impl UiLibs {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn update(libs: &mut UiLibs, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::TabSelected(tab) => {
            libs.selected_tab = tab;
        }
        Msg::ButtonPressed => {
            libs.click_count += 1;
        }
        Msg::ToggleChanged(value) => {
            libs.toggle_value = value;
        }
    }
    Task::none()
}