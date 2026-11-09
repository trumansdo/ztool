//! 判别器引擎：业务逻辑层
//!
//! 处理管线：
//! 1. 显著性检查 → 低于阈值直接丢弃
//! 2. 灰区判断 → 显著性在灰区标记 Tentative
//! 3. 归属检查 → 置信度高则挂已有球
//! 4. 兜底创建 → 以上都不满足则创建新球

use dayu_domain::traits::*;
use dayu_domain::types::*;
use dayu_domain::error::*;

/// 判别器引擎：业务逻辑层
pub struct JudgeEngine {
    /// 显著性阈值：低于此值丢弃
    significance_threshold: f64,
    /// 灰区下界：显著性在 [threshold, gray_zone) 标记 Tentative
    gray_zone_threshold: f64,
    /// 归属置信度阈值：高于此值挂已有球
    belong_confidence_threshold: f64,
}

impl JudgeEngine {
    /// 创建判别器引擎（使用默认阈值）
    pub fn new() -> Self {
        Self {
            significance_threshold: 0.3,
            gray_zone_threshold: 0.5,
            belong_confidence_threshold: 0.7,
        }
    }

    /// 自定义阈值
    pub fn with_thresholds(
        significance: f64,
        gray_zone: f64,
        belong_confidence: f64,
    ) -> Self {
        Self {
            significance_threshold: significance,
            gray_zone_threshold: gray_zone,
            belong_confidence_threshold: belong_confidence,
        }
    }

    /// 执行判别
    pub async fn judge(
        &self,
        backend: &dyn JudgeBackend,
        input: &JudgeInput,
    ) -> Result<JudgeResult, JudgeError> {
        let verdict = backend.judge(input).await?;
        Ok(self.process(verdict, input))
    }

    /// 处理 Verdict → JudgeResult
    fn process(&self, verdict: Verdict, input: &JudgeInput) -> JudgeResult {
        // 1. 显著性检查
        if verdict.significance < self.significance_threshold {
            return JudgeResult::Discard {
                reason: verdict.reason,
            };
        }

        // 2. 灰区判断
        let status = if verdict.significance < self.gray_zone_threshold {
            BallCreationStatus::Tentative
        } else {
            BallCreationStatus::Confirmed
        };

        // 3. 归属检查
        if let Some(belong_id) = verdict.belongs_to
            && verdict.belong_confidence >= self.belong_confidence_threshold
        {
            return JudgeResult::Attach {
                ball_id: belong_id,
                confidence: verdict.belong_confidence,
                significance: verdict.significance,
            };
        }

        // 4. 兜底创建新球
        JudgeResult::Create {
            significance: verdict.significance,
            status,
            reason: verdict.reason,
            embedding: input.candidate_embedding.clone(),
            text: input.candidate_text.clone(),
        }
    }
}

impl Default for JudgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// VerdictPostProcessor
// ============================================================

/// Verdict 后处理器（当前透传，预留扩展点）
pub struct VerdictPostProcessor;

impl VerdictPostProcessor {
    pub fn new() -> Self {
        Self
    }

    /// 处理 Verdict（当前透传）
    pub fn process(&self, verdict: Verdict) -> Verdict {
        verdict
    }
}

impl Default for VerdictPostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_verdict(
        significance: f64,
        belongs_to: Option<BallId>,
        belong_confidence: f64,
    ) -> Verdict {
        Verdict {
            should_judge: significance > 0.0,
            significance,
            offset_direction: String::new(),
            emotion_valence: 0.0,
            emotion_intensity: 0.0,
            hit_dimensions: vec![],
            belongs_to,
            belong_confidence,
            reason: "test".to_string(),
        }
    }

    #[test]
    fn post_processor_passthrough() {
        let processor = VerdictPostProcessor::new();
        let original = make_verdict(0.7, None, 0.0);
        let processed = processor.process(original.clone());
        assert_eq!(processed.significance, original.significance);
        assert_eq!(processed.reason, original.reason);
    }
}
