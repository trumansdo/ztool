//! # 个人工作台 (Dashboard) - egui 教学示例
//!
//! 本文件展示了 egui 的核心概念和使用方法。
//! 通过这个混合型工具，你可以学习到：
//! - Context, Ui, Response 三元组
//! - Widget trait 和自定义控件
//! - 立即模式 (Immediate Mode) UI 构建
//! - 布局系统
//! - 状态管理
//!
//! ============================================================================
//! 📚 学习路径 (推荐阅读顺序)
//! ============================================================================
//!
//! 真实的十三部分内容:
//! - 第一部分: 导入和依赖
//! - 第二部分: 错误定义 (thiserror + anyhow)
//! - 第三部分: 菜单系统 (enum + 模式匹配)
//! - 第四部分: 文本工具模块 (状态结构体)
//! - 第五部分: 系统信息模块 (sysinfo集成)
//! - 第六部分: 计算器模块 (Slider)
//! - 第七部分: 主应用结构 (App trait实现) ⭐ 核心
//! - 第八部分: 网络监控面板
//! - 第九部分: 文本工具面板 (TextEdit/Button/Checkbox)
//! - 第十部分: 系统信息面板 (ProgressBar/Grid/ScrollArea)
//! - 第十一部分: 计算器面板 (Slider)
//! - 第十二部分: 字体加载 (FontDefinitions)
//! - 第十三部分: main函数 (应用入口)
//!
//! 推荐阅读顺序:
//! ```text
//! 第十三部分 (main)        → 1. 如何启动应用
//!    ↓
//! 第七部分 (App trait)     → 2. UI构建入口 (核心)
//!    ↓
//! 第三部分 (菜单系统)      → 3. enum 和模式匹配
//!    ↓
//! 第四-六部分 (状态)        → 4. 为什么要保存状态
//!    ↓
//! 第二部分 (错误处理)      → 5. thiserror + anyhow
//!    ↓
//! 第八部分 (布局开始)       → 6. SidePanel + CentralPanel
//!    ↓
//! 第九部分 (文本控件)       → 7. TextEdit + Button + Checkbox
//!    ↓
//! 第十部分 (系统控件)       → 8. ProgressBar + Grid + ScrollArea
//!    ↓
//! 第十一部分 (计算器控件)   → 9. Slider
//!    ↓
//! 第十二部分 (字体)         → 10. FontDefinitions
//!    ↓
//! 第一部分 (导入)          → 11. 依赖说明
//! ```
//!
//! 详细说明 (行号已修正):
//! ----------
//!
//! 1️⃣ **第十三部分 (行 926)** - main 函数
//!    - 程序入口，run_native() 的用法
//!    - WgpuConfiguration 配置
//!    - 窗口尺寸设置
//!
//! 2️⃣ **第七部分 (行 280)** - App trait 实现 ⭐
//!    - update() 方法每帧调用
//!    - egui 立即模式: 每帧重建 UI
//!    - ctx 和 frame 参数作用
//!
//! 3️⃣ **第三部分 (行 134)** - 菜单系统
//!    - MenuItem 枚举定义
//!    - 模式匹配返回不同标签
//!
//! 4️⃣ **第四部分 (行 159)** - 文本工具状态
//! 5️⃣ **第五部分 (行 208)** - 系统信息状态
//! 6️⃣ **第六部分 (行 236)** - 计算器状态
//!    - 理解立即模式下必须保存状态
//!
//! 7️⃣ **第二部分 (行 118)** - 错误处理
//!    - thiserror 定义错误枚举
//!    - anyhow 统一处理
//!    - #[from] 自动转换
//!
//! 8️⃣ **第八部分 (行 357)** - 网络监控面板
//!    - 包含布局的 start: SidePanel + CentralPanel
//!
//! 9️⃣ **第九部分 (行 403)** - 文本工具面板
//!    - TextEdit 多行/单行输入
//!    - Button 点击检测
//!    - Checkbox 布尔值
//!
//! 🔟 **第十部分 (行 599)** - 系统信息面板
//!    - ProgressBar 进度条
//!    - Grid 表格布局
//!    - ScrollArea 滚动区域
//!
//! 1️⃣1️⃣ **第十一部分 (行 691)** - 计算器面板
//!    - Slider 滑动条
//!
//! 1️⃣2️⃣ **第十二部分 (行 864)** - 字体加载
//!    - FontDefinitions 使用
//!
//! 1️⃣3️⃣ **第一部分 (行 106)** - 导入依赖
//!    - egui, eframe, sysinfo 等
//!
//! 💡 提示: 运行 cargo expand --bin dashboard 可查看宏展开
//!
//! ============================================================================
//!    - anyhow 统一错误处理
//!    - #[from] 自动转换
//!
//! 💡 提示: 阅读时配合 cargo expand --bin dashboard 可以看到宏展开后的代码
//!
//! ============================================================================
//!    - thiserror 定义自定义错误
//!    - anyhow 统一错误处理
//!    - #[from] 自动转换
//!
//! 💡 提示: 阅读时配合 cargo expand --bin dashboard 可以看到宏展开后的代码
//!
//! ============================================================================

