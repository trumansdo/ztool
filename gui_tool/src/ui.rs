//! UI 层入口模块
//!
//! 提供应用启动、状态管理、视图渲染的顶层接口。
//!
//! # Rust 语法要点
//!
//! ## 枚举 (enum)
//! Rust 的枚举比大多数语言更强大 —— 每个变体 (variant) 可以携带数据：
//! ```text
//! enum Message {
//!     UnitVariant,                    // 无数据变体
//!     TupleVariant(u32, String),      // 携带匿名元组
//!     StructVariant { x: u32, y: u32 } // 携带命名字段
//! }
//! ```
//! 这种枚举称为"代数数据类型" (Algebraic Data Type, ADT)，
//! 相当于"和类型" (sum type) —— 值可以是"这个或那个"。
//!
//! ## `derive` 属性
//! `#[derive(Debug, Clone)]` 是过程宏 (procedural macro)，自动生成 trait 实现：
//! - `Debug`: 生成 `{:?}` 格式化输出（用于调试打印）
//! - `Clone`: 生成 `.clone()` 方法（深拷贝）
//! - `Copy`: 位复制语义（Copy 是 Clone 的 marker trait 特化，要求所有字段都是 Copy）
//! - `PartialEq/Eq`: 相等比较
//! - `Default`: 生成 `Default::default()` 的默认值（对于 enum，默认值是带 `#[default]` 的变体）
//!
//! ## Trait (类似接口但更强大)
//! Rust 的 trait 定义共享行为，类似 Java interface 或 Haskell typeclass：
//! - `impl TraitName for TypeName { ... }` 为类型实现 trait
//! - 孤儿规则 (Orphan Rule): 实现 trait 时，trait 或 type 至少有一个在当前 crate 中定义
//! - trait 可以包含默认方法实现，实现者只需覆盖需要定制的方法
//!
//! ## `impl` 块
//! `impl TypeName { ... }` 为类型添加方法。与 trait impl 不同，
//! 这里定义的是"固有方法" (inherent methods)，属于该类型自己的方法。
//!
//! # 架构图
//! ```text
//! fn main() -> Result<()>
//!   └── ui::run() -> Result<()>
//!        ├── iced::application(new, update, view)
//!        │    ├── fn new() -> (App, Task<Message>)       // 初始化状态
//!        │    ├── fn update(&mut App, Message) -> Task   // 处理消息
//!        │    ├── fn view(&App) -> Element<Message>      // 渲染界面
//!        │    ├── fn title(&App) -> String               // 窗口标题
//!        │    └── fn theme(&App) -> Theme                // 应用主题
//!        └── .window() → .run()    // 窗口配置 + 启动事件循环
//! ```
//!
//! # 为什么 iced 需要这么多函数？
//! iced 采用类似于 Elm 的架构，这些函数定义了应用的 5 个核心面：
//! - `new`: 创建初始状态 → 程序启动时调用一次
//! - `update`: 响应消息 → 用户操作/定时器等触发
//! - `view`: 渲染界面 → 每次状态变化后自动调用
//! - `title`: 窗口标题 → 可随状态动态变化
//! - `theme`: 主题选择 → 返回 Light/Dark/Custom 等

// `pub mod` 声明：当前模块 `ui` 下的子模块
// 编译器会查找 ui/app.rs, ui/menu.rs, ui/widgets.rs (或 ui/widgets/mod.rs)
pub mod app;
pub mod menu;
pub mod widgets;

// re-export: 将 `App` 从 `app` 模块提升到 `ui` 模块的公共接口
// 这样外部可以直接 `use crate::ui::App` 而非 `use crate::ui::app::App`
pub use app::App;

use iced::{Element, Task};
use anyhow::Result;

