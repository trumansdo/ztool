//! 结构体：核心数据模型

use serde::{Deserialize, Serialize};
use super::ids::*;
use super::enums::*;

/// 球（记忆节点）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ball {
    pub id: BallId,
    /// 语义嵌入向量（EMA 更新）
    pub embedding: Vec<f32>,
    /// 球大小（记忆强度）
    pub ball_stored_size: f64,
    /// 球大小饱和上限
    pub ball_size_ceiling: f64,
    /// 内容列表
    pub content_list: Vec<Content>,
    /// 内容列表上限
    pub content_list_max: usize,
    /// 最后注水时间
    pub last_watered: chrono::DateTime<chrono::Utc>,
    /// 记忆稳定度 S（天）：衰减时间常数，强化时饱和增长（docs/03 §2.3）
    #[serde(default = "default_ball_stability")]
    pub ball_stability: f64,
    /// 被点亮次数
    pub lit_count: u64,
    /// 球状态
    pub status: BallStatus,
    /// 领域标签（可空）
    pub domain: Option<String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// schema 版本
    pub schema_version: u32,
}

impl Ball {
    /// 有效大小 = 存储大小 × 时间衰减（docs/03 §2.3）
    ///
    /// `exp(-Δt/S)`：S 越大衰减越慢。只算不写回——物化在结算时做。
    pub fn size_effective(&self, now: chrono::DateTime<chrono::Utc>) -> f64 {
        let days = (now - self.last_watered).num_seconds().max(0) as f64 / 86_400.0;
        self.ball_stored_size * (-days / self.ball_stability.max(1e-6)).exp()
    }
}

impl Pipe {
    /// 有效粗细 = 存储粗细 × 时间衰减（docs/03 §3.4）
    pub fn effective_thickness(&self, now: chrono::DateTime<chrono::Utc>) -> f64 {
        let days = (now - self.last_flowed).num_seconds().max(0) as f64 / 86_400.0;
        self.pipe_base_thickness * (-days / self.pipe_stability.max(1e-6)).exp()
    }

    /// 有效阻挡次数：长时间未被阻挡则逐渐遗忘（docs/03 §3.5）
    pub fn blocked_effective(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        decay_days: f64,
    ) -> f64 {
        let Some(t) = self.last_blocked_at else {
            return f64::from(self.blocked_pass_count_stored);
        };
        let days = (now - t).num_seconds().max(0) as f64 / 86_400.0;
        f64::from(self.blocked_pass_count_stored) * (-days / decay_days.max(1e-6)).exp()
    }
}

fn default_ball_stability() -> f64 {
    7.0
}

fn default_pipe_stability() -> f64 {
    3.0
}

/// 内容条目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub id: ContentId,
    /// 文本内容（可能已压缩）
    pub text: String,
    /// 编码方式
    pub encoding: ContentEncoding,
    /// 来源回合
    pub source: TurnId,
    /// 被注入次数
    pub injected_count: u32,
    /// 状态
    pub status: ContentStatus,
}

/// 管道（无向边）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pipe {    pub id: PipeId,
    /// 端点 A（min(a,b) 语义）
    pub a: BallId,
    /// 端点 B（max(a,b) 语义）
    pub b: BallId,
    /// 管道粗细，A↔B 共享
    pub pipe_base_thickness: f64,
    /// 管道稳定度 S_pipe（天）：衰减时间常数（docs/03 §3.4）
    #[serde(default = "default_pipe_stability")]
    pub pipe_stability: f64,
    /// 最后一次被阻挡的时间（用于阻挡次数衰减，docs/03 §3.5）
    #[serde(default)]
    pub last_blocked_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 被阻挡次数，A↔B 共享
    pub blocked_pass_count_stored: u32,
    /// 最后流动时间
    pub last_flowed: chrono::DateTime<chrono::Utc>,
    /// 创建来源
    pub source: PipeSource,
    /// 状态
    pub status: PipeStatus,
}

/// 流动中点亮球的快照
#[derive(Clone, Debug)]
pub struct LitBall {
    pub ball_id: BallId,
    /// 到达该球时的水量
    pub water: f64,
    /// 该球产生的补充量
    pub supplement: f64,
    /// 到达跳数
    pub depth: usize,
    /// 从哪条管道到达（起始球为 None）——用于“只强化真正被水走过的管道”
    pub via_pipe: Option<PipeId>,
}

/// 流动路径
#[derive(Clone, Debug)]
pub struct FlowPath {
    pub steps: Vec<LitBall>,
    pub pipes: Vec<PipeId>,
}

/// 注入块
#[derive(Clone, Debug)]
pub struct InjectionBlock {
    pub text: String,
    pub ball_ids: Vec<BallId>,
    pub confidence: InjectionConfidence,
    pub token_count: usize,
}

/// 判别器输入
#[derive(Clone, Debug)]
pub struct JudgeInput {
    pub candidate_text: String,
    pub candidate_embedding: Vec<f32>,
    pub recent_window: Vec<String>,
    pub candidate_archetypes: Vec<(BallId, f32)>,
}

