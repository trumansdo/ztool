//! 音乐波形可视化 —— 双八度音阶波形与和弦组合
//!
//! - 单音横轴固定显示 ±SOLO_CYCLES 个正弦周期
//! - 组合横轴显示 ±COMBO_CYCLES 个谐波循环周期（以各分量有理逼近 LCM 为基频）
//! - 动画：PERIOD_SECONDS 秒完成一个完整相位循环
//! - 频率比：2^(n/12)，n 为相对 A4 半音数。

pub mod canvas;
pub mod types;
pub mod update;
pub mod view;

pub use types::{Msg, MusicWave};
pub use update::update;
pub use view::view;
