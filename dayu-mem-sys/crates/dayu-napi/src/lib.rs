//! dayu-napi: napi-rs 绑定，只做 DTO 翻译
//!
//! 边界层：JS 友好 DTO ↔ 领域类型。
//! 导出用例级 API：init / recall / judge_turn / archive_turn / correct_memory / get_stats。
//!
//! 错误映射：CoreError → 带 `[CODE]` 前缀的 napi::Error。

use napi_derive::napi;
use napi::{Error, Status};
use std::sync::Arc;
use dayu_core::DayuCore;
use dayu_domain::types::*;
use dayu_storage::TriviumStorage;
use tracing::Instrument;

// ============================================================
// 全局状态（单例）
// ============================================================

use std::sync::Mutex;
static CORE: once_cell::sync::Lazy<Mutex<Option<Arc<CoreState>>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

struct CoreState {
    core: DayuCore,
    storage: TriviumStorage,
    /// 日志/指标写入守卫，必须与状态同生命周期（提前 drop 会丢尾部日志）
    _observability: Option<dayu_observability::ObservabilityGuard>,
}

/// 取出状态快照（Arc）
///
/// 不跨 `await` 持锁：`MutexGuard` 不是 `Send`，且 `Handle::block_on` 在
/// runtime 线程内会 panic。所有 async API 先 `state()?` 拿到 Arc，再 await。
fn state() -> napi::Result<Arc<CoreState>> {
    let guard = CORE
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("[CORE_LOCK] {e}")))?;
    guard.clone().ok_or_else(|| {
        Error::new(
            Status::GenericFailure,
            "[CORE_NOT_INIT] DayuCore not initialized, call init() first",
        )
    })
}

// ============================================================
// DTO 类型（JS 友好）
// ============================================================

/// 注入块 DTO
#[napi(object)]
#[derive(Clone)]
pub struct InjectionBlockDto {
    pub confidence: String,
    pub text: String,
    pub ball_ids: Vec<u32>,
    pub token_count: u32,
}

/// 回忆结果 DTO
///
/// `qid` 是本轮查询标识：调用方传入则原值返回，未传则由本系统生成。
/// 透传到 `judgeTurn` / `archiveTurn` 即可在日志里串成一条链。
#[napi(object)]
#[derive(Clone)]
pub struct RecallResultDto {
    pub qid: String,
    pub blocks: Vec<InjectionBlockDto>,
    /// 本轮流动与结算的统计量
    ///
    /// 冒烟与对照实验靠它判断「水走了多远、被挡了几条、强化了几个」，
    /// 不必再去解析日志或指标文件。
    pub stats: RecallStatsDto,
}

/// 单轮回忆的统计量
#[napi(object)]
#[derive(Clone)]
pub struct RecallStatsDto {
    /// 起始球数量
    pub start_count: u32,
    /// 被点亮的球数量
    pub lit_count: u32,
    /// 拿到路径强化的球数量
    pub strengthened_count: u32,
    /// 只拿到最低强化值的印象球数量
    pub impression_count: u32,
    /// 生成的注入块数量
    pub block_count: u32,
    /// 注入的总 token 数
    pub token_count: u32,
    /// 被阻力挡住的管道数量
    pub blocked_pipe_count: u32,
    /// 本轮耗时（毫秒）
    pub elapsed_ms: u32,
}

impl From<dayu_core::RecallStats> for RecallStatsDto {
    fn from(s: dayu_core::RecallStats) -> Self {
        Self {
            start_count: s.start_count as u32,
            lit_count: s.lit_count as u32,
            strengthened_count: s.strengthened_count as u32,
            impression_count: s.impression_count as u32,
            block_count: s.block_count as u32,
            token_count: s.token_count as u32,
            blocked_pipe_count: s.blocked_pipe_count as u32,
            elapsed_ms: s.elapsed_ms as u32,
        }
    }
}

