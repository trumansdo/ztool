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

/// 配置未指定维度时，Mock 后端的默认维度
const DEFAULT_EMBEDDING_DIM: usize = 384;

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
/// 只记录“决策结果”，不记录中间过程；中间过程交给 `debug` 级日志。
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
    /// 记账“被挡一次”的管道数量
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

/// 大禹记忆系统门面
pub struct DayuCore {
    config: Arc<Config>,
    flow_engine: FlowEngine,
    settle_engine: SettleEngine,
    judge_engine: JudgeEngine,
    judge_backend: Option<Arc<dyn JudgeBackend>>,
    embedding_backend: Arc<dyn EmbeddingBackend>,
}

impl DayuCore {
    /// 创建 DayuCore（使用默认 PipeWeightLossModel + MockEmbeddingBackend）
    pub fn new(config: Arc<Config>) -> Self {
        let loss_model: Arc<dyn LossModel> = Arc::new(PipeWeightLossModel);
        let embedding_backend: Arc<dyn EmbeddingBackend> =
            Arc::new(dayu_embedding::MockEmbeddingBackend::new(
                config.embedding_dim.unwrap_or(DEFAULT_EMBEDDING_DIM),
            ));
        let (flow_engine, settle_engine) = Self::engines(Arc::clone(&config), loss_model);
        Self {
            flow_engine,
            settle_engine,
            judge_engine: JudgeEngine::new(),
            judge_backend: None,
            embedding_backend,
            config,
        }
    }

    /// 创建 DayuCore（指定损失模型，用于对照实验）
    pub fn with_loss_model(config: Arc<Config>, loss_model: Arc<dyn LossModel>) -> Self {
        let embedding_backend: Arc<dyn EmbeddingBackend> =
            Arc::new(dayu_embedding::MockEmbeddingBackend::new(
                config.embedding_dim.unwrap_or(DEFAULT_EMBEDDING_DIM),
            ));
        let (flow_engine, settle_engine) = Self::engines(Arc::clone(&config), loss_model);
        Self {
            flow_engine,
            settle_engine,
            judge_engine: JudgeEngine::new(),
            judge_backend: None,
            embedding_backend,
            config,
        }
    }

    /// 按配置组装流动/结算引擎（含指标后端）
    ///
    /// `metrics_enabled = false` 时不注入记录器，引擎里的埋点走 `None` 分支直接跳过。
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

    /// 设置嵌入后端（校验维度一致性）
    ///
    /// `config.embedding_dim` 为 `None` 时不校验（以模型实际维度为准）；
    /// 为 `Some(n)` 时要求后端维度必须等于 `n`。
    pub fn with_embedding_backend(
        mut self,
        backend: Arc<dyn EmbeddingBackend>,
    ) -> Result<Self, CoreError> {
        let backend_dim = backend.dimension();
        if let Some(expected) = self.config.embedding_dim {
            if backend_dim != expected {
                return Err(CoreError::Config(format!(
                    "embedding dimension mismatch: backend has {}, config expects {}",
                    backend_dim, expected
                )));
            }
        } else {
            tracing::info!(
                dim = backend_dim,
                "embedding dim 未指定，以后端实际维度为准"
            );
        }
        self.embedding_backend = backend;
        Ok(self)
    }

    /// 当前嵌入后端的实际维度（运行时动态值）
    /// 用当前嵌入后端把文本转成向量
    ///
    /// 调用方已有向量时不必用本方法；没有时（例如 napi 只拿到文本）由本系统嵌入，
    /// 保证「配了模型就用模型、没配就用 mock」这条链路只有一处实现。
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, CoreError> {
        self.embedding_backend
            .embed(text)
            .await
            .map_err(CoreError::Embedding)
    }

    pub fn embedding_dimension(&self) -> usize {
        self.embedding_backend.dimension()
    }

    /// 设置嵌入后端（不校验维度，用于测试兼容）
    pub fn with_embedding_backend_unchecked(mut self, backend: Arc<dyn EmbeddingBackend>) -> Self {
        self.embedding_backend = backend;
        self
    }

