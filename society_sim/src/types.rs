//! 共享辅助类型
//!
//! 全部配置驱动，无硬编码。
//! 阶级/制度不是数据结构——阶级由 V 值连续函数动态体现，制度只是事件配置的标签。

// ============================================================================
// V 值区间（事件过滤用）
// ============================================================================

/// V 值区间，用于事件的目标过滤
/// 高 V = 高阶层，低 V = 低阶层，一切由动态函数运算
#[derive(Debug, Clone, Copy)]
pub struct VRange {
    /// 下限（含）
    pub min: f32,
    /// 上限（含），f32::MAX 表示无上限
    pub max: f32,
}

impl VRange {
    /// 所有 V 值
    pub const ALL: Self = Self { min: f32::NEG_INFINITY, max: f32::INFINITY };
    /// 仅低 V（<10）
    pub const LOW: Self = Self { min: f32::NEG_INFINITY, max: 10.0 };
    /// 仅高 V（≥100）
    pub const HIGH: Self = Self { min: 100.0, max: f32::INFINITY };

    pub fn contains(self, v: f32) -> bool {
        v >= self.min && v <= self.max
    }
}

// ============================================================================
// 事件相关
// ============================================================================

/// 事件发布主体（三层）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    World,
    Org,
    Citizen,
}

/// 事件作用范围
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventScope {
    Point,
    Area,
    Global,
}

/// 信息失真规则（跨 Grid 传播时强制执行）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationRule {
    Identity,
    Drift,
    Amplify,
}

// ============================================================================
// 修正器
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcType {
    Add,
    Mul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrTarget {
    TCurrent,
    TMax,
    TRegen,
    VCurrent,
    VRegen,
}

/// 属性修正器：事件对 T/V 的具体影响
#[derive(Debug, Clone, Copy)]
pub struct Modifier {
    pub target: AttrTarget,
    pub calc: CalcType,
    pub value: f32,
}

// ============================================================================
// 组织关系
// ============================================================================

/// 组织间关系
#[derive(Debug, Clone)]
pub struct Relation {
    pub target: u64,
    pub hostility: f32,
    pub tribute: f32,
    pub trade: f32,
}
