//! dayu-core: 门面 crate，组装所有模块
//!
//! DayuCore 是系统唯一入口，负责：
//! - 组装 StorageBackend + JudgeBackend + EmbeddingBackend + Config + LossModel
//! - 暴露 recall / judge_turn / archive_turn / correct_memory / get_stats

use dayu_domain::traits::*;
use dayu_domain::types::*;
use dayu_domain::error::*;
use dayu_flow::engine::FlowEngine;
use dayu_flow::settle::SettleEngine;
use dayu_flow::loss::{LossModel, PipeWeightLossModel};
use dayu_judge::JudgeEngine;
use std::sync::Arc;
use tracing::Instrument;

/// 归档扫描结果
#[derive(Clone, Debug, Default)]
pub struct ArchiveReport {
    /// 本次新归档的球数
    pub archived: usize,
    /// 本次被唤醒的球数（由 recall 触发，手动唤醒时也计）
    pub woken: usize,
}

/// 核心统计
#[derive(Clone, Debug, Default)]
pub struct CoreStats {
    /// 活跃球数
    pub active_balls: usize,
    /// 归档球数
    pub archived_balls: usize,
    /// 嵌入维度（由后端实际输出决定）
    pub embedding_dim: usize,
}

/// 单轮回忆的统计快照
///
/// 只记录"决策结果"，不记录中间过程；中间过程交给 `debug` 级日志。
#[derive(Clone, Debug, Default)]
pub struct RecallStats {
    /// 起始球数量
    pub start_count: usize,
    /// 被点亮的球数量
    pub lit_count: usize,
    /// 拿到路径强化的球数量
    pub strengthened_count: usize,
    /// 拿到最低强化值的印象球数量
    pub impression_count: usize,
    /// 生成的注入块数量
    pub block_count: usize,
    /// 注入的总 token 数
    pub token_count: usize,
    /// 记账"被挡一次"的管道数量
    pub blocked_pipe_count: usize,
    /// 本轮耗时（毫秒）
    pub elapsed_ms: u64,
}

/// 回忆输出：注入块 + 本轮统计 + 查询标识
#[derive(Clone, Debug)]
pub struct RecallOutput {
    /// 本轮查询标识，调用方可透传到后续 `judge_turn` / `archive_turn`
    pub qid: QueryId,
    /// 注入块
    pub injection_blocks: Vec<InjectionBlock>,
    /// 本轮统计
    pub stats: RecallStats,
}

/// 批量建球的单条输入
///
/// `chain_key` 用于把同一会话的相邻回合连成管道：相同 key 的相邻两条
/// 在 `create_turn_balls_batch` 里会自动建一条边。
pub struct NewBall<'a> {
    pub content: &'a str,
    pub embedding: &'a [f32],
    pub size: f64,
    pub source: TurnId,
    pub chain_key: Option<String>,
}

/// 球构造的唯一入口：所有创建路径（单个/批量/带来源）共用，避免字段漂移
fn build_ball(
    id: BallId,
    content: &str,
    embedding: &[f32],
    size: f64,
    source: TurnId,
    now: chrono::DateTime<chrono::Utc>,
) -> Ball {
    Ball {
        id,
        embedding: embedding.to_vec(),
        ball_stored_size: size.clamp(0.0, 10.0),
        ball_size_ceiling: 10.0,
        content_list: vec![Content {
            id: ContentId(0),
            text: content.to_string(),
            encoding: ContentEncoding::Raw,
            source,
            injected_count: 0,
            status: ContentStatus::Active,
        }],
        content_list_max: 100,
        last_watered: now,
        ball_stability: 7.0,
        lit_count: 0,
        status: BallStatus::Active,
        domain: None,
        created_at: now,
        schema_version: 1,
    }
}

/// 大禹记忆系统门面
pub struct DayuCore {
    config: Arc<Config>,
    flow_engine: FlowEngine,
    settle_engine: SettleEngine,
    judge_engine: JudgeEngine,
    judge_backend: Option<Arc<dyn JudgeBackend>>,
    embedding_backend: Arc<dyn EmbeddingBackend>,
}

/// 按 `config.embedding_backend` 加载嵌入后端
///
/// 三种取值：
/// - `"http"` — OpenAI 兼容 HTTP 接口（需 `http` feature）
/// - `"onnx"` — ONNX Runtime 本地模型（需 `onnx` feature）
/// - `"test"` — 确定性哈希后端（需 `test-support` feature，仅测试）
///
/// 不做事后回退：选中的后端不可用就直接报错。
/// `config.embedding_dim` 为 `None` 时由后端自己报告维度（http 会发一次探测请求）。
pub async fn load_embedding_backend(
    config: &Config,
) -> Result<Arc<dyn EmbeddingBackend>, CoreError> {
    let kind = config.embedding_backend.trim().to_ascii_lowercase();
    match kind.as_str() {
        "http" => {
            #[cfg(feature = "http")]
            {
                if config.embedding_api_base.is_empty() || config.embedding_api_model.is_empty() {
                    return Err(CoreError::Config(
                        "http 后端需要 embedding_api_base 与 embedding_api_model".into(),
                    ));
                }
                let dim = match config.embedding_dim {
                    Some(d) => d,
                    None => dayu_embedding::HttpBackend::probe_dimension(
                        &config.embedding_api_base,
                        &config.embedding_api_key,
                        &config.embedding_api_model,
                    )
                    .await
                    .map_err(|e| CoreError::Config(format!("HTTP 嵌入维度探测失败: {e}")))?,
                };
                let backend = dayu_embedding::HttpBackend::new(
                    &config.embedding_api_base,
                    &config.embedding_api_key,
                    &config.embedding_api_model,
                    dim,
                );
                tracing::info!(
                    dim,
                    base = %config.embedding_api_base,
                    model = %config.embedding_api_model,
                    "HTTP embedding backend loaded"
                );
                Ok(Arc::new(backend))
            }
            #[cfg(not(feature = "http"))]
            {
                Err(CoreError::Config(
                    "config 选了 http 后端，但编译时未启用 http feature".into(),
                ))
            }
        }
        "onnx" => {
            #[cfg(feature = "onnx")]
            {
                let model_path = std::path::Path::new(&config.embedding_model_path);
                let tokenizer_path = std::path::Path::new(&config.embedding_tokenizer_path);
                if model_path.as_os_str().is_empty() || tokenizer_path.as_os_str().is_empty() {
                    return Err(CoreError::Config(
                        "onnx 后端需要 embedding_model_path 与 embedding_tokenizer_path".into(),
                    ));
                }
                let backend = dayu_embedding::OrtEmbeddingBackend::load(model_path, tokenizer_path, 512)
                    .map_err(|e| CoreError::Config(format!("ONNX 嵌入后端加载失败: {e}")))?;
                Ok(Arc::new(backend) as Arc<dyn EmbeddingBackend>)
            }
            #[cfg(not(feature = "onnx"))]
            {
                Err(CoreError::Config(
                    "config 选了 onnx 后端，但编译时未启用 onnx feature".into(),
                ))
            }
        }
        "test" => {
            #[cfg(feature = "test-support")]
            {
                let dim = config.embedding_dim.unwrap_or(512);
                Ok(Arc::new(dayu_embedding::TestEmbeddingBackend::new(dim)))
            }
            #[cfg(not(feature = "test-support"))]
            {
                Err(CoreError::Config(
                    "config 选了 test 后端，但编译时未启用 test-support feature".into(),
                ))
            }
        }
        other => Err(CoreError::Config(format!(
            "未知的 embedding_backend: `{other}`（可选 onnx / http / test）"
        ))),
    }
}

