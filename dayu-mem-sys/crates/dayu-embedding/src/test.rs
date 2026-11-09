//! TestEmbeddingBackend: 确定性测试嵌入后端
//!
//! 基于文本哈希生成确定性向量，用于冒烟测试和规模验证。
//! 不依赖任何外部模型或网络，纯 Rust 实现。
//!
//! 仅在 `test-support` feature 启用时编译。

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use async_trait::async_trait;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::error::EmbeddingError;

/// 确定性测试嵌入后端
///
/// 使用 SipHash 对文本生成 64 位哈希，扩展为指定维度的伪随机向量。
/// 相同文本始终产生相同向量，不同文本产生近似正交向量。
///
/// 向量经 L2 归一化，余弦相似度 ≈ 0（不同文本）或 1（相同文本）。
pub struct TestEmbeddingBackend {
    dim: usize,
}

impl TestEmbeddingBackend {
    /// 创建测试嵌入后端
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    /// 将文本哈希扩展为 dim 维向量
    fn hash_to_vec(&self, text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; self.dim];

        // 使用多个种子生成伪随机向量
        for (i, slot) in vec.iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            i.hash(&mut hasher);
            // 将 u64 映射到 [-1, 1]
            let h = hasher.finish();
            *slot = ((h as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32;
        }

        // L2 归一化
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-12 {
            for x in &mut vec {
                *x /= norm;
            }
        }

        vec
    }
}

#[async_trait]
impl EmbeddingBackend for TestEmbeddingBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(self.hash_to_vec(text))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        Ok(texts.iter().map(|t| self.hash_to_vec(t)).collect())
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_text_same_vector() {
        let backend = TestEmbeddingBackend::new(128);
        let v1 = backend.embed("你好世界").await.unwrap();
        let v2 = backend.embed("你好世界").await.unwrap();
        assert_eq!(v1, v2);
    }

    #[tokio::test]
    async fn different_texts_different_vectors() {
        let backend = TestEmbeddingBackend::new(128);
        let v1 = backend.embed("你好").await.unwrap();
        let v2 = backend.embed("世界").await.unwrap();
        assert_ne!(v1, v2);
    }

    #[tokio::test]
    async fn vectors_are_normalized() {
        let backend = TestEmbeddingBackend::new(128);
        let v = backend.embed("测试文本").await.unwrap();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn batch_matches_single() {
        let backend = TestEmbeddingBackend::new(128);
        let texts: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let batch = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(batch.len(), 3);
        for (i, t) in texts.iter().enumerate() {
            let single = backend.embed(t).await.unwrap();
            assert_eq!(batch[i], single);
        }
    }

    #[tokio::test]
    async fn dimension_matches() {
        for dim in [64, 128, 256, 512] {
            let backend = TestEmbeddingBackend::new(dim);
            assert_eq!(backend.dimension(), dim);
            let v = backend.embed("test").await.unwrap();
            assert_eq!(v.len(), dim);
        }
    }
}
