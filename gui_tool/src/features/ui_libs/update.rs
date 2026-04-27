//! # UI 组件库状态管理与消息处理
//!
//! 管理组件示例页面中所有子 tab 的交互状态。
//!
//! ## Rust 概念 — Copy trait
//! `#[derive(Copy)]` 意味着类型在赋值/传参时是「按位复制」而非「移动所有权」。
//! Copy 只能用于简单类型（所有字段都是 Copy 的）。这里 ComponentTab 是纯枚举，
//! 没有堆数据，所以可以 Copy。
//! 一旦实现 Copy，就不需要 `.clone()` 调用。
//!
//! ## Rust 概念 — Self-referential 结构体
//! `UiLibs` 存储 `Vec<ToastItem>`，而 ToastItem 有 `u64 id`。
//! `next_toast_id` 是一个自增计数器（不是 pub 的），用于生成唯一 ID。
//! 这是避免 ID 冲突的常见模式。

use crate::ui::widgets::toast::{ToastItem, ToastLevel, ToastPosition};
use iced::Task;

/// UI 组件库的子 tab 枚举
///
/// ## Rust 概念 — 密集枚举
/// 这里 13 个变体涵盖了 iced_aw 库的主要组件。
/// `#[derive(Copy)]` 使这个枚举可以按值传递而无须 clone。
/// `#[derive(Hash)]` 允许用作 HashMap 的键。
/// `#[default]` 指定 Badge 为默认变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComponentTab {
    #[default]
    Badge,          // 徽章
    Card,           // 卡片
    Button,         // 按钮
    Toggle,         // 开关
    Separator,      // 分隔线
    Tab,            // 标签页
    NumberInput,    // 数字输入
    Spinner,        // 加载动画
    Wrap,           // 自动换行
    Split,          // 分屏
    Toast,          // 提示消息
    ColorPicker,    // 颜色选择器
    DatePicker,     // 日期选择器
}

/// UI 组件库的消息
///
/// ## Rust 概念 — 枚举变体携带不同类型
/// - `TabSelected(ComponentTab)` — 携带一个 Copy 的枚举值
/// - `ButtonPressed` — 无数据变体（单元变体）
/// - `ToggleChanged(bool)` — 携带布尔值
/// - `NumberChanged(i32)` — 携带 32 位有符号整数
/// - `ToastShow(ToastLevel, String, ToastPosition)` — 携带 3 个不同类型
/// - `AutoDismiss(u64)` — 携带 toast ID
/// 注意：不使用 thiserror，因为不需要错误消息显示。
#[derive(Debug, Clone)]
pub enum Msg {
    TabSelected(ComponentTab),
    ButtonPressed,
    ToggleChanged(bool),
    NumberChanged(i32),
    ToastShow(ToastLevel, String, ToastPosition),
    AutoDismiss(u64),
}

/// UI 组件库的完整状态
///
/// ## Rust 概念 — 字段默认值
/// 通过 `#[derive(Default)]`，所有字段都使用其类型的默认值：
/// - `bool` → false
/// - `u32` / `i32` → 0
/// - `Vec<T>` → 空向量
/// - `iced::Color` → 默认黑色
#[derive(Default)]
pub struct UiLibs {
    /// 当前选中的子 tab
    pub selected_tab: ComponentTab,
    /// 按钮点击累计次数（演示状态变更）
    pub click_count: u32,
    /// 开关状态（演示布尔状态管理）
    pub toggle_value: bool,
    /// 数字值（演示整数状态管理）
    pub number_value: i32,
    /// 当前选中的颜色
    pub selected_color: iced::Color,
    /// 活跃的 Toast 通知列表
    pub toasts: Vec<ToastItem>,
    /// 下一个 Toast 的 ID（自增，非 pub — 外部不可直接访问）
    next_toast_id: u64,
}

impl UiLibs {
    /// 添加一个 Toast 通知，返回其唯一 ID
    ///
    /// ## Rust 概念 — 私有方法
    /// `fn` 前没有 `pub`，表示这是私有方法，只能在当前模块内调用。
    /// Rust 的可见性默认是私有的（与 Java/C++ 的默认 public 相反）。
    ///
    /// ## Rust 概念 — 返回值使用
    /// 返回 `u64` 类型的 ID，调用者可以后续用它来消除此 Toast。
    fn push_toast(&mut self, level: ToastLevel, text: String, position: ToastPosition) -> u64 {
        let id = self.next_toast_id;
        self.next_toast_id += 1;  // 自增，下次 ID 不重复
        self.toasts.push(ToastItem {
            id,
            level,
            text,
            position,
        });
        id
    }
}

/// 处理消息，更新状态
///
/// ## Rust 概念 — 模式匹配绑定
/// `Msg::ToastShow(level, text, position)` 不仅匹配变体，
/// 还同时把携带的 3 个值绑定到局部变量 `level`, `text`, `position`。
///
/// ## Rust 概念 — 运算符重载
/// `libs.click_count += 1` 使用了 `AddAssign` trait 的运算符重载，
/// 等价于 `libs.click_count = libs.click_count + 1`。
///
/// ## Rust 概念 — `return` 关键字
/// `return` 用于提前返回。在 ToastShow 分支中直接返回一个异步任务，
/// 而其他分支走底部的 `Task::none()`。return 可以避免需要 else 分支。
pub fn update(libs: &mut UiLibs, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::TabSelected(tab) => {
            libs.selected_tab = tab;
        }
        Msg::ButtonPressed => {
            libs.click_count += 1;
        }
        Msg::ToggleChanged(value) => {
            libs.toggle_value = value;
        }
        Msg::NumberChanged(value) => {
            libs.number_value = value;
        }
        Msg::ToastShow(level, text, position) => {
            let id = libs.push_toast(level, text, position);
            // return 提前返回：创建异步任务，等待 3 秒后自动消除 Toast
            // 注意这里不需要 .clone() 因为 level 和 position 实现了 Copy
            return iced::Task::perform(
                tokio::time::sleep(std::time::Duration::from_secs(3)),
                move |_| Msg::AutoDismiss(id),
            );
        }
        Msg::AutoDismiss(id) => {
            // Vec::retain — 保留满足条件的元素，删除不满足的
            // 闭包 `|t| t.id != id` — 保留 id 不匹配的 toast，即删除指定 id 的 toast
            libs.toasts.retain(|t| t.id != id);
        }
    }
    Task::none()
}