/// 判别结果 DTO
#[napi(object)]
#[derive(Clone)]
pub struct JudgeResultDto {
    pub kind: String,
    pub significance: f64,
    pub ball_id: Option<u32>,
    pub reason: String,
}

/// 球 DTO
#[napi(object)]
#[derive(Clone)]
pub struct BallDto {
    pub id: u32,
    pub size: f64,
    pub size_effective: f64,
    pub stability: f64,
    pub status: String,
    pub text: String,
    pub lit_count: f64,
    pub last_watered: String,
}

/// 管道 DTO
#[napi(object)]
#[derive(Clone)]
pub struct PipeDto {
    pub id: u32,
    pub a: u32,
    pub b: u32,
    /// 管道粗细（A↔B 共享）
    pub thickness: f64,
    /// 管道稳定度（天）
    pub stability: f64,
    /// 被阻挡次数（结算时累加，随时间衰减）
    pub blocked_count: f64,
}

/// 统计 DTO
#[napi(object)]
#[derive(Clone)]
pub struct StatsDto {
    pub active_ball_count: u32,
    pub archived_ball_count: u32,
    pub embedding_dim: u32,
}

/// 归档扫描结果 DTO
#[napi(object)]
#[derive(Clone)]
pub struct ArchiveReportDto {
    pub archived: u32,
    pub woken: u32,
}

// ============================================================
// 内部工具
// ============================================================

fn parse_status(s: &str) -> napi::Result<BallStatus> {
    match s {
        "active" => Ok(BallStatus::Active),
        "contested" => Ok(BallStatus::Contested),
        "archived" => Ok(BallStatus::Archived),
        "deleted" => Ok(BallStatus::Deleted),
        other => Err(Error::new(
            Status::InvalidArg,
            format!("[STATUS] 未知状态: {other}"),
        )),
    }
}

fn to_ball_dto(b: &Ball) -> BallDto {
    let now = chrono::Utc::now();
    let text = b
        .content_list
        .iter()
        .find(|c| c.status == ContentStatus::Active)
        .map(|c| c.text.clone())
        .unwrap_or_default();
    BallDto {
        id: b.id.0 as u32,
        size: b.ball_stored_size,
        size_effective: b.size_effective(now),
        stability: b.ball_stability,
        status: b.status.as_str().to_string(),
        text,
        lit_count: b.lit_count as f64,
        last_watered: b.last_watered.to_rfc3339(),
    }
}

fn to_block_dto(b: InjectionBlock) -> InjectionBlockDto {
    InjectionBlockDto {
        confidence: b.confidence.as_str().to_string(),
        text: b.text,
        ball_ids: b.ball_ids.iter().map(|id| id.0 as u32).collect(),
        token_count: b.token_count as u32,
    }
}

// ============================================================
// 导出 API
// ============================================================

