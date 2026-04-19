//! # 个人工作台 (Dashboard) - iced 0.14 教学示例
//!
//! 本文件展示了 iced 与 egui 的核心架构差异。
//!
//! ## 架构对比
//!
//! | 特性 | egui (立即模式) | iced (Elm 架构) |
//! |------|----------------|-----------------|
//! | UI 构建 | 每帧重建 | 声明式渲染 |
//! | 状态修改 | 直接在闭包中修改 | 通过 Message 更新 |
//! | 数据流 | 单向 | 单向 (State → View → Message → Update → State) |
//! | 类型安全 | 运行时检查 | 编译时检查 |
//!
//! ## iced 0.14 核心概念
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Elm 架构数据流                           │
//! │                                                             │
//! │   ┌─────────┐      ┌─────────┐      ┌─────────┐            │
//! │   │  State  │ ───→ │  View   │ ───→ │Element  │            │
//! │   │  (状态)  │      │ (渲染)  │      │ (UI)    │            │
//! │   └─────────┘      └─────────┘      └─────────┘            │
//! │        ↑                                    │               │
//! │        │                                    ↓               │
//! │   ┌─────────┐      ┌─────────┐      ┌─────────┐            │
//! │   │  Task   │ ←─── │ Update  │ ←─── │ Message │            │
//! │   │ (命令)   │      │ (更新)  │      │ (消息)  │            │
//! │   └─────────┘      └─────────┘      └─────────┘            │
//! │                                                             │
//! │   用户点击按钮 → 产生 Message → update() 处理 → 更新 State   │
//! │                    ↓                                        │
//! │              view() 重新渲染                                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 学习路径
//!
//! 1. `main()` 函数 → 应用启动方式
//! 2. `DashboardState` → 状态定义
//! 3. `Message` 枚举 → 所有用户交互类型
//! 4. `new()` 函数 → 初始化状态
//! 5. `update()` 函数 → 状态更新逻辑 (核心)
//! 6. `view()` 函数 → UI 渲染
//! 7. 各面板视图 → 控件使用示例

// ============================================================================
// 第一部分：导入依赖
// ============================================================================

// base64 编解码引擎 trait
use base64::Engine;

// chrono 时间处理库
// - TimeZone: 时区转换 trait
// - Utc: UTC 时区
use chrono::{TimeZone, Utc};

// iced 核心控件导入
// iced 使用 "widget" 模块组织所有 UI 控件
use iced::widget::{
    button,       // 按钮
    checkbox,     // 复选框
    column,       // 垂直布局容器
    container,    // 容器（用于装饰和布局）
    progress_bar, // 进度条
    row,          // 水平布局容器
    scrollable,   // 可滚动区域
    slider,       // 滑块
    text,         // 文本
    text_input,   // 文本输入框
};

// iced 核心类型导入
use iced::{
    window,  // 窗口模块
    Element, // UI 元素类型（所有控件的最终返回类型）
    Length,  // 尺寸类型（Fill 填充、Shrink 收缩、Fixed 固定）
    Padding, // 内边距类型
    Size,    // 尺寸结构体（宽高）
    Task,    // 异步任务（替代旧版的 Command）
    Theme,   // 主题类型
};

// sysinfo: 跨平台系统信息库
use sysinfo::System;

// ============================================================================
// 第二部分：枚举定义
// ============================================================================

/// 菜单项枚举
///
/// Rust 枚举非常适合表示 UI 中的有限选项集合。
/// 配合 #[derive(Default)] 可指定默认值。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum MenuItem {
    #[default] // 指定 Network 为默认值
    Network,
    TextTool,
    System,
    Calculator,
}

/// 文本工具标签页
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum TextToolTab {
    #[default]
    Base64,
    UrlEncode,
    Timestamp,
    JsonFormat,
}

/// 计算器标签页
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum CalculatorTab {
    #[default]
    Bmi,
    Simple,
}

// ============================================================================
// 第三部分：Message 枚举 ⭐ 核心概念
// ============================================================================

