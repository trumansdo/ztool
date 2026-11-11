//! TriviumDB 存储实现（真实嵌入式三模引擎，v0.9.0）
//!
//! 映射关系：
//! - `Ball` → Node：`embedding` 存 Vector 字段，其余字段存 JSON payload
//! - `Pipe` → 双向 Edge（label = `pipe`）：`pipe_base_thickness` 存 edge weight，
//!   其余字段存 edge metadata。无向管道落成两条有向边，两个方向都能走通。
//!
//! 事务：`begin_transaction` 收集脏球/脏管，`commit` 时一次性写入 TriviumDB 并 flush。
//! 持久化：TriviumDB 的 WAL + checkpoint 负责，不再有 JSON 全量重写。

use dayu_domain::error::*;
use dayu_domain::traits::*;
use dayu_domain::types::*;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use triviumdb::{Database, NodeId};

/// 无向管道的边标签
const PIPE_LABEL: &str = "pipe";

/// `BallId` → `NodeId`
///
/// TriviumDB 把 NodeId 0 留作内部哨兵，而大禹的 `BallId` 从 0 开始，
/// 所以统一偏移 1。payload 里保留原始 `BallId`，读回时以 payload 为准。
#[inline]
fn to_node(id: BallId) -> NodeId {
    id.0 + 1
}

/// `NodeId` → `BallId`
#[inline]
fn to_ball(id: NodeId) -> BallId {
    BallId(id.saturating_sub(1))
}
/// 默认向量维度（与 `Config::embedding_dim` 默认值一致）
const DEFAULT_DIM: usize = 512;

// ============================================================
// 编码 / 解码
// ============================================================

/// `Ball` → (vector, payload)；payload 里 `embedding` 置空，避免重复存储
fn ball_to_payload(ball: &Ball) -> Result<(Vec<f32>, serde_json::Value), StorageError> {
    let mut v = serde_json::to_value(ball)
        .map_err(|e| StorageError::Serialization(e.to_string()))?;
    if let Some(obj) = v.as_object_mut() {
        obj.insert("embedding".into(), serde_json::json!([]));
    }
    Ok((ball.embedding.clone(), v))
}

/// (vector, payload) → `Ball`
fn payload_to_ball(vector: Vec<f32>, mut payload: serde_json::Value) -> Option<Ball> {
    let obj = payload.as_object_mut()?;
    obj.insert("embedding".into(), serde_json::json!(vector));
    serde_json::from_value(payload).ok()
}

/// `Pipe` → (src, dst, weight, metadata)
fn pipe_to_edge(pipe: &Pipe) -> Result<(NodeId, NodeId, f32, serde_json::Value), StorageError> {
    let meta = serde_json::to_value(pipe)
        .map_err(|e| StorageError::Serialization(e.to_string()))?;
    Ok((
        to_node(pipe.a),
        to_node(pipe.b),
        pipe.pipe_base_thickness as f32,
        meta,
    ))
}

/// edge(weight, metadata) → `Pipe`
///
/// **metadata 里的 f64 才是权威值**：edge weight 是 f32，只有 7 位有效数字，
/// 用它回填会让 0.9 变成 0.8999999761581421，水量计算随之漂移。
/// weight 只作为 metadata 缺字段时的兜底。
fn edge_to_pipe(weight: f32, meta: serde_json::Value) -> Option<Pipe> {
    let mut pipe: Pipe = serde_json::from_value(meta).ok()?;
    if pipe.pipe_base_thickness <= 0.0 {
        pipe.pipe_base_thickness = weight as f64;
    }
    Some(pipe)
}

// ============================================================
// 存储后端
// ============================================================

/// TriviumDB 存储后端
pub struct TriviumStorage {
    db: Arc<Mutex<Database<f32>>>,
    next_ball_id: Arc<AtomicU64>,
    next_pipe_id: Arc<AtomicU64>,
    dim: usize,
}

impl TriviumStorage {
    /// 用默认维度（384）打开数据库
    pub fn new(path: &str) -> Result<Self, StorageError> {
        Self::with_dim(path, DEFAULT_DIM)
    }

    /// 用指定维度打开数据库（维度一旦创建不可更改）
    pub fn with_dim(path: &str, dim: usize) -> Result<Self, StorageError> {
        let db = Database::<f32>::open(path, dim)
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_OPEN] {e}")))?;

