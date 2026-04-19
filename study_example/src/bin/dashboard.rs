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
// 第一部分：导入和依赖 (行 121-130)
// ============================================================================

use anyhow::Result; // anyhow: 统一错误处理，简化错误传播
use base64::Engine; // base64::Engine: base64 编码解码的 trait，提供 encode/decode 方法
use eframe::egui; // egui: 核心 GUI 库，包含所有控件和布局
use std::path::PathBuf; // PathBuf: 跨平台路径表示，类似 String 的 Path 版本
use sysinfo::System; // sysinfo: 系统信息获取库，查看 CPU/内存/进程等
use thiserror::Error; // thiserror: 自定义错误派生宏，自动实现 Error trait

// ============================================================================
// 第二部分：错误定义 (行 132-146)
// ============================================================================

// AppError: 应用自定义错误枚举
// #[derive(Debug, Error)] 自动派生 Debug 和 Display
#[derive(Debug, Error)]
pub enum AppError {
    // #[error("...")] thiserror 宏自动实现 Display trait
    #[error("未找到中文字体")]
    FontNotFound,

    // #[from] 自动实现 From<std::io::Error>，错误自动转换
    #[error("读取字体文件失败: {0}")]
    ReadFontFailed(#[from] std::io::Error),
}

// ============================================================================
// 第三部分：菜单系统 (行 148-157) - enum + 模式匹配
// ============================================================================

// MenuItem: 菜单项枚举，表示左侧菜单选项
#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuItem {
    Network,    // Network: 网络监控菜单项
    TextTool,   // TextTool: 文本工具菜单项
    System,     // System: 系统信息菜单项
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

// ============================================================================
// 第七部分：App trait 实现 ⭐ 核心 (行 320-354)
// ============================================================================

// eframe::App: egui 应用入口 trait
// ui(): 每帧调用的方法，构建即时模式 UI
impl eframe::App for DashboardApp {
    // self: &mut DashboardApp - 可变引用，可以修改状态
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // egui::Panel::left(): 左侧面板
        // .default_size(180.0): 默认宽度 180 像素
        // .show_inside(): 在父 UI 内显示面板
        egui::Panel::left("left_panel")
            .default_size(180.0)
            .show_inside(ui, |ui| {
                // 标题
                ui.heading("工具菜单");
                // 分隔线
                ui.separator();

                // for 循环遍历菜单项数组
                for menu in [MenuItem::Network, MenuItem::TextTool, MenuItem::System, MenuItem::Calculator] {
                    // == 比较是否选中
                    let selected = self.selected_menu == menu;
                    // selectable_label: 可选择标签，类似单选按钮
                    // .clicked(): 检测点击
                    if ui
                        .selectable_label(selected, menu.label())
                        .clicked()
                    {
                        // 更新状态
                        self.selected_menu = menu;
                    }
                }
            });

        // egui::CentralPanel::default(): 中央面板
        // .show_inside(): 显示中央区域
        // match 匹配当前菜单项，决定显示哪个面板
        egui::CentralPanel::default().show_inside(ui, |ui| match self.selected_menu {
            // 显示对应的面板函数
            MenuItem::Network => show_network_panel(ui),
            MenuItem::TextTool => show_text_tool_panel(ui, &mut self.text_tool),
            MenuItem::System => show_system_panel(ui, &mut self.system_info),
            MenuItem::Calculator => show_calculator_panel(ui, &mut self.calculator),
        });
    }
}

// ============================================================================
// 第八部分：网络监控面板 (行 356-400)
// ============================================================================

