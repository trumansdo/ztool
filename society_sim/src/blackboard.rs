//! Blackboard — 黑板
//!
//! Grid 级别的事件存储。
//! 固定 4 槽位 + 组织新闻流（非定域传播）。
//! 写入策略：查重续命 / 优先级覆盖（模拟"新闻热搜榜"）。

/// 黑板槽位条目
#[derive(Debug, Clone, Copy)]
pub struct EventSlot {
    /// 事件实例 ID
    pub event_id: u64,
    /// 优先级（抢占依据，0=最低，255=最高）
    pub priority: u8,
    /// 剩余 tick 数
    pub remaining: u32,
}

/// 黑板：固定槽位 + 组织新闻流
#[derive(Debug, Clone)]
pub struct Blackboard {
    /// 固定 4 槽位事件数组（分类隔离：T生死层 / V分配层）
    pub slots: [Option<EventSlot>; 4],
    /// 组织新闻流（非定域传播：圣旨/谣言/股价崩盘瞬间覆盖）
    pub news: Vec<EventSlot>,
}
