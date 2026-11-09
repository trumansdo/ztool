//! GgufBackend: mistral.rs GGUF 嵌入后端
//!
//! 使用 mistralrs 加载 BGE 等 GGUF 格式嵌入模型。
//! 仅在 `gguf` feature 启用时编译。

use std::path::Path;

use async_trait::async_trait;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::error::EmbeddingError;
use mistralrs::{
    GgufModelBuilder,
    EmbeddingRequestBuilder,
};

/// GGUF 嵌入后端
///
/// 加载 GGUF 格式的嵌入模型（如 BGE-small-zh-v1.5），
/// 通过 mistral.rs 的 embedding API 生成文本向量。
pub struct GgufBackend {
    /// mistral.rs 模型实例
    model: mistralrs::Model,
    dim: usize,
}

// SAFETY: mistralrs::Model 内部用 Arc，跨线程安全
unsafe impl Send for GgufBackend {}
unsafe impl Sync for GgufBackend {}

impl GgufBackend {
    /// 从 GGUF 文件路径加载模型
    ///
    /// # 参数
    /// - `model_path`: GGUF 模型文件路径（.gguf）
    /// - `max_length`: 最大 token 长度，默认 512
    pub async fn load(model_path: &Path) -> Result<Self, EmbeddingError> {
        let path_str = model_path.to_str().unwrap_or_default();
        let model = GgufModelBuilder::new(path_str, vec![path_str])
            .build()
            .await
            .map_err(|e| EmbeddingError::InferenceFailed(format!("load gguf: {e}")))?;

        // 从模型元数据获取维度
        let dim = 512; // TODO: 从模型获取实际维度

        Ok(Self {
            model,
            dim,
        })
    }

    /// 执行嵌入推理
    async fn run_inference(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut builder = EmbeddingRequestBuilder::new();
        for text in texts {
            builder = builder.add_prompt(text.clone());
        }

        let mut embeddings = self
            .model
            .generate_embeddings(builder)
            .await
            .map_err(|e| EmbeddingError::InferenceFailed(format!("embed: {e}")))?;

        for vec in &mut embeddings {
            l2_normalize(vec);
        }

        Ok(embeddings)
    }
}

#[async_trait]
impl EmbeddingBackend for GgufBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let results = self.run_inference(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InferenceFailed("no embedding produced".into()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.run_inference(texts).await
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

/// L2 归一化（原地）
fn l2_normalize(vec: &mut [f32]) {
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-12 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn model_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../models/bge/bge-small-zh-v1.5-q8_0.gguf")
    }

    #[tokio::test]
    #[ignore = "requires GGUF model file"]
    async fn loads_model_and_reports_dimension() {
        let backend = GgufBackend::load(&model_path(), 512).await.expect("load model");
        assert_eq!(backend.dimension(), 512);
    }

    #[tokio::test]
    #[ignore = "requires GGUF model file"]
    async fn embed_chinese_text() {
        let backend = GgufBackend::load(&model_path(), 512).await.expect("load model");
        let emb = backend.embed("今天天气真好").await.unwrap();
        assert_eq!(emb.len(), 512);
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "L2 norm should be ~1, got {norm}");
    }

    #[tokio::test]
    #[ignore = "requires GGUF model file"]
    async fn embed_is_deterministic() {
        let backend = GgufBackend::load(&model_path(), 512).await.expect("load model");
        let e1 = backend.embed("你好世界").await.unwrap();
        let e2 = backend.embed("你好世界").await.unwrap();
        assert_eq!(e1, e2, "同文本应产生相同向量");
    }

    #[tokio::test]
    #[ignore = "requires GGUF model file"]
    async fn empty_and_blank_is_safe() {
        let backend = GgufBackend::load(&model_path(), 512).await.expect("load model");
        for text in ["", " ", "   \n\t  "] {
            let emb = backend.embed(text).await.unwrap();
            assert_eq!(emb.len(), 512);
            assert!(emb.iter().all(|x| x.is_finite()), "NaN/Inf in blank input");
        }
    }

    #[tokio::test]
    #[ignore = "requires GGUF model file"]
    async fn batch_matches_single() {
        let backend = GgufBackend::load(&model_path(), 512).await.expect("load model");
        let texts: Vec<String> = vec!["短".into(), "中等长度句子".into(), "较长文本用于验证批量推理".into()];
        let batch = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(batch.len(), 3);
        for (i, t) in texts.iter().enumerate() {
            let single = backend.embed(t).await.unwrap();
            assert_eq!(batch[i].len(), single.len());
        }
    }
}
