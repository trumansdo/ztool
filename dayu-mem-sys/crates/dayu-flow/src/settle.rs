//! 结算引擎
//!
//! 对流动点亮结果进行结算：
//! 1. 路径相关性计算
//! 2. 多路径取最大值
//! 3. 强化（球大小 + 管道粗细）
//! 4. 饱和（上限裁剪）
//! 5. 注入排序
//! 6. 渲染注入块

use dayu_domain::traits::*;
use dayu_domain::types::*;
use dayu_domain::error::*;
use dayu_observability::{
    MetricsRecorder, METRIC_PATH_SUPPLEMENT_LOSS_RATIO, METRIC_INJECTION_TOKEN_USED,
    METRIC_REINFORCED_BALL_COUNT, METRIC_REINFORCED_PIPE_COUNT,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// ============================================================
// 结算引擎
// ============================================================

/// 结算引擎：将点亮结果转化为持久化变更
pub struct SettleEngine {
    config: Arc<Config>,
    metrics: Option<Arc<dyn MetricsRecorder>>,
    clock: Arc<dyn dayu_domain::traits::Clock>,
}

/// 一条路径的结算度量。
///
/// 拆成两个量是因为它们回答不同问题（docs/03 §7.1）：
/// - `semantic_sim`：这条记忆够不够相关、值得被强化（**资格**）
/// - `path_relevance = semantic_sim × 跳数折扣`：强化多少、排序多靠前（**幅度**）
///
/// 路径长度不该否定“想起来”，只该决定“想得多深”——否则最深的球
/// 在 `strengthen_eligibility_threshold > min_discount` 时永远过不了资格线。
#[derive(Clone, Copy, Debug)]
struct PathRelevance {
    ball_id: BallId,
    /// `None` = 无法计算（嵌入维度不匹配 / 球不在子图中），不是“相关度为 0”。
    /// 两者必须区分：无法计算时不能给最低强化值，否则切模型后所有球都被微弱强化。
    semantic_sim: Option<f64>,
    path_relevance: f64,
}

/// 记忆档位（docs/03 §7.1）：由语义相似度划分，决定强化幅度与是否参与注入
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BallTier {
    /// 强化球：拿到完整强化，参与注入
    Strengthened,
    /// 印象球：拿到最低强化值，参与注入
    Impression,
    /// 最低强化球：拿到最低强化值，**不参与注入**
    Minimum,
}

impl BallTier {
    /// 日志/指标用名称
    fn as_str(self) -> &'static str {
        match self {
            Self::Strengthened => "strengthened",
            Self::Impression => "impression",
            Self::Minimum => "minimum",
        }
    }
}

/// 结算结果
#[derive(Debug)]
pub struct SettleResult {
    /// 拿到路径强化的球数量（语义相似度 ≥ 强化球下限）
    pub reinforced_balls: usize,
    /// 拿到最低强化值的印象球数量（`impression_threshold ≤ sim < 强化球下限`）
    pub impression_balls: usize,
    /// 拿到最低强化值的最低强化球数量（`sim < impression_threshold`，不注入）
    pub min_reinforced_balls: usize,
    /// 强化的管道数量
    pub reinforced_pipes: usize,
    /// 记账“被挡一次”的管道数量
    pub blocked_pipes: usize,
    /// 生成的注入块
    pub injection_blocks: Vec<InjectionBlock>,
    /// 总 token 数
    pub total_tokens: usize,
}

