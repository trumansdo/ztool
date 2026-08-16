//! Org — 组织
//!
//! 抽象统计容器 + 文明载体。
//! 无大脑设计：组织"决策"由规则阈值自动触发。
//! 原子计数器：多线程无锁更新。
//! 制度不是数据结构——制度只是事件配置的标签，通过事件影响 T/V。

use std::collections::HashMap;

use crate::types::Relation;

/// 组织
#[derive(Debug, Clone)]
pub struct Org {
    /// 唯一标识
    pub id: u64,
    /// 组织名称
    pub name: String,

    // ── 原子计数器（无锁更新） ──
    /// 成员总数
    pub population: u64,
    /// 成员 V 值总和
    pub v_sum: f64,
    /// 危机人口数（V 值低于生存线的成员数）
    pub crisis_count: u64,

    // ── V 值池 ──
    /// 组织 V 值池（高 V 成员从此抽取 V_Regen）
    /// 池枯竭 → 高 V 成员 V_Regen 降速 → 组织解体
    pub v_pool: f64,

    // ── 代谢消耗 ──
    /// 每成员每 tick 的 V 消耗速率（组织越大消耗越高）
    pub metabolic_rate: f32,

    // ── 空间属性 ──
    /// 组织核心坐标
    pub center: (u32, u32),
    /// 热点 Grid 索引（Min_V_Grid_ID，靶向决策用）
    pub hotspot_grid: u32,

    // ── 文明属性（科技/文化） ──
    /// 科技水平（修正 T_Max 和 V 转化效率）
    pub tech_level: f32,
    /// 文化水平（修正 V_Regen 和成员忠诚度）
    pub culture_level: f32,

    // ── 对外关系 ──
    /// 与其他组织的关系表
    pub relations: HashMap<u64, Relation>,

    /// 组织存亡标记
    pub alive: bool,
}

impl Org {
    /// 计算组织代谢消耗（每 tick）
    pub fn metabolic_cost(&self) -> f64 {
        self.population as f64 * self.metabolic_rate as f64
    }

    /// 检查组织是否应解体
    pub fn should_collapse(&self) -> bool {
        self.v_pool <= 0.0 || self.population == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metabolic_cost() {
        let org = Org {
            id: 0,
            name: "test".into(),
            population: 1000,
            v_sum: 5000.0,
            crisis_count: 0,
            v_pool: 1000.0,
            metabolic_rate: 0.01,
            center: (0, 0),
            hotspot_grid: 0,
            tech_level: 1.0,
            culture_level: 1.0,
            relations: HashMap::new(),
            alive: true,
        };
        assert!((org.metabolic_cost() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_should_collapse() {
        let mut org = Org {
            id: 0,
            name: "doomed".into(),
            population: 100,
            v_sum: 0.0,
            crisis_count: 100,
            v_pool: -1.0,
            metabolic_rate: 0.01,
            center: (0, 0),
            hotspot_grid: 0,
            tech_level: 1.0,
            culture_level: 1.0,
            relations: HashMap::new(),
            alive: true,
        };
        assert!(org.should_collapse());

        org.v_pool = 10.0;
        assert!(!org.should_collapse());

        org.population = 0;
        assert!(org.should_collapse());
    }
}
