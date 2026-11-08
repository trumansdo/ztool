//! ONNX 判别器后端
//!
//! 使用 ONNX Runtime 加载判别模型，通过 prompt 模板构造输入，
//! 解析 JSON 输出为 Verdict。
//!
//! 仅在 `onnx` feature 启用时可用。

#[cfg(feature = "onnx")]
use dayu_domain::traits::*;
#[cfg(feature = "onnx")]
use dayu_domain::types::*;
#[cfg(feature = "onnx")]
use dayu_domain::error::*;

/// ONNX 判别器后端
#[cfg(feature = "onnx")]
pub struct OrtJudgeBackend {
    session: ort::Session,
    tokenizer: tokenizers::Tokenizer,
    prompt_template: String,
    max_length: usize,
}

#[cfg(feature = "onnx")]
impl OrtJudgeBackend {
    /// 加载 ONNX 判别模型
    ///
    /// # 参数
    /// - `model_path`: ONNX 模型文件路径
    /// - `tokenizer_path`: tokenizer.json 路径
    /// - `prompt_template`: 判别 prompt 模板（含 {role_state}、{recent_window}、{candidates} 占位符）
    /// - `max_length`: 最大 token 长度
    pub fn load(
        model_path: &std::path::Path,
        tokenizer_path: &std::path::Path,
        prompt_template: &str,
        max_length: usize,
    ) -> Result<Self, JudgeError> {
        let session = ort::Session::builder()
            .map_err(|e| JudgeError::BackendUnavailable)?
            .with_model_from_file(model_path)
            .map_err(|e| {
                tracing::error!(error = %e, path = %model_path.display(), "failed to load judge ONNX model");
                JudgeError::BackendUnavailable
            })?;

        let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
            .map_err(|e| {
                tracing::error!(error = %e, path = %tokenizer_path.display(), "failed to load judge tokenizer");
                JudgeError::BackendUnavailable
            })?;

        tracing::info!(
            model = %model_path.display(),
            "ONNX judge backend loaded"
        );

        Ok(Self {
            session,
            tokenizer,
            prompt_template: prompt_template.to_string(),
            max_length,
        })
    }

    /// 构造判别 prompt
    fn build_prompt(&self, input: &JudgeInput) -> String {
        let recent = input.recent_window.join("\n");
        let candidates = input.candidate_text.clone();
        self.prompt_template
            .replace("{role_state}", "你是一个记忆判别助手")
            .replace("{recent_window}", &recent)
            .replace("{candidates}", &candidates)
    }

    /// 解析模型输出为 Verdict
    fn parse_output(&self, text: &str) -> Result<Verdict, JudgeError> {
        let json_start = text.find('{').unwrap_or(0);
        let json_end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());
        let json_str = &text[json_start..json_end];

        serde_json::from_str::<Verdict>(json_str)
            .map_err(|e| {
                tracing::warn!(error = %e, raw = %text, "failed to parse judge output as Verdict");
                JudgeError::BackendUnavailable
            })
    }
}

#[cfg(feature = "onnx")]
#[async_trait::async_trait]
impl JudgeBackend for OrtJudgeBackend {
    async fn judge(&self, input: &JudgeInput) -> Result<Verdict, JudgeError> {
        let prompt = self.build_prompt(input);

        let encoding = self.tokenizer.encode(prompt, true)
            .map_err(|e| {
                tracing::error!(error = %e, "tokenizer encode failed");
                JudgeError::BackendUnavailable
            })?;

        let token_ids: Vec<i64> = encoding.get_ids()
            .iter()
            .take(self.max_length)
            .map(|&id| id as i64)
            .collect();

        let input_tensor = ort::Value::from_array(
            self.session.allocator(),
            &ndarray::Array1::from_vec(token_ids).into_shape((1, token_ids.len()))
                .map_err(|_| JudgeError::BackendUnavailable)?,
        )
        .map_err(|_| JudgeError::BackendUnavailable)?;

        let outputs = self.session.run(vec![input_tensor])
            .map_err(|e| {
                tracing::error!(error = %e, "ONNX inference failed");
                JudgeError::BackendUnavailable
            })?;

        let output_text = outputs.iter()
            .next()
            .and_then(|(_, v)| {
                v.try_extract::<i64>()
                    .ok()
                    .map(|arr| {
                        arr.iter()
                            .map(|&id| id as u32)
                            .collect::<Vec<u32>>()
                    })
            })
            .and_then(|ids| {
                self.tokenizer.decode(&ids, true).ok()
            })
            .unwrap_or_default();

        self.parse_output(&output_text)
    }

    fn backend_kind(&self) -> BackendKind {
        BackendKind::Onnx
    }
}

// ============================================================
// 默认判别 prompt 模板
// ============================================================

/// 默认判别器 prompt 模板
///
/// 占位符：
/// - `{role_state}`: 角色状态描述
/// - `{recent_window}`: 最近对话窗口
/// - `{candidates}`: 候选片段
pub const DEFAULT_JUDGE_PROMPT_TEMPLATE: &str = r#"{role_state}

最近对话：
{recent_window}

候选片段：
{candidates}

请判断以上候选片段是否值得创建新的记忆球。输出 JSON：
{{
  "should_judge": true/false,
  "significance": 0.0-1.0,
  "offset_direction": "方向描述",
  "emotion_valence": -1.0到1.0,
  "emotion_intensity": 0.0-1.0,
  "hit_dimensions": ["维度1", "维度2"],
  "belongs_to": null或球ID数字,
  "belong_confidence": 0.0-1.0,
  "reason": "判断理由"
}}"#;