impl DayuCore {
    /// 校验后端维度与配置期望值一致
    ///
    /// `config.embedding_dim` 为 `None` 时不校验（以模型实际维度为准）；
    /// 为 `Some(n)` 时要求后端维度必须等于 `n`。
    fn check_embedding_dim(config: &Config, backend_dim: usize) -> Result<(), CoreError> {
        match config.embedding_dim {
            Some(expected) if backend_dim != expected => Err(CoreError::Config(format!(
                "embedding dimension mismatch: backend has {backend_dim}, config expects {expected}"
            ))),
            _ => Ok(()),
        }
    }

    /// 创建 DayuCore（嵌入后端必传，不再有 mock 默认值）
    ///
    /// 使用默认 PipeWeightLossModel。
    /// 若 config.embedding_dim 为 Some(n)，校验后端维度必须等于 n。
    pub fn new(
        config: Arc<Config>,
        embedding_backend: Arc<dyn EmbeddingBackend>,
    ) -> Result<Self, CoreError> {
        Self::check_embedding_dim(&config, embedding_backend.dimension())?;
        let loss_model: Arc<dyn LossModel> = Arc::new(PipeWeightLossModel);
        let (flow_engine, settle_engine) = Self::engines(Arc::clone(&config), loss_model);
        Ok(Self {
            flow_engine,
            settle_engine,
            judge_engine: JudgeEngine::new(),
            judge_backend: None,
            embedding_backend,
            config,
        })
    }

    /// 创建 DayuCore（指定损失模型，用于对照实验）
    pub fn with_loss_model(
        config: Arc<Config>,
        embedding_backend: Arc<dyn EmbeddingBackend>,
        loss_model: Arc<dyn LossModel>,
    ) -> Result<Self, CoreError> {
        Self::check_embedding_dim(&config, embedding_backend.dimension())?;
        let (flow_engine, settle_engine) = Self::engines(Arc::clone(&config), loss_model);
        Ok(Self {
            flow_engine,
            settle_engine,
            judge_engine: JudgeEngine::new(),
            judge_backend: None,
            embedding_backend,
            config,
        })
    }

    /// 按配置组装流动/结算引擎（含指标后端）
    fn engines(
        config: Arc<Config>,
        loss_model: Arc<dyn LossModel>,
    ) -> (FlowEngine, SettleEngine) {
        let flow = FlowEngine::with_loss_model(Arc::clone(&config), loss_model);
        let settle = SettleEngine::new(Arc::clone(&config));
        if !config.metrics_enabled {
            return (flow, settle);
        }
        let recorder: Arc<dyn dayu_observability::MetricsRecorder> =
            Arc::new(dayu_observability::TracingMetricsRecorder);
        (
            flow.with_metrics(Arc::clone(&recorder)),
            settle.with_metrics(recorder),
        )
    }

    /// 设置判别器后端
    pub fn with_judge_backend(mut self, backend: Arc<dyn JudgeBackend>) -> Self {
        self.judge_backend = Some(backend);
        self
    }

    /// 当前嵌入后端的实际维度
    pub fn embedding_dimension(&self) -> usize {
        self.embedding_backend.dimension()
    }

    /// 获取嵌入后端的引用（用于创建 scoped DayuCore）
    pub fn embedding_backend(&self) -> &Arc<dyn EmbeddingBackend> {
        &self.embedding_backend
    }

    /// 用当前嵌入后端把文本转成向量
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, CoreError> {
        self.embedding_backend
            .embed(text)
            .await
            .map_err(CoreError::Embedding)
    }

    /// 批量嵌入：一次调用送多条文本
    ///
    /// HTTP 后端会合并成一个请求，避免 N 次往返。
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, CoreError> {
        self.embedding_backend
            .embed_batch(texts)
            .await
            .map_err(CoreError::Embedding)
    }

    /// 尝试加载 ONNX 嵌入后端（从 config 读取路径），失败即返回 Err
    ///
    /// 仅在 `onnx` feature 启用时可用。不再回退到 mock。
    #[cfg(feature = "onnx")]
    pub fn with_ort_embedding(mut self) -> Result<Self, CoreError> {
        let model_path = std::path::Path::new(&self.config.embedding_model_path);
        let tokenizer_path = std::path::Path::new(&self.config.embedding_tokenizer_path);

        if model_path.as_os_str().is_empty() || tokenizer_path.as_os_str().is_empty() {
            return Err(CoreError::Config(
                "embedding_model_path 或 embedding_tokenizer_path 未配置".to_string(),
            ));
        }

        let ort_backend = dayu_embedding::OrtEmbeddingBackend::load(model_path, tokenizer_path, 512)
            .map_err(|e| CoreError::Config(format!(
                "ONNX 嵌入后端加载失败: {} (model={}, tokenizer={})",
                e,
                model_path.display(),
                tokenizer_path.display(),
            )))?;

        let backend_dim = ort_backend.dimension();
        Self::check_embedding_dim(&self.config, backend_dim)?;

        tracing::info!(
            dim = backend_dim,
            model = %model_path.display(),
            "ONNX embedding backend loaded"
        );
        self.embedding_backend = Arc::new(ort_backend);
        Ok(self)
    }

