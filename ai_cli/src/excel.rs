//! Excel 读写模块
//!
//! 接入 calamine(读)、rust_xlsxwriter(写)、umya-spreadsheet(读写) 三个引擎，
//! 根据操作类型和参数自动选择最优引擎。

pub mod engine;
pub mod reader;
pub mod writer;