/// Message 枚举 - 定义所有可能的用户交互
///
/// Elm 架构的核心：所有用户交互都通过 Message 表示。
/// 类似于 Redux 中的 Action，Flux 中的 Dispatcher Action。
///
/// ## 设计原则
///
/// 1. 每个 Message 代表一个独立的用户意图
/// 2. Message 可以携带数据（如 String, bool, f32）
/// 3. Message 命名应该描述"发生了什么"，而不是"怎么做"
///
/// ## 示例
///
/// ```text
/// 用户点击"编码"按钮
///     ↓
/// 产生 Message::EncodeBase64
///     ↓
/// update() 函数接收并处理
///     ↓
/// 更新 state.text_tool_output
/// ```
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
enum Message {
    // ==================== 菜单导航 ====================
    /// 选择菜单项（携带 MenuItem 数据）
    SelectMenu(MenuItem),

    // ==================== 文本工具 ====================
    /// 文本输入变化（携带输入的字符串）
    /// 注意：每次按键都会触发这个消息
    TextToolInputChanged(String),
    /// 切换文本工具标签页
    TextToolTabChanged(TextToolTab),
    /// Base64 编码
    EncodeBase64,
    /// Base64 解码
    DecodeBase64,
    /// URL 编码
    EncodeUrl,
    /// URL 解码
    DecodeUrl,
    /// 清空文本工具
    ClearTextTool,

    // ==================== 时间戳工具 ====================
    /// 时间戳输入变化
    TimestampInputChanged(String),
    /// 转换时间戳
    ConvertTimestamp,
    /// 切换"使用当前时间"复选框（携带 bool 表示新状态）
    UseCurrentTimeChanged(bool),

    // ==================== JSON 工具 ====================
    /// JSON 输入变化
    JsonInputChanged(String),
    /// 格式化 JSON
    FormatJson,
    /// 压缩 JSON
    CompactJson,

    // ==================== 系统信息 ====================
    /// 刷新系统信息
    RefreshSystemInfo,

    // ==================== 计算器 ====================
    /// 切换计算器标签页
    CalculatorTabChanged(CalculatorTab),
    /// 身高变化（携带 f32 新值）
    HeightChanged(f32),
    /// 体重变化（携带 f32 新值）
    WeightChanged(f32),
    /// 计算 BMI
    CalculateBmi,
    /// 计算器输入变化
    CalcInputChanged(String),
    /// 执行计算
    Calculate,
    /// 清空计算器
    ClearCalc,
}

// ============================================================================
// 第四部分：State 状态结构 ⭐ 核心概念
// ============================================================================

/// DashboardState - 应用状态
///
/// Elm 架构中的 "Model" 部分。
/// 所有 UI 状态都应该在这里定义，view() 函数根据状态渲染。
///
/// ## 状态管理原则
///
/// 1. 单一数据源：所有状态集中在一个结构体中
/// 2. 不可变渲染：view() 只读取状态，不修改
/// 3. 纯函数更新：update() 是唯一修改状态的地方
///
/// ## 与 React 的类比
///
/// ```text
/// React:  const [count, setCount] = useState(0);
/// iced:   struct State { count: u32 }
///         Message::Increment => state.count += 1
/// ```
#[derive(Default)]
struct DashboardState {
    // ==================== 导航状态 ====================
    /// 当前选中的菜单项
    selected_menu: MenuItem,

    // ==================== 文本工具状态 ====================
    /// 文本工具输入内容
    text_tool_input: String,
    /// 文本工具输出内容
    text_tool_output: String,
    /// 当前选中的文本工具标签页
    text_tool_tab: TextToolTab,

    // ==================== 时间戳工具状态 ====================
    /// 时间戳输入
    timestamp_input: String,
    /// 时间戳转换结果
    timestamp_output: String,
    /// 是否显示当前时间
    use_current_time: bool,

    // ==================== JSON 工具状态 ====================
    /// JSON 输入内容
    json_input: String,

