//! Event — 事件
//!
//! 驱动系统运转的唯一动力。
//! 固定 4 槽位黑板存储，写入策略 = 查重续命 / 优先级覆盖。

use crate::types::{
    EventScope, EventSource, Modifier, MutationRule, VRange,
};

/// 事件实例
#[derive(Debug, Clone)]
pub struct Event {
    /// 事件实例唯一 ID（因果溯源）
    pub instance_id: u64,
    /// 事件模板 ID（逻辑与数据分离）
    pub config_id: u32,

    // ── 因果链 ──
    /// 发布主体类型
    pub source: EventSource,
    /// 发布者 ID（责任归属 → "剥削-仇恨"链）
    pub source_id: u64,

    // ── 时空属性 ──
    /// 事件中心坐标
    pub pos: (u32, u32),
    /// 影响半径（0=点状，>0=区域，u32::MAX=全局）
    pub radius: u32,
    /// 作用范围类型
    pub scope: EventScope,

    // ── 动态参数 ──
    /// 事件强度
    pub intensity: f32,
    /// 剩余 tick 数（归零自动清除）
    pub remaining: u32,

    // ── V 值区间过滤 ──
    /// 影响的 V 值区间（高 V = 高阶层，低 V = 低阶层）
    pub target_v: VRange,

    // ── 修正器（固定 4 槽位） ──
    /// 对 T/V 属性的修正列表
    pub modifiers: [Option<Modifier>; 4],

    // ── 传播与失真 ──
    /// 跨 Grid 传播时的失真规则
    pub mutation: MutationRule,

    // ── 优先级（黑板抢占用） ──
    /// 0=最低（劳动），255=最高（崩溃/屠杀）
    pub priority: u8,
}

impl Event {
    /// 检查事件是否对指定 V 值的公民生效
    pub fn affects(&self, v: f32) -> bool {
        self.target_v.contains(v)
    }

    /// 检查事件是否已过期
    pub fn is_expired(&self) -> bool {
        self.remaining == 0
    }

    /// 每 tick 递减剩余时间
    pub fn tick(&mut self) {
        if self.remaining > 0 {
            self.remaining -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_lifecycle() {
        let mut event = Event {
            instance_id: 0,
            config_id: 0,
            source: EventSource::World,
            source_id: 0,
            pos: (0, 0),
            radius: 0,
            scope: EventScope::Point,
            intensity: 1.0,
            remaining: 3,
            target_v: VRange::ALL,
            modifiers: [None; 4],
            mutation: MutationRule::Identity,
            priority: 128,
        };

        assert!(!event.is_expired());
        event.tick();
        assert_eq!(event.remaining, 2);
        event.tick();
        event.tick();
        assert!(event.is_expired());
    }

    #[test]
    fn test_event_affects() {
        let event = Event {
            instance_id: 0,
            config_id: 0,
            source: EventSource::World,
            source_id: 0,
            pos: (0, 0),
            radius: 0,
            scope: EventScope::Point,
            intensity: 1.0,
            remaining: 1,
            target_v: VRange::HIGH,
            modifiers: [None; 4],
            mutation: MutationRule::Identity,
            priority: 128,
        };

        assert!(event.affects(150.0));  // 高 V → 生效
        assert!(!event.affects(5.0));   // 低 V → 不生效
        assert!(event.affects(100.0));  // 边界 → 生效
    }
}
