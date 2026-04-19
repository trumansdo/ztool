use iced::Task;

#[derive(Default)]
pub struct NetScanner {
    pub target: String,
    pub results: Vec<String>,
}

impl NetScanner {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum Msg {
    TargetChanged(String),
    StartScan,
    Clear,
}

pub fn update(scanner: &mut NetScanner, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::TargetChanged(s) => scanner.target = s,
        Msg::StartScan => {
            scanner.results.push(format!("扫描: {}", scanner.target));
            scanner.results.push("端口 22: 开放".to_string());
            scanner.results.push("端口 80: 开放".to_string());
            scanner.results.push("端口 443: 开放".to_string());
        }
        Msg::Clear => {
            scanner.results.clear();
            scanner.target.clear();
        }
    }
    Task::none()
}