//! # 个人工作台 (Dashboard) - iced 0.14 教学示例
//!
//! 本文件展示了 iced 与 egui 的核心架构差异。
//! - egui: 立即模式 (Immediate Mode) - 每帧重建 UI，直接在 UI 闭包中修改状态
//! - iced: Elm 架构 (保留模式/Retained Mode) - 通过 Message + Update + View 模式
//!
//! iced 核心概念:
//! - Message: 用户交互产生的消息（类似 Redux action）
//! - update(): 处理消息，更新状态纯函数
//! - view(): 根据状态渲染 UI（类似 React render）
//!
//! ============================================================================
//! 📚 学习路径 (推荐阅读顺序)
//! ============================================================================
//!
//! iced 0.14 使用简单的 `iced::run(update, view)` API
//!
//! 推荐阅读顺序:
//! ```text
//! 第十三部分 (main)        → 1. 如何启动应用
//!    ↓
//! 第七部分 (view函数)      → 2. UI构建入口 ⭐ 核心
//!    ↓
//! 第四部分 (状态定义)       → 3. 为什么要定义状态
//!    ↓
//! 第五部分 (Message)       → 4. 消息枚举定义
//!    ↓
//! 第六部分 (update函数)     → 5. 状态更新逻辑
//!    ↓
//! 第八部分 (各面板view)     → 6. 各功能面板实现
//!    ↓
//! 第一部分 (导入)          → 7. 依赖说明
//! ```
//!
//! 详细说明:
//! ----------
//!
//! 1️⃣ **第十三部分 (行 476)** - main 函数
//!    - iced::run() 的用法
//!    - 状态初始化
//!
//! 2️⃣ **第七部分 (行 277)** - view 函数 ⭐
//!    - 根据状态渲染整个应用 UI
//!    - 分发到各个面板视图
//!
//! 3️⃣ **第四部分 (行 97)** - DashboardState 状态结构
//!    - 保存应用所有状态
//!    - 类似于 React 的 useState
//!
//! 4️⃣ **第五部分 (行 72)** - Message 枚举
//!    - 定义所有用户交互类型
//!    - 类似于 Redux action type
//!
//! 5️⃣ **第六部分 (行 120)** - update 函数
//!    - 处理每个 Message
//!    - 纯函数：不修改 state，而是通过参数 &mut state
//!
//! 6️⃣ **第八部分 (行 288-470)** - 各面板视图函数
//!    - view_menu_panel: 左侧菜单
//!    - view_network_panel: 网络监控
//!    - view_text_tool_panel: 文本工具
//!    - view_system_panel: 系统信息
//!    - view_calculator_panel: 计算器
//!
//! 7️⃣ **第一部分 (导入)** - 依赖说明
//!    - iced::widget: UI 组件
//!    - sysinfo: 系统信息
//!    - chrono: 时间处理
//!    - base64/urlencoding: 编解码
//!
//! 💡 提示: iced 与 egui 的最大区别是交互处理方式
//!    - egui: 用 `.clicked()` 检测点击，直接修改状态
//!    - iced: 用 `.on_press(Message)` 回调，通过 update 处理
//!
//! ============================================================================
//! 第一部分：导入和依赖
// ============================================================================

use base64::Engine;
use chrono::{TimeZone, Utc};
use iced::widget::{button, checkbox, column, container, progress_bar, row, scrollable, slider, text, text_input};
use iced::{Element, Length};
use sysinfo::System;

// ============================================================================
// 第四部分：状态定义 (类似 React useState / Vue data)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum MenuItem {
    #[default]
    Network,
    TextTool,
    System,
    Calculator,
}