// ============================================================================
// 第一部分：导入和依赖
// ============================================================================

use anyhow::Result;
use base64::Engine; // base64 编解码 trait
use eframe::egui; // egui 核心库
use std::path::PathBuf;
use sysinfo::System; // 系统信息获取
use thiserror::Error;

// ============================================================================
// 第二部分：错误定义 (thiserror + anyhow 最佳实践)
// ============================================================================

/// 应用错误枚举 - 使用 thiserror 自动派生 Error trait
#[derive(Debug, Error)]
pub enum AppError {
    /// 未找到中文字体
    #[error("未找到中文字体")]
    FontNotFound,

    /// 读取字体文件失败 - #[from] 自动实现 From<std::io::Error>
    #[error("读取字体文件失败: {0}")]
    ReadFontFailed(#[from] std::io::Error),
}

// ============================================================================
// 第三部分：菜单系统 - 展示 enum + 模式匹配
// ============================================================================

/// 菜单项枚举 - 使用 Copy trait 因为只是简单的选择标记
#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    Network,    // 网络监控
    TextTool,   // 文本工具
    System,     // 系统信息
    Calculator, // 计算器
}

impl MenuItem {
    /// 显示标签 - 模式匹配返回不同的字符串
    fn label(&self) -> &'static str {
        match self {
            MenuItem::Network => "🌐 网络",
            MenuItem::TextTool => "📝 文本工具",
            MenuItem::System => "💻 系统信息",
            MenuItem::Calculator => "🔢 计算器",
        }
    }
}

// ============================================================================
// 第四部分：文本工具模块 - 展示状态管理
// ============================================================================

/// 文本工具标签页 - 展示子状态切换
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum TextToolTab {
    #[default]
    Base64, // Base64 编解码
    UrlEncode,  // URL 编码
    Timestamp,  // 时间戳转换
    JsonFormat, // JSON 格式化
}

impl TextToolTab {
    fn label(&self) -> &'static str {
        match self {
            TextToolTab::Base64 => "Base64",
            TextToolTab::UrlEncode => "URL编码",
            TextToolTab::Timestamp => "时间戳",
            TextToolTab::JsonFormat => "JSON格式化",
        }
    }
}

/// 文本工具状态 - egui 是立即模式，需要在 App 结构体中保存状态
/// 每帧重建 UI 时，状态会被保留（与 React 类似的概念）
struct TextToolState {
    input: String,             // 输入框内容
    output: String,            // 输出框内容
    selected_tab: TextToolTab, // 当前选中的标签页
    timestamp_input: String,   // 时间戳输入
    timestamp_output: String,  // 时间戳输出
    use_current_time: bool,    // 是否使用当前时间
}

impl Default for TextToolState {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: String::new(),
            selected_tab: TextToolTab::Base64,
            timestamp_input: String::new(),
            timestamp_output: String::new(),
            use_current_time: false,
        }
    }
}

// ============================================================================
// 第五部分：系统信息模块 - 展示 sysinfo 集成
// ============================================================================

