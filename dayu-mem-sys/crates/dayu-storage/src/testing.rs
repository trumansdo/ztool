//! 测试辅助：临时目录里的真实 TriviumDB 实例
//!
//! 只在 `feature = "test-support"` 下编译。生产的 TriviumDB 需要真实文件路径，
//! 测试用临时目录隔离，`TempDir` 句柄必须活到测试结束——它一 Drop，数据库文件就被删。
//!
//! 维度惰性确定：`seed` 时按第一个球的向量维度打开数据库。测试里既有 4 维的手算场景，
//! 也有 512 维的贴近生产场景，写死维度会让一半测试失败。

use crate::TriviumStorage;
use async_trait::async_trait;
use dayu_domain::error::StorageError;
use dayu_domain::traits::{StorageBackend, StorageTx};
use dayu_domain::types::*;
use std::sync::OnceLock;

/// 未提供球时的默认维度
const DEFAULT_DIM: usize = 512;

/// 测试用存储：临时目录 + 惰性打开的真实 TriviumDB
pub struct TestStore {
    /// 必须持有到测试结束，否则数据库文件会被删除
    _dir: std::path::PathBuf,
    path: String,
    inner: OnceLock<TriviumStorage>,
}

impl TestStore {
    /// 按指定维度打开（重复调用忽略维度参数）
    pub fn ensure(&self, dim: usize) -> &TriviumStorage {
        self.inner.get_or_init(|| {
            TriviumStorage::with_dim(&self.path, dim.max(1))
                .unwrap_or_else(|e| panic!("[TEST] 无法打开 TriviumDB: {e}"))
        })
    }

    /// 已打开的实例；未显式指定维度时按默认 512 维打开
    fn opened(&self) -> &TriviumStorage {
        self.ensure(DEFAULT_DIM)
    }

    /// 强制落盘
    pub fn flush(&self) -> Result<(), StorageError> {
        self.opened().flush()
    }
}

#[async_trait]
impl StorageBackend for TestStore {
    async fn get_ball(&self, id: BallId) -> Result<Option<Ball>, StorageError> {
        self.opened().get_ball(id).await
    }

    async fn get_balls(&self, ids: &[BallId]) -> Result<Vec<Ball>, StorageError> {
        self.opened().get_balls(ids).await
    }

    async fn load_subgraph(
        &self,
        starts: &[BallId],
        depth: usize,
    ) -> Result<SubGraph, StorageError> {
        self.opened().load_subgraph(starts, depth).await
    }

    async fn search_by_embedding(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<(BallId, f32)>, StorageError> {
        self.opened().search_by_embedding(query, top_k).await
    }

    async fn active_ball_count(&self) -> Result<usize, StorageError> {
        self.opened().active_ball_count().await
    }

    async fn allocate_ball_id(&self) -> Result<BallId, StorageError> {
        self.opened().allocate_ball_id().await
    }

    async fn allocate_pipe_id(&self) -> Result<PipeId, StorageError> {
        self.opened().allocate_pipe_id().await
    }

    async fn list_balls(&self, status: Option<BallStatus>) -> Result<Vec<Ball>, StorageError> {
        self.opened().list_balls(status).await
    }

    async fn list_pipes(&self) -> Result<Vec<Pipe>, StorageError> {
        self.opened().list_pipes().await
    }

    async fn begin_transaction(&self) -> Result<Box<dyn StorageTx>, StorageError> {
        self.opened().begin_transaction().await
    }
}

/// 创建临时目录里的测试存储（首次 `seed` 时按球的维度打开）
///
/// 目录保留在 `test/run-*/` 下，测试结束不删除，便于事后查看 db。
pub fn temp_store() -> TestStore {
    let dir = dayu_testkit::temp_dir("store-");
    let path = dir.join("test.tdb");
    TestStore {
        _dir: dir,
        path: path.to_str().expect("[TEST] 路径非 UTF-8").to_string(),
        inner: OnceLock::new(),
    }
}

/// 批量写入球与管道
pub async fn seed(store: &TestStore, balls: Vec<Ball>, pipes: Vec<Pipe>) {
    let dim = balls
        .first()
        .map(|b| b.embedding.len())
        .filter(|d| *d > 0)
        .unwrap_or(DEFAULT_DIM);
    store.ensure(dim);

    let mut tx = store
        .begin_transaction()
        .await
        .expect("[TEST] begin_transaction 失败");
    for b in &balls {
        tx.upsert_ball(b).await.expect("[TEST] upsert_ball 失败");
    }
    for p in &pipes {
        tx.upsert_pipe(p).await.expect("[TEST] upsert_pipe 失败");
    }
    tx.commit().await.expect("[TEST] commit 失败");
}