impl MenuItem {
    fn label(&self) -> &'static str {
        match self {
            MenuItem::Network => "网络",
            MenuItem::TextTool => "文本工具",
            MenuItem::System => "系统信息",
            MenuItem::Calculator => "计算器",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum TextToolTab {
    #[default]
    Base64,
    UrlEncode,
    Timestamp,
    JsonFormat,
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum CalculatorTab {
    #[default]
    Bmi,
    Simple,
}

impl CalculatorTab {
    fn label(&self) -> &'static str {
        match self {
            CalculatorTab::Bmi => "BMI 计算",
            CalculatorTab::Simple => "简单计算",
        }
    }
}

// ============================================================================
// 第五部分：Message 枚举 (类似 Redux Action - 定义所有用户交互)
// ============================================================================

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
enum Message {
    SelectMenu(MenuItem),
    TextToolInputChanged(String),
    TextToolTabChanged(TextToolTab),
    EncodeBase64,
    DecodeBase64,
    EncodeUrl,
    DecodeUrl,
    ClearTextTool,
    TimestampInputChanged(String),
    ConvertTimestamp,
    UseCurrentTimeChanged(bool),
    JsonInputChanged(String),
    FormatJson,
    CompactJson,
    RefreshSystemInfo,
    CalculatorTabChanged(CalculatorTab),
    HeightChanged(f32),
    WeightChanged(f32),
    CalculateBmi,
    CalcInputChanged(String),
    Calculate,
    ClearCalc,
}

// ============================================================================
// 第四部分：DashboardState 状态结构 (类似 React useState / Vue data)
// ============================================================================

#[derive(Default)]
struct DashboardState {
    selected_menu: MenuItem,
    text_tool_input: String,
    text_tool_output: String,
    text_tool_tab: TextToolTab,
    timestamp_input: String,
    timestamp_output: String,
    use_current_time: bool,
    json_input: String,
    system: System,
    calc_tab: CalculatorTab,
    height: f32,
    weight: f32,
    bmi_result: String,
    calc_input: String,
    calc_result: String,
}

// ============================================================================
// 第六部分：Update 函数 (行 200-)
// 纯函数处理消息，更新状态，类似 Redux reducer
// ============================================================================

// update: 状态更新函数 (Elm 架构)
// 参数 state: &mut DashboardState - 可变引用，修改状态
// 参数 message: Message - 用户交互产生的消息
fn update(state: &mut DashboardState, message: Message) {
    // match 匹配消息类型，处理对应的状态更新
    match message {
        // 菜单选择
        Message::SelectMenu(menu) => state.selected_menu = menu,

        // 文本输入变化
        Message::TextToolInputChanged(s) => state.text_tool_input = s,

        // 标签页切换
        Message::TextToolTabChanged(tab) => {
            state.text_tool_tab = tab;
            state.text_tool_output.clear();
        }

        // Base64 编码
        Message::EncodeBase64 => {
            // base64::engine::general_purpose::STANDARD: 标准编码器
            // .encode(): 编码
            state.text_tool_output = base64::engine::general_purpose::STANDARD.encode(&state.text_tool_input);
        }

        // Base64 解码
        Message::DecodeBase64 => {
            state.text_tool_output = base64::engine::general_purpose::STANDARD
                .decode(&state.text_tool_input)
                .map(|v| String::from_utf8_lossy(&v).to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }

        // URL 编码
        Message::EncodeUrl => {
            state.text_tool_output = urlencoding::encode(&state.text_tool_input).to_string();
        }

        // URL 解码
        Message::DecodeUrl => {
            state.text_tool_output = urlencoding::decode(&state.text_tool_input)
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }

        // 清空
        Message::ClearTextTool => {
            state.text_tool_input.clear();
            state.text_tool_output.clear();
        }

        // 时间戳输入
        Message::TimestampInputChanged(s) => state.timestamp_input = s,

        // 时间戳转换
        Message::ConvertTimestamp => {
            // .parse::<i64>(): 字符串解析为 i64
            if let Ok(ts) = state
                .timestamp_input
                .parse::<i64>()
            {
                // chrono::Utc: UTC 时间
                // .timestamp_opt(): 从时间戳创建 DateTime
                if let Some(datetime) = Utc
                    .timestamp_opt(ts, 0)
                    .single()
                {
                    // .format(): 格式化
                    state.timestamp_output = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
                } else {
                    state.timestamp_output = "时间戳无效".to_string();
                }
            } else {
                state.timestamp_output = "请输入有效的数字".to_string();
            }
        }
        Message::UseCurrentTimeChanged(b) => state.use_current_time = b,
        Message::JsonInputChanged(s) => state.json_input = s,
        Message::FormatJson => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&state.json_input) {
                state.text_tool_output = serde_json::to_string_pretty(&value).unwrap_or_default();
            } else {
                state.text_tool_output = "JSON 格式错误".to_string();
            }
        }
        Message::CompactJson => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&state.json_input) {
                state.text_tool_output = serde_json::to_string(&value).unwrap_or_default();
            } else {
                state.text_tool_output = "JSON 格式错误".to_string();
            }
        }
        Message::RefreshSystemInfo => state.system.refresh_all(),
        Message::CalculatorTabChanged(tab) => state.calc_tab = tab,
        Message::HeightChanged(h) => state.height = h,
        Message::WeightChanged(w) => state.weight = w,
        Message::CalculateBmi => {
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
        Message::CalcInputChanged(s) => state.calc_input = s,
        Message::Calculate => {
            state.calc_result = calculate_simple(&state.calc_input);
        }
        Message::ClearCalc => {
            state.calc_input.clear();
            state.calc_result.clear();
        }
    }
}

