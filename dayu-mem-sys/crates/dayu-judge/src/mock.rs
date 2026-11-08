//! Mock 判别器后端：可编程 verdict 队列，用于测试

use dayu_domain::traits::*;
use dayu_domain::types::*;
use dayu_domain::error::*;
use std::sync::Mutex;

/// Mock 判别器后端：可编程返回 Verdict
///
/// 通过 enqueue() 预设返回值序列，按 FIFO 顺序消费。
/// fail_next() 让下一次调用返回错误。
pub struct MockJudgeBackend {
    verdicts: Mutex<Vec<Verdict>>,
    fail_next: Mutex<bool>,
}

impl MockJudgeBackend {
    pub fn new() -> Self {
        Self {
            verdicts: Mutex::new(Vec::new()),
            fail_next: Mutex::new(false),
        }
    }

    /// 入队一个预设 Verdict
    pub fn enqueue(&self, verdict: Verdict) {
        self.verdicts.lock().unwrap().push(verdict);
    }

    /// 让下一次 judge() 调用返回错误
    pub fn fail_next(&self) {
        *self.fail_next.lock().unwrap() = true;
    }

    /// 剩余预设 verdict 数量
    pub fn remaining(&self) -> usize {
        self.verdicts.lock().unwrap().len()
    }
}

impl Default for MockJudgeBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl JudgeBackend for MockJudgeBackend {
    async fn judge(&self, _input: &JudgeInput) -> Result<Verdict, JudgeError> {
        if *self.fail_next.lock().unwrap() {
            *self.fail_next.lock().unwrap() = false;
            return Err(JudgeError::BackendUnavailable);
        }

        let mut guard = self.verdicts.lock().unwrap();
        if guard.is_empty() {
            return Ok(Verdict {
                should_judge: false,
                significance: 0.0,
                offset_direction: String::new(),
                emotion_valence: 0.0,
                emotion_intensity: 0.0,
                hit_dimensions: vec![],
                belongs_to: None,
                belong_confidence: 0.0,
                reason: "no verdict enqueued".to_string(),
            });
        }

        Ok(guard.remove(0))
    }

    fn backend_kind(&self) -> BackendKind {
        BackendKind::Mock
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn mock_backend_returns_enqueued_verdict() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.8, None, 0.0));

        let input = sample_input();
        let verdict = backend.judge(&input).await.unwrap();
        assert_eq!(verdict.significance, 0.8);
        assert_eq!(backend.remaining(), 0);
    }

    #[tokio::test]
    async fn mock_backend_fifo_order() {
        let backend = MockJudgeBackend::new();
        backend.enqueue(make_verdict(0.3, None, 0.0));
        backend.enqueue(make_verdict(0.9, None, 0.0));

        let input = sample_input();
        let v1 = backend.judge(&input).await.unwrap();
        let v2 = backend.judge(&input).await.unwrap();
        assert_eq!(v1.significance, 0.3);
        assert_eq!(v2.significance, 0.9);
    }

    #[tokio::test]
    async fn mock_backend_fail_next() {
        let backend = MockJudgeBackend::new();
        backend.fail_next();

        let input = sample_input();
        let result = backend.judge(&input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn mock_backend_default_when_empty() {
        let backend = MockJudgeBackend::new();
        let input = sample_input();
        let verdict = backend.judge(&input).await.unwrap();
        assert!(!verdict.should_judge);
        assert_eq!(verdict.significance, 0.0);
    }

    #[test]
    fn mock_backend_kind_is_mock() {
        let backend = MockJudgeBackend::new();
        assert_eq!(backend.backend_kind(), BackendKind::Mock);
    }
}