    // ==================== 系统信息状态 ====================
    /// 系统信息（sysinfo 库）
    /// System 实现了 Default，但我们需要手动刷新数据
    system: System,

    // ==================== 计算器状态 ====================
    /// 计算器当前标签页
    calc_tab: CalculatorTab,
    /// BMI 计算器 - 身高 (cm)
    height: f32,
    /// BMI 计算器 - 体重 (kg)
    weight: f32,
    /// BMI 计算结果
    bmi_result: String,
    /// 简单计算器 - 输入表达式
    calc_input: String,
    /// 简单计算器 - 计算结果
    calc_result: String,
}

// ============================================================================
// 第五部分：核心函数 (Elm 架构三大核心)
// ============================================================================

/// new - 应用初始化函数
///
/// iced 0.14 使用函数式初始化，不再需要 trait。
///
/// ## 返回值
///
/// 返回元组 `(State, Task<Message>)`：
/// - State: 初始化的应用状态
/// - Task: 初始任务（如异步加载数据），Task::none() 表示无任务
///
/// ## 与 iced 旧版本对比
///
/// ```text
/// // 旧版本 (Application trait)
/// fn new(flags: Flags) -> (Self, Command<Message>) { ... }
///
/// // 新版本 (函数式)
/// fn new() -> (State, Task<Message>) { ... }
/// ```
fn new() -> (DashboardState, Task<Message>) {
    // 创建默认状态
    let mut state = DashboardState::default();

    // 初始化系统信息
    // System::default() 创建空实例，需要 refresh_all() 填充数据
    state.system.refresh_all();

    // 返回状态和无任务
    // Task::none() 类似于返回空的 Command
    (state, Task::none())
}