/// 标签页枚举 —— 定义应用中的所有功能页面
///
/// # Rust: enum 的属性宏
/// - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]`
///   自动生成多个 trait 的实现：
///   - `Debug`: 调试输出 `Tab::JsonFmt` 打印为 `JsonFmt`
///   - `Clone`: 深拷贝（无堆数据的类型 clone 即 copy）
///   - `Copy`: 位拷贝，Copy 类型赋值后原变量仍可用（无 move 语义）
///   - `PartialEq/Eq`: 可以用 `==` 和 `!=` 比较
///   - `Hash`: 可放入 `HashSet` 或 `HashMap` 作为 key
///   - `Default`: 生成默认值，`#[default]` 标记的变体 = `Tab::JsonFmt`
///
/// # 何时用 Copy？
/// 只有字段全是 Copy 类型（整数、浮点、bool、Copy enum、无堆内存的 struct）时
/// 才能 derive Copy。包含 String/Vec 等堆分配类型的 struct 不能 Copy。
/// 当前 enum 无携带数据，全部变体都是 Copy。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Tab {
    /// JSON 格式化工具（默认页面）
    #[default]
    JsonFmt,
    /// 端口扫描工具，基于 rayon 并行扫描
    NetPortScan,
    /// 网络抓包工具，基于 pnet 捕获数据包
    NetCapture,
    /// UI 组件库展示，测试 iced_aw 各种组件
    UiLibs,
}

// `impl From<Tab> for String`: 实现类型转换 —— 从 Tab 转为 String
//
// # Rust: From<T> trait
// `From<T>` 是标准库的转换 trait，定义 `fn from(value: T) -> Self`。
// 实现 `From<Tab> for String` 后，可以使用 `String::from(tab)` 或 `tab.into()`。
// 标准库自动提供反向的 `Into<String> for Tab`（只要实现了 From）。
impl From<Tab> for String {
    fn from(t: Tab) -> Self {
        // `match` 模式匹配 —— Rust 最核心的控制流结构之一
        // 编译器强制检查所有分支是否穷尽（exhaustiveness checking）
        // 如果遗漏任何变体，编译报错
        match t {
            Tab::JsonFmt => "json_fmt".to_string(),
            Tab::NetPortScan => "net_port_scan".to_string(),
            Tab::NetCapture => "net_capture".to_string(),
            Tab::UiLibs => "ui_libs".to_string(),
        }
    }
}

/// `impl Tab` —— 为 Tab 类型添加固有方法（非 trait 实现）
impl Tab {
    /// 从菜单项标识符反查对应的 Tab 变体
    ///
    /// # Rust: `Option<T>` 类型
    /// `Option<T>` 是 Rust 的"可能存在的值"，代替 null：
    /// - `Some(T)` —— 有值
    /// - `None` —— 无值
    /// 使用 Option 的好处：编译器强制处理 None 情况，消除 null pointer 错误。
    ///
    /// # Rust: match 守卫与 `_` 通配符
    /// `_ => None` 中的 `_` 匹配任何值，通常放在 match 最后作为兜底分支。
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "json_fmt" => Some(Tab::JsonFmt),
            "net_port_scan" => Some(Tab::NetPortScan),
            "net_capture" => Some(Tab::NetCapture),
            "ui_libs" => Some(Tab::UiLibs),
            _ => None,
        }
    }
}

/// 全局消息枚举 —— 定义应用中所有可能的消息/事件
///
/// # 设计模式: 分层消息包装
/// 该枚举采用"消息树"模式：顶层消息包含子模块消息作为变体数据。
/// ```text
/// Message::JsonFmt(json_fmt::Msg)
///         ↑                    ↑
///    顶层消息变体          子模块自己的消息枚举
/// ```
/// 这种模式在 iced/Elm 应用中非常常见：
/// 1. 子模块定义自己的 Msg 枚举
/// 2. 顶层 Message 用一个变体包装子模块的 Msg
/// 3. update 中先解包再转交给子模块的 update 函数
///
/// # Rust: 携带数据的 enum 变体
/// `JsonFmt(crate::features::json_fmt::Msg)` 是 tuple-style 变体，
/// 携带一个 `json_fmt::Msg` 类型的值。
#[derive(Debug, Clone)]
pub enum Message {
    /// 切换顶部菜单栏的下拉展开状态
    ToggleMenu(Option<usize>),
    /// 切换侧边栏分类的展开/折叠
    ToggleCategory(String),
    /// 切换到指定标签页
    TabSelected(Tab),
    /// JSON 格式化模块消息（格式化输入、复制结果等）
    JsonFmt(crate::features::json_fmt::Msg),
    /// 端口扫描模块消息（修改目标IP、扫描结果更新等）
    NetPortScan(crate::features::net_port_scan::Msg),
    /// 网络抓包模块消息（开始/停止抓包、数据包到达等）
    NetCapture(crate::features::net_capture::Msg),
    /// UI 组件库消息（切换子标签页、组件交互等）
    UiLibs(crate::features::ui_libs::Msg),
}

