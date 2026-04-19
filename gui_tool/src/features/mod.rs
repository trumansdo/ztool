pub mod json_fmt;
pub mod net_capture;
pub mod net_scan;
pub mod theme;
pub mod ui_libs;

#[derive(Debug, Clone)]
pub enum FeatureMsg {
    None,
    JsonFmt(json_fmt::Msg),
    NetScan(net_scan::Msg),
    NetCapture(net_capture::Msg),
}