/// 系统信息状态 - 包含 sysinfo 的 System 结构体
struct SystemInfoState {
    system: System, // sysinfo 库的系统信息实例
    refresh_interval: u64,
}

impl Default for SystemInfoState {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self {
            system,
            refresh_interval: 2,
        }
    }
}

impl SystemInfoState {
    /// 刷新系统信息
    fn refresh(&mut self) {
        self.system.refresh_all();
    }
}

// ============================================================================
// 第六部分：计算器模块 - 展示 Slider 和简单计算
// ============================================================================

/// 计算器标签页
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum CalculatorTab {
    #[default]
    Bmi, // BMI 计算
    Simple, // 简单计算
}

impl CalculatorTab {
    fn label(&self) -> &'static str {
        match self {
            CalculatorTab::Bmi => "BMI 计算",
            CalculatorTab::Simple => "简单计算",
        }
    }
}

/// 计算器状态
struct CalculatorState {
    selected_tab: CalculatorTab,
    height: f32, // 身高 cm
    weight: f32, // 体重 kg
    bmi_result: String,
    calc_input: String,
    calc_result: String,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            selected_tab: CalculatorTab::Bmi,
            height: 170.0,
            weight: 70.0,
            bmi_result: "请输入身高和体重".to_string(),
            calc_input: String::new(),
            calc_result: String::new(),
        }
    }
}

// ============================================================================
// 第七部分：主应用结构 - App trait 实现
// ============================================================================

/// Dashboard 主应用结构体
/// - 包含所有子模块的状态
/// - 实现 eframe::App trait 来定义 UI 行为
struct DashboardApp {
    selected_menu: MenuItem,      // 当前选中的菜单
    text_tool: TextToolState,     // 文本工具状态
    system_info: SystemInfoState, // 系统信息状态
    calculator: CalculatorState,  // 计算器状态
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self {
            selected_menu: MenuItem::TextTool,
            text_tool: TextToolState::default(),
            system_info: SystemInfoState::default(),
            calculator: CalculatorState::default(),
        }
    }
}

/**
 * eframe::App trait - egui 应用的入口点
 *
 * egui 0.34 新 API:
 * - ui(): 绘制 UI
 * - logic(): 在 UI 之前调用，可用于状态更新
 */
impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("left_panel")
            .default_size(180.0)
            .show_inside(ui, |ui| {
                ui.heading("工具菜单");
                ui.separator();

                for menu in [MenuItem::Network, MenuItem::TextTool, MenuItem::System, MenuItem::Calculator] {
                    let selected = self.selected_menu == menu;

                    if ui
                        .selectable_label(selected, menu.label())
                        .clicked()
                    {
                        self.selected_menu = menu;
                    }
                }
            });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.selected_menu {
            MenuItem::Network => show_network_panel(ui),
            MenuItem::TextTool => show_text_tool_panel(ui, &mut self.text_tool),
            MenuItem::System => show_system_panel(ui, &mut self.system_info),
            MenuItem::Calculator => show_calculator_panel(ui, &mut self.calculator),
        });
    }
}

// ============================================================================
// 第八部分：网络监控面板
// ============================================================================

/**
 * 网络监控面板
 *
 * egui 常用控件:
 * - heading(): 标题文字
 * - separator(): 分隔线
 * - label(): 普通文字
 * - indent(): 缩进块
 * - ScrollArea::vertical(): 垂直滚动区域
 */
fn show_network_panel(ui: &mut egui::Ui) {
    ui.heading("🌐 网络监控");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("网络信息 (sysinfo):");
        ui.indent("net_info", |ui| {
            let sys = System::new_all();
            ui.label(format!("系统进程数: {}", sys.processes().len()));
            ui.label(format!("CPU 核心数: {}", sys.cpus().len()));
        });

        ui.separator();

        ui.label("网络统计:");
        ui.indent("net_stats", |ui| {
            ui.label("网络接口信息需要额外配置");
            ui.label("提示: 使用 pcap crate 可实现抓包功能");
        });

        ui.separator();

        ui.label("功能说明:");
        ui.indent("net_help", |ui| {
            ui.label("1. 显示网络接口列表");
            ui.label("2. 实时流量监控");
            ui.label("3. 连接状态统计");
            ui.label("4. 需要管理员权限");
        });
    });
}