    /// 尝试加载 ONNX 判别器后端（从 config 读取路径），失败即返回 Err
    #[cfg(feature = "judge-onnx")]
    pub fn with_ort_judge(mut self) -> Result<Self, CoreError> {
        let model_path = std::path::Path::new(&self.config.embedding_model_path);
        let tokenizer_path = std::path::Path::new(&self.config.embedding_tokenizer_path);

        if model_path.as_os_str().is_empty() || tokenizer_path.as_os_str().is_empty() {
            return Err(CoreError::Config(
                "embedding_model_path 或 embedding_tokenizer_path 未配置（判别器需要模型）".to_string(),
            ));
        }

        let ort_backend = dayu_judge::OrtJudgeBackend::load(
            model_path,
            tokenizer_path,
            dayu_judge::DEFAULT_JUDGE_PROMPT_TEMPLATE,
            512,
        )
        .map_err(|e| CoreError::Config(format!(
            "ONNX 判别器后端加载失败: {} (model={})",
            e,
            model_path.display(),
        )))?;

        tracing::info!(
            model = %model_path.display(),
            "ONNX judge backend loaded"
        );
        self.judge_backend = Some(Arc::new(ort_backend));
        Ok(self)
    }

    /// 回忆：query → 流动 → 结算 → 注入块
    ///
    /// 便利方法：内部生成 `QueryId`。需要串联链路时用 [`Self::recall_with_qid`]。
    pub async fn recall(
        &self,
        storage: &dyn StorageBackend,
        query_embedding: &[f32],
        significance: f64,
    ) -> Result<Vec<InjectionBlock>, Box<dyn std::error::Error + Send + Sync>> {
        let output = self
            .recall_with_qid(storage, query_embedding, significance, QueryId::new())
            .await?;
        Ok(output.injection_blocks)
    }

    /// 回忆（带查询标识）：整轮打一个 `info` span，退出时回填统计字段
    pub async fn recall_with_qid(
        &self,
        storage: &dyn StorageBackend,
        query_embedding: &[f32],
        significance: f64,
        qid: QueryId,
    ) -> Result<RecallOutput, Box<dyn std::error::Error + Send + Sync>> {
        let span = tracing::info_span!(
            "recall",
            qid = %qid,
            query_dim = query_embedding.len(),
            significance,
            starts = tracing::field::Empty,
            lit = tracing::field::Empty,
            strengthened = tracing::field::Empty,
            impression = tracing::field::Empty,
            blocks = tracing::field::Empty,
            tokens = tracing::field::Empty,
            blocked = tracing::field::Empty,
            elapsed_ms = tracing::field::Empty,
        );
        let recorder = span.clone();
        let qid_out = qid.clone();
        let started = std::time::Instant::now();

        let result: Result<RecallOutput, Box<dyn std::error::Error + Send + Sync>> = async move {
            let outcome = self
                .flow_engine
                .flow(storage, query_embedding, significance)
                .await?;
            let start_ids: Vec<BallId> = outcome
                .lit_balls
                .iter()
                .filter(|l| l.depth == 0)
                .map(|l| l.ball_id)
                .collect();
            self.wake_archived(storage, &start_ids).await?;
            let ball_ids: Vec<BallId> = outcome.iter().map(|l| l.ball_id).collect();
            let sub = storage.load_subgraph(&ball_ids, self.config.max_flow_depth).await?;
            let settled = self
                .settle_engine
                .settle(
                    storage,
                    &outcome.lit_balls,
                    &sub,
                    &[query_embedding],
                    &outcome.blocked_pipes,
                )
                .await?;

            let stats = RecallStats {
                start_count: start_ids.len(),
                lit_count: outcome.lit_balls.len(),
                strengthened_count: settled.reinforced_balls,
                impression_count: settled.impression_balls,
                block_count: settled.injection_blocks.len(),
                token_count: settled.total_tokens,
                blocked_pipe_count: settled.blocked_pipes,
                elapsed_ms: started.elapsed().as_millis() as u64,
            };
            recorder.record("starts", stats.start_count);
            recorder.record("lit", stats.lit_count);
            recorder.record("strengthened", stats.strengthened_count);
            recorder.record("impression", stats.impression_count);
            recorder.record("blocks", stats.block_count);
            recorder.record("tokens", stats.token_count);
            recorder.record("blocked", stats.blocked_pipe_count);
            recorder.record("elapsed_ms", stats.elapsed_ms);

            Ok(RecallOutput {
                qid: qid_out,
                injection_blocks: settled.injection_blocks,
                stats,
            })
        }
        .instrument(span)
        .await;

        result
    }

    /// 回忆（文本输入）：自动嵌入 → 流动 → 结算 → 注入块
    pub async fn recall_with_text(
        &self,
        storage: &dyn StorageBackend,
        query_text: &str,
        significance: f64,
    ) -> Result<Vec<InjectionBlock>, Box<dyn std::error::Error + Send + Sync>> {
        let output = self
            .recall_with_text_qid(storage, query_text, significance, QueryId::new())
            .await?;
        Ok(output.injection_blocks)
    }

    /// 回忆（文本输入 + 查询标识）
    pub async fn recall_with_text_qid(
        &self,
        storage: &dyn StorageBackend,
        query_text: &str,
        significance: f64,
        qid: QueryId,
    ) -> Result<RecallOutput, Box<dyn std::error::Error + Send + Sync>> {
        let query_embedding = self.embedding_backend.embed(query_text)
            .await
            .map_err(CoreError::Embedding)?;
        self.recall_with_qid(storage, &query_embedding, significance, qid).await
    }

