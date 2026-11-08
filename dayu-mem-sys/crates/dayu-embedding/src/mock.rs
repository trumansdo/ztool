//! 确定性嵌入后端：同一文本永远返回同一向量
//!
//! 使用 std hash 生成伪随机向量，映射到 [-1, 1]。
//! 适用于单元测试和集成测试，无需真实模型。

use async_trait::async_trait;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::error::EmbeddingError;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// 确定性嵌入后端
pub struct MockEmbeddingBackend {
    dim: usize,
}

impl MockEmbeddingBackend {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// 确定性嵌入：使用 std hash 生成伪随机向量
    pub fn deterministic_embed(&self, text: &str) -> Vec<f32> {
        (0..self.dim)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                i.hash(&mut hasher);
                let h = hasher.finish();
                (h as f64 / u64::MAX as f64 * 2.0 - 1.0) as f32
            })
            .collect()
    }
}

#[async_trait]
impl EmbeddingBackend for MockEmbeddingBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(self.deterministic_embed(text))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|t| self.deterministic_embed(t)).collect())
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_embedding_is_deterministic() {
        let backend = MockEmbeddingBackend::new(384);
        let e1 = backend.embed("hello").await.unwrap();
        let e2 = backend.embed("hello").await.unwrap();
        assert_eq!(e1, e2);
    }

    #[tokio::test]
    async fn mock_embedding_different_texts_differ() {
        let backend = MockEmbeddingBackend::new(384);
        let e1 = backend.embed("hello").await.unwrap();
        let e2 = backend.embed("world").await.unwrap();
        assert_ne!(e1, e2);
    }

    #[tokio::test]
    async fn mock_embedding_correct_dimension() {
        let backend = MockEmbeddingBackend::new(768);
        let e = backend.embed("test").await.unwrap();
        assert_eq!(e.len(), 768);
        assert_eq!(backend.dimension(), 768);
    }

    #[tokio::test]
    async fn mock_embedding_batch() {
        let backend = MockEmbeddingBackend::new(128);
        let texts: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let results = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].len(), 128);
        assert_ne!(results[0], results[1]);
    }
}
