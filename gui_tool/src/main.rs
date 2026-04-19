//! gui_tool - 综合工具集
//!
//! 基于 iced GUI 框架的多功能桌面工具

mod features;
mod ui;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    ui::run().context("Failed to run application")?;
    Ok(())
}