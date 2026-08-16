//! EventConfig — 事件模板
//!
//! 逻辑与数据分离：config 定义"什么事件"，Event 是"事件的一次发生"。
//! 全部配置驱动，无硬编码。

use crate::types::{EventScope, EventSource, Modifier, MutationRule, VRange};

/// 事件模板（参数化配置）
#[derive(Debug, Clone)]
pub struct EventConfig {
    /// 模板 ID
    pub config_id: u32,
    /// 事件名称
    pub name: String,
    /// 发布主体类型
    pub source: EventSource,
    /// 作用范围
    pub scope: EventScope,
    /// 基础强度
    pub base_intensity: f32,
    /// 基础持续 tick 数
    pub base_duration: u32,
    /// 目标 V 值区间
    pub target_v: VRange,
    /// 固定 4 修正器
    pub modifiers: [Option<Modifier>; 4],
    /// 传播失真规则
    pub mutation: MutationRule,
    /// 黑板抢占优先级
    pub priority: u8,
    /// 触发条件表达式（占位，后续由配置系统解析）
    pub trigger_condition: String,
}
