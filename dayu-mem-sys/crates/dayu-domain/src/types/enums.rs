//! 枚举：状态与类型
//!
//! 封闭世界，新增变体编译器强制处理。

use serde::{Deserialize, Serialize};
use super::ids::BallId;

/// 球的状态
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BallStatus {
    /// 活跃，参与流动和结算
    Active,
    /// 被隐式纠正标记，待离线巩固确认
    Contested,
    /// 已归档，不参与常规流动，强线索可临时唤醒
    Archived,
    /// 软删除
    Deleted,
}

impl BallStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BallStatus::Active => "active",
            BallStatus::Contested => "contested",
            BallStatus::Archived => "archived",
            BallStatus::Deleted => "deleted",
        }
    }
}

/// 管道状态
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PipeStatus {
    Active,
    Archived,
}

impl PipeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PipeStatus::Active => "active",
            PipeStatus::Archived => "archived",
        }
    }
}

/// 内容编码方式
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ContentEncoding {
    /// 原始文本
    Raw,
    /// rmw-utf8 压缩
    RmwUtf8,
}

/// 内容状态
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ContentStatus {
    Active,
    Evicted,
}

/// 管道创建来源
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PipeSource {
    /// 新球接图
    NewBall,
    /// 共激活建边
    CoActivation,
    /// 离线相似补边
    OfflineSimilarity,
    /// 手动建边
    Manual,
}

/// 纠正类型
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CorrectionType {
    DeleteContent,
    UpdateContent,
    DeleteBall,
    UpdateBall,
    MergeBall,
    SplitBall,
    DeletePipe,
    ForgetBall,
    RejectRecall,
}

/// 注入置信度
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum InjectionConfidence {
    /// 清晰记忆（sim ≥ 0.7）
    Clear,
    /// 有比较深的印象（0.6 ≤ sim < 0.7）
    DeepImpression,
    /// 有印象（0.5 ≤ sim < 0.6）
    Impression,
    /// 有模糊的印象（0.3 ≤ sim < 0.5）
    FaintImpression,
}

impl InjectionConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            InjectionConfidence::Clear => "clear",
            InjectionConfidence::DeepImpression => "deep_impression",
            InjectionConfidence::Impression => "impression",
            InjectionConfidence::FaintImpression => "faint_impression",
        }
    }
}

/// 注入预算模式
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum InjectionMode {
    /// 按 token 预算
    Token,
    /// 按球数上限
    BallCount,
}

/// 多出边水量分配模式
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum SplitMode {
    /// 按 `1 + w` 加权：粗管多分、细管少分但保底非零（生产默认）
    #[default]
    Weighted,
    /// 平均分配：仅用于对照实验，不作为生产配置
    Equal,
}

/// 冲突处理策略
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// 重算增量
    Recompute,
    /// 中止
    Abort,
}

/// 判别器后端类型
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BackendKind {
    Ort,
    Mock,
}

/// 球创建状态
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BallCreationStatus {
    /// 灰区：显著性不足，暂挂 tentative
    Tentative,
    /// 确认创建
    Confirmed,
}

/// 判别器处理结果
#[derive(Clone, Debug)]
pub enum JudgeResult {
    /// 丢弃：显著性不足
    Discard {
        reason: String,
    },
    /// 归属已有球
    Attach {
        ball_id: BallId,
        confidence: f64,
        significance: f64,
    },
    /// 创建新球
    Create {
        significance: f64,
        status: BallCreationStatus,
        reason: String,
        embedding: Vec<f32>,
        text: String,
    },
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ball_status_as_str() {
        assert_eq!(BallStatus::Active.as_str(), "active");
        assert_eq!(BallStatus::Archived.as_str(), "archived");
    }
}