        // 扫描已有节点与边，恢复 ID 分配器
        let ids = db
            .all_node_ids()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_SCAN] {e}")))?;
        // NodeId = BallId + 1，故下一个 BallId = max NodeId
        let max_node = ids.iter().copied().max().unwrap_or(0);
        let mut max_pipe = 0u64;
        for id in &ids {
            if let Ok(edges) = db.get_edges(*id) {
                for e in edges.iter().filter(|e| e.label == PIPE_LABEL) {
                    if let Some(pid) = e.metadata.get("id").and_then(|v| v.as_u64()) {
                        max_pipe = max_pipe.max(pid);
                    }
                }
            }
        }

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            next_ball_id: Arc::new(AtomicU64::new(max_node.max(1))),
            next_pipe_id: Arc::new(AtomicU64::new(max_pipe + 1)),
            dim,
        })
    }

    /// 向量维度
    pub fn dimension(&self) -> usize {
        self.dim
    }

    /// 强制落盘（测试与优雅关闭用）
    pub fn flush(&self) -> Result<(), StorageError> {
        let mut db = self.lock()?;
        db.flush()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_FLUSH] {e}")))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Database<f32>>, StorageError> {
        self.db
            .lock()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_LOCK] {e}")))
    }
}

#[async_trait::async_trait]
impl StorageBackend for TriviumStorage {
    async fn get_ball(&self, id: BallId) -> Result<Option<Ball>, StorageError> {
        let db = self.lock()?;
        match db.get(to_node(id)) {
            Ok(Some(view)) => Ok(payload_to_ball(view.vector, view.payload)),
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::Io(format!("[TRIVIUM_GET] {e}"))),
        }
    }

    async fn get_balls(&self, ids: &[BallId]) -> Result<Vec<Ball>, StorageError> {
        let db = self.lock()?;
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Ok(Some(view)) = db.get(to_node(*id))
                && let Some(ball) = payload_to_ball(view.vector, view.payload)
            {
                out.push(ball);
            }
        }
        Ok(out)
    }

    async fn load_subgraph(
        &self,
        starts: &[BallId],
        depth: usize,
    ) -> Result<SubGraph, StorageError> {
        let db = self.lock()?;

        // BFS 收集 depth 跳内的节点
        let mut visited: HashSet<NodeId> = starts.iter().map(|b| to_node(*b)).collect();
        let mut frontier: HashSet<NodeId> = visited.clone();
        for _ in 0..depth {
            if frontier.is_empty() {
                break;
            }
            let mut next = HashSet::new();
            for id in &frontier {
                let ns = db
                    .neighbors(*id, 1)
                    .map_err(|e| StorageError::Io(format!("[TRIVIUM_NEIGHBORS] {e}")))?;
                next.extend(ns);
            }
            next.retain(|id| !visited.contains(id));
            if next.is_empty() {
                break;
            }
            visited.extend(next.iter().copied());
            frontier = next;
        }

        // 读球
        let mut balls = Vec::with_capacity(visited.len());
        for id in &visited {
            if let Ok(Some(view)) = db.get(*id)
                && let Some(ball) = payload_to_ball(view.vector, view.payload)
            {
                balls.push(ball);
            }
        }

        // 读管道：双向边按 (min,max) 去重
        //
        // 只保留**两端都在子图内**的边。`db.get_edges(id)` 会返回跨出深度边界的边，
        // 直接收下就会得到一个悬空边（另一端没加载）——那等于绕过 `depth` 预算
        // 告诉调用方“外面还有个球”。当前两个消费者都做了防御跳过，但那是隐患：
        // 任何新消费者只要假设「子图是闭合的」就会读到不存在的球。
        let mut seen: HashSet<(u64, u64)> = HashSet::new();
        let mut pipes = Vec::new();
        for id in &visited {
            let edges = db
                .get_edges(*id)
                .map_err(|e| StorageError::Io(format!("[TRIVIUM_EDGES] {e}")))?;
            for e in edges.iter().filter(|e| e.label == PIPE_LABEL) {
                if !visited.contains(&e.target_id) {
                    continue;
                }
                let key = if *id < e.target_id {
                    (*id, e.target_id)
                } else {
                    (e.target_id, *id)
                };
                if !seen.insert(key) {
                    continue;
                }
                if let Some(pipe) = edge_to_pipe(e.weight, e.metadata.clone()) {
                    pipes.push(pipe);
                }
            }
        }

        // 稳定顺序：TriviumDB 返回的边/节点顺序不保证稳定，而流动的 BFS 顺序会影响
        // 多源汇合结果（先到的路径先点亮）。按 id 排序保证同输入同输出（docs/07 第三层）。
        balls.sort_by_key(|b| b.id.0);
        pipes.sort_by_key(|p| p.id.0);
        Ok(SubGraph { balls, pipes })
    }

    async fn search_by_embedding(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<(BallId, f32)>, StorageError> {
        if query.len() != self.dim {
            return Err(StorageError::DimensionMismatch {
                expected: self.dim,
                actual: query.len(),
            });
        }
        let db = self.lock()?;
        // expand_depth = 0：只做向量召回，图扩散由 FlowEngine 自己控制
        let hits = db
            .search(query, top_k, 0, 0.0)
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_SEARCH] {e}")))?;

        let mut out = Vec::with_capacity(hits.len());
        for hit in hits {
            // 排除已删除
            let deleted = hit
                .payload
                .get("status")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == "Deleted");
            if !deleted {
                out.push((to_ball(hit.id), hit.score));
            }
        }
        Ok(out)
    }

    async fn active_ball_count(&self) -> Result<usize, StorageError> {
        Ok(self
            .list_balls(Some(BallStatus::Active))
            .await?
            .len())
    }

    async fn allocate_ball_id(&self) -> Result<BallId, StorageError> {
        Ok(BallId(self.next_ball_id.fetch_add(1, Ordering::SeqCst)))
    }

    async fn allocate_pipe_id(&self) -> Result<PipeId, StorageError> {
        Ok(PipeId(self.next_pipe_id.fetch_add(1, Ordering::SeqCst)))
    }

    async fn list_balls(&self, status: Option<BallStatus>) -> Result<Vec<Ball>, StorageError> {
        let db = self.lock()?;
        let ids = db
            .all_node_ids()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_IDS] {e}")))?;
        let mut out = Vec::new();
        for id in ids {
            if let Ok(Some(view)) = db.get(id)
                && let Some(ball) = payload_to_ball(view.vector, view.payload)
                && status.is_none_or(|s| ball.status == s)
            {
                out.push(ball);
            }
        }
        Ok(out)
    }

    async fn list_pipes(&self) -> Result<Vec<Pipe>, StorageError> {
        let db = self.lock()?;
        let ids = db
            .all_node_ids()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_IDS] {e}")))?;
        // 无向边在两端各存一条，按 (min,max) 去重
        let mut seen: HashSet<(u64, u64)> = HashSet::new();
        let mut out = Vec::new();
        for id in ids {
            let edges = db
                .get_edges(id)
                .map_err(|e| StorageError::Io(format!("[TRIVIUM_EDGES] {e}")))?;
            for e in edges.iter().filter(|e| e.label == PIPE_LABEL) {
                let key = if id < e.target_id {
                    (id, e.target_id)
                } else {
                    (e.target_id, id)
                };
                if !seen.insert(key) {
                    continue;
                }
                if let Some(pipe) = edge_to_pipe(e.weight, e.metadata.clone()) {
                    out.push(pipe);
                }
            }
        }
        out.sort_by_key(|p| p.id.0);
        Ok(out)
    }

    async fn begin_transaction(&self) -> Result<Box<dyn StorageTx>, StorageError> {
        Ok(Box::new(TriviumTx {
            db: Arc::clone(&self.db),
            dirty_balls: HashMap::new(),
            dirty_pipes: HashMap::new(),
        }))
    }
}

