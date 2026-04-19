pub mod app;
pub mod menu;
pub mod widgets;

pub use app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tab {
    #[default]
    JsonFmt,
    NetPortScan,
    NetCapture,
    UiLibs,
}

impl From<Tab> for String {
    fn from(t: Tab) -> Self {
        match t {
            Tab::JsonFmt => "json_fmt".to_string(),
            Tab::NetPortScan => "net_port_scan".to_string(),
            Tab::NetCapture => "net_capture".to_string(),
            Tab::UiLibs => "ui_libs_page".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleCategory(String),
    TabSelected(Tab),
    JsonFmt(crate::features::json_fmt::Msg),
    NetPortScan(crate::features::net_port_scan::Msg),
    NetCapture(crate::features::net_capture::Msg),
    UiLibs(crate::features::ui_libs::Msg),
}

use anyhow::Result;
use iced::{Element, Task};

fn new() -> (App, Task<Message>) {
    (App::new(), Task::none())
}

fn update(state: &mut App, message: Message) -> Task<Message> {
    state.update(message)
}

fn view(state: &App) -> Element<'_, Message> {
    state.view()
}

fn title(_state: &App) -> String {
    App::title()
}

fn theme(_state: &App) -> iced::Theme {
    App::theme()
}

pub fn run() -> Result<()> {
    iced::application(new, update, view)
        .title(title)
        .theme(theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1000.0, 700.0),
            min_size: Some(iced::Size::new(800.0, 600.0)),
            resizable: true,
            ..iced::window::Settings::default()
        })
        .run()?;
    Ok(())
}