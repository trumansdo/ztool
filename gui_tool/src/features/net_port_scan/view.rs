//! 端口扫描 —— 视图层
//!
//! 渲染端口扫描工具的 UI：目标输入、模式选择、扫描按钮、结果和日志区域。
//!
//! # UI 结构
//! ```text
//! ┌─────────────────────────────────┐
//! │ 端口扫描                         │
//! │ [目标IP输入框                   ]│
//! │ [模式选择▼] [扫描] [清空]        │
//! │ 扫描结果:                        │
//! │ ┌─────────────────────────────┐│
//! │ │ 192.168.1.1:22 开放 - SSH    ││
//! │ └─────────────────────────────┘│
//! │ 扫描日志:                        │
//! │ ┌─────────────────────────────┐│
//! │ │ [*] 开始扫描...              ││
//! │ └─────────────────────────────┘│
//! └─────────────────────────────────┘
//! ```

use crate::features::theme;
use crate::features::net_port_scan::{ScanMode, Msg, NetScanner};
use crate::ui::widgets::Layered;
use iced::widget::{button, column, container, row, text, text_input, pick_list, scrollable};
use iced::Element;
use iced::Length;

/// 渲染端口扫描界面
///
/// # Rust: `text_input` —— 单行文本输入
/// `text_input(placeholder, &content).on_input(msg)` 创建输入框。
/// `placeholder` 在输入为空时显示灰色提示文字。
/// 宽度设为 `Length::Fill` 以填充可用空间（适应不同窗口大小）。
///
/// # Rust: `pick_list` —— 下拉选择器
/// `pick_list(options, selected, on_selected)` 创建下拉列表。
/// 第二个参数 `Some(scanner.scan_mode)` 是当前选中项（None 表示未选择）。
pub fn view(scanner: &NetScanner) -> (Element<'_, Msg>, Vec<Layered<'_, Msg>>) {
    let input = text_input("目标IP/网段 (如: 192.168.1.1, 192.168.1.1-10, 192.168.1.0/24)", &scanner.target)
        .on_input(Msg::TargetChanged)
        .padding(theme::padding2(0.36, 1.0))
        .width(Length::Fill);

    // 扫描模式下拉列表
    // `vec!` 宏创建 Vec
    let modes: Vec<ScanMode> = vec![ScanMode::Common, ScanMode::Top100, ScanMode::All];
    let mode_list = pick_list(
        modes,
        Some(scanner.scan_mode),
        Msg::ScanModeChanged,
    );

    // 扫描按钮：根据扫描状态切换文字和样式
    //
    // Rust: `if` 作为表达式直接赋值
    let scan_btn = if scanner.is_scanning {
        button("扫描中...")
            .padding(theme::padding2(0.36, 0.5))
            .style(button::secondary)  // 无 on_press —— 按钮被禁用
    } else {
        button("扫描")
            .on_press(Msg::StartScan)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::primary)
    };

    // 扫描结果文本
    let results_text = if scanner.results.is_empty() {
        "暂无结果".to_string()
    } else {
        scanner.results.join("\n")
    };

    // 扫描进度文本（仅在扫描中显示）
    let progress_text: String = if scanner.is_scanning && scanner.total_ports > 0 {
        format!("已扫描 {}/{} 端口", scanner.scanned_count, scanner.total_ports)
    } else {
        String::new()  // 空字符串，不显示
    };

    // 扫描结果区域（带滚动条，自动扩展高度）
    //
    // `scrollable(text(...))` 创建可滚动的文本区域。
    // `container(...).height(Length::Fill)` 让结果区域占据剩余空间。
    let results_col = column![
        text("扫描结果:").size(theme::font(1.0)),
        container(
            scrollable(
                text(results_text)
                    .size(theme::font(0.9))
            )
        )
        .height(Length::Fill)
        .padding(theme::padding(0.5))
    ];

    // 扫描日志文本
    let logs_text = if scanner.logs.is_empty() {
        "等待扫描...".to_string()
    } else {
        scanner.logs.join("\n")
    };

    // 扫描日志区域（固定高度 150px + 滚动条，自动滚到底部）
    let logs_col = column![
        text("扫描日志:").size(theme::font(1.0)),
        container(
            scrollable(
                text(logs_text)
                    .size(theme::font(0.8))
            )
            .anchor_bottom()  // 始终显示最新日志（滚动到底部）
        )
        .height(Length::Fixed(150.0))
        .padding(theme::padding(0.5))
    ];

    // 按钮行
    let btn_row = row![
        scan_btn,
        button("清空")
            .on_press(Msg::Clear)
            .padding(theme::padding2(0.36, 0.5))
            .style(button::secondary),
    ]
    .spacing(theme::size(0.86).0 as u32);

    // 主容器：垂直排列所有组件
    let content = container(
        column![
            text("端口扫描").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),
            input,
            text("").size(theme::font(0.3)),
            row![mode_list, btn_row]
                .spacing(theme::size(0.86).0 as u32),
            text("").size(theme::font(0.5)),
            results_col,
            text("").size(theme::font(0.3)),
            text(progress_text).size(theme::font(0.8)),
            text("").size(theme::font(0.3)),
            logs_col,
        ]
        .spacing(theme::size(0.5).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0))
    .height(Length::Fill)
    .into();

    // 端口扫描页面没有 Toast 叠加层
    (content, vec![])
}