// ============================================================
// 写事务
// ============================================================

struct TriviumTx {
    db: Arc<Mutex<Database<f32>>>,
    dirty_balls: HashMap<BallId, Ball>,
    dirty_pipes: HashMap<PipeId, Pipe>,
}

#[async_trait::async_trait]
impl StorageTx for TriviumTx {
    async fn upsert_ball(&mut self, ball: &Ball) -> Result<(), StorageError> {
        self.dirty_balls.insert(ball.id, ball.clone());
        Ok(())
    }

    async fn increment_ball_size(
        &mut self,
        id: BallId,
        delta: f64,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), StorageError> {
        let ball = self
            .dirty_balls
            .get(&id)
            .cloned()
            .or_else(|| {
                self.db
                    .lock()
                    .ok()
                    .and_then(|db| db.get(to_node(id)).ok().flatten())
                    .and_then(|v| payload_to_ball(v.vector, v.payload))
            })
            .ok_or(StorageError::BallNotFound(id))?;

        // 物化：先按衰减折到“现在”，再加增量
        let effective = ball.size_effective(now);
        let mut updated = ball;
        updated.ball_stored_size = (effective + delta).clamp(0.0, updated.ball_size_ceiling);
        updated.last_watered = now;
        // 被注水即计一次点亮：`lit_count` 是「这条记忆被想起过多少次」，
        // 稳定性增长与归档判定都依赖它，漏记会让 S 永远停在初始值。
        updated.lit_count = updated.lit_count.saturating_add(1);
        self.dirty_balls.insert(id, updated);
        Ok(())
    }

