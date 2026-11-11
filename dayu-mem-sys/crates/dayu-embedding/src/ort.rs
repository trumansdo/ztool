//! OrtEmbeddingBackend: ONNX Runtime 嵌入后端
//!
//! 使用 ort 2.0 + tokenizers 加载中文嵌入模型，执行：
//! tokenize → 前向推理 → mean-pooling → L2 归一化
//!
//! 仅在 `onnx` feature 启用时编译。

use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::error::EmbeddingError;
use ort::environment::Environment;
use ort::session::Session;
use ort::value::TensorRef;
use tokenizers::Tokenizer;

// ============================================================
// OrtEmbeddingBackend
// ============================================================

/// ONNX Runtime 嵌入后端
///
/// 加载 ONNX 模型 + tokenizer，执行文本嵌入。
/// 模型输出经 mean-pooling（对 token 维度取平均）后 L2 归一化。
pub struct OrtEmbeddingBackend {
    /// ort 2.0: Environment 必须与 Session 同生命周期
    _env: Arc<Environment>,
    /// ort 2.0 Session::run 需要 &mut self，而 EmbeddingBackend::embed 是 &self，
    /// 用 Mutex 串行化推理（推理本身即串行，锁开销可忽略）
    session: Mutex<Session>,
    tokenizer: Arc<Tokenizer>,
    dim: usize,
    /// 最大序列长度（token 数），超出截断
    max_length: usize,
    /// 模型是否需要 token_type_ids 输入（BERT 系模型通常需要）
    need_token_type_ids: bool,
}

// SAFETY: Mutex<Session> 由标准库保证 Sync；Session 自身跨线程可用

impl OrtEmbeddingBackend {
    /// 从文件路径加载模型和 tokenizer
    ///
    /// # 参数
    /// - `model_path`: ONNX 模型文件路径（.onnx）
    /// - `tokenizer_path`: tokenizer.json 文件路径
    /// - `max_length`: 最大 token 长度，默认 512
    pub fn load(
        model_path: &Path,
        tokenizer_path: &Path,
        max_length: usize,
    ) -> Result<Self, EmbeddingError> {
        // 创建 ONNX Runtime 环境
        let env = Arc::new(
            ort::init()
                .with_name("dayu-embedding")
                .build()
                .map_err(|e| EmbeddingError::InferenceFailed(format!("ort init: {e}")))?
        );

        // 加载 ONNX session（ort 2.0: builder 需要 &Environment）
        let session = Session::builder(&env)
            .map_err(|e| EmbeddingError::InferenceFailed(format!("ort builder: {e}")))?
            .commit_from_file(model_path)
            .map_err(|e| EmbeddingError::InferenceFailed(format!("ort load model: {e}")))?;

        // 从输出形状推断嵌入维度
        // ort 2.0: outputs() 是方法，返回 &[Outlet]
        // Outlet.dtype() → &ValueType, ValueType.tensor_shape() → Option<&[i64]>
        let dim = session
            .outputs()
            .first()
            .and_then(|o| o.dtype().tensor_shape())
            .and_then(|shape| shape.last().copied())
            .unwrap_or(768) as usize;

        // 加载 tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| EmbeddingError::TokenizationFailed(format!("tokenizer load: {e}")))?;

        // 检查模型是否需要 token_type_ids（BERT 系模型通常需要，缺了会在 Gather 节点报 Missing Input）
        let need_token_type_ids = session
            .inputs()
            .iter()
            .any(|i| i.name() == "token_type_ids");

        Ok(Self {
            _env: env,
            session: Mutex::new(session),
            tokenizer: Arc::new(tokenizer),
            dim,
            max_length,
            need_token_type_ids,
        })
    }

