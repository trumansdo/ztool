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
    use crate::mock::MockJudgeBackend;

    fn sample_input() -> JudgeInput {
        JudgeInput {
            candidate_text: "今天天气不错".to_string(),
            candidate_embedding: vec![0.1; 384],
            recent_window: vec!["你好".to_string()],
            candidate_archetypes: vec![],
        }
    }

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

    #[tokio::test]
    async fn engine_discards_below_threshold() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.1, None, 0.0));

        let engine = JudgeEngine::new();
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Discard { .. } => {}
            other => panic!("expected Discard, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_marks_tentative_in_gray_zone() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.4, None, 0.0));

        let engine = JudgeEngine::new();
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Create { status, .. } => {
                assert_eq!(status, BallCreationStatus::Tentative);
            }
            other => panic!("expected Create, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_confirms_above_gray_zone() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.6, None, 0.0));

        let engine = JudgeEngine::new();
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Create { status, .. } => {
                assert_eq!(status, BallCreationStatus::Confirmed);
            }
            other => panic!("expected Create, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_attaches_when_belong_confidence_high() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.8, Some(BallId(42)), 0.85));

        let engine = JudgeEngine::new();
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Attach { ball_id, confidence, .. } => {
                assert_eq!(ball_id, BallId(42));
                assert_eq!(confidence, 0.85);
            }
            other => panic!("expected Attach, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_creates_when_belong_confidence_low() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.8, Some(BallId(42)), 0.5));

        let engine = JudgeEngine::new();
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Create { .. } => {}
            other => panic!("expected Create (fallback), got {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_creates_when_no_belong_target() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.8, None, 0.0));

        let engine = JudgeEngine::new();
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Create { significance, .. } => {
                assert_eq!(significance, 0.8);
            }
            other => panic!("expected Create, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn engine_custom_thresholds() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.5, None, 0.0));

        let engine = JudgeEngine::with_thresholds(0.6, 0.8, 0.9);
        let result = engine.judge(&backend, &sample_input()).await.unwrap();

        match result {
            JudgeResult::Discard { .. } => {}
            other => panic!("expected Discard with custom threshold, got {:?}", other),
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
