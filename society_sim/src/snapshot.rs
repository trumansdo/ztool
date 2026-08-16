//! Snapshot — 双缓冲快照
//!
//! 用于 UI 渲染线程与模拟线程解耦。
//! 只存聚合统计，不存个体数据（Arc 指针交换，零拷贝）。

/// Grid 聚合统计快照
#[derive(Debug, Clone)]
pub struct GridSummary {
    /// Grid ID
    pub grid_id: u32,
    /// 人口
    pub population: u32,
    /// 平均 T 值
    pub t_avg: f32,
    /// 平均 V 值
    pub v_avg: f32,
    /// 危机人口数（V < 生存线）
    pub crisis_count: u32,
}

/// 组织统计快照
#[derive(Debug, Clone)]
pub struct OrgStat {
    /// 组织 ID
    pub org_id: u64,
    /// 人口
    pub population: u64,
    /// V 值池
    pub v_pool: f64,
    /// 危机人口数
    pub crisis_count: u64,
}

/// 世界快照（双缓冲：模拟线程写 → UI 线程读）
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// 快照时刻的 tick
    pub tick: u64,
    /// 所有 Grid 的聚合统计
    pub grid_summaries: Vec<GridSummary>,
    /// 所有组织的统计
    pub org_stats: Vec<OrgStat>,
}