/// 初始化 DayuCore
///
/// config_toml: TOML 格式配置字符串
#[napi]
pub fn init(config_toml: String, db_path: Option<String>) -> napi::Result<()> {
    let config: Config = toml::from_str(&config_toml)
        .map_err(|e| Error::new(Status::InvalidArg, format!("[CONFIG_PARSE] {e}")))?;

    // 优先级：显式参数 > 配置里的 db_path > 默认文件名
    let path = db_path
        .or_else(|| config.db_path.clone())
        .unwrap_or_else(|| "dayu-mem.tdb".to_string());

    let config = Arc::new(config);

    // 嵌入后端由 `config.embedding_backend` 决定（onnx / http / test）。
    // 不再有静默回退：选中什么就必须可用，否则 init 直接失败。
    // init 是同步导出函数，而 http 后端探测维度是异步的，这里开一个临时 runtime 跑一次。
    let dcore = {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::new(Status::GenericFailure, format!("[RUNTIME] {e}")))?;
        let backend = rt
            .block_on(dayu_core::load_embedding_backend(&config))
            .map_err(|e| Error::new(Status::GenericFailure, format!("[EMBEDDING_INIT] {e}")))?;
        DayuCore::new(Arc::clone(&config), backend)
            .map_err(|e| Error::new(Status::GenericFailure, format!("[CORE_INIT] {e}")))?
    };

    // 日志初始化失败不阻断主流程：可观测性是增强，不是前提。
    // AlreadyInitialized 说明已有 subscriber（例如重复调 init），直接沿用。
    let observability = dayu_observability::init(
        &config.log_dir,
        &config.log_level,
        config.log_retention_days,
        config.metrics_enabled,
        config.metrics_retention_days,
    )
    .map(Some)
    .unwrap_or_else(|e| {
        eprintln!("[dayu] 日志初始化跳过: {e}");
        None
    });

    // 存储维度跟随后端实际维度：模型 512 就开 512 的库，
    // 库已存在且维度不同时由 TriviumDB 拒绝打开（防止混用两套向量）。
    let dim = dcore.embedding_dimension();
    let storage = TriviumStorage::with_dim(&path, dim)
        .map_err(|e| Error::new(Status::GenericFailure, format!("[STORAGE_OPEN] {e}")))?;

    let mut guard = CORE
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("[CORE_LOCK] {e}")))?;
    *guard = Some(Arc::new(CoreState { core: dcore, storage, _observability: observability }));
    Ok(())
}

/// 关闭当前实例：释放存储与嵌入后端，允许再次 `init`
///
/// 用途：同一进程内跑多组对照实验（不同配置各开一个库）。
/// 不调用也能重新 `init`，但旧实例的存储句柄与 ONNX 会话会一直占着。
#[napi]
pub fn shutdown() -> napi::Result<()> {
    let mut guard = CORE
        .lock()
        .map_err(|e| Error::new(Status::GenericFailure, format!("[CORE_LOCK] {e}")))?;
    *guard = None;
    Ok(())
}

/// 回忆：文本查询 → 注入块列表
///
/// `qid` 省略时由本系统生成，并在返回值里带回。
#[napi]
pub async fn recall(
    query_text: String,
    significance: f64,
    qid: Option<String>,
) -> napi::Result<RecallResultDto> {
    let st = state()?;
    let qid = QueryId::from_input(qid.as_deref());
    let output = st
        .core
        .recall_with_text_qid(&st.storage, &query_text, significance, qid)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[RECALL] {e}")))?;
    Ok(RecallResultDto {
        qid: output.qid.0,
        blocks: output.injection_blocks.into_iter().map(to_block_dto).collect(),
        stats: output.stats.into(),
    })
}

/// 回忆（直接传嵌入）：调用方已有向量时省去一次模型调用
///
/// 与 `recall` 的区别只在入口：`recall` 接受文本并内部嵌入，
/// 本接口接受已算好的向量（插件侧通常已为 `judgeTurn` 算过）。
#[napi]
pub async fn recall_by_embedding(
    embedding: Vec<f64>,
    significance: f64,
    qid: Option<String>,
) -> napi::Result<RecallResultDto> {
    let st = state()?;
    let qid = QueryId::from_input(qid.as_deref());
    let emb: Vec<f32> = embedding.into_iter().map(|x| x as f32).collect();
    let output = st
        .core
        .recall_with_qid(&st.storage, &emb, significance, qid)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[RECALL_EMB] {e}")))?;
    Ok(RecallResultDto {
        qid: output.qid.0,
        blocks: output.injection_blocks.into_iter().map(to_block_dto).collect(),
        stats: output.stats.into(),
    })
}

/// 多输入源回忆：用户消息 / 工具结果 / 模型输出各自产生的嵌入一起注水
#[napi]
pub async fn recall_multi(
    query_embeddings: Vec<Vec<f64>>,
    significance: f64,
    qid: Option<String>,
) -> napi::Result<RecallResultDto> {
    let st = state()?;
    let qid = QueryId::from_input(qid.as_deref());
    let owned: Vec<Vec<f32>> = query_embeddings
        .into_iter()
        .map(|v| v.into_iter().map(|x| x as f32).collect())
        .collect();
    let refs: Vec<&[f32]> = owned.iter().map(|v| v.as_slice()).collect();
    let output = st
        .core
        .recall_multi_with_qid(&st.storage, &refs, significance, qid)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[RECALL_MULTI] {e}")))?;
    Ok(RecallResultDto {
        qid: output.qid.0,
        blocks: output.injection_blocks.into_iter().map(to_block_dto).collect(),
        stats: output.stats.into(),
    })
}

