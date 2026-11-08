//! Toast 消息列表管理
//!
//! 封装 Toast 的增删操作，消除各 feature 中的重复代码。
//!
//! # 使用方式
//! ```ignore
//! use crate::ui::widgets::toast::{ToastList, ToastLevel, ToastPosition};
//!
//! // 在状态结构体中
//! toasts: ToastList,
//!
//! // 在 update 中
//! Msg::CloseToast(i) => self.toasts.close(i),
//!
//! // 在 view 中
//! Toaster::new(content, &self.toasts, Msg::CloseToast)
//! ```

use super::{Toast, ToastLevel, ToastPosition};

/// Toast 消息列表
///
/// 封装 `Vec<Toast>` 并提供语义化的增删方法。
/// 实现 `Deref<Target = Vec<Toast>>` 以兼容 `Toaster::new()` 的 `&[Toast]` 参数。
#[derive(Debug, Clone, Default)]
pub struct ToastList {
    items: Vec<Toast>,
}

impl ToastList {
    /// 添加一条 Toast
    pub fn push(&mut self, level: ToastLevel, text: impl Into<String>, position: ToastPosition) {
        self.items.push(Toast {
            level,
            text: text.into(),
            position,
        });
    }

    /// 关闭指定索引的 Toast
    pub fn close(&mut self, index: usize) {
        self.items.remove(index);
    }
}

impl ToastList {
    /// 以切片形式访问 Toast 列表（兼容 `Toaster::new()` 的 `&[Toast]` 参数）
    pub fn as_slice(&self) -> &[Toast] {
        &self.items
    }
}
