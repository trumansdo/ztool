//! HttpBackend: OpenAI 兼容 HTTP API 嵌入后端
//!
//! 通过 HTTP 调用远程嵌入服务（如 Qwen3-Embedding-8B），
//! 使用 OpenAI 兼容的 /v1/embeddings 接口。
//! 仅在 `http` feature 启用时编译。

use async_trait::async_trait;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::error::EmbeddingError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// HTTP 嵌入后端
///
/// 通过 OpenAI 兼容 API 调用远程嵌入模型。
pub struct HttpBackend {
    client: Client,
    /// API 基础 URL（如 http://10.89.188.246:8000/llmapi/v1）
    api_base: String,
    /// API Key
    api_key: String,
    /// 模型名称（如 Qwen3-Embedding-8B）
    model: String,
    dim: usize,
}

impl HttpBackend {
    /// 创建 HTTP 嵌入后端
    ///
    /// # 参数
    /// - `api_base`: API 基础 URL（不含尾部斜杠）
    /// - `api_key`: API 密钥
    /// - `model`: 模型名称
    /// - `dim`: 嵌入维度（需与模型一致）
    pub fn new(api_base: &str, api_key: &str, model: &str, dim: usize) -> Self {
        Self {
            client: Client::new(),
            api_base: api_base.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            dim,
        }
    }

    /// 探测远程模型的输出维度
    ///
    /// 向服务发一次最小请求（单条短文本），用返回向量长度作为维度。
    /// 配置未指定 `embedding_dim` 时由 init 阶段调用。
    pub async fn probe_dimension(
        api_base: &str,
        api_key: &str,
        model: &str,
    ) -> Result<usize, EmbeddingError> {
        let probe = Self::new(api_base, api_key, model, 0);
        let emb = probe.embed("维度探测").await?;
        if emb.is_empty() {
            return Err(EmbeddingError::InferenceFailed(
                "probe 返回空向量，无法确定维度".into(),
            ));
        }
        Ok(emb.len())
    }

    /// 执行 HTTP 嵌入请求
    async fn request_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let url = format!("{}/embeddings", self.api_base);

        let body = EmbeddingRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::InferenceFailed(format!("http request: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::InferenceFailed(format!(
                "http {status}: {text}"
            )));
        }

        let result: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::InferenceFailed(format!("parse response: {e}")))?;

        let mut embeddings: Vec<Vec<f32>> = result
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect();

        // L2 归一化
        for emb in &mut embeddings {
            l2_normalize(emb);
        }

        Ok(embeddings)
    }
}

#[async_trait]
impl EmbeddingBackend for HttpBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let results = self.request_embeddings(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InferenceFailed("no embedding produced".into()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.request_embeddings(texts).await
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

// ============================================================
// OpenAI 兼容 API 类型
// ============================================================

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用后端：连接信息从环境变量读取，不把内网地址与密钥写进仓库
    ///
    /// - `DAYU_HTTP_API_BASE`（如 `http://host:port/v1`）
    /// - `DAYU_HTTP_API_KEY`
    /// - `DAYU_HTTP_MODEL`（默认 `Qwen3-Embedding-8B`）
    /// - `DAYU_HTTP_DIM`（默认 4096）
    fn test_backend() -> HttpBackend {
        let api_base = std::env::var("DAYU_HTTP_API_BASE").unwrap_or_default();
        let api_key = std::env::var("DAYU_HTTP_API_KEY").unwrap_or_default();
        let model = std::env::var("DAYU_HTTP_MODEL")
            .unwrap_or_else(|_| "Qwen3-Embedding-8B".to_string());
        let dim = std::env::var("DAYU_HTTP_DIM")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(4096);
        HttpBackend::new(&api_base, &api_key, &model, dim)
    }

    #[test]
    fn reports_dimension() {
        let backend = test_backend();
        assert_eq!(backend.dimension(), 4096);
    }

    #[tokio::test]
    #[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
    async fn embed_single_text() {
        let backend = test_backend();
        let emb = backend.embed("今天天气真好").await.unwrap();
        assert_eq!(emb.len(), 4096);
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "L2 norm should be ~1, got {norm}");
    }

    #[tokio::test]
    #[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
    async fn embed_batch() {
        let backend = test_backend();
        let texts: Vec<String> = vec!["你好".into(), "世界".into(), "测试".into()];
        let batch = backend.embed_batch(&texts).await.unwrap();
        assert_eq!(batch.len(), 3);
        for emb in &batch {
            assert_eq!(emb.len(), 4096);
        }
    }

    #[tokio::test]
    #[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
    async fn empty_batch_returns_empty() {
        let backend = test_backend();
        assert!(backend.embed_batch(&[]).await.unwrap().is_empty());
    }
}