/// 多输入源回忆（文本版）：每个文本各自嵌入后一起注水
///
/// 与 `recallMulti` 的区别只在入口：`recallMulti` 接受已算好的向量，
/// 本接口接受文本，嵌入交给本系统（配了模型走模型，没配走 mock）。
#[napi]
pub async fn recall_multi_by_text(
    query_texts: Vec<String>,
    significance: f64,
    qid: Option<String>,
) -> napi::Result<RecallResultDto> {
    let st = state()?;
    let qid = QueryId::from_input(qid.as_deref());
    let span = tracing::info_span!("recall_multi", qid = %qid, sources = query_texts.len());
    let mut owned: Vec<Vec<f32>> = Vec::with_capacity(query_texts.len());
    for text in &query_texts {
        let emb = st
            .core
            .embed_text(text)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("[EMBED] {e}")))?;
        owned.push(emb);
    }
    let refs: Vec<&[f32]> = owned.iter().map(|v| v.as_slice()).collect();
    let output = st
        .core
        .recall_multi_with_qid(&st.storage, &refs, significance, qid)
        .instrument(span)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[RECALL_MULTI] {e}")))?;
    Ok(RecallResultDto {
        qid: output.qid.0,
        blocks: output.injection_blocks.into_iter().map(to_block_dto).collect(),
        stats: output.stats.into(),
    })
}

/// 判别轮次：判断候选片段是否产生新球
#[napi]
pub async fn judge_turn(
    candidate_text: String,
    candidate_embedding: Vec<f64>,
    recent_window: Vec<String>,
    qid: Option<String>,
) -> napi::Result<JudgeResultDto> {
    let st = state()?;
    let qid = QueryId::from_input(qid.as_deref());
    let span = tracing::info_span!("judge_turn", qid = %qid);
    let input = JudgeInput {
        candidate_text,
        candidate_embedding: candidate_embedding.into_iter().map(|x| x as f32).collect(),
        recent_window,
        candidate_archetypes: vec![],
    };
    let result = st
        .core
        .judge_turn(&input)
        .instrument(span)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[JUDGE] {e}")))?;

    Ok(match result {
        JudgeResult::Discard { reason } => JudgeResultDto {
            kind: "discard".to_string(),
            significance: 0.0,
            ball_id: None,
            reason,
        },
        JudgeResult::Create {
            significance, reason, ..
        } => JudgeResultDto {
            kind: "create".to_string(),
            significance,
            ball_id: None,
            reason,
        },
        JudgeResult::Attach {
            ball_id,
            confidence,
            significance,
        } => JudgeResultDto {
            kind: "attach".to_string(),
            significance,
            ball_id: Some(ball_id.0 as u32),
            reason: format!("attached with confidence {confidence}"),
        },
    })
}

/// 直接创建新球（AI 主动“记一件事”，不经判别器）
#[napi]
pub async fn create_ball(
    content: String,
    embedding: Option<Vec<f64>>,
    size: f64,
    qid: Option<String>,
) -> napi::Result<u32> {
    let st = state()?;
    let qid = QueryId::from_input(qid.as_deref());
    let span = tracing::info_span!("create_ball", qid = %qid, size);
    // 传了向量就用，没传就由本系统嵌入（配了模型走模型，没配走 mock）
    let emb: Vec<f32> = match embedding {
        Some(v) => v.into_iter().map(|x| x as f32).collect(),
        None => st
            .core
            .embed_text(&content)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("[EMBED] {e}")))?,
    };
    let id = st
        .core
        .create_ball(&st.storage, &content, &emb, size)
        .instrument(span)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[CREATE_BALL] {e}")))?;
    Ok(id.0 as u32)
}

