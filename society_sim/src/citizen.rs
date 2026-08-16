//! Citizen — 个体公民
//!
//! 极简 POD，零槽位、零状态存储。
//! 阶级不由枚举决定——V 值本身就是阶级：高 V = 高阶层，低 V = 低阶层。
//! V_Regen 由 V 值的连续函数动态计算，而非固定三档。

/// 个体公民
#[derive(Debug, Clone)]
pub struct Citizen {
    /// 唯一标识
    pub id: u64,
    /// Grid 坐标
    pub pos: (u32, u32),
    /// 所属组织（None = 流民）
    pub org_id: Option<u64>,
    /// 当前所在 Grid ID
    pub grid_id: u32,
    /// 来源 Grid ID（迁移追踪 / 引用传播）
    pub last_grid_id: u32,

    // ── T 值（真生命）三字段 ──
    /// 真生命当前值，≤0 即死亡
    pub t_current: f32,
    /// 真生命上限（受事件/科技修正）
    pub t_max: f32,
    /// 真生命恢复速率（受 V 值修正：高 V → 更快恢复）
    pub t_regen: f32,

    // ── V 值（虚生命）— 结果，不是工具 ──
    /// 虚生命当前值（社会地位/权力的量化结果）
    /// V 值本身就是阶级：高 V = 高阶层，低 V = 低阶层
    pub v_current: f32,
    /// 虚生命流速（由 V 值的连续函数动态计算）
    pub v_regen: f32,

    /// 事件记忆槽位（最近 2 次关键事件 outcome）
    pub memory: [i16; 2],

    /// 迁移冷却剩余 tick（迟滞机制，防弹簧床效应）
    pub cooldown: u16,

    /// 存活标记（配合对象池，死亡不销毁内存）
    pub alive: bool,
}

impl Citizen {
    /// V_Regen 连续函数：V 值越高，流速越快
    ///
    /// 这不是固定三档，而是连续曲线：
    /// - V ≤ 0   → V_Regen ≈ -0.1（流失）
    /// - V = 50  → V_Regen ≈ 0.5（缓慢积累）
    /// - V ≥ 200 → V_Regen ≈ 2.0（高速虹吸）
    ///
    /// 公式：V_Regen = ln(1 + V/10) - 0.1
    pub fn compute_v_regen(v: f32) -> f32 {
        let v_clamped = v.max(0.0);
        (1.0 + v_clamped / 10.0).ln() - 0.1
    }

    /// 更新 V_Regen（每 tick 或 V 值变化后调用）
    pub fn update_v_regen(&mut self) {
        self.v_regen = Self::compute_v_regen(self.v_current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v_regen_continuous() {
        // V=0 → 接近 -0.1（流失）
        let r0 = Citizen::compute_v_regen(0.0);
        assert!(r0 < 0.0 && r0 > -0.11);

        // V=50 → 正值（积累）
        let r50 = Citizen::compute_v_regen(50.0);
        assert!(r50 > 0.0);

        // V=200 → 高速虹吸
        let r200 = Citizen::compute_v_regen(200.0);
        assert!(r200 > 1.5);

        // 单调递增
        assert!(r200 > r50);
        assert!(r50 > r0);
    }

    #[test]
    fn test_update_v_regen() {
        let mut c = Citizen {
            id: 0,
            pos: (0, 0),
            org_id: None,
            grid_id: 0,
            last_grid_id: 0,
            t_current: 100.0,
            t_max: 100.0,
            t_regen: 1.0,
            v_current: 0.0,
            v_regen: 0.0,
            memory: [0; 2],
            cooldown: 0,
            alive: true,
        };

        c.update_v_regen();
        assert!(c.v_regen < 0.0); // V=0 → 流失

        c.v_current = 200.0;
        c.update_v_regen();
        assert!(c.v_regen > 1.5); // V=200 → 高速虹吸
    }
}