/// update - 状态更新函数 ⭐ Elm 架构核心
///
/// 这是 Elm 架构的心脏：所有状态变化都在这里发生。
///
/// ## 函数签名解读
///
/// ```text
/// fn update(state: &mut State, message: Message) -> Task<Message>
///            ↑                    ↑                    ↑
///            可变引用             消息枚举              可能的异步任务
/// ```
///
/// ## 设计原则
///
/// 1. 纯函数：相同输入产生相同输出
/// 2. 唯一修改点：所有状态修改都在这里
/// 3. 返回 Task：用于异步操作（网络请求、定时器等）
///
/// ## match 模式匹配
///
/// Rust 的 match 是穷尽的，确保处理所有 Message 变体。
/// 编译器会在遗漏时报错，这是类型安全的体现。
fn update(state: &mut DashboardState, message: Message) -> Task<Message> {
    match message {
        // ==================== 菜单导航 ====================
        // 简单的状态赋值，Message 携带的数据直接更新状态
        Message::SelectMenu(menu) => {
            state.selected_menu = menu;
        }

        // ==================== 文本工具 ====================
        // 文本输入：每次按键都触发，直接更新字符串
        Message::TextToolInputChanged(s) => {
            state.text_tool_input = s;
        }

        // 标签页切换：同时清空输出
        Message::TextToolTabChanged(tab) => {
            state.text_tool_tab = tab;
            state.text_tool_output.clear();
        }

        // Base64 编码
        // base64::engine::general_purpose::STANDARD 是预定义的编码器
        Message::EncodeBase64 => {
            state.text_tool_output = base64::engine::general_purpose::STANDARD.encode(&state.text_tool_input);
        }

        // Base64 解码
        // 使用 Result 处理可能的解码失败
        Message::DecodeBase64 => {
            state.text_tool_output = base64::engine::general_purpose::STANDARD
                .decode(&state.text_tool_input)
                .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }

        // URL 编码
        Message::EncodeUrl => {
            state.text_tool_output = urlencoding::encode(&state.text_tool_input).to_string();
        }

        // URL 解码
        Message::DecodeUrl => {
            state.text_tool_output = urlencoding::decode(&state.text_tool_input)
                .map(|cow| cow.to_string())
                .unwrap_or_else(|_| "解码失败".to_string());
        }

        // 清空
        Message::ClearTextTool => {
            state.text_tool_input.clear();
            state.text_tool_output.clear();
        }

        // ==================== 时间戳工具 ====================
        Message::TimestampInputChanged(s) => {
            state.timestamp_input = s;
        }

        Message::ConvertTimestamp => {
            // 解析字符串为 i64
            if let Ok(ts) = state
                .timestamp_input
                .parse::<i64>()
            {
                // chrono 的 timestamp_opt 返回多种可能结果
                // .single() 确保是唯一有效的时间
                if let Some(datetime) = Utc
                    .timestamp_opt(ts, 0)
                    .single()
                {
                    // 格式化输出
                    state.timestamp_output = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
                } else {
                    state.timestamp_output = "时间戳无效".to_string();
                }
            } else {
                state.timestamp_output = "请输入有效的数字".to_string();
            }
        }

        // 复选框切换：on_toggle 传递新的 bool 值
        Message::UseCurrentTimeChanged(b) => {
            state.use_current_time = b;
        }

        // ==================== JSON 工具 ====================
        Message::JsonInputChanged(s) => {
            state.json_input = s;
        }

        Message::FormatJson => {
            // serde_json 解析为 Value，然后美化输出
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

        // ==================== 系统信息 ====================
        Message::RefreshSystemInfo => {
            state.system.refresh_all();
        }

        // ==================== 计算器 ====================
        Message::CalculatorTabChanged(tab) => {
            state.calc_tab = tab;
        }

        // slider 的 on_change 回调传递新值
        Message::HeightChanged(h) => {
            state.height = h;
        }

        Message::WeightChanged(w) => {
            state.weight = w;
        }

        Message::CalculateBmi => {
            let height_m = state.height / 100.0;
            let bmi = state.weight / (height_m * height_m);

            // BMI 分类（中国标准）
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

        Message::CalcInputChanged(s) => {
            state.calc_input = s;
        }

        Message::Calculate => {
            state.calc_result = calculate_simple(&state.calc_input);
        }

        Message::ClearCalc => {
            state.calc_input.clear();
            state.calc_result.clear();
        }
    }

    // 返回空任务
    // 如果需要异步操作，可以返回 Task::perform(...)
    Task::none()
}

/// view - UI 渲染函数 ⭐ Elm 架构核心
///
/// 根据状态渲染 UI，类似于 React 的 render 函数。
///
/// ## 重要原则
///
/// 1. **纯函数**：相同状态产生相同 UI
/// 2. **只读**：不能修改状态（注意参数是 & 而不是 &mut）
/// 3. **声明式**：描述"UI 应该是什么样子"，而不是"怎么画"
///
/// ## Element 类型
///
/// ```text
/// Element<'a, Message, Theme = Theme, Renderer = Renderer>
///         │    │
///         │    └── Message: 这个元素产生的消息类型
///         └── 生命周期，通常使用 '_ 自动推断
/// ```
///
/// ## 宏的使用
///
/// iced 提供了便捷的宏来创建布局：
/// - `column![...]`: 垂直布局
/// - `row![...]`: 水平布局
fn view(state: &DashboardState) -> Element<'_, Message> {
    // 渲染左侧菜单面板
    let menu_panel = view_menu_panel(state.selected_menu);

    // 根据当前菜单选择，渲染对应的内容面板
    // match 表达式确保处理所有菜单项
    let content = match state.selected_menu {
        MenuItem::Network => view_network_panel(),
        MenuItem::TextTool => view_text_tool_panel(state),
        MenuItem::System => view_system_panel(state),
        MenuItem::Calculator => view_calculator_panel(state),
    };

    // row! 宏创建水平布局
    // .into() 将 Row 转换为 Element
    row![menu_panel, content].into()
}

/// theme - 主题设置函数
///
/// 返回当前应用使用的主题。
/// 可以根据状态动态切换主题（如深色/浅色模式）。
fn theme(_state: &DashboardState) -> Theme {
    Theme::Dark
}

/// title - 窗口标题函数
///
/// 返回窗口标题，可以根据状态动态设置。
fn title(_state: &DashboardState) -> String {
    "个人工作台".to_string()
}

// ============================================================================
// 第六部分：辅助函数
// ============================================================================

/// 简单表达式计算包装函数
fn calculate_simple(input: &str) -> String {
    let input = input.replace(' ', "");
    match evaluate_expression(&input) {
        Ok(result) => format!("{}", result),
        Err(e) => e,
    }
}

/// 简单表达式求值（从左到右计算，不支持运算符优先级）
///
/// 算法：遍历字符，遇到运算符时计算上一个运算符的结果
fn evaluate_expression(input: &str) -> Result<f64, String> {
    let input = input.replace(' ', "");

    if input.is_empty() {
        return Err("请输入表达式".to_string());
    }

    let mut result = 0.0;
    let mut current_num = 0.0;
    let mut last_op = '+'; // 初始为 '+'，处理第一个数字
    let mut has_number = false;

    for c in input.chars() {
        if c.is_ascii_digit() {
            // 处理多位数：123 = 1*100 + 2*10 + 3
            current_num = current_num * 10.0 + (c as u8 - b'0') as f64;
            has_number = true;
        } else if "+-*/".contains(c) {
            if !has_number {
                return Err("表达式格式错误".to_string());
            }

            // 执行上一个运算符的计算
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
// 第七部分：各面板视图函数
// ============================================================================

/// 菜单面板
///
/// 展示控件：
/// - column!: 垂直布局
/// - button: 按钮
/// - text: 文本
/// - container: 容器
/// - Length::Fill: 填充宽度
fn view_menu_panel(_selected: MenuItem) -> Element<'static, Message> {
    let items = [
        (MenuItem::Network, "网络"),
        (MenuItem::TextTool, "文本工具"),
        (MenuItem::System, "系统信息"),
        (MenuItem::Calculator, "计算器"),
    ];

    let mut col = column![text("工具菜单").size(20), text(""),].spacing(10);

    for (menu, label) in items {
        // 添加样式：使用 button::primary 预设样式
        let btn = button(text(label))
            .on_press(Message::SelectMenu(menu))
            .width(Length::Fill)
            .padding(Padding::from(8))
            .style(button::primary);
        col = col.push(btn);
    }

    // 容器添加背景和边框样式
    container(col)
        .width(180)
        .padding(10)
        .style(container::transparent)
        .into()
}

/// 网络监控面板
///
/// 展示控件：
/// - System: 系统信息获取
/// - scrollable: 可滚动区域
fn view_network_panel() -> Element<'static, Message> {
    // System::new_all() 创建包含所有信息的实例
    let sys = System::new_all();

    let col = column![
        text("网络监控").size(24),
        text(""),
        text(format!("进程数: {}", sys.processes().len())),
        text(format!("CPU核心: {}", sys.cpus().len())),
    ]
    .spacing(10);

    container(scrollable(col))
        .padding(10)
        .style(container::transparent)
        .into()
}

/// 文本工具面板
///
/// 展示控件：
/// - text_input: 文本输入框
/// - checkbox: 复选框
/// - row!: 水平布局
fn view_text_tool_panel(state: &DashboardState) -> Element<'_, Message> {
    let tabs = [
        (TextToolTab::Base64, "Base64"),
        (TextToolTab::UrlEncode, "URL编码"),
        (TextToolTab::Timestamp, "时间戳"),
        (TextToolTab::JsonFormat, "JSON"),
    ];

    let mut tab_row = row![].spacing(5);
    for (tab, label) in tabs {
        let btn = button(text(label))
            .on_press(Message::TextToolTabChanged(tab))
            .padding(Padding::from(5))
            .style(button::secondary);
        tab_row = tab_row.push(btn);
    }

    let mut col = column![text("文本工具").size(24), text(""), tab_row, text(""),].spacing(10);

    match state.text_tool_tab {
        TextToolTab::Base64 => {
            col = col.push(text("输入:"));
            col = col.push(
                text_input("", &state.text_tool_input)
                    .on_input(Message::TextToolInputChanged)
                    .padding(Padding::from(8)),
            );
            col = col.push(text(""));

            let btn_row = row![
                button("编码")
                    .on_press(Message::EncodeBase64)
                    .padding(Padding::from(8))
                    .style(button::primary),
                button("解码")
                    .on_press(Message::DecodeBase64)
                    .padding(Padding::from(8))
                    .style(button::secondary),
                button("清空")
                    .on_press(Message::ClearTextTool)
                    .padding(Padding::from(8))
                    .style(button::danger),
            ]
            .spacing(10);
            col = col.push(btn_row);
            col = col.push(text(""));
            col = col.push(text("输出:"));
            col = col.push(text(&state.text_tool_output));
        }

        TextToolTab::UrlEncode => {
            col = col.push(text("输入:"));
            col = col.push(
                text_input("", &state.text_tool_input)
                    .on_input(Message::TextToolInputChanged)
                    .padding(Padding::from(8)),
            );
            col = col.push(text(""));

            let btn_row = row![
                button("编码")
                    .on_press(Message::EncodeUrl)
                    .padding(Padding::from(8))
                    .style(button::primary),
                button("解码")
                    .on_press(Message::DecodeUrl)
                    .padding(Padding::from(8))
                    .style(button::secondary),
                button("清空")
                    .on_press(Message::ClearTextTool)
                    .padding(Padding::from(8))
                    .style(button::danger),
            ]
            .spacing(10);
            col = col.push(btn_row);
            col = col.push(text("输出:"));
            col = col.push(text(&state.text_tool_output));
        }

        TextToolTab::Timestamp => {
            col = col.push(
                checkbox(state.use_current_time)
                    .label("使用当前时间")
                    .on_toggle(Message::UseCurrentTimeChanged),
            );
            col = col.push(text(""));

            let input_row = row![
                text("时间戳: "),
                text_input("", &state.timestamp_input)
                    .width(200)
                    .on_input(Message::TimestampInputChanged)
                    .padding(Padding::from(8)),
                button("转换")
                    .on_press(Message::ConvertTimestamp)
                    .padding(Padding::from(8))
                    .style(button::primary),
            ]
            .spacing(10);
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
            col = col.push(
                text_input("", &state.json_input)
                    .on_input(Message::JsonInputChanged)
                    .padding(Padding::from(8)),
            );
            col = col.push(text(""));

            let btn_row = row![
                button("格式化")
                    .on_press(Message::FormatJson)
                    .padding(Padding::from(8))
                    .style(button::primary),
                button("压缩")
                    .on_press(Message::CompactJson)
                    .padding(Padding::from(8))
                    .style(button::secondary),
            ]
            .spacing(10);
            col = col.push(btn_row);
            col = col.push(text("输出:"));
            col = col.push(text(&state.text_tool_output));
        }
    }

    container(scrollable(col))
        .padding(10)
        .style(container::transparent)
        .into()
}

/// 系统信息面板
///
/// 展示控件：
/// - progress_bar: 进度条
/// - scrollable 嵌套
fn view_system_panel(state: &DashboardState) -> Element<'_, Message> {
    let mut col = column![text("系统信息").size(24), text(""),];

    col = col.push(
        button("刷新")
            .on_press(Message::RefreshSystemInfo)
            .padding(Padding::from(8))
            .style(button::primary),
    );
    col = col.push(text(""));

    let sys = &state.system;

    let cpu: f32 = sys
        .cpus()
        .iter()
        .map(|c| c.cpu_usage())
        .sum::<f32>()
        / sys.cpus().len() as f32;

    col = col.push(text(format!("CPU: {:.1}%", cpu)));
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

    let mut grid = column![].spacing(5);
    for (pid, p) in procs.iter().take(15) {
        grid =
            grid.push(text(format!("{} - {} - {:.0}MB", pid, p.name().to_string_lossy(), p.memory() as f64 / 1024.0)));
    }

    col = col.push(scrollable(grid).height(200));

    container(scrollable(col))
        .padding(10)
        .style(container::transparent)
        .into()
}