impl SettleEngine {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            metrics: None,
            clock: Arc::new(dayu_domain::traits::SystemClock),
        }
    }

    /// 注入时钟（测试/回放用）
    pub fn with_clock(mut self, clock: Arc<dyn dayu_domain::traits::Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// 设置指标记录器
    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsRecorder>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// 执行结算
    pub async fn settle(
        &self,
        storage: &dyn StorageBackend,
        lit_balls: &[LitBall],
        sub: &SubGraph,
        queries: &[&[f32]],
        blocked_pipes: &[PipeId],
    ) -> Result<SettleResult, SettleError> {
        if lit_balls.is_empty() {
            return Err(SettleError::NoLitBalls);
        }

        let path_relevance = self.calculate_path_relevance(lit_balls, sub, queries);
        let max_relevance = self.max_relevance_per_ball(&path_relevance);

        let mut tx = storage.begin_transaction().await?;
        let (reinforced_balls, impression_balls, min_reinforced_balls) = self
            .reinforce_balls(&mut *tx, &max_relevance, lit_balls, sub)
            .await?;
        let reinforced_pipes = self
            .reinforce_pipes(&mut *tx, &max_relevance, lit_balls, sub)
            .await?;
        let blocked_pipes = self
            .record_blocked_pipes(&mut *tx, blocked_pipes, sub)
            .await?;
        tx.commit().await?;

        let sorted = self.sort_for_injection(&max_relevance, sub);
        let injection_blocks = self.render_injection_blocks(&sorted, lit_balls, sub);

        let total_tokens: usize = injection_blocks.iter().map(|b| b.token_count).sum();

        // 记录结算指标
        if let Some(ref m) = self.metrics {
            // 补充损失比：按粗管基准损失率估算（精确值需在流动层累计）。
            // 仅用于观测指标，不参与任何决策——水的唯一职责是点亮（docs/03 §6.1）。
            let total_loss: f64 = lit_balls.iter().map(|l| l.water * self.config.base_loss).sum();
            let total_supplement: f64 = lit_balls.iter().map(|l| l.supplement).sum();
            if total_loss + total_supplement > 0.0 {
                let loss_ratio = total_loss / (total_loss + total_supplement);
                let supplement_ratio = total_supplement / (total_loss + total_supplement);
                m.record_gauge(
                    METRIC_PATH_SUPPLEMENT_LOSS_RATIO,
                    loss_ratio,
                    &[("dominant", "loss")],
                );
                m.record_gauge(
                    METRIC_PATH_SUPPLEMENT_LOSS_RATIO,
                    supplement_ratio,
                    &[("dominant", "supplement")],
                );
            }
            m.record_gauge(METRIC_INJECTION_TOKEN_USED, total_tokens as f64, &[]);
            m.record_gauge(METRIC_REINFORCED_BALL_COUNT, reinforced_balls as f64, &[]);
            m.record_gauge(METRIC_REINFORCED_PIPE_COUNT, reinforced_pipes as f64, &[]);
        }

        Ok(SettleResult {
            reinforced_balls,
            impression_balls,
            min_reinforced_balls,
            reinforced_pipes,
            blocked_pipes,
            injection_blocks,
            total_tokens,
        })
    }

    fn calculate_path_relevance(
        &self,
        lit_balls: &[LitBall],
        sub: &SubGraph,
        queries: &[&[f32]],
    ) -> Vec<PathRelevance> {
        let ball_map: HashMap<BallId, &Ball> =
            sub.balls.iter().map(|b| (b.id, b)).collect();

        // 流动加权语境：原始查询占主导，路径只做轻微染色（docs/03 §7.1）。
        // 权重 = 各球在路径上获得的水量占比；水量越分散、染色越弱。
        let context = self.build_flow_context(lit_balls, &ball_map, queries);

        lit_balls
            .iter()
            .map(|lit| {
                // 相关性口径：`cos(流动加权语境, 球)`（docs/03 §7.1）。
                // 多源时语境的基座是**各源均值**（见 `build_flow_context`），
                // 这样第二个源召回的球不会被当成“与当前输入无关”过滤掉。
                let sim: Option<f64> = ball_map
                    .get(&lit.ball_id)
                    .and_then(|ball| {
                        dayu_domain::math::cosine_similarity(&context, &ball.embedding)
                            .map(f64::from)
                    });
                if sim.is_none() {
                    tracing::warn!(
                        ball_id = ?lit.ball_id,
                        ball_dim = ball_map.get(&lit.ball_id).map(|b| b.embedding.len()).unwrap_or(0),
                        query_dim = context.len(),
                        "维度不匹配，该球相关性无法计算（跳过结算）"
                    );
                }

                // 路径长度折扣
                let hop_discount = (1.0 - self.config.discount_per_hop * lit.depth as f64)
                    .max(self.config.min_discount);

                // 路径相关性 = 语义相关性 × 路径长度折扣（docs/03 §7.1）。
                //
                // **不乘 water**：water 是“这次流动中到达的力度”，量纲随
                // total_water_budget / split_mode / 损失参数漂移；乘进来会让
                // `strengthen_eligibility_threshold` 失去语义——强线索（W₀×3）时，
                // 语义仅 0.25 相关的球 relevance 也能达 0.75 而被判为“强化球”。
                // 相关性只回答“这个球与当前输入有多相关”，不回答“被激活得多强”。
                PathRelevance {
                    ball_id: lit.ball_id,
                    semantic_sim: sim,
                    path_relevance: sim.unwrap_or(0.0) * hop_discount,
                }
            })
            .collect()
    }

    /// 加权语境 = `(1-α) × 原始查询 + α × 路径上各球嵌入的水量加权平均`
    ///
    /// `α = 0` 时退化为原始查询（对照基线）；`α` 上限 0.3，
    /// 保证原始查询始终占主导，路径只做轻微染色。
    fn build_flow_context(
        &self,
        lit_balls: &[LitBall],
        ball_map: &HashMap<BallId, &Ball>,
        queries: &[&[f32]],
    ) -> Vec<f32> {
        // 多源时用各源的均值作为“原始查询”——语境是“这一轮输入整体在说什么”，
        // 不是“第一个源说了什么”。相关性逐球取 max（见 `calculate_path_relevance`）。
        let query_embedding = mean_query(queries);
        let alpha = self.config.flow_context_blend_alpha.clamp(0.0, 0.3) as f32;
        if alpha == 0.0 {
            return query_embedding.to_vec();
        }
        let dim = query_embedding.len();
        let total_water: f64 = lit_balls.iter().map(|l| l.water.max(0.0)).sum();
        if total_water <= 0.0 {
            return query_embedding.to_vec();
        }

        let mut blended = vec![0.0f32; dim];
        let mut weight_sum = 0.0f64;
        for lit in lit_balls {
            let Some(ball) = ball_map.get(&lit.ball_id) else {
                continue;
            };
            if ball.embedding.len() != dim {
                continue;
            }
            let w = lit.water.max(0.0) / total_water;
            for (acc, e) in blended.iter_mut().zip(&ball.embedding) {
                *acc += (w as f32) * e;
            }
            weight_sum += w;
        }
        if weight_sum <= 0.0 {
            return query_embedding.to_vec();
        }

        let norm = weight_sum as f32;
        blended
            .iter()
            .zip(query_embedding)
            .map(|(b, q)| (1.0 - alpha) * q + alpha * (b / norm))
            .collect()
    }

    /// 多路径取最大值：按 `path_relevance` 选出最强的那条路径，
    /// 语义与折扣**都取自同一条路径**，避免“语义来自 A 路径、折扣来自 B 路径”的拼接。
    fn max_relevance_per_ball(
        &self,
        path_relevance: &[PathRelevance],
    ) -> HashMap<BallId, PathRelevance> {
        let mut max_map: HashMap<BallId, PathRelevance> = HashMap::new();
        for r in path_relevance {
            max_map
                .entry(r.ball_id)
                .and_modify(|cur| {
                    if r.path_relevance > cur.path_relevance {
                        *cur = *r;
                    }
                })
                .or_insert(*r);
        }
        max_map
    }

    /// 按语义相似度划分记忆档位（docs/03 §7.1）。
    ///
    /// 强化球：拿到完整强化，参与注入
    /// 印象球：拿到最低强化值，参与注入（“有点印象”但不是完整记忆）
    /// 最低强化球：拿到最低强化值，**不参与注入**
    fn tier_of(&self, semantic_sim: f64) -> BallTier {
        if semantic_sim >= self.config.strengthen_eligibility_threshold {
            BallTier::Strengthened
        } else if semantic_sim >= self.config.impression_threshold {
            BallTier::Impression
        } else {
            BallTier::Minimum
        }
    }

    async fn reinforce_balls(
        &self,
        tx: &mut dyn StorageTx,
        max_relevance: &HashMap<BallId, PathRelevance>,
        lit_balls: &[LitBall],
        sub: &SubGraph,
    ) -> Result<(usize, usize, usize), SettleError> {
        let now = self.clock.now();
        let gamma = self.config.size_saturation_gamma;
        let ball_map: HashMap<BallId, &Ball> = sub.balls.iter().map(|b| (b.id, b)).collect();
        let mut strengthened = 0;
        let mut impression = 0;
        let mut minimum = 0;
        for lit in lit_balls {
            let Some(r) = max_relevance.get(&lit.ball_id) else {
                continue;
            };
            // 无法计算相关性（维度不匹配）：跳过，既不强也不弱——
            // 给最低强化值会让切模型后所有球凭空长一圈
            let Some(semantic_sim) = r.semantic_sim else {
                continue;
            };

            // 资格看语义相似度，幅度看路径相关性——两种信号各管一件事。
            // 若把资格也交给 `sim × 折扣`，深度 4/5 的球在
            // `strengthen_eligibility_threshold(0.7) > min_discount(0.5)` 时永远过不了线。
            let tier = self.tier_of(semantic_sim);
            let delta = match tier {
                BallTier::Strengthened => {
                    self.config.base_reinforcement
                        * r.path_relevance
                        * (1.0 + lit.supplement * self.config.supplement_reinforce_ratio)
                }
                // 印象球 / 最低强化球：统一最低值，不乘路径折扣
                // （它们本来就“没被真正想起”，只是路过时留下的微弱痕迹）。
                // 相对 base_reinforcement 取值，避免与达标球最小增量量级倒挂。
                BallTier::Impression | BallTier::Minimum => {
                    self.config.base_reinforcement * self.config.min_strengthen_value
                }
            };

            // 饱和增长：越接近上限，同样多的水留下的痕迹越少（docs/03 §2.4）。
            // 用“剩余空间比例^gamma”缩放，gamma=1 为线性递减，gamma→0 退化为无饱和。
            let delta = match ball_map.get(&lit.ball_id) {
                Some(ball) => {
                    let room =
                        (1.0 - ball.size_effective(now) / ball.ball_size_ceiling.max(1e-9)).max(0.0);
                    delta * room.powf(gamma)
                }
                None => delta,
            };

            if delta >= self.config.min_reinforcement_threshold {
                if tracing::enabled!(tracing::Level::DEBUG) {
                    tracing::debug!(
                        ball_id = lit.ball_id.0,
                        tier = tier.as_str(),
                        semantic_sim,
                        path_relevance = r.path_relevance,
                        depth = lit.depth,
                        delta,
                        "球结算"
                    );
                }
                tx.increment_ball_size(lit.ball_id, delta, now).await?;
                // 稳定度增长：只有**真正被想起**（强化球）才让记忆更牢固。
                // 印象球 / 路过球拿最低强化值，但不动 S——否则“没想起来”也会留下痕迹。
                // 饱和增长 + delta 驱动，已隐含“间隔效应”——连续强化时 delta 递减，
                // 间隔强化时 delta 较大，最终 S 更高（docs/03 §4.6）。
                if matches!(tier, BallTier::Strengthened)
                    && let Some(ball) = ball_map.get(&lit.ball_id)
                {
                    let new_s = grow_stability(ball.ball_stability, delta, &self.config);
                    if new_s > ball.ball_stability {
                        tx.update_ball_stability(lit.ball_id, new_s).await?;
                    }
                }
                match tier {
                    BallTier::Strengthened => strengthened += 1,
                    BallTier::Impression => impression += 1,
                    BallTier::Minimum => minimum += 1,
                }
            } else if tracing::enabled!(tracing::Level::DEBUG) {
                tracing::debug!(
                    ball_id = lit.ball_id.0,
                    tier = tier.as_str(),
                    delta,
                    threshold = self.config.min_reinforcement_threshold,
                    "增量低于阈值，跳过强化"
                );
            }
        }
        Ok((strengthened, impression, minimum))
    }

    async fn reinforce_pipes(
        &self,
        tx: &mut dyn StorageTx,
        max_relevance: &HashMap<BallId, PathRelevance>,
        lit_balls: &[LitBall],
        sub: &SubGraph,
    ) -> Result<usize, SettleError> {
        let now = self.clock.now();
        // 只强化**真正被水冲过**的管道：`via_pipe` 记录每个球是从哪条边到达的。
        // 仅凭“两端都被点亮”不够——两端可能分别经不同路径到达，
        // 它们之间这条边根本没走过，加粗它等于给“没走过的路”发通行证。
        let traversed: HashSet<PipeId> = lit_balls.iter().filter_map(|l| l.via_pipe).collect();
        let mut count = 0;
        for pipe in &sub.pipes {
            if !traversed.contains(&pipe.id) {
                continue;
            }
            let (Some(rel_a), Some(rel_b)) = (max_relevance.get(&pipe.a), max_relevance.get(&pipe.b))
            else {
                continue;
            };
            if rel_a.semantic_sim.is_none() || rel_b.semantic_sim.is_none() {
                continue;
            }
            let avg_rel = (rel_a.path_relevance + rel_b.path_relevance) / 2.0;

            if avg_rel >= self.config.min_pipe_reinforce_threshold {
                let delta = self.config.pipe_strengthen_factor * avg_rel;
                // 物化：先折到“现在”再加粗，并刷新 last_flowed（docs/07 第五层）。
                let effective = pipe.effective_thickness(now);
                let new_weight = (effective + delta).min(self.config.pipe_thickness_ceiling);
                let mut updated = pipe.clone();
                updated.pipe_base_thickness = new_weight;
                updated.last_flowed = now;
                tx.upsert_pipe(&updated).await?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// 记一次“被挡”：`blocked_count += 1`，下次有效阻力降低（docs/03 §3.5）。
    ///
    /// 只记账，不改粗细——被挡不说明这条关联没用，只说明这次水不够。
    async fn record_blocked_pipes(
        &self,
        tx: &mut dyn StorageTx,
        blocked_pipes: &[PipeId],
        sub: &SubGraph,
    ) -> Result<usize, SettleError> {
        let now = self.clock.now();
        let mut seen = HashSet::new();
        let mut count = 0;
        for id in blocked_pipes {
            if !seen.insert(*id) {
                continue;
            }
            let Some(pipe) = sub.pipes.iter().find(|p| p.id == *id) else {
                continue;
            };
            let mut updated = pipe.clone();
            updated.blocked_pass_count_stored =
                updated.blocked_pass_count_stored.saturating_add(1);
            updated.last_blocked_at = Some(now);
            tx.upsert_pipe(&updated).await?;
            count += 1;
        }
        Ok(count)
    }

    /// 注入排序只用 `semantic_sim`，**不乘路径折扣**（docs/03 §7.4）：
    /// “想不想得起来”已经由强化档位回答过了，排序只需回答“跟当前问题多相关”。
    /// 只排强化球与印象球；最低强化球不参与注入。
    fn sort_for_injection(
        &self,
        max_relevance: &HashMap<BallId, PathRelevance>,
        sub: &SubGraph,
    ) -> Vec<(BallId, f64)> {
        let now = self.clock.now();
        let a = self.config.sort_exponent_a;
        let b = self.config.sort_exponent_b;
        let ball_map: HashMap<BallId, &Ball> = sub.balls.iter().map(|b| (b.id, b)).collect();

        let mut sorted: Vec<(BallId, f64)> = max_relevance
            .iter()
            .filter_map(|(&id, r)| {
                let sim = r.semantic_sim?;
                if self.tier_of(sim) == BallTier::Minimum {
                    return None;
                }
                // score = sim^a × (size_eff/ceiling)^b
                // b=0 时退化为 sim^a，即“只看相关性”；大小已通过补充机制
                // 间接影响过流动结果，默认不再二次计权（docs/03 §7.4）。
                let size_ratio = ball_map
                    .get(&id)
                    .map(|ball| {
                        (ball.size_effective(now) / ball.ball_size_ceiling.max(1e-9)).clamp(0.0, 1.0)
                    })
                    .unwrap_or(0.0);
                let score = sim.max(0.0).powf(a) * size_ratio.powf(b);
                Some((id, score))
            })
            .collect();
        sorted.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                // 同相关性时按 id 升序：HashMap 迭代顺序不定，
                // 不 tie-break 会让相同输入产生不同注入顺序，破坏可复现性
                .then(a.0.cmp(&b.0))
        });
        sorted
    }

    fn render_injection_blocks(
        &self,
        sorted: &[(BallId, f64)],
        _lit_balls: &[LitBall],
        sub: &SubGraph,
    ) -> Vec<InjectionBlock> {
        let ball_map: HashMap<BallId, &Ball> =
            sub.balls.iter().map(|b| (b.id, b)).collect();

        let mut blocks = Vec::new();
        let mut token_budget = match self.config.injection_mode {
            InjectionMode::Token => self.config.token_budget,
            InjectionMode::BallCount => usize::MAX,
        };
        let mut ball_count = 0usize;

        for &(ball_id, rel) in sorted {
            match self.config.injection_mode {
                InjectionMode::BallCount => {
                    if ball_count >= self.config.max_balls_per_inject {
                        break;
                    }
                }
                InjectionMode::Token => {
                    if token_budget == 0 {
                        break;
                    }
                }
            }

            let ball = match ball_map.get(&ball_id) {
                Some(b) => b,
                None => continue,
            };

            let confidence = if rel >= self.config.render_clear_threshold {
                InjectionConfidence::Clear
            } else if rel >= self.config.render_deep_impression_threshold {
                InjectionConfidence::DeepImpression
            } else if rel >= self.config.render_impression_threshold {
                InjectionConfidence::Impression
            } else {
                InjectionConfidence::FaintImpression
            };

            // 预筛：按 size 粗估 token（docs/03 §7.3），避免渲染全部内容再估算。
            // 实际计数仍用字符估算（更准），预筛只做“明显装不下”的快速淘汰。
            if let InjectionMode::Token = self.config.injection_mode {
                let est = (ball.ball_stored_size * self.config.tokens_per_size).ceil() as usize;
                if est > token_budget {
                    continue;
                }
            }

            let texts: Vec<String> = ball
                .content_list
                .iter()
                .filter(|c| c.status == ContentStatus::Active)
                .map(|c| c.text.clone())
                .collect();

            if texts.is_empty() {
                continue;
            }

            let combined = texts.join("\n");
            let token_count = estimate_tokens(&combined, self.config.chars_per_token);

            if let InjectionMode::Token = self.config.injection_mode {
                if token_count > token_budget {
                    continue;
                }
                token_budget = token_budget.saturating_sub(token_count);
            }

            blocks.push(InjectionBlock {
                text: combined,
                ball_ids: vec![ball_id],
                confidence,
                token_count,
            });

            ball_count += 1;
        }

        blocks
    }
}

// ============================================================
// 工具函数
// ============================================================

/// 多源查询的均值向量（未归一化——余弦相似度自己会归一化）
fn mean_query(queries: &[&[f32]]) -> Vec<f32> {
    let Some(dim) = queries.first().map(|q| q.len()) else {
        return Vec::new();
    };
    if dim == 0 {
        return Vec::new();
    }
    let mut sum = vec![0.0f32; dim];
    let mut count = 0.0f32;
    for q in queries {
        if q.len() != dim {
            continue;
        }
        for (acc, v) in sum.iter_mut().zip(q.iter()) {
            *acc += v;
        }
        count += 1.0;
    }
    if count > 0.0 {
        for v in &mut sum {
            *v /= count;
        }
    }
    sum
}

/// 稳定度增长：`S += (S_max - S) × delta / (delta + k)`（docs/03 §4.6）
///
/// - 单调不减：S 一旦增长就不回落
/// - 饱和：越接近 `S_max` 增长越慢
/// - delta 驱动：路过球（最低强化值）几乎不增长
fn grow_stability(s: f64, delta: f64, cfg: &Config) -> f64 {
    let s_max = cfg.ball_stability_max;
    if s >= s_max || delta <= 0.0 {
        return s;
    }
    let k = cfg.ball_stability_k.max(1e-9);
    (s + (s_max - s) * delta / (delta + k)).min(s_max)
}

/// 粗略 token 估算：中文按字符数，其他字符每 `chars_per_token` 个计 1 token
fn estimate_tokens(text: &str, chars_per_token: usize) -> usize {
    let chinese_chars = text.chars().filter(|c| *c >= '\u{4e00}' && *c <= '\u{9fff}').count();
    let other_chars = text.chars().count() - chinese_chars;
    let per = chars_per_token.max(1);
    let other_tokens = if other_chars > 0 { (other_chars / per).max(1) } else { 0 };
    chinese_chars + other_tokens
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use dayu_storage::testing::{seed, temp_store};
    use dayu_testkit::*;

    #[tokio::test]
    async fn settle_engine_basic_reinforcement() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(3);
        seed(&store, balls.clone(), pipes.clone()).await;

        let sub = SubGraph {
            balls: balls.clone(),
            pipes: pipes.clone(),
        };

        let lit_balls = vec![
            LitBall {
                ball_id: BallId(0),
                water: 1.0,
                supplement: 0.5,
                depth: 0,
                via_pipe: None,
            },
            LitBall {
                ball_id: BallId(1),
                water: 0.5,
                supplement: 0.3,
                depth: 1,
                via_pipe: Some(PipeId(0)), // 经 pipe_0 从球 0 到达
            },
        ];

        let config = sample_config();
        let engine = SettleEngine::new(config);

        let query = vec![0.5; 512];
        let result = engine
            .settle(&store, &lit_balls, &sub, &[&query], &[])
            .await
            .unwrap();

        assert!(result.reinforced_balls > 0);
        assert!(result.reinforced_pipes > 0);
    }

    #[tokio::test]
    async fn settle_engine_empty_lit_balls() {
        let store = temp_store();
        let sub = SubGraph {
            balls: vec![],
            pipes: vec![],
        };
        let config = sample_config();
        let engine = SettleEngine::new(config);

        let result = engine
            .settle(&store, &[], &sub, &[&vec![0.5; 512][..]], &[])
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn max_relevance_per_ball_takes_max() {
        let config = sample_config();
        let engine = SettleEngine::new(config);
        let path_rel = vec![
            PathRelevance { ball_id: BallId(1), semantic_sim: Some(0.9), path_relevance: 0.5 },
            PathRelevance { ball_id: BallId(1), semantic_sim: Some(0.85), path_relevance: 0.8 },
            PathRelevance { ball_id: BallId(2), semantic_sim: Some(0.4), path_relevance: 0.3 },
        ];
        let max_map = engine.max_relevance_per_ball(&path_rel);
        // 取 path_relevance 最大的一条路径，语义也取自同一条（不拼接）
        assert_eq!(max_map[&BallId(1)].path_relevance, 0.8);
        assert_eq!(max_map[&BallId(1)].semantic_sim, Some(0.85));
        assert_eq!(max_map[&BallId(2)].path_relevance, 0.3);
    }

    #[test]
    fn injection_blocks_respect_token_budget() {
        let config = Arc::new(Config {
            injection_mode: InjectionMode::Token,
            token_budget: 50,
            ..Config::default()
        });
        let engine = SettleEngine::new(config);

        let sorted = vec![(BallId(1), 0.9), (BallId(2), 0.7)];
        let lit_balls = vec![
            LitBall { ball_id: BallId(1), water: 1.0, supplement: 0.5, depth: 0, via_pipe: None },
            LitBall { ball_id: BallId(2), water: 0.5, supplement: 0.3, depth: 1, via_pipe: None },
        ];

        let ball1 = sample_ball_with_content(1, 3.0, vec![0.1; 512], &["hello world"]);
        let ball2 = sample_ball_with_content(2, 2.0, vec![0.2; 512], &["foo bar baz qux"]);
        let sub = SubGraph {
            balls: vec![ball1, ball2],
            pipes: vec![],
        };

        let blocks = engine.render_injection_blocks(&sorted, &lit_balls, &sub);
        assert!(blocks.len() <= 2);
        let total: usize = blocks.iter().map(|b| b.token_count).sum();
        assert!(total <= 50);
    }

    #[test]
    fn injection_blocks_respect_ball_count() {
        let config = Arc::new(Config {
            injection_mode: InjectionMode::BallCount,
            max_balls_per_inject: 1,
            ..Config::default()
        });
        let engine = SettleEngine::new(config);

        let sorted = vec![(BallId(1), 0.9), (BallId(2), 0.7)];
        let lit_balls = vec![
            LitBall { ball_id: BallId(1), water: 1.0, supplement: 0.5, depth: 0, via_pipe: None },
            LitBall { ball_id: BallId(2), water: 0.5, supplement: 0.3, depth: 1, via_pipe: None },
        ];

        let ball1 = sample_ball_with_content(1, 3.0, vec![0.1; 512], &["hello"]);
        let ball2 = sample_ball_with_content(2, 2.0, vec![0.2; 512], &["world"]);
        let sub = SubGraph {
            balls: vec![ball1, ball2],
            pipes: vec![],
        };

        let blocks = engine.render_injection_blocks(&sorted, &lit_balls, &sub);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn estimate_tokens_chinese() {
        let text = "你好世界";
        let tokens = estimate_tokens(text, 4);
        assert_eq!(tokens, 4);
    }

    #[test]
    fn estimate_tokens_english() {
        let text = "hello world";
        let tokens = estimate_tokens(text, 4);
        assert_eq!(tokens, 2);
    }

    #[tokio::test]
    async fn settle_engine_records_metrics() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(3);
        seed(&store, balls.clone(), pipes.clone()).await;

        let sub = SubGraph {
            balls: balls.clone(),
            pipes: pipes.clone(),
        };

        let lit_balls = vec![
            LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.5, depth: 0, via_pipe: None },
            LitBall { ball_id: BallId(1), water: 0.5, supplement: 0.3, depth: 1, via_pipe: None },
        ];

        let config = sample_config();
        let recorder = Arc::new(dayu_observability::MockMetricsRecorder::new());
        let engine = SettleEngine::new(config).with_metrics(recorder.clone());

        let query = vec![0.5; 512];
        let _result = engine.settle(&store, &lit_balls, &sub, &[&query], &[]).await.unwrap();

        // 断言至少记录了 4 种不同指标
        let distinct = recorder.distinct_metric_count();
        assert!(distinct >= 4,
            "should record at least 4 distinct metrics, got {}", distinct);

        // 断言 path_supplement_loss_ratio 被记录（带 dominant 标签）
        let ratios = recorder.filter_by_name(dayu_observability::METRIC_PATH_SUPPLEMENT_LOSS_RATIO);
        assert!(!ratios.is_empty(), "path_supplement_loss_ratio should be recorded");

        // 断言 injection_token_used 被记录
        let tokens = recorder.filter_by_name(dayu_observability::METRIC_INJECTION_TOKEN_USED);
        assert!(!tokens.is_empty(), "injection_token_used should be recorded");

        // 断言 reinforced_ball_count 被记录
        let balls = recorder.filter_by_name(dayu_observability::METRIC_REINFORCED_BALL_COUNT);
        assert!(!balls.is_empty(), "reinforced_ball_count should be recorded");
    }

    #[tokio::test]
    async fn unrelated_ball_is_not_reinforced() {
        // query 与 B 正交 → 加权语境下 cos(ctx,B) ≈ 0.243 < impression_threshold(0.3) → 不强化
        let mut a = vec![0.0f32; 512];
        a[0] = 1.0;
        let mut b = vec![0.0f32; 512];
        b[1] = 1.0;

        let store = temp_store();
        let balls = vec![sample_ball(0, 1.0, a.clone()), sample_ball(1, 1.0, b.clone())];
        let pipes = vec![sample_pipe(1, 0, 1, 0.5)];
        seed(&store, balls.clone(), pipes.clone()).await;
        let sub = SubGraph { balls, pipes };

        let lit_balls = vec![LitBall {
            ball_id: BallId(1),
            water: 1.0,
            supplement: 0.0,
            depth: 1,
            via_pipe: None,
        }];
        let engine = SettleEngine::new(sample_config());

        // 旧口径 cos(query,B)=1 会强化；新口径必须拒绝
        let r = engine
            .settle(&store, &lit_balls, &sub, &[a.as_slice()], &[])
            .await
            .unwrap();
        assert_eq!(r.reinforced_balls, 0, "语义不相关的球不应被强化");
        assert_eq!(r.reinforced_pipes, 0, "语义不相关的路径不应加粗管道");
    }

    #[tokio::test]
    async fn settle_skips_dimension_mismatched_balls() {
        // 切模型场景：query 维度与球的向量维度不一致时，跳过而非报错
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(2); // 384 维球
        seed(&store, balls.clone(), pipes.clone()).await;
        let sub = SubGraph { balls, pipes };

        let lit_balls = vec![LitBall {
            ball_id: BallId(0),
            water: 1.0,
            supplement: 0.5,
            depth: 0,
            via_pipe: None,
        }];
        let engine = SettleEngine::new(sample_config());

        // 维度匹配：relevance > 0，球被强化
        let matched = engine
            .settle(&store, &lit_balls, &sub, &[&vec![0.5; 512][..]], &[])
            .await
            .unwrap();
        assert!(matched.reinforced_balls > 0, "维度匹配时应强化球");

        // 维度不匹配（256 vs 512）：不 panic，relevance 计 0 → 无强化
        let mismatched = engine
            .settle(&store, &lit_balls, &sub, &[&vec![0.5; 256]], &[])
            .await
            .expect("维度不匹配不应报错");
        assert_eq!(mismatched.reinforced_balls, 0, "维度不匹配的球不应被强化");
        assert_eq!(mismatched.reinforced_pipes, 0, "维度不匹配时管道不应被强化");
    }

    /// 第三层：结算增量必须精确等于手算值，不是“> 0”。
    ///
    /// delta = base_reinforcement × relevance × (1 + supplement × supplement_reinforce_ratio)
    /// relevance = sqrt(cos(S,B) × cos(query,B)) × hop_discount
    /// —— 不含 water：相关性只由记忆内容决定（docs/07 第二层）。
    #[tokio::test]
    async fn settle_increment_matches_hand_calculated_value() {
        let store = temp_store();
        let ball = sample_ball(0, 1.0, vec![1.0, 0.0, 0.0, 0.0]);
        let sub = SubGraph { balls: vec![ball.clone()], pipes: vec![] };
        seed(&store, vec![ball], vec![]).await;

        let lit = vec![LitBall {
            ball_id: BallId(0),
            water: 1.0,
            supplement: 0.2,
            depth: 0,
            via_pipe: None,
        }];

        let cfg = sample_config();
        SettleEngine::new(cfg.clone())
            .settle(&store, &lit, &sub, &[&[1.0, 0.0, 0.0, 0.0]], &[])
            .await
            .unwrap();

        // sim = 1（球与自身、与 query 均同向），hop_discount = 1（depth 0）
        // 饱和增长：size 1.0 / ceiling 10.0 → 剩余空间 0.9
        // delta = 0.1 × 1.0 × (1 + 0.2 × 0.5) × 0.9
        let room = 1.0 - 1.0 / 10.0;
        let after = store.get_ball(BallId(0)).await.unwrap().unwrap();
        let expected =
            1.0 + cfg.base_reinforcement * 1.0 * (1.0 + 0.2 * cfg.supplement_reinforce_ratio) * room;
        assert!(
            (after.ball_stored_size - expected).abs() < 1e-9,
            "球大小应为 {expected}，实际 {}",
            after.ball_stored_size
        );
    }

    /// 第二层理念断言：相关性只由记忆内容决定，**与水量无关**（docs/07）。
    ///
    /// 反例：relevance 曾乘 `lit.water`，导致强线索（W₀×3）下语义仅 0.25
    /// 相关的球也被判为“强化球”，`strengthen_threshold_X` 退化成魔数。
    #[tokio::test]
    async fn relevance_is_independent_of_water_amount() {
        async fn increment_with_water(water: f64) -> f64 {
            let store = temp_store();
            let ball = sample_ball(0, 1.0, vec![1.0, 0.0, 0.0, 0.0]);
            let sub = SubGraph { balls: vec![ball.clone()], pipes: vec![] };
            seed(&store, vec![ball], vec![]).await;

            let lit = vec![LitBall {
                ball_id: BallId(0),
                water,
                supplement: 0.0,
                depth: 0,
                via_pipe: None,
            }];

            SettleEngine::new(sample_config())
                .settle(&store, &lit, &sub, &[&[1.0, 0.0, 0.0, 0.0]], &[])
                .await
                .unwrap();

            store.get_ball(BallId(0)).await.unwrap().unwrap().ball_stored_size - 1.0
        }

        let tiny = increment_with_water(0.01).await;
        let huge = increment_with_water(100.0).await;

        assert!(tiny > 0.0, "水再少，语义完全相关的球也应被强化: {tiny}");
        assert!(
            (tiny - huge).abs() < 1e-12,
            "水量不应影响强化量: tiny={tiny}, huge={huge}"
        );
    }

    /// 第二层理念断言：注水再大，偏离水源的球也不会因“水多”被判为强化球。
    #[tokio::test]
    async fn large_water_does_not_promote_unrelated_ball() {
        let mut a = vec![0.0f32; 512];
        a[0] = 1.0;
        let mut b = vec![0.0f32; 512];
        b[1] = 1.0;

        let store = temp_store();
        let balls = vec![sample_ball(0, 1.0, a.clone()), sample_ball(1, 1.0, b.clone())];
        let pipes = vec![sample_pipe(1, 0, 1, 0.5)];
        seed(&store, balls.clone(), pipes.clone()).await;
        let sub = SubGraph { balls, pipes };

        // 水量再大也不能把“语义不相关”变成“相关”
        let lit = vec![LitBall {
            ball_id: BallId(1),
            water: 100.0,
            supplement: 0.0,
            depth: 1,
            via_pipe: None,
        }];

        // 加权语境会把球 1 自身染进来，但只占 α=0.2：
        // cos(ctx, B) = 0.2/sqrt(0.8²+0.2²) ≈ 0.243 < impression_threshold(0.3)
        let r = SettleEngine::new(sample_config())
            .settle(&store, &lit, &sub, &[a.as_slice()], &[])
            .await
            .unwrap();

        assert_eq!(r.reinforced_balls, 0, "水量大不能把“语义不相关”变成“相关”");
        assert_eq!(r.impression_balls, 0, "相关性不足 0.3 的球连印象球都不是");
    }

    /// 第三层：注入排序稳定——相同相关性时按 id 升序，不受 HashMap 迭代顺序影响。
    #[tokio::test]
    async fn injection_order_is_stable_for_equal_relevance() {
        let store = temp_store();
        let balls = vec![
            sample_ball_with_content(0, 1.0, vec![1.0, 0.0], &["a"]),
            sample_ball_with_content(1, 1.0, vec![1.0, 0.0], &["b"]),
            sample_ball_with_content(2, 1.0, vec![1.0, 0.0], &["c"]),
        ];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        let lit: Vec<LitBall> = (0..3)
            .map(|i| LitBall {
                ball_id: BallId(i),
                water: 1.0,
                supplement: 0.0,
                depth: 0,
                via_pipe: None,
            })
            .collect();

        let result = SettleEngine::new(sample_config())
            .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
            .await
            .unwrap();

        let ids: Vec<u64> = result
            .injection_blocks
            .iter()
            .flat_map(|b| b.ball_ids.iter().map(|id| id.0))
            .collect();
        assert_eq!(ids, vec![0, 1, 2], "相同相关性下应按 id 升序稳定排序，实际 {ids:?}");
    }

    /// 方案 A 死区回归：`depth = max_flow_depth` 且语义几乎完全匹配的球
    /// **必须**被强化——资格看 `semantic_sim`，不看 `sim × 折扣`。
    ///
    /// 反例：资格若用 `sim × 折扣`，depth 5 折扣 0.5 < 阈值 0.6，
    /// 无论语义多相关都过不了线，只能拿最低强化值。
    #[tokio::test]
    async fn deep_ball_with_high_semantic_sim_is_still_strengthened() {
        let store = temp_store();
        let origin = vec![1.0f32, 0.0];
        let ball_vec = vec![1.0f32, 0.1];
        let query = vec![1.0f32, 0.05];

        let balls = vec![
            sample_ball(0, 1.0, origin.clone()),
            sample_ball(1, 1.0, ball_vec.clone()),
        ];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        let lit = vec![LitBall {
            ball_id: BallId(1),
            water: 0.1,
            supplement: 0.0,
            depth: 5,
            via_pipe: None,
        }];

        let cfg = sample_config();
        let r = SettleEngine::new(cfg.clone())
            .settle(&store, &lit, &sub, &[query.as_slice()], &[])
            .await
            .unwrap();

        assert_eq!(r.reinforced_balls, 1, "语义达标的深球必须被强化");
        assert_eq!(r.min_reinforced_balls, 0);

        // 与实现同口径：加权语境 = (1-α)×query + α×球1嵌入归一化
        // （lit 中只有球 1，其水量占比为 1）
        let sim = {
            let alpha = cfg.flow_context_blend_alpha as f32;
            let n = ball_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            let ctx: Vec<f32> = query
                .iter()
                .zip(&ball_vec)
                .map(|(q, b)| (1.0 - alpha) * q + alpha * (b / n))
                .collect();
            dayu_domain::math::cosine_similarity(&ctx, &ball_vec).unwrap() as f64
        };
        // 饱和增长：size 1.0 / ceiling 10.0 → 剩余空间 0.9
        let room = 1.0 - 1.0 / 10.0;
        let expected = 1.0 + cfg.base_reinforcement * sim * cfg.min_discount * room;
        let after = store.get_ball(BallId(1)).await.unwrap().unwrap();
        assert!(
            (after.ball_stored_size - expected).abs() < 1e-6,
            "增量应为 sim × 折扣 = {}，实际 {}",
            expected - 1.0,
            after.ball_stored_size - 1.0
        );
    }

    /// 量级不变量：达标球的最小增量必须大于最低强化值。
    ///
    /// 否则“语义刚好达标但路径最远”的球拿到的强化，比“语义不达标只是路过”的球还少。
    #[test]
    fn minimum_reinforcement_is_below_any_strengthened_ball() {
        let cfg = sample_config();
        let weakest_strengthened = cfg.base_reinforcement
            * cfg.strengthen_eligibility_threshold
            * cfg.min_discount;
        let minimum = cfg.base_reinforcement * cfg.min_strengthen_value;

        assert!(
            minimum < weakest_strengthened,
            "最低强化值 {minimum} 不应大于达标球的最小增量 {weakest_strengthened}"
        );
    }

    /// 第二层理念断言：同等相关性下不区分激活强弱——注入顺序不受水量影响。
    ///
    /// 水量差异反映的是路径通畅度，不是回答价值（docs/07 第二层）。
    #[tokio::test]
    async fn injection_order_ignores_water_strength() {
        let store = temp_store();
        let balls = vec![
            sample_ball_with_content(0, 1.0, vec![1.0, 0.0], &["a"]),
            sample_ball_with_content(1, 1.0, vec![1.0, 0.0], &["b"]),
            sample_ball_with_content(2, 1.0, vec![1.0, 0.0], &["c"]),
        ];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        // 语义完全相同（同一 embedding、同一深度、各自为水源），
        // 只有水量相差 250 倍：id 1 被强激活，id 0 只是勉强点亮
        let waters = [0.02, 5.0, 1.0];
        let lit: Vec<LitBall> = (0..3)
            .map(|i| LitBall {
                ball_id: BallId(i),
                water: waters[i as usize],
                supplement: 0.0,
                depth: 0,
                via_pipe: None,
            })
            .collect();

        let result = SettleEngine::new(sample_config())
            .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
            .await
            .unwrap();

        let ids: Vec<u64> = result
            .injection_blocks
            .iter()
            .flat_map(|b| b.ball_ids.iter().map(|id| id.0))
            .collect();
        assert_eq!(
            ids,
            vec![0, 1, 2],
            "同等相关性下水量不应影响顺序，实际 {ids:?}"
        );
    }

    /// 三档划分（docs/03 §7.1）：语义相似度决定强化幅度与是否参与注入。
    #[tokio::test]
    async fn tiers_split_by_semantic_similarity() {
        let store = temp_store();
        let balls = vec![
            sample_ball_with_content(0, 1.0, vec![1.0, 0.0], &["强化"]),
            sample_ball_with_content(1, 1.0, vec![0.5, 0.866], &["印象"]),
            sample_ball_with_content(2, 1.0, vec![0.1, 0.995], &["路过"]),
        ];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        let lit: Vec<LitBall> = (0..3)
            .map(|i| LitBall { ball_id: BallId(i), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None })
            .collect();

        // α=0：语境退化为原始查询，sim 就是 cos(query, ball)
        let mut cfg = (*sample_config()).clone();
        cfg.flow_context_blend_alpha = 0.0;

        let r = SettleEngine::new(Arc::new(cfg))
            .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
            .await
            .unwrap();

        assert_eq!(r.reinforced_balls, 1, "cos=1.0 应为强化球");
        assert_eq!(r.impression_balls, 1, "cos=0.5 应为印象球");
        assert_eq!(r.min_reinforced_balls, 1, "cos=0.1 应为最低强化球");

        let injected: Vec<u64> = r
            .injection_blocks
            .iter()
            .flat_map(|b| b.ball_ids.iter().map(|id| id.0))
            .collect();
        assert_eq!(injected, vec![0, 1], "最低强化球不参与注入，实际 {injected:?}");
    }

    /// 排序只看语义相似度，不乘路径折扣（docs/03 §7.4）：
    /// 深球相关性更低就该排后面，哪怕它的 path_relevance 因为折扣被拉平。
    #[tokio::test]
    async fn injection_order_uses_semantic_similarity_only() {
        let store = temp_store();
        let balls = vec![
            sample_ball_with_content(0, 1.0, vec![1.0, 0.0], &["浅且相关"]),
            sample_ball_with_content(1, 1.0, vec![0.6, 0.8], &["深但一般"]),
        ];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        let lit = vec![
            LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None },
            LitBall { ball_id: BallId(1), water: 1.0, supplement: 0.0, depth: 5, via_pipe: None },
        ];

        let mut cfg = (*sample_config()).clone();
        cfg.flow_context_blend_alpha = 0.0;

        let r = SettleEngine::new(Arc::new(cfg))
            .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
            .await
            .unwrap();

        let order: Vec<u64> = r
            .injection_blocks
            .iter()
            .flat_map(|b| b.ball_ids.iter().map(|id| id.0))
            .collect();
        assert_eq!(order, vec![0, 1], "按 sim 排序（1.0 > 0.6），实际 {order:?}");
    }

    /// 加权语境 α=0 时必须与原始查询完全一致（对照基线可用）。
    #[tokio::test]
    async fn zero_alpha_context_equals_raw_query() {
        let store = temp_store();
        let balls = vec![sample_ball(0, 1.0, vec![0.6, 0.8])];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        let lit = vec![LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None }];
        let mut cfg = (*sample_config()).clone();
        cfg.flow_context_blend_alpha = 0.0;

        let engine = SettleEngine::new(Arc::new(cfg));
        let ctx = engine.build_flow_context(&lit, &HashMap::from([(BallId(0), &sub.balls[0])]), &[&[1.0, 0.0]]);
        assert_eq!(ctx, vec![1.0, 0.0], "α=0 时语境必须等于原始查询");
    }

    /// 饱和增长：越接近上限，同样的水留下的痕迹越少（docs/03 §2.4）。
    #[tokio::test]
    async fn saturation_reduces_increment_near_ceiling() {
        async fn increment_with_size(size: f64) -> f64 {
            let store = temp_store();
            let ball = sample_ball(0, size, vec![1.0, 0.0]);
            let sub = SubGraph { balls: vec![ball.clone()], pipes: vec![] };
            seed(&store, vec![ball], vec![]).await;
            let lit = vec![LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None }];
            let mut cfg = (*sample_config()).clone();
            cfg.flow_context_blend_alpha = 0.0;
            SettleEngine::new(Arc::new(cfg))
                .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
                .await
                .unwrap();
            store.get_ball(BallId(0)).await.unwrap().unwrap().ball_stored_size - size
        }

        let empty = increment_with_size(0.1).await;
        let nearly_full = increment_with_size(9.0).await;
        assert!(empty > nearly_full, "空球增量应大于近满球: {empty} vs {nearly_full}");
        assert!(nearly_full > 0.0, "近满球仍应有增量，不能硬截断: {nearly_full}");
    }

    /// 精确“走过”判据：三角形三条边、三个球全部点亮，
    /// 但只有 pipe_0 / pipe_1 被水冲过 → pipe_2 不得加粗。
    #[tokio::test]
    async fn pipe_not_traversed_is_not_reinforced() {
        let store = temp_store();
        let balls = vec![
            sample_ball(0, 1.0, vec![1.0, 0.0]),
            sample_ball(1, 1.0, vec![0.9, 0.1]),
            sample_ball(2, 1.0, vec![0.8, 0.2]),
        ];
        let pipes = vec![
            sample_pipe(0, 0, 1, 0.5),
            sample_pipe(1, 0, 2, 0.5),
            sample_pipe(2, 1, 2, 0.5), // 两端都亮，但没有水从它流过
        ];
        seed(&store, balls.clone(), pipes.clone()).await;
        let sub = SubGraph { balls, pipes };

        let lit = vec![
            LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None },
            LitBall { ball_id: BallId(1), water: 0.5, supplement: 0.0, depth: 1, via_pipe: Some(PipeId(0)) },
            LitBall { ball_id: BallId(2), water: 0.5, supplement: 0.0, depth: 1, via_pipe: Some(PipeId(1)) },
        ];

        let mut cfg = (*sample_config()).clone();
        cfg.flow_context_blend_alpha = 0.0;

        let r = SettleEngine::new(Arc::new(cfg))
            .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
            .await
            .unwrap();

        assert_eq!(
            r.reinforced_pipes, 2,
            "只有走过的 pipe_0 / pipe_1 应被强化，pipe_2 两端都亮但没走过"
        );
    }

    /// 稳定度增长：被强化后 S 变大（衰减更慢），且不超过上限（docs/03 §4.6）。
    #[tokio::test]
    async fn stability_grows_with_reinforcement_and_caps() {
        async fn reinforce_n(times: usize) -> f64 {
            let store = temp_store();
            let ball = sample_ball(0, 1.0, vec![1.0, 0.0]);
            seed(&store, vec![ball], vec![]).await;

            let lit = vec![LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None }];
            let mut cfg = (*sample_config()).clone();
            cfg.flow_context_blend_alpha = 0.0;
            let engine = SettleEngine::new(Arc::new(cfg));

            for _ in 0..times {
                // 每次结算前重读：S 增长基于存储里的当前值，不是过期快照
                let fresh = SubGraph {
                    balls: store.get_balls(&[BallId(0)]).await.unwrap(),
                    pipes: vec![],
                };
                engine.settle(&store, &lit, &fresh, &[&[1.0, 0.0]], &[]).await.unwrap();
            }
            store.get_ball(BallId(0)).await.unwrap().unwrap().ball_stability
        }

        let once = reinforce_n(1).await;
        let many = reinforce_n(200).await;

        assert!(once > 7.0, "强化一次后 S 应变大，实际 {once}");
        assert!(many > once, "多次强化应继续增长: {once} -> {many}");
        assert!(many <= 30.0 + 1e-9, "S 不得超过 S_max=30，实际 {many}");
    }

    /// 最低强化球（只是路过）几乎不增长 S——它本来就“没被真正想起”。
    #[tokio::test]
    async fn minimum_tier_barely_grows_stability() {
        let store = temp_store();
        // 球 1 与 query 正交 → 最低强化球
        let balls = vec![
            sample_ball(0, 1.0, vec![1.0, 0.0]),
            sample_ball(1, 1.0, vec![0.0, 1.0]),
        ];
        let sub = SubGraph { balls: balls.clone(), pipes: vec![] };
        seed(&store, balls, vec![]).await;

        let lit = vec![
            LitBall { ball_id: BallId(0), water: 1.0, supplement: 0.0, depth: 0, via_pipe: None },
            LitBall { ball_id: BallId(1), water: 1.0, supplement: 0.0, depth: 1, via_pipe: None },
        ];
        let mut cfg = (*sample_config()).clone();
        cfg.flow_context_blend_alpha = 0.0;

        SettleEngine::new(Arc::new(cfg))
            .settle(&store, &lit, &sub, &[&[1.0, 0.0]], &[])
            .await
            .unwrap();

        let passed_by = store.get_ball(BallId(1)).await.unwrap().unwrap().ball_stability;
        assert!(
            passed_by < 7.01,
            "路过球的 S 增长应可忽略，实际 {passed_by}"
        );
    }

    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(500))]

        /// 不变量（docs/07 第一层）：S 只增不减、永不超过 S_max、永不倒退
        #[test]
        fn stability_growth_is_monotone_and_capped(
            s0 in 0.0f64..30.0,
            delta in 0.0f64..5.0,
        ) {
            let cfg = Config::default();
            let s1 = grow_stability(s0, delta, &cfg);
            prop_assert!(s1 >= s0 - 1e-12, "S 倒退了：{} → {}（delta={}）", s0, s1, delta);
            prop_assert!(s1 <= cfg.ball_stability_max + 1e-12,
                "S 超过上限：{} > {}（s0={} delta={}）", s1, cfg.ball_stability_max, s0, delta);
            // 再强化一次仍然单调
            let s2 = grow_stability(s1, delta, &cfg);
            prop_assert!(s2 >= s1 - 1e-12, "第二次强化 S 倒退：{} → {}", s1, s2);
        }

        /// 不变量：零增量的路过球不改变 S（“没想起来”不留痕迹）
        #[test]
        fn zero_delta_never_grows_stability(s0 in 0.0f64..30.0) {
            let cfg = Config::default();
            prop_assert_eq!(grow_stability(s0, 0.0, &cfg), s0);
            prop_assert_eq!(grow_stability(s0, -1.0, &cfg), s0);
        }
    }
}
