//! HTTP 端点：`/health` `/reset` `/store` `/search`
//!
//! 契约见 `docs/11-评测对接.md`。要点：
//! - `/search` 必须回填 `session_id` / `turn_idx`，否则 harness 无法与 gold evidence 对齐
//! - `as_of_date` 当前不支持，忽略而不报错
//! - 返回顺序就是 recall 的注入顺序，服务端不重排

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use dayu_core::NewBall;
use dayu_domain::types::{BallId, ContentStatus, TurnId};
use serde::{Deserialize, Serialize};

use crate::state::{Registry, SourceTurn};

/// 单轮注入时的显著度：评测场景取中性值
const EVAL_SIGNIFICANCE: f64 = 1.0;

pub fn router(reg: Arc<Registry>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/reset", post(reset))
        .route("/store", post(store))
        .route("/store_batch", post(store_batch))
        .route("/search", post(search))
        .with_state(reg)
}

type ApiError = (StatusCode, String);

fn internal(e: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

async fn health() -> &'static str {
    "ok"
}

// ============================================================
// /reset
// ============================================================

#[derive(Deserialize)]
struct ResetReq {
    namespace: String,
}

#[derive(Serialize)]
struct OkResp {
    ok: bool,
}

async fn reset(
    State(reg): State<Arc<Registry>>,
    Json(req): Json<ResetReq>,
) -> Result<Json<OkResp>, ApiError> {
    reg.reset(&req.namespace).map_err(internal)?;
    Ok(Json(OkResp { ok: true }))
}

// ============================================================
// /store
// ============================================================

#[derive(Deserialize)]
struct StoreReq {
    namespace: String,
    turn: TurnIn,
}

#[derive(Deserialize)]
struct TurnIn {
    content: String,
    #[serde(default)]
    #[allow(dead_code)] // 角色暂不参与检索，保留字段以免 harness 传值时 400
    role: String,
    session_id: String,
    turn_idx: i64,
    #[serde(default)]
    session_idx: i64,
    #[serde(default)]
    #[allow(dead_code)] // 时间过滤尚未实现（见 as_of_date 说明）
    timestamp: Option<String>,
}

#[derive(Serialize)]
struct StoreResp {
    memory_id: String,
}

async fn store(
    State(reg): State<Arc<Registry>>,
    Json(req): Json<StoreReq>,
) -> Result<Json<StoreResp>, ApiError> {
    let t = req.turn;
    let st = reg.state(&req.namespace).await.map_err(internal)?;

    let source = SourceTurn {
        session_id: t.session_id.clone(),
        turn_idx: t.turn_idx,
        session_idx: t.session_idx,
    };

    let vec = reg.embed_cached(&t.content).await.map_err(internal)?;

    let id = st
        .core
        .create_ball_with_source(
            &st.storage,
            &t.content,
            &vec,
            1.0,
            TurnId(source.encode()),
        )
        .await
        .map_err(internal)?;

    // 同一会话内相邻回合连管道：这是「水能扩散」的前提。
    // 没有管道时 recall 退化为纯余弦 Top-N，测不出流动的作用。
    if let Some(weight) = reg.link_weight() {
        if let Some(prev) = st.take_last_ball(&t.session_id) {
            // 连边失败不致命：本回合照样可被余弦直接召回
            if let Err(e) = st.core.link_balls(&st.storage, prev, id, weight).await {
                tracing::warn!("连管道失败（session={}）：{e}", t.session_id);
            }
        }
        st.set_last_ball(&t.session_id, id);
    }

    Ok(Json(StoreResp {
        memory_id: id.0.to_string(),
    }))
}

// ============================================================
// /store_batch
// ============================================================

#[derive(Deserialize)]
struct StoreBatchReq {
    namespace: String,
    turns: Vec<TurnIn>,
}

#[derive(Serialize)]
struct StoreBatchResp {
    memory_ids: Vec<String>,
}