    /// 异步判别：对候选片段进行判别，返回处理结果
    ///
    /// 若未设置 judge_backend，返回 `CoreError::Config`。
    pub async fn judge_turn(
        &self,
        input: &JudgeInput,
    ) -> Result<JudgeResult, CoreError> {
        let backend = self.judge_backend.as_ref().ok_or_else(|| {
            CoreError::Config(
                "判别器未初始化：请在 config 中配置 embedding_model_path 与 \
                 embedding_tokenizer_path（判别需要模型），或改用 recall 系列接口"
                    .to_string(),
            )
        })?;
        let result = self.judge_engine.judge(backend.as_ref(), input)
            .await
            .map_err(CoreError::Judge)?;
        Ok(result)
    }

    /// 获取当前配置快照
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    // ============================================================
    // 对外 API（供 napi / CLI / 插件工具调用）
    // ============================================================

    /// 回忆（多输入源）：一轮对话里的用户消息、工具结果、模型输出可各自产生查询嵌入
    pub async fn recall_multi(
        &self,
        storage: &dyn StorageBackend,
        query_embeddings: &[&[f32]],
        significance: f64,
    ) -> Result<Vec<InjectionBlock>, Box<dyn std::error::Error + Send + Sync>> {
        let output = self
            .recall_multi_with_qid(storage, query_embeddings, significance, QueryId::new())
            .await?;
        Ok(output.injection_blocks)
    }

    /// 回忆（多输入源 + 查询标识）
    pub async fn recall_multi_with_qid(
        &self,
        storage: &dyn StorageBackend,
        query_embeddings: &[&[f32]],
        significance: f64,
        qid: QueryId,
    ) -> Result<RecallOutput, Box<dyn std::error::Error + Send + Sync>> {
        let span = tracing::info_span!(
            "recall_multi",
            qid = %qid,
            sources = query_embeddings.len(),
            significance,
            starts = tracing::field::Empty,
            lit = tracing::field::Empty,
            strengthened = tracing::field::Empty,
            impression = tracing::field::Empty,
            blocks = tracing::field::Empty,
            tokens = tracing::field::Empty,
            blocked = tracing::field::Empty,
            elapsed_ms = tracing::field::Empty,
        );
        let recorder = span.clone();
        let started = std::time::Instant::now();

        let result: Result<RecallOutput, Box<dyn std::error::Error + Send + Sync>> = async move {
            if query_embeddings.is_empty() {
                return Ok(RecallOutput {
                    qid,
                    injection_blocks: Vec::new(),
                    stats: RecallStats::default(),
                });
            }
            let outcome = self
                .flow_engine
                .flow_multi(storage, query_embeddings, significance)
                .await?;
            let start_ids: Vec<BallId> = outcome
                .lit_balls
                .iter()
                .filter(|l| l.depth == 0)
                .map(|l| l.ball_id)
                .collect();
            self.wake_archived(storage, &start_ids).await?;
            let ball_ids: Vec<BallId> = outcome.iter().map(|l| l.ball_id).collect();
            let sub = storage
                .load_subgraph(&ball_ids, self.config.max_flow_depth)
                .await?;
            let settled = self
                .settle_engine
                .settle(
                    storage,
                    &outcome.lit_balls,
                    &sub,
                    query_embeddings,
                    &outcome.blocked_pipes,
                )
                .await?;

            let stats = RecallStats {
                start_count: start_ids.len(),
                lit_count: outcome.lit_balls.len(),
                strengthened_count: settled.reinforced_balls,
                impression_count: settled.impression_balls,
                block_count: settled.injection_blocks.len(),
                token_count: settled.total_tokens,
                blocked_pipe_count: settled.blocked_pipes,
                elapsed_ms: started.elapsed().as_millis() as u64,
            };
            recorder.record("starts", stats.start_count);
            recorder.record("lit", stats.lit_count);
            recorder.record("strengthened", stats.strengthened_count);
            recorder.record("impression", stats.impression_count);
            recorder.record("blocks", stats.block_count);
            recorder.record("tokens", stats.token_count);
            recorder.record("blocked", stats.blocked_pipe_count);
            recorder.record("elapsed_ms", stats.elapsed_ms);

            Ok(RecallOutput {
                qid,
                injection_blocks: settled.injection_blocks,
                stats,
            })
        }
        .instrument(span)
        .await;

        result
    }

    /// 直接创建新球（AI 主动"记一件事"，不经判别器）
    pub async fn create_ball(
        &self,
        storage: &dyn StorageBackend,
        content: &str,
        embedding: &[f32],
        size: f64,
    ) -> Result<BallId, Box<dyn std::error::Error + Send + Sync>> {
        self.create_ball_with_source(storage, content, embedding, size, TurnId(String::new()))
            .await
    }

    /// 创建新球并记录来源回合
    ///
    /// `source` 写入 `Content.source`，随 Ball 一起序列化进存储，重启后仍可读回。
    /// 用途：外部系统（评测服务、插件）需要把球映射回原始回合，
    /// 而 `create_ball` 无法携带这一信息。
    pub async fn create_ball_with_source(
        &self,
        storage: &dyn StorageBackend,
        content: &str,
        embedding: &[f32],
        size: f64,
        source: TurnId,
    ) -> Result<BallId, Box<dyn std::error::Error + Send + Sync>> {
        let id = storage.allocate_ball_id().await?;
        let ball = build_ball(id, content, embedding, size, source, chrono::Utc::now());
        let mut tx = storage.begin_transaction().await?;
        tx.upsert_ball(&ball).await?;
        tx.commit().await?;
        Ok(id)
    }

