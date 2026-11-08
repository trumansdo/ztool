//! 固定嵌入后端：所有文本返回同一向量
//!
//! 用于测试流动/结算逻辑，消除嵌入差异对结果的影响。

use async_trait::async_trait;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::error::EmbeddingError;

/// 固定嵌入后端
pub struct FixedEmbeddingBackend {
    dim: usize,
    value: f32,
}

impl FixedEmbeddingBackend {
    /// 创建固定嵌入后端，所有维度填充 `value`
    pub fn new(dim: usize, value: f32) -> Self {
        Self { dim, value }
    }
}

#[async_trait]
impl EmbeddingBackend for FixedEmbeddingBackend {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(vec![self.value; self.dim])
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|_| vec![self.value; self.dim]).collect())
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
    async fn fixed_embedding_all_same() {
        let backend = FixedEmbeddingBackend::new(256, 0.5);
        let e1 = backend.embed("hello").await.unwrap();
        let e2 = backend.embed("world").await.unwrap();
        assert_eq!(e1, e2);
        assert_eq!(e1, vec![0.5; 256]);
    }

    #[tokio::test]
    async fn fixed_embedding_batch() {
        let backend = FixedEmbeddingBackend::new(64, -0.3);
        let texts: Vec<String> = vec!["x".into(), "y".into()];
        let results = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], vec![-0.3; 64]);
        assert_eq!(results[1], vec![-0.3; 64]);
    }

    #[tokio::test]
    async fn fixed_embedding_dimension() {
        let backend = FixedEmbeddingBackend::new(512, 0.0);
        assert_eq!(backend.dimension(), 512);
    }
}