    /// 执行嵌入推理（ort 2.0: run 需要 &mut self，通过 Mutex 取得）
    fn run_inference(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Tokenize：padding + truncation
        let encoding = self.tokenizer.encode_batch(
            texts.to_vec(),
            true, // add_special_tokens
        ).map_err(|e| EmbeddingError::TokenizationFailed(format!("tokenize: {e}")))?;

        // 构建 input_ids 和 attention_mask
        let batch_size = texts.len();
        let seq_len = encoding[0].len().min(self.max_length);

        let mut input_ids: Vec<i64> = Vec::with_capacity(batch_size * seq_len);
        let mut attention_mask: Vec<i64> = Vec::with_capacity(batch_size * seq_len);

        for enc in encoding.iter() {
            let ids = enc.get_ids();
            let mask = enc.get_attention_mask();
            for i in 0..seq_len {
                input_ids.push(if i < ids.len() { ids[i] as i64 } else { 0 });
                attention_mask.push(if i < mask.len() { mask[i] as i64 } else { 0 });
            }
        }

        // 构建 ndarray 输入
        let input_ids_array = ndarray::Array2::from_shape_vec(
            (batch_size, seq_len), input_ids,
        ).map_err(|e| EmbeddingError::InferenceFailed(format!("input_ids shape: {e}")))?;

        let attention_mask_array = ndarray::Array2::from_shape_vec(
            (batch_size, seq_len), attention_mask.clone(),
        ).map_err(|e| EmbeddingError::InferenceFailed(format!("attention_mask shape: {e}")))?;

        // ort 2.0: 用 TensorRef::from_array_view 包装 ndarray
        let input_ids_tensor = TensorRef::from_array_view(&input_ids_array)
            .map_err(|e| EmbeddingError::InferenceFailed(format!("input_ids tensor: {e}")))?;
        let attention_mask_tensor = TensorRef::from_array_view(&attention_mask_array)
            .map_err(|e| EmbeddingError::InferenceFailed(format!("attention_mask tensor: {e}")))?;

        // 运行推理（ort 2.0: run 需要 &mut self）
        // 锁中毒说明前一次推理 panic，Session 无内部可变状态，取回内部值继续使用
        let mut session = self
            .session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let outputs = if self.need_token_type_ids {
            // BERT 系模型需要 token_type_ids；句子对场景才用 0/1，单句全 0
            let token_type_ids = ndarray::Array2::<i64>::zeros((batch_size, seq_len));
            let token_type_ids_tensor = TensorRef::from_array_view(&token_type_ids)
                .map_err(|e| EmbeddingError::InferenceFailed(format!("token_type_ids tensor: {e}")))?;
            session
                .run(ort::inputs![
                    "input_ids" => input_ids_tensor,
                    "attention_mask" => attention_mask_tensor,
                    "token_type_ids" => token_type_ids_tensor,
                ])
                .map_err(|e| EmbeddingError::InferenceFailed(format!("ort run: {e}")))?
        } else {
            session
                .run(ort::inputs![
                    "input_ids" => input_ids_tensor,
                    "attention_mask" => attention_mask_tensor,
                ])
                .map_err(|e| EmbeddingError::InferenceFailed(format!("ort run: {e}")))?
        };

        // 提取输出张量 (batch, seq_len, dim)
        // ort 2.0: outputs["name"] → &DynValue, .try_extract_tensor::<T>() → Result<(&Shape, &[T])>
        let (_output_shape, output_data) = outputs["last_hidden_state"]
            .try_extract_tensor::<f32>()
            .map_err(|e| EmbeddingError::InferenceFailed(format!("extract output: {e}")))?;

        // Mean-pooling：对 seq_len 维度取平均
        // output_data 布局: [batch, seq_len, dim] row-major
        let mut embeddings = Vec::with_capacity(batch_size);

        for b in 0..batch_size {
            let mut emb = vec![0.0f32; self.dim];
            let mut count = 0usize;

            for s in 0..seq_len {
                // 检查 attention_mask（非零才参与 pooling）
                let mask_val = attention_mask.get(b * seq_len + s).copied().unwrap_or(0);
                if mask_val != 0 {
                    let offset = (b * seq_len + s) * self.dim;
                    for (e, v) in emb.iter_mut().zip(&output_data[offset..offset + self.dim]) {
                        *e += v;
                    }
                    count += 1;
                }
            }

            if count > 0 {
                let inv = 1.0 / count as f32;
                for e in emb.iter_mut() {
                    *e *= inv;
                }
            }

            // L2 归一化
            l2_normalize(&mut emb);

            embeddings.push(emb);
        }

        Ok(embeddings)
    }
}