    /// 批量建球 + 同会话相邻回合自动连管道（评测灌库）
    ///
    /// 与逐个调 `create_ball_with_source` + `link_balls` 语义等价，但整批只开一个事务。
    /// 实测逐球提交 ~37ms/球（fsync 主导），批量后降到毫秒级——评测每题要灌
    /// 几百到几千回合，不批量的话单次评测跑不完。
    ///
    /// `link_weight` 为 `Some(w)` 时，`chain_key` 相同的相邻两球之间建一条粗细 `w`
    /// 的管道（`PipeSource::NewBall`），且与球同事务写入。
    pub async fn create_turn_balls_batch(
        &self,
        storage: &dyn StorageBackend,
        items: &[NewBall<'_>],
        link_weight: Option<f64>,
    ) -> Result<Vec<BallId>, Box<dyn std::error::Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        // ID 在事务外分配：allocate 失败时尚未写任何数据，重试安全
        let mut ids = Vec::with_capacity(items.len());
        for _ in items {
            ids.push(storage.allocate_ball_id().await?);
        }

        let now = chrono::Utc::now();
        let mut tx = storage.begin_transaction().await?;
        let mut last_in_chain: std::collections::HashMap<&str, BallId> =
            std::collections::HashMap::new();

        for (item, id) in items.iter().zip(ids.iter().copied()) {
            let ball = build_ball(
                id,
                item.content,
                item.embedding,
                item.size,
                item.source.clone(),
                now,
            );
            tx.upsert_ball(&ball).await?;

            let Some(weight) = link_weight else { continue };
            let Some(key) = item.chain_key.as_deref() else {
                continue;
            };
            if let Some(prev) = last_in_chain.insert(key, id) {
                let (lo, hi) = if prev < id { (prev, id) } else { (id, prev) };
                let pipe = Pipe {
                    id: storage.allocate_pipe_id().await?,
                    a: lo,
                    b: hi,
                    pipe_base_thickness: weight.clamp(0.0, self.config.pipe_thickness_ceiling),
                    pipe_stability: 3.0,
                    last_blocked_at: None,
                    blocked_pass_count_stored: 0,
                    last_flowed: now,
                    source: PipeSource::NewBall,
                    status: PipeStatus::Active,
                };
                tx.upsert_pipe(&pipe).await?;
            }
        }

        tx.commit().await?;
        Ok(ids)
    }