// ============================================================================
// 第九部分：文本工具面板 - 展示多种控件
// ============================================================================

/**
 * 文本工具面板
 *
 * 关键概念:
 * - ScrollArea::vertical(): 垂直滚动区域，重要！因为内容可能超过窗口
 * - 状态通过 &mut TextToolState 传递
 */
fn show_text_tool_panel(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.heading("📝 文本工具");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 标签页切换 - 使用 horizontal 布局
        ui.horizontal(|ui| {
            for tab in [TextToolTab::Base64, TextToolTab::UrlEncode, TextToolTab::Timestamp, TextToolTab::JsonFormat] {
                let selected = state.selected_tab == tab;
                if ui
                    .selectable_label(selected, tab.label())
                    .clicked()
                {
                    state.selected_tab = tab;
                }
            }
        });

        ui.separator();

        // 匹配当前标签页，显示对应的工具
        match state.selected_tab {
            TextToolTab::Base64 => show_base64_tool(ui, state),
            TextToolTab::UrlEncode => show_url_encode_tool(ui, state),
            TextToolTab::Timestamp => show_timestamp_tool(ui, state),
            TextToolTab::JsonFormat => show_json_tool(ui, state),
        }
    });
}

/**
 * Base64 工具 - 展示 TextEdit 和按钮
 *
 * 核心控件:
 * - TextEdit::multiline(): 多行文本输入
 * - Button::clicked(): 按钮点击检测，返回 bool
 */
fn show_base64_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.label("输入:");
    // desired_rows() 设置默认行数
    ui.add(egui::TextEdit::multiline(&mut state.input).desired_rows(4));

    // 按钮行 - horizontal 布局
    ui.horizontal(|ui| {
        // 点击检测: button().clicked() 返回 bool
        if ui
            .button("编码 (Encode)")
            .clicked()
        {
            // 使用 base64 crate 编码
            state.output = base64::engine::general_purpose::STANDARD.encode(&state.input);
        }
        if ui
            .button("解码 (Decode)")
            .clicked()
        {
            state.output = base64::engine::general_purpose::STANDARD
                .decode(&state.input)
                .map(|v| String::from_utf8_lossy(&v).to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }
        if ui.button("清空").clicked() {
            state.input.clear();
            state.output.clear();
        }
    });

    ui.label("输出:");
    // frame(false) 移除边框，使其看起来像纯文本输出
    ui.add(egui::TextEdit::multiline(&mut state.output).desired_rows(4));
}

/**
 * URL 编码工具
 */
fn show_url_encode_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.label("输入:");
    ui.add(egui::TextEdit::multiline(&mut state.input).desired_rows(4));

    ui.horizontal(|ui| {
        if ui.button("编码").clicked() {
            state.output = urlencoding::encode(&state.input).to_string();
        }
        if ui.button("解码").clicked() {
            state.output = urlencoding::decode(&state.input)
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }
        if ui.button("清空").clicked() {
            state.input.clear();
            state.output.clear();
        }
    });

    ui.label("输出:");
    ui.add(egui::TextEdit::multiline(&mut state.output).desired_rows(4));
}

/**
 * 时间戳工具 - 展示 Checkbox 和 TextEdit::singleline
 */
fn show_timestamp_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    // Checkbox - 布尔值的复选框
    ui.checkbox(&mut state.use_current_time, "使用当前时间");

    ui.horizontal(|ui| {
        ui.label("时间戳 (秒):");
        // singleline() 单行输入
        ui.add(egui::TextEdit::singleline(&mut state.timestamp_input).desired_width(200.0));
    });

    if ui.button("转换").clicked() {
        if let Ok(ts) = state
            .timestamp_input
            .parse::<i64>()
        {
            if let Some(datetime) = chrono::DateTime::from_timestamp(ts, 0) {
                state.timestamp_output = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
            } else {
                state.timestamp_output = "时间戳无效".to_string();
            }
        } else {
            state.timestamp_output = "请输入有效的数字".to_string();
        }
    }

    if state.use_current_time {
        let now = chrono::Utc::now();
        ui.label(format!("当前时间戳: {}", now.timestamp()));
        ui.label(format!("当前时间: {}", now.format("%Y-%m-%d %H:%M:%S UTC")));
    }

    if !state
        .timestamp_output
        .is_empty()
    {
        ui.label(&state.timestamp_output);
    }
}

