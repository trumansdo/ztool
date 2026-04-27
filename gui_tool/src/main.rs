//! gui_tool - 综合工具集
//!
//! 基于 iced GUI 框架的多功能桌面工具，采用 Elm Architecture (TEA) 架构。
//!
//! # 功能模块
//! - **JSON格式化工具**: 格式化、压缩、验证 JSON 数据
//! - **网络端口扫描**: 并行扫描主机端口，实时显示进度
//! - **网络抓包分析**: 捕获网络流量，解析 HTTP/TCP/UDP 协议
//! - **UI组件库展示**: 展示按钮、卡片、徽章等组件
//!
//! # 架构
//! 采用 The Elm Architecture (TEA) —— 纯函数式 UI 架构模式：
//! - **State** (模型): 应用状态，存储在 `App` 结构体中
//! - **Message** (消息): 用户交互和事件，定义为 `Message` 枚举
//! - **Update** (更新): 纯函数 `fn(&mut State, Message) -> Task<Message>`，
//!   接收消息后更新状态，返回后续异步任务
//! - **View** (视图): 纯函数 `fn(&State) -> Element<Message>`，
//!   根据状态渲染 UI，不修改状态
//!
//! # Rust 语法要点
//!
//! ## `!` 作为 crate 级注释
//! `//!` 是模块/crate 级文档注释（inner doc comment），用于描述当前模块。
//! 它与 `///` 不同 —— `///` 用于下一个声明项（outer doc comment）。
//! 两者都会生成 `cargo doc` 文档。
//!
//! ## `mod` 模块系统
//! Rust 使用 `mod` 关键字声明子模块。编译器会按以下顺序查找模块文件：
//! 1. `<module_name>.rs` (如 `features.rs`)
//! 2. `<module_name>/mod.rs` (旧式，如 `features/mod.rs`)
//! 模块树在编译时静态确定，不依赖文件系统。
//!
//! ## `use` vs 完全限定路径
//! `use` 将外部路径引入当前作用域，减少重复书写。
//! 例如 `use anyhow::Result` 之后可以直接写 `Result` 而非 `anyhow::Result`。
//! Rust 的 `use` 类似于其他语言的 `import`。
//!
//! ## Cargo Workspace (工作区)
//! 本项目根 `Cargo.toml` 定义了工作区成员 `gui_tool` 和 `study_example`。
//! 工作区共享同一个 `Cargo.lock` 和 `target/` 构建目录，减少重复编译。
//!
//! # 调用链 (从 main 到最深层)
//! ```text
//! fn main()                            // 第7层 - 程序入口
//!   └── ui::run()                      // 第6层 - 启动 iced 事件循环
//!        └── iced::application()       // 框架层
//!             ├── new() -> App         // 第5层 - 初始化应用状态
//!             │    └── App::new()      // 第4层 - 创建各功能模块
//!             ├── update(Message)      // 第5层 - 处理消息
//!             │    └── App::update()   // 第4层 - 分发到子模块
//!             │         └── 各feature::update()
//!             └── view() -> Element    // 第5层 - 渲染 UI
//!                  └── App::view()     // 第4层 - 组装左右布局
//!                       ├── tree_menu  // 第3层 - 左侧导航菜单
//!                       └── feature::view() // 第2层 - 右侧功能面板
//!                            └── theme::font/size/padding // 第0层
//! ```

// `mod` 声明子模块：`features` 和 `ui` 是 gui_tool 的直接子模块。
// 这些声明告诉编译器去查找 features.rs 和 ui.rs (或 features/mod.rs, ui/mod.rs)。

/// 功能模块集合
///
/// 包含各个独立的功能实现，每个功能模块遵循 view/update 分离模式：
/// - `json_fmt`: JSON 格式化/验证工具
/// - `net_port_scan`: 基于 rayon 并行扫描的网络端口扫描器
/// - `net_capture`: pnet 抓包 + 协议解析
/// - `ui_libs`: iced_aw 组件库展示
/// - `theme`: 全局主题尺寸/字体/内边距计算
///
/// # Rust: `mod` vs `pub mod`
/// - `mod foo;` —— 声明私有子模块，外部不可访问
/// - `pub mod foo;` —— 声明公开子模块，外部可访问
/// 这里用 `mod` 而非 `pub mod`，因为这些模块仅在本 crate 内部使用。
/// 对于 binary crate (有 `main` 函数的)，外部无法依赖它，所以 `pub` 没有实际影响。
mod features;

/// UI 层模块集合
///
/// 包含应用界面的结构和布局：
/// - `app`: 应用状态管理 (持有所选标签页、各功能模块状态)
/// - `menu`: 顶部下拉菜单逻辑
/// - `widgets`: 自定义 UI 组件 (树形菜单、Toast、叠加层、菜单栏)
mod ui;

// `use` 语句: 将外部类型/函数引入当前作用域
//
// Rust 语法: `use path::to::Thing;`
// 路径可以是:
// - 绝对路径: `::crate_name::module::Thing` (crate 根)
// - crate-relative: `crate::module::Thing`
// - self-relative: `self::module::Thing`
// `use ... as ...` 可以重命名引入的项

// anyhow 是一个错误处理库，提供:
// - `Result<T>`: 标准 Result 的别名为 anyhow::Result<T>，错误类型是 anyhow::Error
// - `Context` trait: 为 Result 提供 `.context("msg")?` 方法，在错误上附加人类可读的上下文
use anyhow::{Context, Result};

/// 应用程序入口函数
///
/// Rust 程序的入口点永远是 `fn main()`。对于 binary crate 必须存在。
/// 可以有多种形式：
/// - `fn main()` —— 无返回值的简单入口
/// - `fn main() -> Result<(), E>` —— 可返回错误的入口（常见于 CLI 工具）
///   当 main 返回 Err 时，程序会以非零退出码退出并打印错误信息
///
/// # 执行流程
/// 1. 调用 `ui::run()` 启动 iced 桌面应用
/// 2. iced 内部创建窗口 (默认 1000×700)、初始化 OpenGL 上下文
/// 3. 进入事件循环 —— 等待用户交互、调用 update/view
/// 4. 关闭窗口后，iced 返回 Ok(())，函数正常结束
///
/// # Rust: `?` 运算符
/// `expression?` 是 Rust 的错误传播语法糖：
/// ```text
/// ui::run().context("...")?
/// // 展开后等价于:
/// match ui::run().context("...") {
///     Ok(value) => value,
///     Err(e) => return Err(e.into()),
/// }
/// ```
/// `?` 只能在返回 `Result` 或 `Option` 的函数中使用。
///
/// # Rust: `Result<()>` 返回类型
/// `()` 是 unit type（单元类型），类似其他语言的 void，但 `()` 是一个实际的值。
/// `Result<()>` 表示操作只有成功/失败两种结果，成功时不携带数据。
/// 当 `main` 返回 `Err` 时，Rust 运行时会打印 Debug 格式的错误并退出。
fn main() -> Result<()> {
    // `context("Failed to run application")` 为潜在错误添加描述性前缀
    // 如果 `ui::run()` 失败，错误信息会是: "Failed to run application: <原始错误>"
    ui::run().context("Failed to run application")?;
    Ok(())
}