#[async_trait]
impl EmbeddingBackend for OrtEmbeddingBackend {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let results = self.run_inference(&[text.to_string()])?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::InferenceFailed("no embedding produced".into()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        self.run_inference(texts)
    }

    fn dimension(&self) -> usize {
        self.dim
    }
}

// ============================================================
// 辅助函数
// ============================================================

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
// 测试（真模型；默认 #[ignore]，需要模型文件 + ONNX Runtime DLL）
// ============================================================
//
// 运行方式：
//   ORT_DYLIB_PATH=<...>/onnxruntime.dll \
//   cargo test -p dayu-embedding --features onnx -- --ignored
//
// 模型路径可用 DAYU_EMBEDDING_MODEL_PATH / DAYU_EMBEDDING_TOKENIZER_PATH 覆盖，
// 默认 models/infinilabs/pizza-embed-zh-small-v1。
// 模型缺失时测试直接失败（不静默跳过）——跳过等于没测。

#[cfg(test)]
mod tests {
    use super::*;
    use dayu_domain::math::cosine_similarity;
    use std::sync::OnceLock;

    /// 共享后端实例：模型加载 ~0.5s，只加载一次。
    ///
    /// 模型缺失由 `require_onnx_model_paths` 直接 panic（含处理指引），不跳过。
    fn backend() -> &'static OrtEmbeddingBackend {
        static BACKEND: OnceLock<OrtEmbeddingBackend> = OnceLock::new();
        BACKEND.get_or_init(|| {
            dayu_testkit::ensure_ort_dylib();
            let (mp, tp) = dayu_testkit::require_onnx_model_paths();
            OrtEmbeddingBackend::load(&mp, &tp, 512).expect("load model")
        })
    }

    fn norm(v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    fn assert_unit_norm(v: &[f32], ctx: &str) {
        let n = norm(v);
        assert!(
            (n - 1.0).abs() < 0.01,
            "{ctx}: L2 norm 应约为 1，实际 {n}"
        );
    }

    // ---------- 加载 / 维度 ----------

    #[test]
    #[ignore = "requires ONNX model files"]
    fn loads_model_and_reports_dimension() {
        let dim = backend().dimension();
        assert!(dim > 0, "维度应为正");
        assert!(dim <= 4096, "维度异常: {dim}");
    }

    #[test]
    #[ignore = "requires ONNX model files"]
    fn missing_model_file_returns_err() {
        // 错误路径：不需要真模型，但走同一条 load 路径
        let bogus = std::env::temp_dir().join("dayu-no-such-model.onnx");
        let (_, tokenizer) = dayu_testkit::require_onnx_model_paths();
        let err = OrtEmbeddingBackend::load(&bogus, &tokenizer, 512);
        assert!(err.is_err(), "不存在的模型文件应返回 Err");
    }

    // ---------- 基本嵌入 ----------

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn embed_chinese_text_normalized() {
        let b = backend();
        let emb = b.embed("今天天气真好").await.unwrap();
        assert_eq!(emb.len(), b.dimension());
        assert_unit_norm(&emb, "中文短句");
    }

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn embed_is_deterministic() {
        let b = backend();
        let e1 = b.embed("你好世界").await.unwrap();
        let e2 = b.embed("你好世界").await.unwrap();
        assert_eq!(e1, e2, "同文本应产生完全相同的向量");
    }

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn semantic_similarity_ordering() {
        let b = backend();
        let e1 = b.embed("算了").await.unwrap();
        let e2 = b.embed("不说了").await.unwrap();
        let e3 = b.embed("今天天气好").await.unwrap();

        let cos12 = cosine_similarity(&e1, &e2).unwrap();
        let cos13 = cosine_similarity(&e1, &e3).unwrap();

        assert!(cos12 > 0.5, "近义短语 cos 应 > 0.5，实际 {cos12}");
        assert!(cos13 < cos12, "不相关应低于近义：{cos13} vs {cos12}");
    }

    // ---------- 边界输入 ----------

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn long_text_is_truncated_without_panic() {
        let b = backend();
        // 远超 max_length=512 的长文本（5000 汉字）
        let long: String = "大禹治水，三过家门而不入。".repeat(400);
        let emb = b.embed(&long).await.unwrap();
        assert_eq!(emb.len(), b.dimension(), "截断后维度应不变");
        assert_unit_norm(&emb, "超长文本");
        assert!(emb.iter().all(|x| x.is_finite()), "向量含 NaN/Inf");
    }

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn empty_and_blank_text_is_safe() {
        let b = backend();
        for text in ["", " ", "   \n\t  ", "\u{200b}"] {
            let emb = b.embed(text).await.unwrap_or_else(|e| {
                panic!("空白输入 {text:?} 不应报错: {e}")
            });
            assert_eq!(emb.len(), b.dimension(), "输入 {text:?}");
            assert!(
                emb.iter().all(|x| x.is_finite()),
                "空白输入 {text:?} 产生 NaN/Inf"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn special_chars_and_mixed_language() {
        let b = backend();
        let emb = b.embed("🤖 Hello, 世界！\n\t<b>html</b> \u{0}").await.unwrap();
        assert_eq!(emb.len(), b.dimension());
        assert_unit_norm(&emb, "混合字符");
    }

    // ---------- 批量一致性 ----------

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn batch_matches_single_embedding() {
        let b = backend();
        let texts = vec![
            "短".to_string(),
            "中等长度的句子，包含标点。".to_string(),
            "这是一条明显更长的文本，用来验证批量推理时的 padding 与 attention mask 是否正确。".to_string(),
        ];

        let batch = b.embed_batch(&texts).await.unwrap();
        assert_eq!(batch.len(), texts.len());

        for (i, t) in texts.iter().enumerate() {
            let single = b.embed(t).await.unwrap();
            let sim = cosine_similarity(&batch[i], &single).unwrap();
            assert!(
                (sim - 1.0).abs() < 1e-4,
                "第 {i} 条批量与单条结果应一致，cos={sim}"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires ONNX model files"]
    async fn empty_batch_returns_empty() {
        let b = backend();
        assert!(b.embed_batch(&[]).await.unwrap().is_empty());
    }

    // ---------- 并发安全 ----------

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires ONNX model files"]
    async fn concurrent_embedding_is_safe() {
        let (model, tokenizer) = dayu_testkit::require_onnx_model_paths();
        let b = Arc::new(OrtEmbeddingBackend::load(&model, &tokenizer, 512).expect("load model"));

        let mut handles = Vec::new();
        for i in 0..8 {
            let b = Arc::clone(&b);
            handles.push(tokio::spawn(async move {
                let text = format!("并发测试第 {i} 条文本");
                let emb = b.embed(&text).await.expect("concurrent embed");
                assert_eq!(emb.len(), b.dimension());
                emb
            }));
        }

        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.expect("task join"));
        }
        assert_eq!(results.len(), 8);

        // 并发结果与串行结果一致
        let serial = b.embed("并发测试第 0 条文本").await.unwrap();
        let sim = cosine_similarity(&results[0], &serial).unwrap();
        assert!((sim - 1.0).abs() < 1e-4, "并发结果应与串行一致，cos={sim}");
    }
}