    async fn update_ball_stability(
        &mut self,
        id: BallId,
        stability: f64,
    ) -> Result<(), StorageError> {
        let ball = self
            .dirty_balls
            .get(&id)
            .cloned()
            .or_else(|| {
                self.db
                    .lock()
                    .ok()
                    .and_then(|db| db.get(to_node(id)).ok().flatten())
                    .and_then(|v| payload_to_ball(v.vector, v.payload))
            })
            .ok_or(StorageError::BallNotFound(id))?;
        let mut updated = ball;
        updated.ball_stability = stability;
        self.dirty_balls.insert(id, updated);
        Ok(())
    }

    async fn update_ball_status(
        &mut self,
        id: BallId,
        status: BallStatus,
    ) -> Result<(), StorageError> {
        let ball = self
            .dirty_balls
            .get(&id)
            .cloned()
            .or_else(|| {
                self.db
                    .lock()
                    .ok()
                    .and_then(|db| db.get(to_node(id)).ok().flatten())
                    .and_then(|v| payload_to_ball(v.vector, v.payload))
            })
            .ok_or(StorageError::BallNotFound(id))?;
        let mut updated = ball;
        updated.status = status;
        self.dirty_balls.insert(id, updated);
        Ok(())
    }

    async fn upsert_pipe(&mut self, pipe: &Pipe) -> Result<(), StorageError> {
        self.dirty_pipes.insert(pipe.id, pipe.clone());
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), StorageError> {
        let mut db = self
            .db
            .lock()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_LOCK] {e}")))?;

        for ball in self.dirty_balls.values() {
            let (vector, payload) = ball_to_payload(ball)?;
            db.upsert_with_id(to_node(ball.id), &vector, payload)
                .map_err(|e| StorageError::Io(format!("[TRIVIUM_UPSERT_BALL] {e}")))?;
        }

        for pipe in self.dirty_pipes.values() {
            let (a, b, weight, meta) = pipe_to_edge(pipe)?;
            // 无向管道落成双向边，两个方向都能走通
            db.upsert_edge(a, b, PIPE_LABEL, weight, meta.clone())
                .map_err(|e| StorageError::Io(format!("[TRIVIUM_UPSERT_EDGE] {e}")))?;
            db.upsert_edge(b, a, PIPE_LABEL, weight, meta)
                .map_err(|e| StorageError::Io(format!("[TRIVIUM_UPSERT_EDGE] {e}")))?;
        }

