//! 损失模型：管道粗细 w_eff 同时决定阻力和损耗
//!
//! 管道双重角色：
//! - 阻力：能不能过（resistance = 1/(1+w_eff)）
//! - 损耗：过了剩多少（loss = f(w_eff)）
//!
//! LossModel trait 可替换，支持对照实验：
//! - PipeWeightLossModel：默认，损失由管道粗细决定
//! - ConstantLossModel：对照用，退化为固定损失，精确复现旧规则

use dayu_domain::types::{Config, LossRate};

/// 损失模型 trait：可替换，支持 A/B 对照实验
pub trait LossModel: Send + Sync {
    /// 根据管道有效粗细计算损失率
    fn loss(&self, w_eff: f64, cfg: &Config) -> LossRate;
}

/// 默认损失模型：损失由管道粗细决定
///
/// loss(w_eff) = min(base_loss × (1 + loss_pipe_factor × (1 - w_eff)), loss_max)
///
/// - w_eff = 1（最粗）→ loss = base_loss
/// - w_eff → 0（最细）→ loss → base_loss × (1 + loss_pipe_factor)，被 loss_max 截断
pub struct PipeWeightLossModel;

impl LossModel for PipeWeightLossModel {
    fn loss(&self, w_eff: f64, cfg: &Config) -> LossRate {
        flow_loss(w_eff, cfg)
    }
}

/// 对照实验用：退化为固定损失，精确复现旧规则
///
/// 每跳损失恒等于 base_loss，与管道粗细无关。
/// 用于验证"管道决定损耗"是否真的带来行为差异。
pub struct ConstantLossModel;

impl LossModel for ConstantLossModel {
    fn loss(&self, _w_eff: f64, cfg: &Config) -> LossRate {
        LossRate::new(cfg.base_loss).expect("base_loss < 1.0 由配置校验保证")
    }
}

/// 核心损失函数：管道粗细 → 损失率
///
/// 损失对 w_eff 单调递减：越粗损失越小，越细损失越大。
/// 损失率有上限 loss_max，防止细管道把水全部吃掉。
pub fn flow_loss(w_eff: f64, cfg: &Config) -> LossRate {
    let raw = cfg.base_loss * (1.0 + cfg.loss_pipe_factor * (1.0 - w_eff));
    LossRate::new(raw.min(cfg.loss_max)).expect("loss_max < 1.0 由配置校验保证")
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use dayu_testkit::sample_config;

    #[test]
    fn thick_pipe_has_minimal_loss() {
        let cfg = sample_config();
        let loss = flow_loss(1.0, &cfg).get();
        assert!((loss - cfg.base_loss).abs() < 1e-9,
            "w_eff=1 should give base_loss={}, got {}", cfg.base_loss, loss);
    }

    #[test]
    fn thin_pipe_loses_more_than_thick_pipe() {
        let cfg = sample_config();
        let thick = flow_loss(1.0, &cfg).get();
        let thin = flow_loss(0.1, &cfg).get();
        assert!(thin > thick,
            "thin pipe loss ({}) should exceed thick pipe loss ({})", thin, thick);
    }

    #[test]
    fn loss_is_monotonic_decreasing_in_w() {
        let cfg = sample_config();
        let mut prev = flow_loss(0.01, &cfg).get();
        for w in [0.1, 0.3, 0.5, 0.7, 0.9, 1.0] {
            let curr = flow_loss(w, &cfg).get();
            assert!(curr <= prev + 1e-9,
                "loss should decrease as w increases: w={}, prev={}, curr={}", w, prev, curr);
            prev = curr;
        }
    }

    #[test]
    fn loss_never_exceeds_max() {
        let cfg = sample_config();
        for w in [0.001, 0.01, 0.1, 0.5, 1.0] {
            let loss = flow_loss(w, &cfg).get();
            assert!(loss <= cfg.loss_max,
                "loss {} exceeds max {} at w={}", loss, cfg.loss_max, w);
        }
    }

    #[test]
    fn loss_never_below_base() {
        let cfg = sample_config();
        for w in [0.001, 0.01, 0.1, 0.5, 1.0] {
            let loss = flow_loss(w, &cfg).get();
            assert!(loss >= cfg.base_loss - 1e-9,
                "loss {} below base {} at w={}", loss, cfg.base_loss, w);
        }
    }

    #[test]
    fn constant_loss_model_matches_base_loss() {
        let cfg = sample_config();
        let model = ConstantLossModel;
        for w in [0.1, 0.5, 1.0] {
            let loss = model.loss(w, &cfg).get();
            assert!((loss - cfg.base_loss).abs() < 1e-9,
                "ConstantLossModel should always return base_loss={}, got {} at w={}",
                cfg.base_loss, loss, w);
        }
    }

    #[test]
    fn pipe_weight_vs_constant_loss_differ_on_thin_pipe() {
        let cfg = sample_config();
        let pipe_model = PipeWeightLossModel;
        let const_model = ConstantLossModel;

        let w_thin = 0.1;
        let pipe_loss = pipe_model.loss(w_thin, &cfg).get();
        let const_loss = const_model.loss(w_thin, &cfg).get();

        assert!(pipe_loss > const_loss,
            "PipeWeightLossModel ({}) should exceed ConstantLossModel ({}) on thin pipe",
            pipe_loss, const_loss);
    }
}

// ============================================================
// 属性测试（第一层：物理不变量）
// ============================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// 不变量：损失率 ∈ [base_loss, loss_max]，永不超上限、永不为负
        #[test]
        fn loss_is_bounded(w in 0.0f64..=1.0) {
            let cfg = Config::default();
            let l = flow_loss(w, &cfg).get();
            prop_assert!(l >= cfg.base_loss - 1e-9, "loss({}) = {} < base_loss", w, l);
            prop_assert!(l <= cfg.loss_max + 1e-9, "loss({}) = {} > loss_max", w, l);
        }

        /// 不变量：损失是 w_eff 的单调递减函数——管道越细损失越大
        #[test]
        fn loss_is_monotone_decreasing(w1 in 0.0f64..=1.0, w2 in 0.0f64..=1.0) {
            let cfg = Config::default();
            let (l1, l2) = (flow_loss(w1, &cfg).get(), flow_loss(w2, &cfg).get());
            if w1 >= w2 {
                prop_assert!(l1 <= l2 + 1e-9, "w1={} l1={} 应 <= w2={} l2={}", w1, l1, w2, l2);
            } else {
                prop_assert!(l1 >= l2 - 1e-9, "w1={} l1={} 应 >= w2={} l2={}", w1, l1, w2, l2);
            }
        }

        /// 不变量：阻力有界且有限——w→0 时趋近 1 但永不为无穷
        #[test]
        fn resistance_is_finite_and_bounded(w in 0.0f64..=1.0) {
            let r = 1.0 / (1.0 + w);
            prop_assert!(r.is_finite(), "resistance 非有限: w={}", w);
            prop_assert!(r > 0.0 && r <= 1.0, "resistance 越界: w={} r={}", w, r);
        }

        /// 不变量（“不封死”）：任意管道在足够大的水量下都能被突破
        #[test]
        fn any_pipe_is_breakable_with_enough_water(
            w in 0.0001f64..=1.0,
            extra in 0.0f64..100.0
        ) {
            let r = 1.0 / (1.0 + w);
            prop_assert!(1.0 + extra > r, "w={} 的水量 {} 无法突破阻力 {}", w, 1.0 + extra, r);
        }
    }
}