/// 批量灌库：整批一个事务
///
/// 评测 harness 每题要灌几百到几千回合，逐个 `/store` 会让 fsync 成为瓶颈
/// （实测 ~37ms/球）。批量后同量级数据降到毫秒级。
async fn store_batch(
    State(reg): State<Arc<Registry>>,
    Json(req): Json<StoreBatchReq>,
) -> Result<Json<StoreBatchResp>, ApiError> {
    if req.turns.is_empty() {
        return Ok(Json(StoreBatchResp {
            memory_ids: Vec::new(),
        }));
    }

    let st = reg.state(&req.namespace).await.map_err(internal)?;

    // 先嵌入（带缓存），再一次性建球建边
    let mut sources = Vec::with_capacity(req.turns.len());
    let mut vectors = Vec::with_capacity(req.turns.len());
    for t in &req.turns {
        sources.push(
            SourceTurn {
                session_id: t.session_id.clone(),
                turn_idx: t.turn_idx,
                session_idx: t.session_idx,
            }
            .encode(),
        );
        vectors.push(reg.embed_cached(&t.content).await.map_err(internal)?);
    }

    let items: Vec<NewBall<'_>> = req
        .turns
        .iter()
        .zip(vectors.iter())
        .zip(sources.iter())
        .map(|((t, v), src)| NewBall {
            content: &t.content,
            embedding: v.as_slice(),
            size: 1.0,
            source: TurnId(src.clone()),
            chain_key: Some(t.session_id.clone()),
        })
        .collect();

    let ids = st
        .core
        .create_turn_balls_batch(&st.storage, &items, reg.link_weight())
        .await
        .map_err(internal)?;

    // 保持与 /store 的链式语义一致：记下每个会话的最后一球
    if reg.link_weight().is_some() {
        for (t, id) in req.turns.iter().zip(ids.iter()) {
            st.set_last_ball(&t.session_id, *id);
        }
    }

    Ok(Json(StoreBatchResp {
        memory_ids: ids.iter().map(|i| i.0.to_string()).collect(),
    }))
}

// ============================================================
// /search
// ============================================================

#[derive(Deserialize)]
struct SearchReq {
    namespace: String,
    query: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
    #[serde(default)]
    #[allow(dead_code)] // dayu-core 暂无时间过滤，按契约忽略而非报错
    as_of_date: Option<String>,
}

fn default_top_k() -> usize {
    10
}

#[derive(Serialize)]
struct MemoryOut {
    memory_id: String,
    content: String,
    session_id: Option<String>,
    turn_idx: Option<i64>,
    session_idx: Option<i64>,
    score: f64,
}

#[derive(Serialize)]
struct SearchResp {
    memories: Vec<MemoryOut>,
}

async fn search(
    State(reg): State<Arc<Registry>>,
    Json(req): Json<SearchReq>,
) -> Result<Json<SearchResp>, ApiError> {
    let st = reg.state(&req.namespace).await.map_err(internal)?;
    let qvec = reg.embed_cached(&req.query).await.map_err(internal)?;

    let blocks = st
        .core
        .recall(&st.storage, &qvec, EVAL_SIGNIFICANCE)
        .await
        .map_err(internal)?;

    // 注入块顺序即流动的排序结果；同一球可能出现在多个块里，按首次出现去重
    let mut seen = std::collections::HashSet::new();
    let mut ids: Vec<BallId> = Vec::new();
    for block in &blocks {
        for id in &block.ball_ids {
            if seen.insert(*id) {
                ids.push(*id);
            }
        }
    }
    ids.truncate(req.top_k);

    let mut memories = Vec::with_capacity(ids.len());
    for id in ids {
        let Some(ball) = st.core.get_ball(&st.storage, id).await.map_err(internal)? else {
            continue;
        };
        let content = ball
            .content_list
            .iter()
            .find(|c| c.status == ContentStatus::Active)
            .or_else(|| ball.content_list.first());
        let (text, source) = match content {
            Some(c) => (c.text.clone(), SourceTurn::decode(&c.source.0)),
            None => (String::new(), None),
        };
        let score = dot(&qvec, &ball.embedding);
        memories.push(MemoryOut {
            memory_id: id.0.to_string(),
            content: text,
            session_id: source.as_ref().map(|s| s.session_id.clone()),
            turn_idx: source.as_ref().map(|s| s.turn_idx),
            session_idx: source.as_ref().map(|s| s.session_idx),
            score,
        });
    }

    Ok(Json(SearchResp { memories }))
}

fn dot(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum()
}