/// 手动建管道（AI 主动建立关联）
#[napi]
pub async fn link_balls(a: u32, b: u32, weight: f64) -> napi::Result<u32> {
    let st = state()?;
    let id = st
        .core
        .link_balls(&st.storage, BallId(a as u64), BallId(b as u64), weight)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[LINK_BALLS] {e}")))?;
    Ok(id.0 as u32)
}

/// 批量建管道：三条平行数组（起点、终点、粗细）
///
/// 规模冒烟时逐条 `linkBalls` 太慢（每条一个事务），这里一次事务写入整批。
/// 自连或端点不存在的边会被跳过，返回实际写入数。
#[napi]
pub async fn link_balls_batch(
    a: Vec<u32>,
    b: Vec<u32>,
    weight: Vec<f64>,
) -> napi::Result<u32> {
    let st = state()?;
    if a.len() != b.len() || a.len() != weight.len() {
        return Err(Error::new(
            Status::InvalidArg,
            "[LINK_BATCH] a / b / weight 长度必须一致",
        ));
    }
    let edges: Vec<(BallId, BallId, f64)> = a
        .into_iter()
        .zip(b)
        .zip(weight)
        .map(|((x, y), w)| (BallId(x as u64), BallId(y as u64), w))
        .collect();
    let written = st
        .core
        .link_balls_batch(&st.storage, &edges)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[LINK_BATCH] {e}")))?;
    Ok(written as u32)
}

/// 批量建球：三条平行数组（内容、嵌入、初始 size）
///
/// 一个事务写入整批，适合规模冒烟与批量导入。
#[napi]
pub async fn create_balls_batch(
    contents: Vec<String>,
    embeddings: Vec<Vec<f64>>,
    sizes: Vec<f64>,
) -> napi::Result<Vec<u32>> {
    let st = state()?;
    if contents.len() != embeddings.len() || contents.len() != sizes.len() {
        return Err(Error::new(
            Status::InvalidArg,
            "[CREATE_BATCH] contents / embeddings / sizes 长度必须一致",
        ));
    }
    let items: Vec<(String, Vec<f32>, f64)> = contents
        .into_iter()
        .zip(embeddings)
        .zip(sizes)
        .map(|((text, emb), size)| {
            (text, emb.into_iter().map(|x| x as f32).collect(), size)
        })
        .collect();
    let ids = st
        .core
        .create_balls_batch(&st.storage, &items)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[CREATE_BATCH] {e}")))?;
    Ok(ids.into_iter().map(|id| id.0 as u32).collect())
}

/// 批量嵌入：一次 HTTP 请求送多条文本，显著降低远程调用开销
#[napi]
pub async fn embed_batch(texts: Vec<String>) -> napi::Result<Vec<Vec<f64>>> {
    let st = state()?;
    let vectors = st
        .core
        .embed_batch(&texts)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[EMBED_BATCH] {e}")))?;
    Ok(vectors
        .into_iter()
        .map(|v| v.into_iter().map(|x| x as f64).collect())
        .collect())
}

/// 读单个球
#[napi]
pub async fn get_ball(id: u32) -> napi::Result<Option<BallDto>> {
    let st = state()?;
    let ball = st
        .core
        .get_ball(&st.storage, BallId(id as u64))
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[GET_BALL] {e}")))?;
    Ok(ball.as_ref().map(to_ball_dto))
}

/// 按状态列球（status 省略 = 全部，含归档）
#[napi]
pub async fn list_balls(status: Option<String>) -> napi::Result<Vec<BallDto>> {
    let st = state()?;
    let filter = status.as_deref().map(parse_status).transpose()?;
    let balls = st
        .core
        .list_balls(&st.storage, filter)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[LIST_BALLS] {e}")))?;
    Ok(balls.iter().map(to_ball_dto).collect())
}

