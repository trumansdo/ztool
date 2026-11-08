//! 错误类型定义
//!
//! 每个模块有独立错误枚举，通过 thiserror 派生 Display + Error。

use thiserror::Error;
use crate::types::{BallId, PipeId};

// ============================================================
// 值错误（newtype 构造失败）
// ============================================================

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("pipe weight out of range: {0}, expected (0, 1]")]
    PipeWeightOutOfRange(f64),

    #[error("negative water amount: {0}")]
    NegativeWater(f64),

    #[error("relevance out of range: {0}, expected [0, 1]")]
    RelevanceOutOfRange(f64),

    #[error("negative ball size: {0}")]
    NegativeSize(f64),

    #[error("loss rate out of range: {0}, expected [0, 1)")]
    LossRateOutOfRange(f64),
}

// ============================================================
// 存储错误
// ============================================================

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("ball not found: {0:?}")]
    BallNotFound(BallId),

    #[error("pipe not found: {0:?}")]
    PipeNotFound(PipeId),

    #[error("transaction failed: {0}")]
    TransactionFailed(String),

    #[error("version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("io error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("operation not supported (skeleton/stub)")]
    Unsupported,
}

// ============================================================
// 判别器错误
// ============================================================

#[derive(Debug, Error)]
pub enum JudgeError {
    #[error("model not loaded")]
    ModelNotLoaded,

    #[error("inference failed: {0}")]
    InferenceFailed(String),

    #[error("output parse failed: {0}")]
    OutputParseFailed(String),

    #[error("input too long: {tokens} tokens, limit {limit}")]
    InputTooLong { tokens: usize, limit: usize },

    #[error("backend unavailable")]
    BackendUnavailable,
}

// ============================================================
// 嵌入错误
// ============================================================

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("model not loaded")]
    ModelNotLoaded,

    #[error("tokenization failed: {0}")]
    TokenizationFailed(String),

    #[error("inference failed: {0}")]
    InferenceFailed(String),

    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

// ============================================================
// 流动/结算错误
// ============================================================

#[derive(Debug, Error)]
pub enum FlowError {
    #[error("no starting balls found")]
    NoStartingBalls,

    #[error("max flow depth exceeded: {0}")]
    MaxDepthExceeded(usize),

    #[error("max lit balls exceeded: {0}")]
    MaxLitBallsExceeded(usize),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("embedding error: {0}")]
    Embedding(#[from] EmbeddingError),
}

#[derive(Debug, Error)]
pub enum SettleError {
    #[error("no lit balls to settle")]
    NoLitBalls,

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("version conflict during settle")]
    VersionConflict,
}

// ============================================================
// 核心错误（dayu-core 门面）
// ============================================================

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("storage: {0}")]
    Storage(#[from] StorageError),

    #[error("judge: {0}")]
    Judge(#[from] JudgeError),

    #[error("embedding: {0}")]
    Embedding(#[from] EmbeddingError),

    #[error("flow: {0}")]
    Flow(#[from] FlowError),

    #[error("settle: {0}")]
    Settle(#[from] SettleError),

    #[error("value: {0}")]
    ValueError(#[from] ValueError),

    #[error("config: {0}")]
    Config(String),

    #[error("not initialized")]
    NotInitialized,
}