/// 计算器面板
fn view_calculator_panel(state: &DashboardState) -> Element<'_, Message> {
    let tabs = [(CalculatorTab::Bmi, "BMI"), (CalculatorTab::Simple, "计算")];

    let mut tab_row = row![].spacing(5);
    for (tab, label) in tabs {
        let btn = button(text(label))
            .on_press(Message::CalculatorTabChanged(tab))
            .padding(Padding::from(5))
            .style(button::secondary);
        tab_row = tab_row.push(btn);
    }

    let mut col = column![text("计算器").size(24), text(""), tab_row, text(""),].spacing(10);

    match state.calc_tab {
        CalculatorTab::Bmi => {
            col = col.push(text(format!("身高: {:.0} cm", state.height)));
            col = col.push(slider(50.0..=250.0, state.height, Message::HeightChanged));

            col = col.push(text(format!("体重: {:.1} kg", state.weight)));
            col = col.push(slider(10.0..=200.0, state.weight, Message::WeightChanged));

            col = col.push(text(""));
            col = col.push(
                button("计算BMI")
                    .on_press(Message::CalculateBmi)
                    .padding(Padding::from(8))
                    .style(button::success),
            );
            col = col.push(text(""));
            col = col.push(text(&state.bmi_result).size(20));
        }

        CalculatorTab::Simple => {
            col = col.push(
                text_input("表达式", &state.calc_input)
                    .on_input(Message::CalcInputChanged)
                    .padding(Padding::from(8)),
            );
            col = col.push(text(""));

            let btn_row = row![
                button("计算")
                    .on_press(Message::Calculate)
                    .padding(Padding::from(8))
                    .style(button::primary),
                button("清空")
                    .on_press(Message::ClearCalc)
                    .padding(Padding::from(8))
                    .style(button::danger),
            ]
            .spacing(10);
            col = col.push(btn_row);

            if !state.calc_result.is_empty() {
                col = col.push(text(format!("结果: {}", state.calc_result)).size(20));
            }
        }
    }

    container(scrollable(col))
        .padding(10)
        .style(container::transparent)
        .into()
}