/// 列出全部管道
///
/// 管道粗细是否随「被走通」而增长，是「联想被强化」最直接的证据。
#[napi]
pub async fn list_pipes() -> napi::Result<Vec<PipeDto>> {
    let st = state()?;
    let pipes = st
        .core
        .list_pipes(&st.storage)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[LIST_PIPES] {e}")))?;
    Ok(pipes
        .iter()
        .map(|p| PipeDto {
            id: p.id.0 as u32,
            a: p.a.0 as u32,
            b: p.b.0 as u32,
            thickness: p.pipe_base_thickness,
            stability: p.pipe_stability,
            blocked_count: p.blocked_pass_count_stored as f64,
        })
        .collect())
}

/// 改球状态：active / contested / archived / deleted
#[napi]
pub async fn set_ball_status(id: u32, status: String) -> napi::Result<()> {
    let st = state()?;
    let target = parse_status(&status)?;
    st.core
        .set_ball_status(&st.storage, BallId(id as u64), target)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[SET_STATUS] {e}")))?;
    Ok(())
}

/// 归档扫描：定时任务与手动调用共用
///
/// `qid` 省略时按 `sweep-{日期}` 生成，便于把一轮扫描的所有日志归组。
#[napi]
pub async fn archive_sweep(
    inactive_days: Option<f64>,
    max_balls: Option<u32>,
    qid: Option<String>,
) -> napi::Result<ArchiveReportDto> {
    let st = state()?;
    let qid = qid
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("sweep-{}", chrono::Utc::now().format("%Y%m%d")));
    let span = tracing::info_span!("archive_sweep", qid = %qid);
    let report = async {
        if inactive_days.is_some() || max_balls.is_some() {
            let mut cfg = (**st.core.config()).clone();
            if let Some(d) = inactive_days {
                cfg.archive_inactive_days = d;
            }
            if let Some(n) = max_balls {
                cfg.max_active_balls = n as usize;
            }
            let scoped = DayuCore::new(
                Arc::new(cfg),
                Arc::clone(st.core.embedding_backend()),
            )
            .map_err(|e| Error::new(Status::GenericFailure, format!("[CORE_INIT] {e}")))?;
            scoped
                .archive_sweep(&st.storage)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("[ARCHIVE] {e}")))
        } else {
            st.core
                .archive_sweep(&st.storage)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("[ARCHIVE] {e}")))
        }
    }
    .instrument(span)
    .await?;
    Ok(ArchiveReportDto {
        archived: report.archived as u32,
        woken: report.woken as u32,
    })
}

/// 归档轮次：将当前轮次内容写入存储（保留旧签名，内部走 create_ball）
#[napi]
pub async fn archive_turn(
    turn_id: String,
    user_text: String,
    assistant_text: String,
    embedding: Vec<f64>,
    qid: Option<String>,
) -> napi::Result<()> {
    let st = state()?;
    // `turn_id` 是插件侧的回合标识，仅作元数据；链路标识统一用 `qid`。
    let qid = qid
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| QueryId::from_input(Some(&turn_id)).0);
    let span = tracing::info_span!("archive_turn", qid = %qid);
    let text = if assistant_text.is_empty() {
        user_text
    } else {
        format!("{user_text}\n{assistant_text}")
    };
    let emb: Vec<f32> = embedding.into_iter().map(|x| x as f32).collect();
    st.core
        .create_ball(&st.storage, &text, &emb, 1.0)
        .instrument(span)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[ARCHIVE_TURN] {e}")))?;
    Ok(())
}

/// 纠正记忆：标记球为争议状态（离线巩固再确认）
#[napi]
pub async fn correct_memory(ball_id: u32, _correction: String) -> napi::Result<()> {
    let st = state()?;
    st.core
        .set_ball_status(&st.storage, BallId(ball_id as u64), BallStatus::Contested)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[CORRECT] {e}")))?;
    Ok(())
}