fn calculate_simple(input: &str) -> String {
    let input = input.replace(' ', "");
    match shunting_yard(&input) {
        Ok(result) => format!("{}", result),
        Err(e) => e,
    }
}

fn shunting_yard(input: &str) -> Result<f64, String> {
    let input = input.replace(' ', "");
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
// 第七部分：View 函数 (行 357-)
// 根据状态渲染 UI，类似 React render
// ============================================================================

// view: UI 渲染函数
// 参数 state: &DashboardState - 不可变引用，只读状态
// 返回 Element<Message>: iced 的 UI 元素类型
fn view(state: &DashboardState) -> Element<Message> {
    // view_menu_panel: 渲染左侧菜单面板
    let menu_panel = view_menu_panel(state.selected_menu);

    // match 匹配当前选中的菜单项
    // let content = 根据选中的菜单显示对应的面板
    let content = match state.selected_menu {
        MenuItem::Network => view_network_panel(),
        MenuItem::TextTool => view_text_tool_panel(state),
        MenuItem::System => view_system_panel(state),
        MenuItem::Calculator => view_calculator_panel(state),
    };

    // row!: 水平布局容器
    // 将菜单面板和内容面板水平排列
    // .into(): 转换为 Element<Message>
    row![menu_panel, content].into()
}

// ============================================================================
// 菜单面板视图函数 (行 384-)
// iced widget: button, text, column, container
// ============================================================================

// view_menu_panel: 左侧菜单面板渲染
fn view_menu_panel(selected: MenuItem) -> Element<'static, Message> {
    // 数组: 菜单项和标签文本对
    let items = [
        (MenuItem::Network, "网络"),
        (MenuItem::TextTool, "文本工具"),
        (MenuItem::System, "系统信息"),
        (MenuItem::Calculator, "计算器"),
    ];

    // column!: 垂直布局容器
    // text!: 文本控件
    let mut col = column![text("工具菜单").size(20), text("")];

    // for 遍历菜单数组
    for (menu, label) in items {
        // button!: 按钮控件
        // .on_press(): 设置点击回调，返回 Message
        let btn = button(text(label)).on_press(Message::SelectMenu(menu));
        // .push(): 添加元素到 column
        // .width(): 设置宽度
        // Length::Fill: 填充父容器
        col = col.push(btn.width(Length::Fill));
    }

    // container!: 容器控件，添加布局
    // .width(): 设置宽度 180
    // .padding(): 设置内边距 10
    container(col)
        .width(180)
        .padding(10)
        .into()
}

// ============================================================================
// 网络监控面板 (行 423-)
// Sysinfo 集成
// ============================================================================

// view_network_panel: 网络监控面板渲染
fn view_network_panel() -> Element<'static, Message> {
    // System::new_all(): 创建包含所有信息的系统实例
    let sys = System::new_all();

    // column!: 垂直布局
    let mut col = column![
        text("网络监控").size(24), // 标题
        text(""),                  // 空行
        // format!: 格式化字符串
        text(format!("进程数: {}", sys.processes().len())),
        text(format!("CPU核心: {}", sys.cpus().len())),
    ];

    // scrollable!: 可滚动区域
    // .into(): 转换为 Element
    scrollable(col).into()
}

// ============================================================================
// 文本工具面板 (行 447-)
// 核心控件: row, text_input, checkbox
// ============================================================================