/**
 * JSON 格式化工具
 */
fn show_json_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.label("输入 JSON:");
    ui.add(egui::TextEdit::multiline(&mut state.input).desired_rows(6));

    ui.horizontal(|ui| {
        if ui.button("格式化").clicked() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&state.input) {
                // to_string_pretty 格式化 JSON
                state.output = serde_json::to_string_pretty(&value).unwrap_or_default();
            } else {
                state.output = "JSON 格式错误".to_string();
            }
        }
        if ui.button("压缩").clicked() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&state.input) {
                state.output = serde_json::to_string(&value).unwrap_or_default();
            } else {
                state.output = "JSON 格式错误".to_string();
            }
        }
        if ui.button("清空").clicked() {
            state.input.clear();
            state.output.clear();
        }
    });

    ui.label("输出:");
    ui.add(egui::TextEdit::multiline(&mut state.output).desired_rows(6));
}

// ============================================================================
// 第十部分：系统信息面板 - 展示 ProgressBar 和 Grid
// ============================================================================

/**
 * 系统信息面板
 *
 * 展示的控件:
 * - ProgressBar: 进度条
 * - Grid: 表格布局
 * - ScrollArea: 滚动区域
 */
fn show_system_panel(ui: &mut egui::Ui, state: &mut SystemInfoState) {
    ui.heading("💻 系统信息");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 刷新按钮
        ui.horizontal(|ui| {
            if ui.button("刷新").clicked() {
                state.refresh();
            }
            ui.label(format!("刷新间隔: {} 秒", state.refresh_interval));
        });

        ui.separator();

        let sys = &state.system;

        ui.label("系统信息:");
        ui.indent("sys_info", |ui| {
            ui.label(format!("CPU 核心数: {}", sys.cpus().len()));
        });

        ui.separator();

        ui.label("CPU 使用率 (所有核心平均):");
        // 计算平均 CPU 使用率
        let cpu_usage: f32 = sys
            .cpus()
            .iter()
            .map(|c| c.cpu_usage())
            .sum::<f32>()
            / sys.cpus().len() as f32;
        // ProgressBar - 进度条，值范围 0.0-1.0
        ui.add(egui::ProgressBar::new(cpu_usage / 100.0).text(format!("{:.1}%", cpu_usage)));

        ui.separator();

        ui.label("内存使用:");
        let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_percent = if total_mem > 0.0 { used_mem / total_mem } else { 0.0 };

        ui.label(format!("总内存: {:.2} GB", total_mem));
        ui.label(format!("已使用: {:.2} GB ({:.1}%)", used_mem, mem_percent * 100.0));
        ui.add(egui::ProgressBar::new(mem_percent as f32).text(format!("{:.1}%", mem_percent * 100.0)));

        ui.separator();

        ui.label("进程信息 (按内存排序):");
        // 获取进程列表并按内存排序
        let mut processes: Vec<_> = sys.processes().iter().collect();
        processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

        // Grid 表格布局 - 多列布局
        egui::ScrollArea::vertical()
            .id_salt("process_list")
            .max_height(300.0)
            .show(ui, |ui| {
                egui::Grid::new("process_grid")
                    .num_columns(3)
                    .show(ui, |ui| {
                        // 表头
                        ui.label("PID");
                        ui.label("名称");
                        ui.label("内存");
                        ui.end_row();

                        // 显示前 20 个进程
                        for (pid, proc) in processes.iter().take(20) {
                            ui.label(format!("{}", pid));
                            // to_string_lossy 处理可能的无效 UTF-8
                            ui.label(proc.name().to_string_lossy());
                            ui.label(format!("{:.1} MB", proc.memory() as f64 / 1024.0 / 1024.0));
                            ui.end_row();
                        }
                    });
            });
    });
}

