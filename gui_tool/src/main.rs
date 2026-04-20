//! gui_tool - 综合工具集
//!
//! 基于 iced GUI 框架的多功能桌面工具
//!
//! 调用时序（从外到内，从上层到下层）:
//! main()
//!   └─> ui::run()
//!         └─> iced::application(new, update, view, title, theme)
//!               ├─> new() → App::new() [创建初始状态]
//!               ├─> update(state, msg) → App::update() [处理用户消息]
//!               │     └─> features::xxx::update() [更新功能模块状态]
//!               ├─> view(state) → App::view() [渲染UI]
//!               │     ├─> menu::view_menu_panel() [渲染左侧菜单]
//!               │     │   └─> widgets::render_tree_item() [递归渲染树形菜单]
//!               │     └─> features::xxx::view() [渲染功能内容]
//!               ├─> title() → App::title() [窗口标题]
//!               └─> theme() → App::theme() [应用主题]

mod features;
mod ui;

use anyhow::Context;

/// 程序入口点
/// 使用 anyhow::Result 返回值实现生产级错误处理
fn main() -> anyhow::Result<()> {
    // 调用 ui::run() 启动 iced 应用程序
    // context() 添加错误上下文信息
    // ? 操作符传播错误给调用者
    ui::run().context("Failed to run application")?;
    Ok(())
}