// view_text_tool_panel: 文本工具面板渲染
fn view_text_tool_panel(state: &DashboardState) -> Element<Message> {
    // 标签页数组
    let tabs = [
        (TextToolTab::Base64, "Base64"),
        (TextToolTab::UrlEncode, "URL编码"),
        (TextToolTab::Timestamp, "时间戳"),
        (TextToolTab::JsonFormat, "JSON"),
    ];

    // row!: 水平布局容器
    let mut tab_row = row![];
    // for 遍历标签页
    for (tab, label) in tabs {
        // button: 按钮
        // .on_press: 点击时发送消息
        let btn = button(text(label)).on_press(Message::TextToolTabChanged(tab));
        // .push(): 添加到 row
        tab_row = tab_row.push(btn);
    }

    // column!: 垂直布局容器
    let mut col = column![text("文本工具").size(24), text(""), tab_row, text("")];

    // match 匹配当前标签页
    match state.text_tool_tab {
        TextToolTab::Base64 => {
            col = col.push(text("输入:"));
            col = col.push(text_input("", &state.text_tool_input).on_input(Message::TextToolInputChanged));
            col = col.push(text(""));
            let btn_row = row![
                button("编码").on_press(Message::EncodeBase64),
                button("解码").on_press(Message::DecodeBase64),
                button("清空").on_press(Message::ClearTextTool),
            ];
            col = col.push(btn_row);
            col = col.push(text(""));
            col = col.push(text("输出:"));
            col = col.push(text(&state.text_tool_output));
        }
        TextToolTab::UrlEncode => {
            col = col.push(text("输入:"));
            col = col.push(text_input("", &state.text_tool_input).on_input(Message::TextToolInputChanged));
            col = col.push(text(""));
            let btn_row = row![
                button("编码").on_press(Message::EncodeUrl),
                button("解码").on_press(Message::DecodeUrl),
                button("清空").on_press(Message::ClearTextTool),
            ];
            col = col.push(btn_row);
            col = col.push(text("输出:"));
            col = col.push(text(&state.text_tool_output));
        }
        TextToolTab::Timestamp => {
            col = col.push(checkbox(state.use_current_time).on_toggle(Message::UseCurrentTimeChanged));
            col = col.push(text(""));
            let input_row = row![
                text("时间戳: "),
                text_input("", &state.timestamp_input)
                    .width(200)
                    .on_input(Message::TimestampInputChanged),
                button("转换").on_press(Message::ConvertTimestamp),
            ];
            col = col.push(input_row);
            if state.use_current_time {
                let now = Utc::now();
                col = col.push(text(format!("当前: {}", now.timestamp())));
            }
            if !state
                .timestamp_output
                .is_empty()
            {
                col = col.push(text(&state.timestamp_output));
            }
        }
        TextToolTab::JsonFormat => {
            col = col.push(text("输入:"));
            col = col.push(text_input("", &state.json_input).on_input(Message::JsonInputChanged));
            col = col.push(text(""));
            let btn_row =
                row![button("格式化").on_press(Message::FormatJson), button("压缩").on_press(Message::CompactJson),];
            col = col.push(btn_row);
            col = col.push(text("输出:"));
            col = col.push(text(&state.text_tool_output));
        }
    }

    scrollable(col).into()
}

// ============================================================================
// 系统信息面板 (行 542-)
// 核心控件: progress_bar, scrollable
// ============================================================================