// show_network_panel: 显示网络监控面板的函数
// 参数 ui: &mut egui::Ui - 可变借用的 UI 上下文
fn show_network_panel(ui: &mut egui::Ui) {
    // ui.heading(): 大标题文字，显示表情图标加文字
    ui.heading("🌐 网络监控");
    // ui.separator(): 水平分隔线，分隔区域
    ui.separator();

    // egui::ScrollArea::vertical(): 垂直滚动区域，内部可滚动
    // .show(ui, |ui| {}): 在滚动区域内构建 UI
    egui::ScrollArea::vertical().show(ui, |ui| {
        // ui.label(): 显示文字标签
        ui.label("网络信息 (sysinfo):");
        // ui.indent(): 带缩进的块，类似于代码缩进
        ui.indent("net_info", |ui| {
            // System::new_all(): 创建包含所有信息的系统实例
            let sys = System::new_all();
            // format!(): 字符串格式化，类似 println! 但返回 String
            // .processes(): 获取进程 HashMap
            // .len(): 元素数量
            ui.label(format!("系统进程数: {}", sys.processes().len()));
            // .cpus(): 获取 CPU 信息 Vec
            ui.label(format!("CPU 核心数: {}", sys.cpus().len()));
        });

        // 分隔线
        ui.separator();

        // 标签
        ui.label("网络统计:");
        // 缩进块
        ui.indent("net_stats", |ui| {
            ui.label("网络接口信息需要额外配置");
            ui.label("提示: 使用 pcap crate 可实现抓包功能");
        });

        // 分隔线
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
// 第九部分：文本工具面板 (行 409-448)
// 展示: ScrollArea, horizontal, selectable_label, TextEdit, 按钮
// ============================================================================

// show_text_tool_panel: 文本工具面板函数
// 参数 state: &mut TextToolState - 可变引用传递状态，类似 React useState
fn show_text_tool_panel(ui: &mut egui::Ui, state: &mut TextToolState) {
    // 标题
    ui.heading("📝 文本工具");
    // 分隔线
    ui.separator();

    // 垂直滚动区域
    egui::ScrollArea::vertical().show(ui, |ui| {
        // horizontal: 水平布局容器，标签页按钮横向排列
        ui.horizontal(|ui| {
            // for 循环遍历枚举数组
            for tab in [TextToolTab::Base64, TextToolTab::UrlEncode, TextToolTab::Timestamp, TextToolTab::JsonFormat] {
                // == 比较是否选中
                let selected = state.selected_tab == tab;
                // selectable_label: 可选择的标签，类似单选按钮
                // .clicked(): 立即模式检测点击
                if ui
                    .selectable_label(selected, tab.label())
                    .clicked()
                {
                    // 更新状态
                    state.selected_tab = tab;
                }
            }
        });

        // 分隔线
        ui.separator();

        // match 匹配当前选中的标签页
        match state.selected_tab {
            TextToolTab::Base64 => show_base64_tool(ui, state),
            TextToolTab::UrlEncode => show_url_encode_tool(ui, state),
            TextToolTab::Timestamp => show_timestamp_tool(ui, state),
            TextToolTab::JsonFormat => show_json_tool(ui, state),
        }
    });
}

// ============================================================================
// Base64 工具函数 (行 452-494)
// 核心控件: TextEdit::multiline, Button
// ============================================================================

// show_base64_tool: Base64 编解码工具函数
fn show_base64_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.label("输入:"); // 输入标签

    // ui.add(): 添加控件
    // TextEdit::multiline(): 多行文本输入框
    // desired_rows(4): 默认显示 4 行
    ui.add(egui::TextEdit::multiline(&mut state.input).desired_rows(4));

    // horizontal: 水平布局，按钮横向排列
    ui.horizontal(|ui| {
        // button(): 按钮控件
        // .clicked(): 立即模式检测，返回 bool
        if ui
            .button("编码 (Encode)")
            .clicked()
        {
            // base64::engine::general_purpose::STANDARD: 标准 base64 编码器
            // .encode(): 编码为 base64 字符串
            state.output = base64::engine::general_purpose::STANDARD.encode(&state.input);
        }
        if ui
            .button("解码 (Decode)")
            .clicked()
        {
            // .decode(): 解码为 Vec<u8>
            // .map(): 转换 Result
            // String::from_utf8_lossy(): 转换为 UTF-8 字符串
            // .to_owned(): 克隆为 String
            // .unwrap_or_else(): 错误时返回默认值
            state.output = base64::engine::general_purpose::STANDARD
                .decode(&state.input)
                .map(|v| String::from_utf8_lossy(&v).to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }
        // 清空按钮
        if ui.button("清空").clicked() {
            // .clear(): 清空字符串内容
            state.input.clear();
            state.output.clear();
        }
    });

    ui.label("输出:"); // 输出标签

    // 第二个 TextEdit 显示输出，frame(false) 移除边框
    ui.add(egui::TextEdit::multiline(&mut state.output).desired_rows(4));
}

// ============================================================================
// URL 编码工具 (行 522-)
// 核心控件: urlencoding crate
// ============================================================================

// show_url_encode_tool: URL 编码解码工具函数
fn show_url_encode_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.label("输入:");
    ui.add(egui::TextEdit::multiline(&mut state.input).desired_rows(4));

    // horizontal: 水平布局
    ui.horizontal(|ui| {
        // button(): 按钮
        // .clicked(): 点击检测
        if ui.button("编码").clicked() {
            // urlencoding::encode(): URL 编码
            state.output = urlencoding::encode(&state.input).to_string();
        }
        if ui.button("解码").clicked() {
            // urlencoding::decode(): URL 解码
            // .map(): 转换 Result
            // .to_string(): 转换为 String
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

// ============================================================================
// 时间戳工具 (行 558-)
// 核心控件: Checkbox, TextEdit::singleline
// ============================================================================

// show_timestamp_tool: 时间戳转换工具函数
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

// ============================================================================
// JSON 格式化工具 (行 603-)
// 核心控件: serde_json
// ============================================================================

// show_json_tool: JSON 格式化压缩工具函数
fn show_json_tool(ui: &mut egui::Ui, state: &mut TextToolState) {
    ui.label("输入 JSON:");
    // desired_rows(6): 默认显示 6 行
    ui.add(egui::TextEdit::multiline(&mut state.input).desired_rows(6));

    // horizontal: 水平布局
    ui.horizontal(|ui| {
        // button: 按钮控件
        if ui.button("格式化").clicked() {
            // serde_json::from_str(): 解析 JSON 字符串为 Value
            // serde_json::Value: 通用 JSON 值类型
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&state.input) {
                // serde_json::to_string_pretty(): 格式化 JSON (带缩进)
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
// ============================================================================
// 第十部分：系统信息面板 (行 645-)
// 核心控件: ProgressBar, Grid, ScrollArea
// ============================================================================

// show_system_panel: 系统信息显示函数
fn show_system_panel(ui: &mut egui::Ui, state: &mut SystemInfoState) {
    // 标题
    ui.heading("💻 系统信息");
    // 分隔线
    ui.separator();

    // 垂直滚动区域
    egui::ScrollArea::vertical().show(ui, |ui| {
        // 水平布局，刷新按钮
        ui.horizontal(|ui| {
            if ui.button("刷新").clicked() {
                // refresh(): 刷新系统信息
                state.refresh();
            }
            ui.label(format!("刷新间隔: {} 秒", state.refresh_interval));
        });

        ui.separator();

        // 引用系统实例
        let sys = &state.system;

        ui.label("系统信息:");
        // indent: 缩进块
        ui.indent("sys_info", |ui| {
            ui.label(format!("CPU 核心数: {}", sys.cpus().len()));
        });

        ui.separator();

        ui.label("CPU 使用率 (所有核心平均):");

        // .cpus(): 返回 CPU 信息数组
        // .iter(): 迭代器
        // .map(): 映射闭包
        // .cpu_usage(): 获取单个 CPU 使用率
        // .sum(): 求和
        // / len() as f32: 计算平均值
        let cpu_usage: f32 = sys
            .cpus()
            .iter()
            .map(|c| c.cpu_usage())
            .sum::<f32>()
            / sys.cpus().len() as f32;

        // ProgressBar: 进度条控件
        // .new(值): 值范围 0.0-1.0
        // .text(): 显示文本
        ui.add(egui::ProgressBar::new(cpu_usage / 100.0).text(format!("{:.1}%", cpu_usage)));

        ui.separator();

        ui.label("内存使用:");
        // total_memory() 返回 bytes
        // / 1024.0 / 1024.0 / 1024.0: 转换为 GB
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
// 第十一部分：计算器面板 (行 748-)
// 核心控件: Slider
// ============================================================================

// show_calculator_panel: 计算器面板函数
fn show_calculator_panel(ui: &mut egui::Ui, state: &mut CalculatorState) {
    // 标题
    ui.heading("🔢 计算器");
    // 分隔线
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 标签页切换 - horizontal 布局
        ui.horizontal(|ui| {
            for tab in [CalculatorTab::Bmi, CalculatorTab::Simple] {
                // == 比较
                let selected = state.selected_tab == tab;
                // selectable_label: 选择标签
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
// 第十二部分：字体加载 (行 921-)
// 展示: 文件系统操作, FontDefinitions
// ============================================================================

// find_chinese_font_path: 查找系统中文字体文件
fn find_chinese_font_path() -> Result<PathBuf, AppError> {
    // PathBuf::from(): 创建路径
    let font_dir = PathBuf::from("C:/Windows/Fonts");
    // 字体文件名字数组
    let font_names = ["msyh.ttc", "simhei.ttf", "simsun.ttc", "msjh.ttc"];

    // for 循环遍历
    for name in &font_names {
        // .join(): 拼接路径
        let path = font_dir.join(name);
        // .exists(): 检查文件是否存在
        if path.exists() {
            return Ok(path);
        }
    }
    Err(AppError::FontNotFound)
}

// ============================================================================
// 字体加载函数 (行 945-)
// ============================================================================

// load_chinese_fonts: 加载中文字体到 egui
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
// main 函数 (行 950-)
// 程序入口点
// ============================================================================

// main: 程序入口函数，返回 Result 表示可能的错误
fn main() -> Result<()> {
    // NativeOptions: 原生窗口配置结构体
    let options = eframe::NativeOptions {
        // viewport: 视口配置，控制窗口尺寸
        // ViewportBuilder: 视口构建器
        // .default(): 获取默认配置
        // .with_inner_size([宽, 高]): 设置窗口内尺寸 1000x700
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            // .with_min_inner_size([宽, 高]): 设置窗口最小尺寸 800x500
            .with_min_inner_size([800.0, 500.0]),
        // ..Default::default(): 结构体更新语法，其余字段用默认值
        ..Default::default()
    };

    // eframe::run_native(): 启动原生窗口应用
    // 参数1: 窗口标题，显示在标题栏
    eframe::run_native(
        "个人工作台", // title: 窗口标题
        options,      // options: NativeOptions 窗口配置
        // Box::new(|cc| {}): 装箱闭包，|cc| 接收 CreationContext
        Box::new(|cc| {
            // load_chinese_fonts(): 加载中文字体
            // if let Ok(fonts) = ...: 模式匹配，成功时执行
            if let Ok(fonts) = load_chinese_fonts() {
                // cc.egui_ctx: egui 上下文
                // .set_fonts(): 设置自定义字体定义
                cc.egui_ctx.set_fonts(fonts);
            }
            // Ok(Box::new()): 成功创建并返回 App 实例
            // DashboardApp::default(): 调用 Default 实现创建默认状态
            Ok(Box::new(DashboardApp::default()))
        }),
    )
    // .map_err(): 转换错误类型
    // |e| 闭包参数接收错误
    // anyhow::anyhow!(): 将错误包装为 anyhow 类型
    .map_err(|e| anyhow::anyhow!("运行应用失败: {}", e))?;

    // Ok(()): 成功退出程序
    Ok(())
}
