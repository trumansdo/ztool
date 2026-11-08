//! 诊断规则（纯函数）
//!
//! 输入指标值，输出诊断建议。无副作用，无状态。

/// 诊断结果
#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub rule: String,
    pub severity: DiagnosisSeverity,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosisSeverity {
    Info,
    Warning,
    Critical,
}

/// 诊断：损失主导占比过高
///
/// 当 path_supplement_loss_ratio{dominant=loss} 超过阈值时触发。
pub fn diagnose_loss_dominant(
    loss_dominant_ratio: f64,
    threshold: f64,
) -> Option<Diagnosis> {
    if loss_dominant_ratio > threshold {
        Some(Diagnosis {
            rule: "loss_dominant_high".to_string(),
            severity: DiagnosisSeverity::Warning,
            suggestion: format!(
                "损失主导路径占比 {:.1}% 超过阈值 {:.1}%，检查管道衰减是否过快，或降低 loss_pipe_factor",
                loss_dominant_ratio * 100.0,
                threshold * 100.0
            ),
        })
    } else {
        None
    }
}

/// 诊断：补充不足占比过高
///
/// 当 path_supplement_loss_ratio{dominant=supplement} 超过阈值时触发。
pub fn diagnose_supplement_weak(
    supplement_weak_ratio: f64,
    threshold: f64,
) -> Option<Diagnosis> {
    if supplement_weak_ratio > threshold {
        Some(Diagnosis {
            rule: "supplement_weak_high".to_string(),
            severity: DiagnosisSeverity::Warning,
            suggestion: format!(
                "补充不足路径占比 {:.1}% 超过阈值 {:.1}%，考虑提高 max_supplement_ratio 或降低 strengthen_threshold",
                supplement_weak_ratio * 100.0,
                threshold * 100.0
            ),
        })
    } else {
        None
    }
}

/// 诊断：损失饱和
///
/// 当 flow_loss_by_pipe_weight 的 p95 接近 loss_max 时触发。
pub fn diagnose_loss_saturation(
    p95_loss: f64,
    loss_max: f64,
    saturation_ratio: f64,
) -> Option<Diagnosis> {
    if p95_loss >= loss_max * saturation_ratio {
        Some(Diagnosis {
            rule: "loss_saturation".to_string(),
            severity: DiagnosisSeverity::Critical,
            suggestion: format!(
                "p95 损失率 {:.3} 接近上限 {:.3}（≥{:.0}%），大量管道处于损失上限，检查 loss_max 是否过低或管道整体过细",
                p95_loss,
                loss_max,
                saturation_ratio * 100.0
            ),
        })
    } else {
        None
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_loss_dominant_triggers_above_threshold() {
        let result = diagnose_loss_dominant(0.7, 0.6);
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.rule, "loss_dominant_high");
        assert_eq!(d.severity, DiagnosisSeverity::Warning);
    }

    #[test]
    fn diagnose_loss_dominant_silent_below_threshold() {
        let result = diagnose_loss_dominant(0.5, 0.6);
        assert!(result.is_none());
    }

    #[test]
    fn diagnose_supplement_weak_triggers_above_threshold() {
        let result = diagnose_supplement_weak(0.6, 0.5);
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.rule, "supplement_weak_high");
    }

    #[test]
    fn diagnose_supplement_weak_silent_below_threshold() {
        let result = diagnose_supplement_weak(0.4, 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn diagnose_loss_saturation_triggers_near_max() {
        let result = diagnose_loss_saturation(0.76, 0.8, 0.95);
        assert!(result.is_some());
        let d = result.unwrap();
        assert_eq!(d.rule, "loss_saturation");
        assert_eq!(d.severity, DiagnosisSeverity::Critical);
    }

    #[test]
    fn diagnose_loss_saturation_silent_far_from_max() {
        let result = diagnose_loss_saturation(0.5, 0.8, 0.95);
        assert!(result.is_none());
    }
}