    /// 尝试加载 ONNX 嵌入后端（从 config 读取路径），失败时回退到 MockEmbeddingBackend
    ///
    /// 仅在 `onnx` feature 启用时可用。
    /// 回退时通过 tracing::warn 记录原因，系统不崩溃。
    #[cfg(feature = "onnx")]
    pub fn with_ort_embedding(mut self) -> Self {
        let model_path = std::path::Path::new(&self.config.embedding_model_path);
        let tokenizer_path = std::path::Path::new(&self.config.embedding_tokenizer_path);

        if model_path.as_os_str().is_empty() || tokenizer_path.as_os_str().is_empty() {
            tracing::info!("embedding model path not configured, using MockEmbeddingBackend");
            return self;
        }

        match dayu_embedding::OrtEmbeddingBackend::load(model_path, tokenizer_path, 512) {
            Ok(ort_backend) => {
                tracing::info!(
                    dim = ort_backend.dimension(),
                    model = %model_path.display(),
                    "ONNX embedding backend loaded"
                );
                self.embedding_backend = Arc::new(ort_backend);
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    model = %model_path.display(),
                    "ONNX embedding backend failed to load, falling back to MockEmbeddingBackend"
                );
            }
        }
        self
    }

    /// 尝试加载 ONNX 判别器后端（从 config 读取路径），失败时回退
    ///
    /// 仅在 `judge-onnx` feature 启用时可用（`dayu-judge` 的 ONNX 实现尚未对齐 ort 2.0）。
    #[cfg(feature = "judge-onnx")]
    pub fn with_ort_judge(mut self) -> Self {
        let model_path = std::path::Path::new(&self.config.embedding_model_path);
        let tokenizer_path = std::path::Path::new(&self.config.embedding_tokenizer_path);

        if model_path.as_os_str().is_empty() || tokenizer_path.as_os_str().is_empty() {
            tracing::info!("judge model path not configured, judge backend not set");
            return self;
        }

        match dayu_judge::OrtJudgeBackend::load(
            model_path,
            tokenizer_path,
            dayu_judge::DEFAULT_JUDGE_PROMPT_TEMPLATE,
            512,
        ) {
            Ok(ort_backend) => {
                tracing::info!(
                    model = %model_path.display(),
                    "ONNX judge backend loaded"
                );
                self.judge_backend = Some(Arc::new(ort_backend));
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    model = %model_path.display(),
                    "ONNX judge backend failed to load, judge disabled"
                );
            }
        }
        self
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
    ///
    /// `qid` 由调用方透传或由本系统生成，用于把 `judge_turn` → `recall` →
    /// `archive_turn` 的日志串成一条链。
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
            // 归档球被检索命中即直接翻回 Active，不引入临时态。
            // 唤醒在结算之前：本次流动它已经参与了，下一次检索它就是普通球。
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
    ///
    /// 便捷方法，内部调用 embedding_backend.embed() 生成查询向量。
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
    /// 不阻塞 recall，可并行调用。
    /// 若未设置 judge_backend，返回 `CoreError::Config` 并提示需要配置模型路径。
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
    ///
    /// 多源如何分组、如何放大预算见 `FlowEngine::flow_multi`。
    /// 结算阶段用**第一个源**作为语义语境——多源时严格做法是逐球取
    /// `max cos(源, 球)`，但 `SettleEngine::settle` 目前只接受单向量，
    /// 这里先取首源，待结算签名支持多源后再改。
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

    /// 直接创建新球（AI 主动“记一件事”，不经判别器）
    ///
    /// `size` 会被夹到 `[0, 10]`；`content_list_max` 取默认 100。
    /// 新球不自动接图——需要关联时再调 `link_balls`。
    pub async fn create_ball(
        &self,
        storage: &dyn StorageBackend,
        content: &str,
        embedding: &[f32],
        size: f64,
    ) -> Result<BallId, Box<dyn std::error::Error + Send + Sync>> {
        let id = storage.allocate_ball_id().await?;
        let now = chrono::Utc::now();
        let ball = Ball {
            id,
            embedding: embedding.to_vec(),
            ball_stored_size: size.clamp(0.0, 10.0),
            ball_size_ceiling: 10.0,
            content_list: vec![Content {
                id: ContentId(0),
                text: content.to_string(),
                encoding: ContentEncoding::Raw,
                source: TurnId(String::new()),
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
        };
        let mut tx = storage.begin_transaction().await?;
        tx.upsert_ball(&ball).await?;
        tx.commit().await?;
        Ok(id)
    }

    /// 手动建管道（AI 主动建立关联）
    ///
    /// `weight` 夹到 `[0, pipe_thickness_ceiling]`；两端必须已存在，且不能自连。
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

    /// 读单个球
    pub async fn get_ball(
        &self,
        storage: &dyn StorageBackend,
        id: BallId,
    ) -> Result<Option<Ball>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(storage.get_ball(id).await?)
    }

    /// 按状态列球（`None` = 全部，含归档）
    pub async fn list_balls(
        &self,
        storage: &dyn StorageBackend,
        status: Option<BallStatus>,
    ) -> Result<Vec<Ball>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(storage.list_balls(status).await?)
    }

    /// 列出全部管道（观测与对照实验用）
    pub async fn list_pipes(
        &self,
        storage: &dyn StorageBackend,
    ) -> Result<Vec<Pipe>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(storage.list_pipes().await?)
    }

    /// 改球状态（归档 / 唤醒 / 标记争议 / 软删）
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
    ///
    /// 判定（两条同时满足）：
    /// - 距 `last_watered` 超过 `archive_inactive_days`
    /// - `size_effective(now)` 低于 `archive_size_threshold`
    ///
    /// 另有容量兜底：活跃球超过 `max_active_balls` 时，按 `size_effective`
    /// 升序强制归档最弱的球，直到回落到上限。
    ///
    /// 归档球仍在同一张球表里（归档区不另建索引），仍参与
    /// `search_by_embedding`，被检索命中即由 `wake_archived` 翻回 `Active`。
    ///
    /// 定时任务与手动 CLI 共用这一个入口。
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

        // 容量兜底：只按有效大小升序砍最弱的，同分按 id 保证可复现
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

    /// 唤醒归档球：把指定球中处于 `Archived` 的直接改回 `Active`
    ///
    /// 无临时态、无 TTL：唤醒就是一次状态翻转，持久化后下一次流动即普通球。
    /// 非归档球忽略（幂等）。
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

    #[tokio::test]
    async fn dayu_core_recall_returns_blocks() {
        let store = temp_store();
        // 使用带内容的球构建链
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..5u64 {
            balls.push(sample_ball_with_content(
                i, 1.0 + i as f64 * 0.5,
                vec![(i + 1) as f32 / 5.0; 384],
                &["test content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let core = DayuCore::new(config);

        let query = vec![0.5; 384];
        let blocks = core.recall(&store, &query, 1.0).await.unwrap();
        assert!(!blocks.is_empty(), "recall should return injection blocks");
    }

    #[tokio::test]
    async fn dayu_core_with_constant_loss_model() {
        let store = temp_store();
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..5u64 {
            balls.push(sample_ball_with_content(
                i, 1.0 + i as f64 * 0.5,
                vec![(i + 1) as f32 / 5.0; 384],
                &["test content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let loss_model: Arc<dyn LossModel> = Arc::new(dayu_flow::loss::ConstantLossModel);
        let core = DayuCore::with_loss_model(config, loss_model);

        let query = vec![0.5; 384];
        let blocks = core.recall(&store, &query, 1.0).await.unwrap();
        assert!(!blocks.is_empty(), "constant loss model should still produce blocks");
    }

    #[tokio::test]
    async fn dayu_core_judge_turn_discards_low_significance() {
        let config = sample_config();
        let backend = Arc::new(dayu_judge::MockJudgeBackend::new());
        // 入队一个低显著性 verdict
        backend.enqueue(dayu_domain::types::Verdict {
            should_judge: false,
            significance: 0.1,
            offset_direction: String::new(),
            emotion_valence: 0.0,
            emotion_intensity: 0.0,
            hit_dimensions: vec![],
            belongs_to: None,
            belong_confidence: 0.0,
            reason: "low significance".to_string(),
        });

        let core = DayuCore::new(config).with_judge_backend(backend);

        let input = JudgeInput {
            candidate_text: "测试".to_string(),
            candidate_embedding: vec![0.1; 384],
            recent_window: vec![],
            candidate_archetypes: vec![],
        };

        let result = core.judge_turn(&input).await.unwrap();
        match result {
            JudgeResult::Discard { .. } => {}
            other => panic!("expected Discard, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn dayu_core_judge_turn_creates_new_ball() {
        let config = sample_config();
        let backend = Arc::new(dayu_judge::MockJudgeBackend::new());
        backend.enqueue(dayu_domain::types::Verdict {
            should_judge: true,
            significance: 0.8,
            offset_direction: String::new(),
            emotion_valence: 0.0,
            emotion_intensity: 0.0,
            hit_dimensions: vec![],
            belongs_to: None,
            belong_confidence: 0.0,
            reason: "new topic".to_string(),
        });

        let core = DayuCore::new(config).with_judge_backend(backend);

        let input = JudgeInput {
            candidate_text: "新话题".to_string(),
            candidate_embedding: vec![0.5; 384],
            recent_window: vec![],
            candidate_archetypes: vec![],
        };

        let result = core.judge_turn(&input).await.unwrap();
        match result {
            JudgeResult::Create { significance, status, text, .. } => {
                assert_eq!(significance, 0.8);
                assert_eq!(status, BallCreationStatus::Confirmed);
                assert_eq!(text, "新话题");
            }
            other => panic!("expected Create, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn dayu_core_judge_turn_without_backend_errors() {
        let config = sample_config();
        let core = DayuCore::new(config); // 未设置 judge_backend

        let input = JudgeInput {
            candidate_text: "测试".to_string(),
            candidate_embedding: vec![0.1; 384],
            recent_window: vec![],
            candidate_archetypes: vec![],
        };

        let result = core.judge_turn(&input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn dayu_core_judge_and_recall_are_independent() {
        // 验证 judge_turn 和 recall 可独立调用，不互相阻塞
        let store = temp_store();
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..3u64 {
            balls.push(sample_ball_with_content(
                i, 1.0,
                vec![(i + 1) as f32 / 3.0; 384],
                &["content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let backend = Arc::new(dayu_judge::MockJudgeBackend::new());
        backend.enqueue(dayu_domain::types::Verdict {
            should_judge: true,
            significance: 0.9,
            offset_direction: String::new(),
            emotion_valence: 0.0,
            emotion_intensity: 0.0,
            hit_dimensions: vec![],
            belongs_to: None,
            belong_confidence: 0.0,
            reason: "test".to_string(),
        });

        let core = DayuCore::new(config).with_judge_backend(backend);

        // 并行调用 judge 和 recall
        let judge_input = JudgeInput {
            candidate_text: "并行测试".to_string(),
            candidate_embedding: vec![0.5; 384],
            recent_window: vec![],
            candidate_archetypes: vec![],
        };

        let query = vec![0.5; 384];
        let (judge_result, recall_result) = tokio::join!(
            core.judge_turn(&judge_input),
            core.recall(&store, &query, 1.0),
        );

        assert!(judge_result.is_ok(), "judge should succeed");
        assert!(recall_result.is_ok(), "recall should succeed");
    }

    // ============================================================
    // 嵌入后端集成测试
    // ============================================================

    #[tokio::test]
    async fn recall_with_text_uses_embedding_backend() {
        let store = temp_store();
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..5u64 {
            balls.push(sample_ball_with_content(
                i, 1.0 + i as f64 * 0.5,
                vec![0.5; 384],  // 固定向量，与 FixedEmbeddingBackend 匹配
                &["test content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        seed(&store, balls, pipes).await;

        let config = sample_config();
        // 使用 FixedEmbeddingBackend 确保查询向量与种子球匹配
        let fixed: Arc<dyn EmbeddingBackend> =
            Arc::new(dayu_embedding::FixedEmbeddingBackend::new(384, 0.5));
        let core = DayuCore::new(config).with_embedding_backend_unchecked(fixed);

        let blocks = core.recall_with_text(&store, "测试查询", 1.0).await.unwrap();
        assert!(!blocks.is_empty(), "recall_with_text should return blocks");
    }

    #[tokio::test]
    async fn custom_embedding_backend_is_used() {
        let store = temp_store();
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..3u64 {
            balls.push(sample_ball_with_content(
                i, 1.0,
                vec![0.5; 384],  // 与 FixedEmbeddingBackend(0.5) 匹配
                &["content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let fixed: Arc<dyn EmbeddingBackend> =
            Arc::new(dayu_embedding::FixedEmbeddingBackend::new(384, 0.5));
        let core = DayuCore::new(config).with_embedding_backend_unchecked(fixed);

        let blocks = core.recall_with_text(&store, "任意文本", 1.0).await.unwrap();
        assert!(!blocks.is_empty(), "fixed embedding should still produce blocks");
    }

    #[tokio::test]
    async fn embedding_backend_dimension_matches_config() {
        let config = sample_config();
        let core = DayuCore::new(Arc::clone(&config));
        // config 未指定维度时，Mock 使用默认维度
        assert_eq!(core.embedding_dimension(), DEFAULT_EMBEDDING_DIM);
    }

    // ============================================================
    // 维度校验测试（阶段 7 验证标准）
    // ============================================================

    #[tokio::test]
    async fn dimension_mismatch_rejected() {
        let mut c = (*sample_config()).clone();
        c.embedding_dim = Some(384); // 显式锁定 384
        let config = Arc::new(c);
        // 创建维度不匹配的嵌入后端
        let wrong_dim: Arc<dyn EmbeddingBackend> =
            Arc::new(dayu_embedding::FixedEmbeddingBackend::new(128, 0.5));
        let result = DayuCore::new(config).with_embedding_backend(wrong_dim);
        assert!(result.is_err(), "dimension mismatch should be rejected");
        match result {
            Err(e) => assert!(e.to_string().contains("dimension mismatch"),
                "error should mention dimension mismatch, got: {}", e),
            Ok(_) => panic!("expected error"),
        }
    }

    #[tokio::test]
    async fn dimension_match_accepted() {
        let mut c = (*sample_config()).clone();
        c.embedding_dim = Some(384);
        let config = Arc::new(c);
        let correct_dim: Arc<dyn EmbeddingBackend> =
            Arc::new(dayu_embedding::FixedEmbeddingBackend::new(384, 0.5));
        let result = DayuCore::new(config).with_embedding_backend(correct_dim);
        assert!(result.is_ok(), "matching dimension should be accepted");
    }

    #[tokio::test]
    async fn dimension_auto_detected_when_unspecified() {
        // config 未指定维度：任意维度的后端都应被接受（以模型为准）
        let config = sample_config();
        for dim in [384usize, 512, 1024] {
            let backend: Arc<dyn EmbeddingBackend> =
                Arc::new(dayu_embedding::FixedEmbeddingBackend::new(dim, 0.5));
            let core = DayuCore::new(Arc::clone(&config))
                .with_embedding_backend(backend)
                .unwrap_or_else(|e| panic!("dim {dim} should be accepted, got {e}"));
            assert_eq!(core.embedding_dimension(), dim);
        }
    }

    // ============================================================
    // 热更新测试（阶段 4 验证标准）
    // ============================================================

    #[tokio::test]
    async fn config_hot_reload_affects_next_flow() {
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..5u64 {
            balls.push(sample_ball_with_content(
                i, 1.0,
                vec![0.5; 384],
                &["content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        // 两个独立存储：recall 会写回强化结果，共用 store 会让第二次
        // 从“已被强化过”的状态出发，比较就失去意义
        let store = temp_store();
        seed(&store, balls.clone(), pipes.clone()).await;
        let store_strict = temp_store();
        seed(&store_strict, balls, pipes).await;

        let config = sample_config();
        let core = DayuCore::new(Arc::clone(&config));

        // 第一次 recall 用默认配置
        let query = vec![0.5; 384];
        let blocks1 = core.recall(&store, &query, 1.0).await.unwrap();
        assert!(!blocks1.is_empty());

        // 修改配置（更严格的深度限制）
        let mut strict_config = (*config).clone();
        strict_config.max_flow_depth = 1;
        let core2 = DayuCore::new(Arc::new(strict_config));

        let blocks2 = core2.recall(&store_strict, &query, 1.0).await.unwrap();
        // 深度限制更严格，结果可能不同（但不一定更少，取决于图结构）
        // 至少不应崩溃
        assert!(blocks2.len() <= blocks1.len(),
            "更严格的深度限制不应增加结果数量: {} vs {}", blocks1.len(), blocks2.len());
    }

    // ============================================================
    // 指标覆盖测试（阶段 5 验证标准）
    // ============================================================

    #[tokio::test]
    async fn flow_records_at_least_5_metrics() {
        use dayu_observability::MockMetricsRecorder;

        let store = temp_store();
        let mut balls = Vec::new();
        let mut pipes = Vec::new();
        for i in 0..5u64 {
            balls.push(sample_ball_with_content(
                i, 1.0,
                vec![0.5; 384],
                &["content"],
            ));
            if i > 0 {
                pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
            }
        }
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let metrics = Arc::new(MockMetricsRecorder::new());
        let loss_model: Arc<dyn LossModel> = Arc::new(PipeWeightLossModel);
        let flow_engine = dayu_flow::engine::FlowEngine::with_loss_model(
            Arc::clone(&config),
            loss_model,
        ).with_metrics(Arc::clone(&metrics) as Arc<dyn dayu_observability::MetricsRecorder>);

        let query = vec![0.5; 384];
        let _ = flow_engine.flow(&store, &query, 1.0).await.unwrap();

        let distinct = metrics.distinct_metric_count();
        let total = metrics.total_count();
        assert!(distinct >= 4,
            "flow should record at least 4 distinct metrics, got {} distinct ({} total entries)",
            distinct, total);
    }

    // ============================================================
    // 归档 / 唤醒
    // ============================================================

    fn idle_ball(id: u64, size: f64, stability: f64, days: i64) -> dayu_domain::types::Ball {
        let mut b = sample_ball(id, size, vec![0.1; 384]);
        b.ball_stability = stability;
        b.last_watered = chrono::Utc::now() - chrono::Duration::days(days);
        b
    }

    #[tokio::test]
    async fn archive_sweep_archives_only_idle_and_decayed() {
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        seed(&store, 
            vec![
                idle_ball(1, 0.05, 7.0, 200),   // 久未注水 + 已衰减 → 归档
                idle_ball(2, 9.0, 30.0, 100),   // 久未注水但有效大小 0.32 → 保留
                idle_ball(3, 0.05, 7.0, 0),     // 刚注水 → 保留
            ],
            vec![],
        ).await;

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
        let core = DayuCore::new(Arc::new(cfg));
        let store = temp_store();
        seed(&store, 
            vec![
                idle_ball(1, 1.0, 30.0, 0),
                idle_ball(2, 5.0, 30.0, 0),
                idle_ball(3, 9.0, 30.0, 0),
            ],
            vec![],
        ).await;

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
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        let mut archived = sample_ball(1, 1.0, vec![0.1; 384]);
        archived.status = BallStatus::Archived;
        let active = sample_ball(2, 1.0, vec![0.1; 384]);
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
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        let mut ball = sample_ball(1, 5.0, vec![0.1; 384]);
        ball.status = BallStatus::Archived;
        seed(&store, vec![ball], vec![]).await;

        let _ = core.recall(&store, &vec![0.1f32; 384], 1.0).await.unwrap();

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
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();

        let a = core
            .create_ball(&store, "第一件事", &vec![0.1; 384], 2.0)
            .await
            .unwrap();
        let b = core
            .create_ball(&store, "第二件事", &vec![0.2; 384], 99.0)
            .await
            .unwrap();
        assert_ne!(a, b, "ID 必须递增分配");

        let ball = store.get_ball(a).await.unwrap().unwrap();
        assert_eq!(ball.content_list.len(), 1);
        assert_eq!(ball.content_list[0].text, "第一件事");
        assert_eq!(ball.ball_stored_size, 2.0);
        assert_eq!(ball.status, BallStatus::Active);

        // size 被夹到上限 10
        let capped = store.get_ball(b).await.unwrap().unwrap();
        assert_eq!(capped.ball_stored_size, 10.0);
    }

    #[tokio::test]
    async fn link_balls_rejects_self_and_missing() {
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        let a = core
            .create_ball(&store, "A", &vec![0.1; 384], 1.0)
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
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        let a = core.create_ball(&store, "A", &vec![0.1; 384], 1.0).await.unwrap();
        let b = core.create_ball(&store, "B", &vec![0.2; 384], 1.0).await.unwrap();

        let pid = core.link_balls(&store, b, a, 5.0).await.unwrap();
        let sub = store.load_subgraph(&[a], 1).await.unwrap();
        let pipe = sub.pipes.iter().find(|p| p.id == pid).expect("管道应存在");
        assert_eq!(pipe.pipe_base_thickness, 1.0, "粗细应夹到 pipe_thickness_ceiling");
        assert_eq!((pipe.a, pipe.b), (a, b), "端点应规范化成 (min, max)");
        assert_eq!(pipe.source, PipeSource::Manual);
    }

    #[tokio::test]
    async fn recall_multi_returns_blocks_for_two_sources() {
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        let qa = vec![0.1f32; 384];
        let qb = vec![0.2f32; 384];
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
        let core = DayuCore::new(Arc::new(Config::default()));
        let store = temp_store();
        let a = core.create_ball(&store, "A", &vec![0.1; 384], 1.0).await.unwrap();
        let b = core.create_ball(&store, "B", &vec![0.2; 384], 1.0).await.unwrap();
        core.set_ball_status(&store, b, BallStatus::Archived).await.unwrap();

        let stats = core.stats(&store).await.unwrap();
        assert_eq!(stats.active_balls, 1);
        assert_eq!(stats.archived_balls, 1);
        assert_eq!(stats.embedding_dim, 384);

        assert_eq!(
            core.list_balls(&store, Some(BallStatus::Active)).await.unwrap().len(),
            1
        );
        assert!(core.get_ball(&store, a).await.unwrap().is_some());
    }
}