// ============================================================================
// 第八部分：main 函数 - 应用启动
// ============================================================================

/// main - 应用入口
///
/// iced 0.14 使用 builder 模式构建应用，替代了旧版的 Application trait。
///
/// ## builder 链式调用
///
/// ```text
/// iced::application(new, update, view)  // 必需：初始化、更新、渲染函数
///     .title(title)                      // 可选：窗口标题
///     .theme(theme)                      // 可选：主题
///     .window_size(Size::new(w, h))      // 可选：窗口大小
///     .resizable(true)                   // 可选：是否可调整大小
///     .run()                             // 启动应用
/// ```
///
/// ## 与旧版本对比
///
/// ```text
/// // 旧版本 (Application trait)
/// DashboardState::run(Settings {
///     window: iced::window::Settings { ... },
///     ...,
/// })
///
/// // 新版本 (builder 模式)
/// iced::application(new, update, view)
///     .window_size(...)
///     .run()
/// ```
///
/// ## 其他配置选项
///
/// - `.subscription(fn)`: 订阅（定时器、事件监听等）
/// - `.scale_factor(fn)`: 缩放因子
/// - `.antialiasing(bool)`: 抗锯齿
/// - `.font(Font)`: 默认字体
/// - `.centered()`: 窗口居中
fn main() -> iced::Result {
    iced::application(new, update, view)
        .title(title)
        .theme(theme)
        .window(window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(0.0, 0.0)), // 设置最小窗口尺寸
            resizable: true,
            ..window::Settings::default()
        })
        .run()
}