// view_system_panel: 系统信息面板渲染
fn view_system_panel(state: &DashboardState) -> Element<Message> {
    // column!: 垂直布局
    let mut col = column![text("系统信息").size(24), text("")];
    // button: 按钮控件
    // on_press: 点击回调
    col = col.push(button("刷新").on_press(Message::RefreshSystemInfo));
    col = col.push(text(""));

    // &state.system: 引用系统实例
    let sys = &state.system;

    // 计算 CPU 平均使用率
    // .cpus(): CPU 数组
    // .iter(): 迭代器
    // .map(): 映射闭包
    // .cpu_usage(): CPU 使用率
    // .sum(): 求和
    // / len() as f32: 计算平均值
    let cpu: f32 = sys
        .cpus()
        .iter()
        .map(|c| c.cpu_usage())
        .sum::<f32>()
        / sys.cpus().len() as f32;

    // text!: 显示文本
    col = col.push(text(format!("CPU: {:.1}%", cpu)));

    // progress_bar!: 进度条控件
    // 0.0..=100.0: 范围
    col = col.push(progress_bar(0.0..=100.0, cpu));

    let total = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let pct = if total > 0.0 { used / total * 100.0 } else { 0.0 };
    col = col.push(text(format!("内存: {:.1} / {:.1} GB", used, total)));
    col = col.push(progress_bar(0.0..=100.0, pct as f32));

    col = col.push(text(""));
    col = col.push(text("进程 (按内存):"));

    let mut procs: Vec<_> = sys.processes().iter().collect();
    procs.sort_by(|a, b| b.1.memory().cmp(&a.1.memory()));

    let mut grid = column![];
    for (pid, p) in procs.iter().take(15) {
        grid =
            grid.push(text(format!("{} - {} - {:.0}MB", pid, p.name().to_string_lossy(), p.memory() as f64 / 1024.0)));
    }
    col = col.push(scrollable(grid).height(200));

    scrollable(col).into()
}

// ============================================================================
// 计算器面板 (行 602-)
// 核心控件: slider
// ============================================================================

// view_calculator_panel: 计算器面板渲染
fn view_calculator_panel(state: &DashboardState) -> Element<Message> {
    // 标签页数组
    let tabs = [(CalculatorTab::Bmi, "BMI"), (CalculatorTab::Simple, "计算")];
    // row!: 水平布局
    let mut tab_row = row![];
    // for 遍历
    for (tab, label) in tabs {
        // button: 按钮
        // .on_press: 点击回调
        let btn = button(text(label)).on_press(Message::CalculatorTabChanged(tab));
        tab_row = tab_row.push(btn);
    }

    let mut col = column![text("计算器").size(24), text(""), tab_row, text("")];

    match state.calc_tab {
        CalculatorTab::Bmi => {
            col = col.push(text(format!("身高: {:.0} cm", state.height)));
            col = col.push(slider(50.0..=250.0, state.height, |v| Message::HeightChanged(v)));
            col = col.push(text(format!("体重: {:.1} kg", state.weight)));
            col = col.push(slider(10.0..=200.0, state.weight, |v| Message::WeightChanged(v)));
            col = col.push(text(""));
            col = col.push(button("计算BMI").on_press(Message::CalculateBmi));
            col = col.push(text(""));
            col = col.push(text(&state.bmi_result).size(20));
        }
        CalculatorTab::Simple => {
            col = col.push(text_input("表达式", &state.calc_input).on_input(Message::CalcInputChanged));
            col = col.push(text(""));
            let btn_row =
                row![button("计算").on_press(Message::Calculate), button("清空").on_press(Message::ClearCalc),];
            col = col.push(btn_row);
            if !state.calc_result.is_empty() {
                col = col.push(text(format!("结果: {}", state.calc_result)).size(20));
            }
        }
    }

    scrollable(col).into()
}

// ============================================================================
// 第八部分：各面板视图函数
// ============================================================================

// ============================================================================
// 渲染模式配置 (通过环境变量控制)
// ============================================================================
//
// iced 通过环境变量控制渲染后端:
// | 环境变量 | 值 | 说明 |
// |---------|-----|------|
// | ICED_BACKEND | wgpu | GPU渲染 (默认) |
// | ICED_BACKEND | tiny-skia | 软件渲染 (CPU) |
//
// ============================================================================
// main 函数 (行 570-)
// 程序入口点
// ============================================================================

// main: 程序入口，返回 iced::Result
fn main() -> iced::Result {
    // DashboardState: 主应用状态结构体
    // ::default(): 调用 Default trait 创建默认状态
    let mut state = DashboardState::default();
    // System::refresh_all(): 立即刷新所有系统信息
    state.system.refresh_all();

    // iced::run(): 启动 iced 应用
    // 参数1 update: 状态更新函数，处理消息
    // 参数2 view: UI 渲染函数，根据状态显示 UI
    // iced 内部管理状态，会自动传递给 update 和 view
    // 使用默认后端 (wgpu) 运行
    iced::run(update, view)
}