/// 创建初始应用状态
///
/// # TEA 架构: new 函数
/// 这是 Elm Architecture 的初始化函数，程序启动时 iced 调用它。
/// 返回 `(初始状态, 初始异步任务)`。
/// 初始 Task 可用于启动后台任务（如加载文件、发起网络请求）。
/// 这里返回 `Task::none()`，表示没有初始异步任务。
fn new() -> (App, Task<Message>) {
    (App::new(), Task::none())
}

/// 处理消息，更新应用状态
///
/// # TEA 架构: update 函数
/// iced 在用户交互或其他事件产生 Message 后调用此函数。
/// 签名 `fn(&mut App, Message) -> Task<Message>` 意味着：
/// - 「接收可变引用」: 可以修改状态
/// - 「返回 Task」: 可以触发后续异步操作（如启动端口扫描的 tokio 任务）
/// - 不能在此函数中阻塞！长时间操作必须通过 Task 异步执行
///
/// # Rust: `&mut` 可变引用
/// `&mut state` 是可变借用 (mutable borrow)。
/// Rust 的借用规则：
/// - 同一时刻只能有一个可变借用，或者多个不可变借用
/// - 可变借用期间不能同时有不可变借用
/// 这保证了编译期的线程安全（无数据竞争）。
fn update(state: &mut App, message: Message) -> Task<Message> {
    state.update(message)
}

/// 渲染应用界面
///
/// # TEA 架构: view 函数
/// 纯函数：接收只读引用 `&App`，返回 Element 树。
/// 不能修改状态 —— 所有修改必须通过 update 函数。
/// iced 在状态变化后自动重新调用此函数生成新的虚拟 DOM。
///
/// # Rust: `&` 不可变引用 (共享引用)
/// `&state` 是不可变借用，view 函数只读不写。
/// 可以有任意多个不可变借用同时存在。
fn view(state: &App) -> Element<'_, Message> {
    state.view()
}

/// 获取窗口标题
///
/// # iced 的 title 函数
/// 此函数在应用启动时和标题变化时被调用。
/// 可以返回动态标题（如 "编辑器 - 未命名" / "编辑器 - file.txt"）。
/// 这里始终返回固定标题。
fn title(_state: &App) -> String {
    // Rust: 以下划线开头的变量名 `_state` 抑制"未使用"警告
    App::title()
}

/// 获取应用主题
///
/// # Rust: `iced::Theme` 枚举
/// iced 内置 Light 和 Dark 主题。本项目使用 `iced::Theme::Dark`。
fn theme(_state: &App) -> iced::Theme {
    App::theme()
}

/// 启动 iced 桌面应用
///
/// # iced application builder 模式
/// `iced::application(new, update, view)` 创建应用构建器，
/// 然后通过链式方法调用配置窗口属性和字体。
/// 这是 Rust 中常见的"构建器模式" (Builder Pattern)：
/// 每个 `.method()` 调用返回 `Self`，允许链式调用。
///
/// # Rust: `iced::Font::with_name("Segoe UI")`
/// 设置默认字体为 Segoe UI（Windows 系统字体）。
/// 如果系统中不存在该字体，iced 会回退到内置的默认字体。
///
/// # Rust: `iced::window::Settings` 结构体
/// `..iced::window::Settings::default()` 使用"结构体更新语法" (struct update syntax)：
/// 从默认值复制所有未显式设置的字段，只覆盖指定的字段。
///
/// # `Result<()>` 返回类型
/// `application.run()` 在异常退出时返回错误。
pub fn run() -> Result<()> {
    iced::application(new, update, view)
        .title(title)
        .theme(theme)
        .default_font(iced::Font::with_name("Segoe UI"))
        .window(iced::window::Settings {
            // iced::Size 是一个元组结构体: Size { width: f32, height: f32 }
            size: iced::Size::new(1000.0, 700.0),
            // Option<Size>: 设置最小窗口尺寸，防止窗口被缩得太小
            min_size: Some(iced::Size::new(800.0, 600.0)),
            // 允许用户拖拽窗口边缘调整大小
            resizable: true,
            // `..default()` 将其他字段（如 position, decorations 等）设置为 iced 默认值
            ..iced::window::Settings::default()
        })
        .run()?;
    Ok(())
}
