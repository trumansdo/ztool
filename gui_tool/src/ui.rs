pub mod app;
pub mod menu;
pub mod widgets;

pub use app::App;

/// Tab：当前选中的功能页面
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tab {
    #[default]
    JsonFmt,        // JSON格式化
    NetPortScan,   // 端口扫描
    NetCapture,    // 网络抓包
    UiLibs,       // 组件示例
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

/// 消息类型：桥接各功能模块的消息
#[derive(Debug, Clone)]
pub enum Message {
    ToggleCategory(String),      // 展开/收起分类
    TabSelected(Tab),             // 选中功能页面
    JsonFmt(crate::features::json_fmt::Msg),         // JSON格式化消息
    NetPortScan(crate::features::net_port_scan::Msg), // 端口扫描消息
    NetCapture(crate::features::net_capture::Msg),    // 网络抓包消息
    UiLibs(crate::features::ui_libs::Msg),           // 组件示例消息
}

use anyhow::Result;
use iced::{Element, Task};

/// 创建初始应用状态
fn new() -> (App, Task<Message>) {
    (App::new(), Task::none())
}

/// 处理消息，更新应用状态
fn update(state: &mut App, message: Message) -> Task<Message> {
    state.update(message)
}

/// 渲染应用视图
fn view(state: &App) -> Element<'_, Message> {
    state.view()
}

/// 获取窗口标题
fn title(_state: &App) -> String {
    App::title()
}

/// 获取主题
fn theme(_state: &App) -> iced::Theme {
    App::theme()
}

/// 启动iced应用程序
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