    /// 批量建球：一个事务写入整批，适合规模冒烟与批量导入
    ///
    /// 语义与逐个调 `create_ball` 一致（size 夹到 `[0,10]`、新球不自动接图）。
    /// 逐条建球在十万级规模下会因每球一个事务而慢到不可用。
    pub async fn create_balls_batch(
        &self,
        storage: &dyn StorageBackend,
        items: &[(String, Vec<f32>, f64)],
    ) -> Result<Vec<BallId>, Box<dyn std::error::Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        let now = chrono::Utc::now();
        let mut tx = storage.begin_transaction().await?;
        let mut ids = Vec::with_capacity(items.len());
        for (content, embedding, size) in items {
            let id = storage.allocate_ball_id().await?;
            let ball = build_ball(
                id,
                content,
                embedding,
                *size,
                TurnId(String::new()),
                now,
            );
            tx.upsert_ball(&ball).await?;
            ids.push(id);
        }
        tx.commit().await?;
        Ok(ids)
    }

    /// 手动建管道（AI 主动建立关联）
    pub async fn link_balls(
        &self,
        storage: &dyn StorageBackend,
        a: BallId,
        b: BallId,
        weight: f64,
    ) -> Result<PipeId, Box<dyn std::error::Error + Send + Sync>> {
        if a == b {
            return Err("不能把球连到它自己".into());
        }
        if storage.get_ball(a).await?.is_none() || storage.get_ball(b).await?.is_none() {
            return Err(format!("球不存在: {a:?} / {b:?}").into());
        }
        let id = storage.allocate_pipe_id().await?;
        let now = chrono::Utc::now();
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let pipe = Pipe {
            id,
            a: lo,
            b: hi,
            pipe_base_thickness: weight.clamp(0.0, self.config.pipe_thickness_ceiling),
            pipe_stability: 3.0,
            last_blocked_at: None,
            blocked_pass_count_stored: 0,
            last_flowed: now,
            source: PipeSource::Manual,
            status: PipeStatus::Active,
        };
        let mut tx = storage.begin_transaction().await?;
        tx.upsert_pipe(&pipe).await?;
        tx.commit().await?;
        Ok(id)
    }

    /// 批量建管道：一次事务写入全部边
    ///
    /// 语义与逐个调 `link_balls` 一致（端点规范化、粗细夹取），
    /// 但只为整批开一个事务。规模冒烟时逐条建图会成为瓶颈。
    ///
    /// 返回实际写入的管道数；自连或端点不存在的边会被跳过。
    pub async fn link_balls_batch(
        &self,
        storage: &dyn StorageBackend,
        edges: &[(BallId, BallId, f64)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if edges.is_empty() {
            return Ok(0);
        }
        let mut ids: Vec<BallId> = Vec::with_capacity(edges.len() * 2);
        for (a, b, _) in edges {
            ids.push(*a);
            ids.push(*b);
        }
        let existing: std::collections::HashSet<BallId> = storage
            .get_balls(&ids)
            .await?
            .into_iter()
            .map(|b| b.id)
            .collect();

        let now = chrono::Utc::now();
        let mut tx = storage.begin_transaction().await?;
        let mut written = 0usize;
        for (a, b, weight) in edges {
            if a == b || !existing.contains(a) || !existing.contains(b) {
                continue;
            }
            let id = storage.allocate_pipe_id().await?;
            let (lo, hi) = if a < b { (*a, *b) } else { (*b, *a) };
            let pipe = Pipe {
                id,
                a: lo,
                b: hi,
                pipe_base_thickness: weight.clamp(0.0, self.config.pipe_thickness_ceiling),
                pipe_stability: 3.0,
                last_blocked_at: None,
                blocked_pass_count_stored: 0,
                last_flowed: now,
                source: PipeSource::Manual,
                status: PipeStatus::Active,
            };
            tx.upsert_pipe(&pipe).await?;
            written += 1;
        }
        tx.commit().await?;
        Ok(written)
    }

    /// 读单个球
    pub async fn get_ball(
        &self,
        storage: &dyn StorageBackend,
        id: BallId,
    ) -> Result<Option<Ball>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(storage.get_ball(id).await?)
    }

    /// 按状态列球
    pub async fn list_balls(
        &self,
        storage: &dyn StorageBackend,
        status: Option<BallStatus>,
    ) -> Result<Vec<Ball>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(storage.list_balls(status).await?)
    }

    /// 列出全部管道
    pub async fn list_pipes(
        &self,
        storage: &dyn StorageBackend,
    ) -> Result<Vec<Pipe>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(storage.list_pipes().await?)
    }

    /// 改球状态
    pub async fn set_ball_status(
        &self,
        storage: &dyn StorageBackend,
        id: BallId,
        status: BallStatus,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = storage.begin_transaction().await?;
        tx.update_ball_status(id, status).await?;
        tx.commit().await?;
        Ok(())
    }

    /// 统计
    pub async fn stats(
        &self,
        storage: &dyn StorageBackend,
    ) -> Result<CoreStats, Box<dyn std::error::Error + Send + Sync>> {
        let active = storage.list_balls(Some(BallStatus::Active)).await?;
        let archived = storage.list_balls(Some(BallStatus::Archived)).await?;
        Ok(CoreStats {
            active_balls: active.len(),
            archived_balls: archived.len(),
            embedding_dim: self.embedding_dimension(),
        })
    }

    /// 归档扫描：把长期未注水且已衰减的活跃球移出主图
    pub async fn archive_sweep(
        &self,
        storage: &dyn StorageBackend,
    ) -> Result<ArchiveReport, Box<dyn std::error::Error + Send + Sync>> {
        let now = chrono::Utc::now();
        let cfg = self.config.as_ref();
        let active = storage.list_balls(Some(BallStatus::Active)).await?;

        let mut victims: Vec<BallId> = active
            .iter()
            .filter(|b| {
                let idle_days = (now - b.last_watered).num_seconds().max(0) as f64 / 86_400.0;
                idle_days > cfg.archive_inactive_days
                    && b.size_effective(now) < cfg.archive_size_threshold
            })
            .map(|b| b.id)
            .collect();

        let remaining = active.len().saturating_sub(victims.len());
        if remaining > cfg.max_active_balls {
            let mut rest: Vec<&Ball> = active
                .iter()
                .filter(|b| !victims.contains(&b.id))
                .collect();
            rest.sort_by(|a, b| {
                a.size_effective(now)
                    .partial_cmp(&b.size_effective(now))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.id.cmp(&b.id))
            });
            let need = remaining - cfg.max_active_balls;
            victims.extend(rest.into_iter().take(need).map(|b| b.id));
        }

        if victims.is_empty() {
            return Ok(ArchiveReport::default());
        }

        let mut tx = storage.begin_transaction().await?;
        for id in &victims {
            tx.update_ball_status(*id, BallStatus::Archived).await?;
        }
        tx.commit().await?;

        Ok(ArchiveReport {
            archived: victims.len(),
            woken: 0,
        })
    }

    /// 唤醒归档球
    pub async fn wake_archived(
        &self,
        storage: &dyn StorageBackend,
        ids: &[BallId],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }
        let archived: Vec<BallId> = storage
            .get_balls(ids)
            .await?
            .into_iter()
            .filter(|b| b.status == BallStatus::Archived)
            .map(|b| b.id)
            .collect();
        if archived.is_empty() {
            return Ok(0);
        }
        let mut tx = storage.begin_transaction().await?;
        for id in &archived {
            tx.update_ball_status(*id, BallStatus::Active).await?;
        }
        tx.commit().await?;
        Ok(archived.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dayu_storage::testing::{seed, temp_store};
    use dayu_testkit::*;
    #[cfg(feature = "onnx")]
    use std::sync::LazyLock;

    // ============================================================
    // 全局单例：ort 只允许一个 Environment per process
    // ============================================================

    /// 全局嵌入后端（真模型），进程内只加载一次。
    ///
    /// 模型缺失**直接 panic**，不跳过：跳过会让 ONNX 测试在缺模型时静默变绿。
    /// 详见 `dayu_testkit::require_onnx_model_paths` 的注释。
    #[cfg(feature = "onnx")]
    static GLOBAL_EMBEDDING: LazyLock<Arc<dyn EmbeddingBackend>> = LazyLock::new(|| {
        // 必须在任何 ort 调用之前：系统 PATH 里那个旧 onnxruntime.dll 会先被找到
        dayu_testkit::ensure_ort_dylib();

        let (model_path, tokenizer_path) = dayu_testkit::require_onnx_model_paths();
        let backend = dayu_embedding::OrtEmbeddingBackend::load(&model_path, &tokenizer_path, 512)
            .expect("ONNX 模型加载失败");
        Arc::new(backend) as Arc<dyn EmbeddingBackend>
    });

    #[cfg(feature = "onnx")]
    fn ort_backend() -> Arc<dyn EmbeddingBackend> {
        GLOBAL_EMBEDDING.clone()
    }

    #[cfg(feature = "onnx")]
    fn make_ort_core() -> DayuCore {
        DayuCore::new(Arc::new(Config::default()), ort_backend()).expect("DayuCore::new 应成功")
    }

    // ============================================================
    // 测试后端：确定性哈希向量，不依赖模型文件与 ONNX Runtime
    // ============================================================

    /// 确定性嵌入后端（同文本→同向量，不同文本→近似正交）
    fn test_backend() -> Arc<dyn EmbeddingBackend> {
        Arc::new(dayu_embedding::TestEmbeddingBackend::new(512))
    }

    /// 不需要真实语义的用例统一用这个构造
    fn test_core() -> DayuCore {
        DayuCore::new(Arc::new(Config::default()), test_backend())
            .expect("DayuCore::new with test backend should succeed")
    }

    // ============================================================
    // 基础 recall 测试（需要 ONNX）
    // ============================================================

    #[cfg(feature = "onnx")]
    #[tokio::test]
    async fn dayu_core_recall_returns_blocks() {
        let store = temp_store();
        let core = make_ort_core();

        // 用真实嵌入建球
        let emb_a = core.embed_text("Rust的所有权系统保证内存安全").await.unwrap();
        let emb_b = core.embed_text("Python的装饰器是一种高阶函数").await.unwrap();
        let emb_c = core.embed_text("Rust的借用和生命周期机制").await.unwrap();

        let a = core.create_ball(&store, "Rust所有权", &emb_a, 3.0).await.unwrap();
        let b = core.create_ball(&store, "Python装饰器", &emb_b, 3.0).await.unwrap();
        let c = core.create_ball(&store, "Rust借用生命周期", &emb_c, 3.0).await.unwrap();

        core.link_balls(&store, a, b, 0.3).await.unwrap();
        core.link_balls(&store, a, c, 0.8).await.unwrap();
        core.link_balls(&store, c, b, 0.2).await.unwrap();

        let query_emb = core.embed_text("Rust的内存管理机制").await.unwrap();
        let blocks = core.recall(&store, &query_emb, 1.0).await.unwrap();
        assert!(!blocks.is_empty(), "recall should return injection blocks");
    }

    #[cfg(feature = "onnx")]
    #[tokio::test]
    async fn dayu_core_with_constant_loss_model() {
        let store = temp_store();
        let config = Arc::new(Config::default());
        let loss_model: Arc<dyn LossModel> = Arc::new(dayu_flow::loss::ConstantLossModel);
        let backend = ort_backend();
        let core = DayuCore::with_loss_model(config, backend, loss_model).unwrap();

        let emb = core.embed_text("测试内容").await.unwrap();
        core.create_ball(&store, "测试", &emb, 3.0).await.unwrap();

        let query_emb = core.embed_text("测试查询").await.unwrap();
        let blocks = core.recall(&store, &query_emb, 1.0).await.unwrap();
        assert!(!blocks.is_empty(), "constant loss model should still produce blocks");
    }

    // ============================================================
    // 维度校验（不需要 ONNX，用 config 直接测）
    // ============================================================

    #[tokio::test]
    async fn dimension_mismatch_rejected() {
        // 后端实际 512 维，config 期望 384 维 → 构造阶段就应报错
        let cfg = Config {
            embedding_dim: Some(384),
            ..Config::default()
        };
        let err = match DayuCore::new(Arc::new(cfg), test_backend()) {
            Ok(_) => panic!("维度不匹配时构造应失败"),
            Err(e) => e,
        };
        assert!(
            matches!(err, CoreError::Config(_)),
            "维度不匹配应返回 CoreError::Config，实得 {err:?}"
        );

        // config 不指定维度时不校验，以后端为准
        let cfg = Config {
            embedding_dim: None,
            ..Config::default()
        };
        let core = DayuCore::new(Arc::new(cfg), test_backend()).unwrap();
        assert_eq!(core.embedding_dimension(), 512);
    }

    // ============================================================
    // 归档 / 唤醒（不需要嵌入后端，只测存储逻辑）
    // ============================================================

    fn idle_ball(id: u64, size: f64, stability: f64, days: i64) -> dayu_domain::types::Ball {
        let mut b = sample_ball(id, size, vec![0.1; 512]);
        b.ball_stability = stability;
        b.last_watered = chrono::Utc::now() - chrono::Duration::days(days);
        b
    }

    /// 创建一个最小 DayuCore 用于归档测试（无需真实模型）
    fn archive_test_core() -> DayuCore {
        test_core()
    }

    #[tokio::test]
    async fn archive_sweep_archives_only_idle_and_decayed() {
        let core = archive_test_core();
        let store = temp_store();
        seed(
            &store,
            vec![
                idle_ball(1, 0.05, 7.0, 200),
                idle_ball(2, 9.0, 30.0, 100),
                idle_ball(3, 0.05, 7.0, 0),
            ],
            vec![],
        )
        .await;

        let report = core.archive_sweep(&store).await.unwrap();
        assert_eq!(report.archived, 1);
        assert_eq!(
            store.get_ball(BallId(1)).await.unwrap().unwrap().status,
            BallStatus::Archived
        );
        assert_eq!(
            store.get_ball(BallId(2)).await.unwrap().unwrap().status,
            BallStatus::Active
        );
    }

    #[tokio::test]
    async fn archive_sweep_enforces_active_cap_by_weakest_first() {
        let cfg = Config {
            max_active_balls: 1,
            ..Config::default()
        };
        let config = Arc::new(cfg);
        let core = DayuCore::new(config, test_backend()).unwrap();
        let store = temp_store();
        seed(
            &store,
            vec![
                idle_ball(1, 1.0, 30.0, 0),
                idle_ball(2, 5.0, 30.0, 0),
                idle_ball(3, 9.0, 30.0, 0),
            ],
            vec![],
        )
        .await;

        let report = core.archive_sweep(&store).await.unwrap();
        assert_eq!(report.archived, 2, "超出上限的 2 个应被强制归档");
        let active: Vec<u64> = store
            .list_balls(Some(BallStatus::Active))
            .await
            .unwrap()
            .iter()
            .map(|b| b.id.0)
            .collect();
        assert_eq!(active, vec![3], "应保留最强的球");
    }

    #[tokio::test]
    async fn wake_archived_flips_only_archived_balls() {
        let core = archive_test_core();
        let store = temp_store();
        let mut archived = sample_ball(1, 1.0, vec![0.1; 512]);
        archived.status = BallStatus::Archived;
        let active = sample_ball(2, 1.0, vec![0.1; 512]);
        seed(&store, vec![archived, active], vec![]).await;

        let woken = core
            .wake_archived(&store, &[BallId(1), BallId(2)])
            .await
            .unwrap();
        assert_eq!(woken, 1, "只有归档球被翻转");
        assert_eq!(
            store.get_ball(BallId(1)).await.unwrap().unwrap().status,
            BallStatus::Active
        );
    }

    #[tokio::test]
    async fn recall_wakes_archived_start_ball() {
        let core = archive_test_core();
        let store = temp_store();
        let mut ball = sample_ball(1, 5.0, vec![0.1; 512]);
        ball.status = BallStatus::Archived;
        seed(&store, vec![ball], vec![]).await;

        let _ = core.recall(&store, &vec![0.1f32; 512], 1.0).await.unwrap();

        assert_eq!(
            store.get_ball(BallId(1)).await.unwrap().unwrap().status,
            BallStatus::Active,
            "被检索命中的归档球应直接翻回 Active（无临时态）"
        );
    }

    // ============================================================
    // 对外 API
    // ============================================================

    #[tokio::test]
    async fn create_ball_writes_content_and_allocates_ids() {
        let core = archive_test_core();
        let store = temp_store();

        let a = core
            .create_ball(&store, "第一件事", &vec![0.1; 512], 2.0)
            .await
            .unwrap();
        let b = core
            .create_ball(&store, "第二件事", &vec![0.2; 512], 99.0)
            .await
            .unwrap();
        assert_ne!(a, b, "ID 必须递增分配");

        let ball = store.get_ball(a).await.unwrap().unwrap();
        assert_eq!(ball.content_list.len(), 1);
        assert_eq!(ball.content_list[0].text, "第一件事");
        assert_eq!(ball.ball_stored_size, 2.0);
        assert_eq!(ball.status, BallStatus::Active);

        let capped = store.get_ball(b).await.unwrap().unwrap();
        assert_eq!(capped.ball_stored_size, 10.0);
    }

    #[tokio::test]
    async fn create_ball_with_source_persists_turn_id() {
        let core = archive_test_core();
        let store = temp_store();

        // 外部系统把「会话 + 回合」编码进 TurnId，需能原样读回（TriviumDB 重启后也在）
        let source = TurnId(r#"{"session_id":"s1","turn_idx":7}"#.to_string());
        let id = core
            .create_ball_with_source(&store, "用户说要去成都", &vec![0.3; 512], 1.0, source.clone())
            .await
            .unwrap();

        let ball = store.get_ball(id).await.unwrap().unwrap();
        assert_eq!(ball.content_list[0].source, source, "来源回合必须随球落库");

        // create_ball 保持旧行为：来源为空串
        let plain = core
            .create_ball(&store, "无关的一句话", &vec![0.4; 512], 1.0)
            .await
            .unwrap();
        let plain_ball = store.get_ball(plain).await.unwrap().unwrap();
        assert_eq!(plain_ball.content_list[0].source, TurnId(String::new()));
    }

    #[tokio::test]
    async fn create_turn_balls_batch_chains_same_session_only() {
        let core = archive_test_core();
        let store = temp_store();

        // 两条会话交错灌入：s1 三回合、s2 两回合
        let mk = |text: &'static str, sid: &'static str| NewBall {
            content: text,
            embedding: &[0.1; 512],
            size: 1.0,
            source: TurnId(sid.to_string()),
            chain_key: Some(sid.to_string()),
        };
        let items = vec![
            mk("s1-0", "s1"),
            mk("s2-0", "s2"),
            mk("s1-1", "s1"),
            mk("s2-1", "s2"),
            mk("s1-2", "s1"),
        ];

        let ids = core
            .create_turn_balls_batch(&store, &items, Some(0.5))
            .await
            .unwrap();
        assert_eq!(ids.len(), 5);

        let pipes = store.list_pipes().await.unwrap();
        // 只有同 session 的相邻回合连边：s1 两条、s2 一条；跨会话不得连
        assert_eq!(pipes.len(), 3, "实际管道：{pipes:?}");
        let has = |a: BallId, b: BallId| {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            pipes.iter().any(|p| p.a == lo && p.b == hi)
        };
        assert!(has(ids[0], ids[2]), "s1 的 0↔1 应连边");
        assert!(has(ids[2], ids[4]), "s1 的 1↔2 应连边");
        assert!(has(ids[1], ids[3]), "s2 的 0↔1 应连边");
        assert!(!has(ids[0], ids[1]), "不同会话不得连边");
        assert!(
            pipes.iter().all(|p| (p.pipe_base_thickness - 0.5).abs() < 1e-9),
            "管道粗细应等于 link_weight"
        );

        // link_weight = None：只建球不建边
        let store2 = temp_store();
        core.create_turn_balls_batch(&store2, &items, None)
            .await
            .unwrap();
        assert!(store2.list_pipes().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn link_balls_rejects_self_and_missing() {
        let core = archive_test_core();
        let store = temp_store();
        let a = core
            .create_ball(&store, "A", &vec![0.1; 512], 1.0)
            .await
            .unwrap();

        assert!(core.link_balls(&store, a, a, 0.5).await.is_err(), "自连应报错");
        assert!(
            core.link_balls(&store, a, BallId(999), 0.5).await.is_err(),
            "球不存在应报错"
        );
    }

    #[tokio::test]
    async fn link_balls_creates_pipe_with_clamped_weight() {
        let core = archive_test_core();
        let store = temp_store();
        let a = core.create_ball(&store, "A", &vec![0.1; 512], 1.0).await.unwrap();
        let b = core.create_ball(&store, "B", &vec![0.2; 512], 1.0).await.unwrap();

        let pid = core.link_balls(&store, b, a, 5.0).await.unwrap();
        let sub = store.load_subgraph(&[a], 1).await.unwrap();
        let pipe = sub.pipes.iter().find(|p| p.id == pid).expect("管道应存在");
        assert_eq!(pipe.pipe_base_thickness, 1.0, "粗细应夹到 pipe_thickness_ceiling");
        assert_eq!((pipe.a, pipe.b), (a, b), "端点应规范化成 (min, max)");
        assert_eq!(pipe.source, PipeSource::Manual);
    }

    #[tokio::test]
    async fn recall_multi_returns_blocks_for_two_sources() {
        let core = archive_test_core();
        let store = temp_store();
        let qa = vec![0.1f32; 512];
        let qb = vec![0.2f32; 512];
        core.create_ball(&store, "关于 A", &qa, 3.0).await.unwrap();
        core.create_ball(&store, "关于 B", &qb, 3.0).await.unwrap();

        let blocks = core
            .recall_multi(&store, &[&qa, &qb], 1.0)
            .await
            .unwrap();
        assert!(!blocks.is_empty(), "多源回忆应返回注入块");
    }

    #[tokio::test]
    async fn stats_counts_active_and_archived() {
        let core = archive_test_core();
        let store = temp_store();
        let a = core.create_ball(&store, "A", &vec![0.1; 512], 1.0).await.unwrap();
        let b = core.create_ball(&store, "B", &vec![0.2; 512], 1.0).await.unwrap();
        core.set_ball_status(&store, b, BallStatus::Archived).await.unwrap();

        let stats = core.stats(&store).await.unwrap();
        assert_eq!(stats.active_balls, 1);
        assert_eq!(stats.archived_balls, 1);

        assert_eq!(
            core.list_balls(&store, Some(BallStatus::Active)).await.unwrap().len(),
            1
        );
        assert!(core.get_ball(&store, a).await.unwrap().is_some());
    }
}
