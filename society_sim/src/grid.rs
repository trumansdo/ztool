//! Grid — 空间网格单元
//!
//! 空间划分最小单元，事件发布锚点。
//! 支持动态细分（Quadtree）防"超级都市"单点过载。

use crate::blackboard::Blackboard;

/// 空间网格单元
#[derive(Debug, Clone)]
pub struct Grid {
    /// Grid 唯一标识
    pub id: u32,
    /// Grid 坐标
    pub pos: (u32, u32),
    /// T 值自然产出基底（关中90/荆楚60/长城20）
    pub t_base: f32,
    /// T 值产出上限（受退化/过度开发影响，不可逆衰减）
    pub t_max_base: f32,
    /// T 值疲劳度（过度开发累积，>阈值触发 t_max_base 衰减）
    pub fatigue: f32,
    /// 所属组织（None = 无主之地）
    pub org_id: Option<u64>,
    /// 当前人口数
    pub population: u32,
    /// 本 Grid 的事件黑板
    pub blackboard: Blackboard,
    /// 子 Grid ID（动态细分：人口超阈值自动分裂为4个子Grid）
    /// None = 未分裂，Some([tl, tr, bl, br])
    pub children: Option<[u32; 4]>,
}