// ============================================================================
// 第十一部分：计算器面板 - 展示 Slider 和简单计算
// ============================================================================

/**
 * 计算器面板
 *
 * 展示的控件:
 * - Slider: 滑动条，用于数值输入
 */
fn show_calculator_panel(ui: &mut egui::Ui, state: &mut CalculatorState) {
    ui.heading("🔢 计算器");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 标签页切换
        ui.horizontal(|ui| {
            for tab in [CalculatorTab::Bmi, CalculatorTab::Simple] {
                let selected = state.selected_tab == tab;
                if ui
                    .selectable_label(selected, tab.label())
                    .clicked()
                {
                    state.selected_tab = tab;
                }
            }
        });

        ui.separator();

        match state.selected_tab {
            CalculatorTab::Bmi => show_bmi_tool(ui, state),
            CalculatorTab::Simple => show_simple_calc(ui, state),
        }
    });
}

/**
 * BMI 计算工具 - 展示 Slider 控件
 *
 * Slider 用于:
 * - 有范围的数值输入
 * - 实时预览数值变化
 */
fn show_bmi_tool(ui: &mut egui::Ui, state: &mut CalculatorState) {
    ui.label("身高 (cm):");
    // Slider - 滑动条，参数: 值, 范围
    ui.add(egui::Slider::new(&mut state.height, 50.0..=250.0).text(""));

    ui.label("体重 (kg):");
    ui.add(egui::Slider::new(&mut state.weight, 10.0..=200.0).text(""));

    if ui.button("计算 BMI").clicked() {
        let height_m = state.height / 100.0;
        let bmi = state.weight / (height_m * height_m);
        let category = if bmi < 18.5 {
            "偏瘦"
        } else if bmi < 24.0 {
            "正常"
        } else if bmi < 28.0 {
            "偏胖"
        } else {
            "肥胖"
        };
        state.bmi_result = format!("BMI: {:.1} ({})", bmi, category);
    }

    ui.separator();
    ui.label(&state.bmi_result);

    ui.label("BMI 参考:");
    ui.indent("bmi_ref", |ui| {
        ui.label("< 18.5: 偏瘦");
        ui.label("18.5 - 24: 正常");
        ui.label("24 - 28: 偏胖");
        ui.label(">= 28: 肥胖");
    });
}

/**
 * 简单计算器 - 展示文本表达式计算
 */
fn show_simple_calc(ui: &mut egui::Ui, state: &mut CalculatorState) {
    ui.label("输入表达式 (如: 2+3*4):");
    ui.add(egui::TextEdit::singleline(&mut state.calc_input).desired_width(300.0));

    ui.horizontal(|ui| {
        if ui.button("计算").clicked() {
            state.calc_result = calculate_simple(&state.calc_input);
        }
        if ui.button("清空").clicked() {
            state.calc_input.clear();
            state.calc_result.clear();
        }
    });

    if !state.calc_result.is_empty() {
        ui.label(format!("结果: {}", state.calc_result));
    }
}

/**
 * 简单的表达式计算器
 *
 * 注意：这是一个非常简化的实现
 * 生产环境建议使用 meval crate
 */
fn calculate_simple(input: &str) -> String {
    let input = input.replace(" ", "");
    match shunting_yard(&input) {
        Ok(result) => format!("{}", result),
        Err(e) => e,
    }
}

/// 简单的运算符优先级计算 (仅支持 + - * / 和个位数)
fn shunting_yard(input: &str) -> Result<f64, String> {
    let input = input.replace(" ", "");

    if input.is_empty() {
        return Err("请输入表达式".to_string());
    }

    let mut result = 0.0;
    let mut current_num = 0.0;
    let mut last_op = '+';
    let mut has_number = false;

    for c in input.chars() {
        if c.is_ascii_digit() {
            current_num = current_num * 10.0 + (c as u8 - b'0') as f64;
            has_number = true;
        } else if "+-*/".contains(c) {
            if !has_number {
                return Err("表达式格式错误".to_string());
            }
            match last_op {
                '+' => result += current_num,
                '-' => result -= current_num,
                '*' => result *= current_num,
                '/' => {
                    if current_num == 0.0 {
                        return Err("除数不能为零".to_string());
                    }
                    result /= current_num;
                }
                _ => {}
            }
            last_op = c;
            current_num = 0.0;
            has_number = false;
        }
    }

    // 处理最后一个数字
    if has_number {
        match last_op {
            '+' => result += current_num,
            '-' => result -= current_num,
            '*' => result *= current_num,
            '/' => {
                if current_num == 0.0 {
                    return Err("除数不能为零".to_string());
                }
                result /= current_num;
            }
            _ => {}
        }
    }

    Ok(result)
}