/// 判别器输出（裁决）
#[derive(Clone, Debug)]
pub struct Verdict {
    pub should_judge: bool,
    pub significance: f64,
    pub offset_direction: String,
    pub emotion_valence: f64,
    pub emotion_intensity: f64,
    pub hit_dimensions: Vec<String>,
    pub belongs_to: Option<BallId>,
    pub belong_confidence: f64,
    pub reason: String,
}

/// 纠正审计记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorrectionLog {
    pub id: uuid::Uuid,
    pub target_type: String,
    pub target_id: String,
    pub correction_type: CorrectionType,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub source: String,
    pub reason: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reversible: bool,
}

/// 子图（流动加载）
#[derive(Clone, Debug)]
pub struct SubGraph {
    pub balls: Vec<Ball>,
    pub pipes: Vec<Pipe>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use proptest::prelude::*;

    fn make_ball(size: f64, last_watered: chrono::DateTime<Utc>) -> Ball {
        Ball {
            id: BallId(1),
            embedding: vec![1.0, 0.0],
            ball_stored_size: size,
            ball_size_ceiling: 10.0,
            content_list: vec![],
            content_list_max: 5,
            last_watered,
            ball_stability: 7.0,
            lit_count: 0,
            status: BallStatus::Active,
            domain: None,
            created_at: last_watered,
            schema_version: 1,
        }
    }

    fn make_pipe(thickness: f64, last_flowed: chrono::DateTime<Utc>) -> Pipe {
        Pipe {
            id: PipeId(1),
            a: BallId(1),
            b: BallId(2),
            pipe_base_thickness: thickness,
            pipe_stability: 3.0,
            last_blocked_at: None,
            blocked_pass_count_stored: 0,
            last_flowed,
            source: PipeSource::NewBall,
            status: PipeStatus::Active,
        }
    }

    #[test]
    fn fresh_ball_has_no_decay() {
        let now = Utc::now();
        let b = make_ball(5.0, now);
        assert!((b.size_effective(now) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn ball_size_decays_with_one_stability_constant() {
        let now = Utc::now();
        let b = make_ball(10.0, now - Duration::days(7));
        // Δt = S = 7 天 → 衰减到 1/e
        assert!((b.size_effective(now) - 10.0 * (-1.0f64).exp()).abs() < 1e-6);
    }

    #[test]
    fn pipe_thickness_decays_with_its_own_constant() {
        let now = Utc::now();
        let p = make_pipe(1.0, now - Duration::days(3));
        assert!((p.effective_thickness(now) - (-1.0f64).exp()).abs() < 1e-6);
    }

    #[test]
    fn blocked_count_decays_after_idle_period() {
        let now = Utc::now();
        let mut p = make_pipe(0.5, now);
        p.blocked_pass_count_stored = 4;
        assert_eq!(p.blocked_effective(now, 7.0), 4.0, "无阻挡时间戳时按存储值");

        p.last_blocked_at = Some(now - Duration::days(7));
        assert!((p.blocked_effective(now, 7.0) - 4.0 * (-1.0f64).exp()).abs() < 1e-6);
    }

    #[test]
    fn old_schema_payload_deserializes_with_defaults() {
        // 旧数据没有 ball_stability / pipe_stability / last_blocked_at
        let json = r#"{
            "id": 1, "embedding": [1.0], "ball_stored_size": 2.0,
            "ball_size_ceiling": 10.0, "content_list": [], "content_list_max": 5,
            "last_watered": "2026-01-01T00:00:00Z", "lit_count": 0,
            "status": "Active", "domain": null,
            "created_at": "2026-01-01T00:00:00Z", "schema_version": 1
        }"#;
        let b: Ball = serde_json::from_str(json).expect("旧数据必须可读");
        assert_eq!(b.ball_stability, 7.0);
    }

    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(300))]

        /// 不变量：阻塞计数衰减后非负、不超过存储值（衰减只减不增）
        #[test]
        fn blocked_effective_is_non_negative_and_bounded(
            stored in 0u32..1000,
            days_ago in 0.0f64..3650.0,
            s in 0.1f64..30.0,
        ) {
            let now = Utc::now();
            let mut p = make_pipe(0.5, now);
            p.blocked_pass_count_stored = stored;
            p.last_blocked_at = Some(now - Duration::seconds((days_ago * 86400.0) as i64));
            let eff = p.blocked_effective(now, s);
            prop_assert!(eff >= 0.0, "阻塞计数为负：{}", eff);
            prop_assert!(eff <= stored as f64 + 1e-9, "衰减后反而变大：{} > {}", eff, stored);
        }

        /// 不变量：size_effective ∈ [0, size]（衰减只减不增，永不为负）
        #[test]
        fn size_effective_is_bounded(
            size in 0.0f64..10.0,
            days_ago in 0.0f64..3650.0,
            s in 0.1f64..30.0,
        ) {
            let now = Utc::now();
            let mut b = make_ball(size, now);
            b.ball_stability = s;
            b.last_watered = now - Duration::seconds((days_ago * 86400.0) as i64);
            let eff = b.size_effective(now);
            prop_assert!(eff >= 0.0, "有效大小为负：{}", eff);
            prop_assert!(eff <= size + 1e-9, "衰减后反而变大：{} > {}", eff, size);
        }
    }
}