/// 获取统计信息
#[napi]
pub async fn get_stats() -> napi::Result<StatsDto> {
    let st = state()?;
    let stats = st
        .core
        .stats(&st.storage)
        .await
        .map_err(|e| Error::new(Status::GenericFailure, format!("[STATS] {e}")))?;
    Ok(StatsDto {
        active_ball_count: stats.active_balls as u32,
        archived_ball_count: stats.archived_balls as u32,
        embedding_dim: stats.embedding_dim as u32,
    })
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小合法配置模板；模型路径由调用方填入
    fn config_toml(model: &str, tokenizer: &str) -> String {
        format!(
            r#"
max_flow_depth = 5
max_lit_ball_count = 50
start_top_n = 3
start_ball_threshold = 0.3
total_water_budget = 1.0
supplement_ratio = 0.2
min_water_threshold = 0.01
base_loss = 0.2
loss_pipe_factor = 0.8
loss_max = 0.8
strengthen_eligibility_threshold = 0.6
min_strengthen_value = 0.05
base_reinforcement = 0.1
supplement_reinforce_ratio = 0.5
min_reinforcement_threshold = 0.01
pipe_strengthen_factor = 0.05
min_pipe_reinforce_threshold = 0.005
# embedding_dim 省略时以模型实际输出为准
embedding_model_path = "{model}"
embedding_tokenizer_path = "{tokenizer}"
injection_mode = "Token"
supplement_threshold = 0.45
split_mode = "Weighted"
max_supplement_ratio = 2.0
discount_per_hop = 0.1
min_discount = 0.5
chars_per_token = 4
pipe_thickness_ceiling = 1.0
token_budget = 2048
max_balls_per_inject = 5
render_clear_threshold = 0.7
impression_threshold = 0.4
"#
        )
    }

    /// 未配置模型路径时必须失败：系统已无 mock 回退
    #[test]
    fn init_without_model_path_fails() {
        let dir = dayu_testkit::temp_dir("napi-");
        let path = dir.join("no-model.tdb");
        assert!(
            init(
                config_toml("", ""),
                Some(path.to_str().unwrap().to_string())
            )
            .is_err(),
            "未配置模型路径时 init 应失败（无 mock 回退）"
        );
    }

    /// 配置合法且模型可用时 init 应成功；模型不存在时跳过
    #[test]
    fn init_with_valid_config() {
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("无法定位 workspace 根目录")
            .to_path_buf();
        let model = std::env::var("DAYU_EMBEDDING_MODEL_PATH").map_or_else(
            |_| workspace_root.join("models/infinilabs/pizza-embed-zh-small-v1/onnx/model.onnx"),
            std::path::PathBuf::from,
        );
        let tokenizer = std::env::var("DAYU_EMBEDDING_TOKENIZER_PATH").map_or_else(
            |_| workspace_root.join("models/infinilabs/pizza-embed-zh-small-v1/tokenizer.json"),
            std::path::PathBuf::from,
        );
        if !model.exists() || !tokenizer.exists() {
            eprintln!("跳过 init_with_valid_config: 模型文件不存在");
            return;
        }
        // 模型在 → ort 会真的建 Session，此时必须让 load-dynamic 找到仓库自带的
        // ONNX Runtime，否则会先命中系统 PATH 里的旧版本而版本不符
        dayu_testkit::ensure_ort_dylib();

        let toml = config_toml(
            &model.to_string_lossy().replace('\\', "/"),
            &tokenizer.to_string_lossy().replace('\\', "/"),
        );
        let dir = dayu_testkit::temp_dir("napi-");
        let path = dir.join("t.tdb");
        let result = init(toml, Some(path.to_string_lossy().to_string()));
        assert!(result.is_ok(), "init 应成功，实际错误: {:?}", result.err());
    }

    #[test]
    fn init_with_invalid_config_fails() {
        assert!(init("not valid toml {{{".to_string(), None).is_err());
    }
}
