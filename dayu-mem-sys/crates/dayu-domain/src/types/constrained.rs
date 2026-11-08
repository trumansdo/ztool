//! 受约束标量 newtype
//!
//! 构造时验证，持有即合法。

use std::fmt;
use crate::error::ValueError;

/// 管道粗细，范围 (0, 1]
#[derive(Clone, Copy, Debug)]
pub struct PipeWeight(f64);

impl PipeWeight {
    /// 构造管道粗细，raw 必须在 (0, 1] 范围内
    pub fn new(raw: f64) -> Result<Self, ValueError> {
        if raw > 0.0 && raw <= 1.0 {
            Ok(Self(raw))
        } else {
            Err(ValueError::PipeWeightOutOfRange(raw))
        }
    }

    /// 获取内部值
    pub fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for PipeWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// 水量，范围 [0, +∞)
#[derive(Clone, Copy, Debug)]
pub struct WaterAmount(f64);

impl WaterAmount {
    pub fn new(raw: f64) -> Result<Self, ValueError> {
        if raw >= 0.0 {
            Ok(Self(raw))
        } else {
            Err(ValueError::NegativeWater(raw))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for WaterAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

/// 相关性，范围 [0, 1]
#[derive(Clone, Copy, Debug)]
pub struct Relevance(f64);

impl Relevance {
    pub fn new(raw: f64) -> Result<Self, ValueError> {
        if (0.0..=1.0).contains(&raw) {
            Ok(Self(raw))
        } else {
            Err(ValueError::RelevanceOutOfRange(raw))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Relevance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// 球大小，范围 [0, size_max]
#[derive(Clone, Copy, Debug)]
pub struct BallSize(f64);

impl BallSize {
    pub fn new(raw: f64) -> Result<Self, ValueError> {
        if raw >= 0.0 {
            Ok(Self(raw))
        } else {
            Err(ValueError::NegativeSize(raw))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for BallSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// 损失率，范围 [0, 1)
/// 管道粗细 w_eff 的确定性函数输出：粗管接近 base_loss，细管被 loss_max 截断
#[derive(Clone, Copy, Debug)]
pub struct LossRate(f64);

impl LossRate {
    pub fn new(raw: f64) -> Result<Self, ValueError> {
        if (0.0..1.0).contains(&raw) {
            Ok(Self(raw))
        } else {
            Err(ValueError::LossRateOutOfRange(raw))
        }
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for LossRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_weight_valid() {
        assert!(PipeWeight::new(0.5).is_ok());
        assert!(PipeWeight::new(1.0).is_ok());
    }

    #[test]
    fn pipe_weight_invalid() {
        assert!(PipeWeight::new(0.0).is_err());
        assert!(PipeWeight::new(-0.1).is_err());
        assert!(PipeWeight::new(1.1).is_err());
    }

    #[test]
    fn water_amount_valid() {
        assert!(WaterAmount::new(0.0).is_ok());
        assert!(WaterAmount::new(100.0).is_ok());
    }

    #[test]
    fn water_amount_invalid() {
        assert!(WaterAmount::new(-1.0).is_err());
    }

    #[test]
    fn relevance_valid() {
        assert!(Relevance::new(0.0).is_ok());
        assert!(Relevance::new(0.5).is_ok());
        assert!(Relevance::new(1.0).is_ok());
    }

    #[test]
    fn relevance_invalid() {
        assert!(Relevance::new(-0.1).is_err());
        assert!(Relevance::new(1.1).is_err());
    }

    #[test]
    fn ball_size_valid() {
        assert!(BallSize::new(0.0).is_ok());
        assert!(BallSize::new(10.0).is_ok());
    }

    #[test]
    fn ball_size_invalid() {
        assert!(BallSize::new(-0.1).is_err());
    }

    #[test]
    fn loss_rate_valid() {
        assert!(LossRate::new(0.0).is_ok());
        assert!(LossRate::new(0.5).is_ok());
        assert!(LossRate::new(0.99).is_ok());
    }

    #[test]
    fn loss_rate_invalid() {
        assert!(LossRate::new(-0.1).is_err());
        assert!(LossRate::new(1.0).is_err());
        assert!(LossRate::new(1.5).is_err());
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

        /// 不变量：管道粗细只在 (0, 1] 构造成功，且取值原样保留
        #[test]
        fn pipe_weight_accepts_only_unit_interval(raw in -10.0f64..10.0) {
            match PipeWeight::new(raw) {
                Ok(w) => {
                    prop_assert!(raw > 0.0 && raw <= 1.0, "非法值 {} 被接受", raw);
                    prop_assert_eq!(w.get(), raw);
                }
                Err(_) => prop_assert!(raw <= 0.0 || raw > 1.0, "合法值 {} 被拒绝", raw),
            }
        }

        /// 不变量：球大小永不为负
        #[test]
        fn ball_size_is_never_negative(raw in -10.0f64..10.0) {
            match BallSize::new(raw) {
                Ok(s) => {
                    prop_assert!(raw >= 0.0, "负值 {} 被接受", raw);
                    prop_assert_eq!(s.get(), raw);
                }
                Err(_) => prop_assert!(raw < 0.0, "非负值 {} 被拒绝", raw),
            }
        }

        /// 不变量：水量永不为负
        #[test]
        fn water_amount_is_never_negative(raw in -10.0f64..10.0) {
            match WaterAmount::new(raw) {
                Ok(w) => prop_assert!(raw >= 0.0 && w.get() == raw),
                Err(_) => prop_assert!(raw < 0.0),
            }
        }

        /// 不变量：相关性 ∈ [0, 1]
        #[test]
        fn relevance_is_within_unit_interval(raw in -2.0f64..2.0) {
            match Relevance::new(raw) {
                Ok(r) => prop_assert!((0.0..=1.0).contains(&raw) && r.get() == raw),
                Err(_) => prop_assert!(!(0.0..=1.0).contains(&raw)),
            }
        }
    }
}