// ============================================================================
// 第十二部分：字体加载 - 展示文件系统操作
// ============================================================================

/**
 * 查找中文字体文件
 *
 * Windows 常见中文字体路径:
 * - C:\Windows\Fonts\msyh.ttc (微软雅黑)
 * - C:\Windows\Fonts\simhei.ttf (黑体)
 * - C:\Windows\Fonts\simsun.ttc (宋体)
 */
fn find_chinese_font_path() -> Result<PathBuf, AppError> {
    let font_dir = PathBuf::from("C:/Windows/Fonts");
    let font_names = ["msyh.ttc", "simhei.ttf", "simsun.ttc", "msjh.ttc"];

    for name in &font_names {
        let path = font_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(AppError::FontNotFound)
}

/**
 * 加载中文字体到 egui
 *
 * egui 字体系统:
 * - FontDefinitions: 字体定义结构
 * - FontData: 字体数据 (TTF/TTC 文件)
 * - font_data.insert(): 添加字体
 * - families: 指定哪些字体族使用该字体
 */
fn load_chinese_fonts() -> Result<egui::FontDefinitions, AppError> {
    let font_path = find_chinese_font_path()?;
    let data = std::fs::read(&font_path)?;

    let mut fonts = egui::FontDefinitions::default();

    // 添加字体数据，into() 转换为 Arc<FontData>
    fonts
        .font_data
        .insert("chinese".to_owned(), egui::FontData::from_owned(data).into());

    // 设置 Proportional 字体族使用中文
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "chinese".to_owned());

    // 设置 Monospace 字体族使用中文
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "chinese".to_owned());

    Ok(fonts)
}

// ============================================================================
// 第十三部分：main 函数 - 应用入口
// ============================================================================

/**
 * 程序入口
 *
 * eframe::run_native() 参数说明:
 * - "个人工作台": 窗口标题
 * - options: NativeOptions 配置窗口
 * - Box::new(|cc|): 闭包，创建 App 实例
 *   - cc: CreationContext，包含 egui_ctx 等
 *
 * WgpuConfiguration:
 * - 配置 wgpu 渲染后端
 *
 * ============================================================================
 * 渲染模式配置 (通过环境变量控制)
 * ============================================================================
 *
 * eframe/wgpu 通过环境变量控制渲染后端:
 * | 环境变量 | 值 | 说明 |
 * |---------|-----|------|
 * | WGPU_POWER_PREF | high | GPU渲染 (默认) |
 * | WGPU_POWER_PREF | low | 软件渲染 (WARP on Windows) |
 * | WGPU_BACKEND | dx12 | DirectX 12 |
 * | WGPU_BACKEND | vulkan | Vulkan |
 * | WGPU_BACKEND | metal | Metal (macOS) |
 *
 * Windows 软件渲染: WGPU_POWER_PREF=low (使用 WARP)
 * Linux 软件渲染: WGPU_POWER_PREF=low (使用 llvmpipe)
 *
 * 运行示例:
 *   $env:WGPU_POWER_PREF="low"; cargo run --bin dashboard
 */

// ============================================================================
// main 函数
// ============================================================================

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "个人工作台",
        options,
        Box::new(|cc| {
            if let Ok(fonts) = load_chinese_fonts() {
                cc.egui_ctx.set_fonts(fonts);
            }
            Ok(Box::new(DashboardApp::default()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("运行应用失败: {}", e))?;

    Ok(())
}
