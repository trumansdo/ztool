use iced::Task;

#[derive(Default)]
pub struct PacketCapture {
    pub interface: String,
    pub filter: String,
    pub results: Vec<String>,
}

impl PacketCapture {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    InterfaceChanged(String),
    FilterChanged(String),
    StartCapture,
    StopCapture,
}

pub fn update(capture: &mut PacketCapture, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::InterfaceChanged(s) => capture.interface = s,
        Msg::FilterChanged(s) => capture.filter = s,
        Msg::StartCapture => {
            capture.results.push(format!("开始抓包: {}", capture.interface));
            capture.results.push(format!("过滤器: {}", capture.filter));
            capture.results.push("正在捕获数据包...".to_string());
        }
        Msg::StopCapture => {
            capture.results.push("抓包已停止".to_string());
        }
    }
    Task::none()
}