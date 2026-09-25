//! AI提示词生成器模块
//!
//! 提供基于模板的 AI 提示词生成功能：
//! - 用户定义变量（`#变量名` 格式）
//! - 固定模板中引用变量（`${变量名}` 格式）
//! - JSON 单文件持久化存储
//! - 实时预览渲染结果

pub mod model;
pub mod storage;
pub mod template;
pub mod update;
pub mod view;

pub use update::{update, Msg, PromptGenerator};
pub use view::view;