        db.flush()
            .map_err(|e| StorageError::Io(format!("[TRIVIUM_FLUSH] {e}")))?;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), StorageError> {
        // 脏数据只在内存里，丢弃即可
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dayu_testkit::*;

    fn temp_store() -> (TriviumStorage, std::path::PathBuf) {
        let dir = dayu_testkit::temp_dir("store-");
        let path = dir.join("test.tdb");
        let store = TriviumStorage::new(path.to_str().unwrap()).unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn trivium_storage_insert_and_get() {
        let (store, _dir) = temp_store();
        let ball = sample_ball(1, 3.0, vec![0.1; 512]);

        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&ball).await.unwrap();
        tx.commit().await.unwrap();

        let got = store.get_ball(BallId(1)).await.unwrap().unwrap();
        assert_eq!(got.id, BallId(1));
        assert!((got.ball_stored_size - 3.0).abs() < 1e-9);
        assert_eq!(got.embedding.len(), 512);
    }

    #[tokio::test]
    async fn trivium_storage_rollback() {
        let (store, _dir) = temp_store();
        let ball = sample_ball(1, 3.0, vec![0.1; 512]);

        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&ball).await.unwrap();
        tx.rollback().await.unwrap();

        assert!(store.get_ball(BallId(1)).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn trivium_storage_persistence() {
        let dir = dayu_testkit::temp_dir("store-");
        let path = dir.join("test.tdb");
        let path_str = path.to_str().unwrap().to_string();

        {
            let store = TriviumStorage::new(&path_str).unwrap();
            let ball = sample_ball(7, 4.5, vec![0.2; 512]);
            let mut tx = store.begin_transaction().await.unwrap();
            tx.upsert_ball(&ball).await.unwrap();
            tx.commit().await.unwrap();
            store.flush().unwrap();
        }

        // 重开：数据必须还在
        let store = TriviumStorage::new(&path_str).unwrap();
        let got = store.get_ball(BallId(7)).await.unwrap().unwrap();
        assert!((got.ball_stored_size - 4.5).abs() < 1e-9);
    }

    #[tokio::test]
    async fn trivium_storage_increment_size() {
        let (store, _dir) = temp_store();
        let ball = sample_ball(1, 3.0, vec![0.1; 512]);
        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&ball).await.unwrap();
        tx.commit().await.unwrap();

        let mut tx = store.begin_transaction().await.unwrap();
        tx.increment_ball_size(BallId(1), 2.0, chrono::Utc::now())
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let got = store.get_ball(BallId(1)).await.unwrap().unwrap();
        assert!((got.ball_stored_size - 5.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn trivium_storage_increment_clamped() {
        let (store, _dir) = temp_store();
        let mut ball = sample_ball(1, 3.0, vec![0.1; 512]);
        ball.ball_size_ceiling = 5.0;
        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&ball).await.unwrap();
        tx.commit().await.unwrap();

        let mut tx = store.begin_transaction().await.unwrap();
        tx.increment_ball_size(BallId(1), 10.0, chrono::Utc::now())
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let got = store.get_ball(BallId(1)).await.unwrap().unwrap();
        assert!((got.ball_stored_size - 5.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn trivium_storage_load_subgraph_chain() {
        let (store, _dir) = temp_store();
        let a = sample_ball(1, 1.0, vec![0.1; 512]);
        let b = sample_ball(2, 1.0, vec![0.2; 512]);
        let pipe = sample_pipe(1, 1, 2, 0.7);

        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&a).await.unwrap();
        tx.upsert_ball(&b).await.unwrap();
        tx.upsert_pipe(&pipe).await.unwrap();
        tx.commit().await.unwrap();

        let sub = store.load_subgraph(&[BallId(1)], 1).await.unwrap();
        assert_eq!(sub.balls.len(), 2);
        assert_eq!(sub.pipes.len(), 1, "无向管道去重后只应有一条");
        assert!((sub.pipes[0].pipe_base_thickness - 0.7).abs() < 1e-6);
    }

    #[tokio::test]
    async fn trivium_storage_search_by_embedding() {
        let (store, _dir) = temp_store();
        let mut a = sample_ball(1, 1.0, vec![0.0; 512]);
        a.embedding[0] = 1.0;
        let mut b = sample_ball(2, 1.0, vec![0.0; 512]);
        b.embedding[1] = 1.0;

        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&a).await.unwrap();
        tx.upsert_ball(&b).await.unwrap();
        tx.commit().await.unwrap();

        let mut q = vec![0.0f32; 512];
        q[0] = 1.0;
        let hits = store.search_by_embedding(&q, 2).await.unwrap();
        assert_eq!(hits[0].0, BallId(1), "同向球应排第一");
        println!("DEBUG scores: {:?}", hits.iter().map(|(id, s)| (id.0, s)).collect::<Vec<_>>());
        assert!((hits[0].1 - 1.0).abs() < 1e-3, "同向向量余弦应为 1.0，实际 {}", hits[0].1);
    }

    #[tokio::test]
    async fn trivium_storage_active_ball_count() {
        let (store, _dir) = temp_store();
        let a = sample_ball(1, 1.0, vec![0.1; 512]);
        let mut b = sample_ball(2, 1.0, vec![0.2; 512]);
        b.status = BallStatus::Archived;

        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&a).await.unwrap();
        tx.upsert_ball(&b).await.unwrap();
        tx.commit().await.unwrap();

        assert_eq!(store.active_ball_count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn trivium_storage_pipe_operations() {
        let (store, _dir) = temp_store();
        let a = sample_ball(1, 1.0, vec![0.1; 512]);
        let b = sample_ball(2, 1.0, vec![0.2; 512]);
        let mut pipe = sample_pipe(1, 1, 2, 0.5);

        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_ball(&a).await.unwrap();
        tx.upsert_ball(&b).await.unwrap();
        tx.upsert_pipe(&pipe).await.unwrap();
        tx.commit().await.unwrap();

        // 加粗后应读到新值
        pipe.pipe_base_thickness = 0.9;
        let mut tx = store.begin_transaction().await.unwrap();
        tx.upsert_pipe(&pipe).await.unwrap();
        tx.commit().await.unwrap();

        let sub = store.load_subgraph(&[BallId(2)], 1).await.unwrap();
        let got = sub.pipes.iter().find(|p| p.id == PipeId(1)).unwrap();
        assert!((got.pipe_base_thickness - 0.9).abs() < 1e-6);
    }

    #[tokio::test]
    async fn trivium_storage_dimension_mismatch_rejected() {
        let (store, _dir) = temp_store();
        let err = store.search_by_embedding(&[0.1; 8], 5).await;
        assert!(matches!(err, Err(StorageError::DimensionMismatch { .. })));
    }
}
