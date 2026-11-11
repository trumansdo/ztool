//! 抽象 trait 定义
//!
//! 四个核心 trait：
//! - StorageBackend + StorageTx：存储抽象
//! - JudgeBackend：判别器抽象
//! - EmbeddingBackend：嵌入抽象
//! - Clock：时间抽象（可测试性）

use crate::types::*;
use crate::error::*;
use async_trait::async_trait;

// ============================================================
// 存储抽象
// ============================================================

/// 存储后端：流动和结算通过此 trait 读写数据，不感知底层实现
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 读取单个球
    async fn get_ball(&self, id: BallId) -> Result<Option<Ball>, StorageError>;

    /// 批量读取球
    async fn get_balls(&self, ids: &[BallId]) -> Result<Vec<Ball>, StorageError>;

    /// 加载子图：从起始球出发，加载 N 跳邻居
    async fn load_subgraph(
        &self,
        starts: &[BallId],
        depth: usize,
    ) -> Result<SubGraph, StorageError>;

    /// 按嵌入向量检索 Top-K 球
    async fn search_by_embedding(
        &self,
        query: &[f32],
        top_k: usize,
    ) -> Result<Vec<(BallId, f32)>, StorageError>;

    /// 获取活跃球总数
    async fn active_ball_count(&self) -> Result<usize, StorageError>;

    /// 分配新的球 ID（单调递增；未提交也会消耗，可接受）
    async fn allocate_ball_id(&self) -> Result<BallId, StorageError>;

    /// 分配新的管道 ID
    async fn allocate_pipe_id(&self) -> Result<PipeId, StorageError>;

    /// 按状态列出球（`None` = 全部，含归档）
    ///
    /// 归档扫描需要遍历全部活跃球；检索路径不用它，用 `search_by_embedding`。
    async fn list_balls(&self, status: Option<BallStatus>) -> Result<Vec<Ball>, StorageError>;

    /// 列出全部管道
    ///
    /// 用于观测与对照实验：管道粗细是否随「被走通」而增长，
    /// 是「联想被强化」最直接的证据，光看球的大小看不出来。
    async fn list_pipes(&self) -> Result<Vec<Pipe>, StorageError>;

    /// 开启写事务
    async fn begin_transaction(&self) -> Result<Box<dyn StorageTx>, StorageError>;
}

/// 存储写事务
#[async_trait]
pub trait StorageTx: Send + Sync {
    /// 插入或更新球
    async fn upsert_ball(&mut self, ball: &Ball) -> Result<(), StorageError>;

    /// 原子增量更新球大小（**物化衰减**）
    ///
    /// 写入值为 `size_effective(now) + delta`，并刷新 `last_watered = now`。
    /// 不做物化的话，一个衰减到接近零的球被强化后，`size` 会从历史峰值继续累加，
    /// 等于“衰减从未发生”——违反 docs/07 第五层「衰减必须物化」。
    async fn increment_ball_size(
        &mut self,
        id: BallId,
        delta: f64,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), StorageError>;

    /// 更新球的稳定度 S（衰减时间常数，天）
    ///
    /// 与 `increment_ball_size` 分开是因为两者语义不同：
    /// size 是“现在多强”，S 是“变弱多快”。分开调用可单独观测、单独对照。
    async fn update_ball_stability(
        &mut self,
        id: BallId,
        stability: f64,
    ) -> Result<(), StorageError>;

    /// 更新球的状态
    ///
    /// 归档/唤醒都走这一个方法，不引入临时状态：归档球被检索命中即直接
    /// 翻回 `Active`，下一次流动它就当作普通球参与。
    async fn update_ball_status(
        &mut self,
        id: BallId,
        status: BallStatus,
    ) -> Result<(), StorageError>;

    /// 插入或更新管道
    async fn upsert_pipe(&mut self, pipe: &Pipe) -> Result<(), StorageError>;

    /// 物理删除球（`docs/13` R6）
    ///
    /// **调用方责任**：必须先把该球的边与内容归属处理完（合并到保留球），
    /// 再调此方法。实现侧会顺带断掉该球的所有管道边，但不会帮忙搬内容。
    ///
    /// 幂等：删除不存在的球不报错。合并流程里同一球可能被多簇引用，
    /// 让底层容忍重复删除比让每个调用方自己防重更省事。
    async fn delete_ball(&mut self, id: BallId) -> Result<(), StorageError>;

    /// 提交事务
    async fn commit(self: Box<Self>) -> Result<(), StorageError>;

    /// 回滚事务
    async fn rollback(self: Box<Self>) -> Result<(), StorageError>;
}

// ============================================================
// 判别器抽象
// ============================================================

/// 判别器后端：异步旁路，判定是否产生新球
#[async_trait]
pub trait JudgeBackend: Send + Sync {
    /// 对候选片段进行判别
    async fn judge(&self, input: &JudgeInput) -> Result<Verdict, JudgeError>;

    /// 后端类型（用于指标标签）
    fn backend_kind(&self) -> BackendKind;
}

// ============================================================
// 嵌入抽象
// ============================================================

/// 嵌入后端：文本 → 语义向量
#[async_trait]
pub trait EmbeddingBackend: Send + Sync {
    /// 单条文本嵌入
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// 批量文本嵌入
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbeddingError>;

    /// 嵌入向量维度
    fn dimension(&self) -> usize;
}

// ============================================================
// 时间抽象
// ============================================================

/// 时钟抽象：使时间相关逻辑可测试
pub trait Clock: Send + Sync {
    fn now(&self) -> chrono::DateTime<chrono::Utc>;
}

/// 真实系统时钟
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}
