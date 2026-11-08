//! 流动引擎
//!
//! 管理一次流动的完整生命周期：
//! 1. 选择起始球（嵌入相似度 Top-K）
//! 2. 注水（初始水量 = significance × 基础水量）
//! 3. 逐跳传播（BFS，水量衰减）
//! 4. 补充（球自身贡献水量）
//! 5. 安全阀（深度/点亮数上限）

use dayu_domain::traits::*;
use dayu_domain::types::*;
use dayu_domain::error::*;
use dayu_observability::{
    MetricsRecorder, METRIC_BLOCKED_PIPE_COUNT, METRIC_FLOW_LOSS_BY_PIPE_WEIGHT,
    METRIC_FLOW_SUPPLEMENT_BY_PIPE_WEIGHT, METRIC_FLOW_TRIGGER_RATE, METRIC_LIT_BALL_COUNT,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::loss::LossModel;

/// 按扩散区域重叠度对源做连通分组
///
/// 重叠度 = |A∩B| / |A∪B|；重叠关系有传递性（A↔B、B↔C → 三者同组）。
/// 不扩散的源（可达集只有自己或为空）不构成独立区域，与任何源视为同组。
fn group_by_overlap(
    reach: &[std::collections::HashSet<BallId>],
    threshold: f64,
) -> Vec<Vec<usize>> {
    let n = reach.len();
    if n <= 1 {
        return vec![(0..n).collect()];
    }
    let mut parent: Vec<usize> = (0..n).collect();
    for i in 0..n {
        for j in (i + 1)..n {
            let overlap = if reach[i].len() <= 1 || reach[j].len() <= 1 {
                1.0
            } else {
                let inter = reach[i].intersection(&reach[j]).count() as f64;
                let union = reach[i].union(&reach[j]).count() as f64;
                if union > 0.0 { inter / union } else { 0.0 }
            };
            if overlap >= threshold {
                let ri = find_root(&mut parent, i);
                let rj = find_root(&mut parent, j);
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }
    let mut groups: std::collections::BTreeMap<usize, Vec<usize>> =
        std::collections::BTreeMap::new();
    for i in 0..n {
        let r = find_root(&mut parent, i);
        groups.entry(r).or_default().push(i);
    }
    groups.into_values().collect()
}

/// 并查集查找（路径压缩）：用于起始球分组
fn find_root(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

/// 流动引擎
///
/// 管道粗细 w_eff 同时决定阻力和损耗——粗管好过且损耗小，细管难通过且损耗大。
/// 逐跳计算顺序：先扣损、再判断阻力。损失确定发生，不管通过与否。
pub struct FlowEngine {
    config: Arc<Config>,
    loss_model: Arc<dyn LossModel>,
    metrics: Option<Arc<dyn MetricsRecorder>>,
    clock: Arc<dyn dayu_domain::traits::Clock>,
}

/// 一次流动的完整结果
///
/// `Deref<Target = [LitBall]>` 让既有调用方（`.iter()` / `.len()` / 索引）
/// 无需改动即可继续工作；需要“被挡管道”时再取 `.blocked_pipes`。
#[derive(Debug)]
pub struct FlowOutcome {
    /// 被点亮的球（按到达顺序）
    pub lit_balls: Vec<LitBall>,
    /// 被水冲过但没通过的管道（结算时 `blocked_count += 1`）
    pub blocked_pipes: Vec<PipeId>,
}

impl std::ops::Deref for FlowOutcome {
    type Target = [LitBall];
    fn deref(&self) -> &Self::Target {
        &self.lit_balls
    }
}

impl FlowEngine {
    /// 创建流动引擎（使用默认 PipeWeightLossModel）
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            loss_model: Arc::new(crate::loss::PipeWeightLossModel),
            metrics: None,
            clock: Arc::new(dayu_domain::traits::SystemClock),
        }
    }

    /// 注入时钟（测试/回放用）
    pub fn with_clock(mut self, clock: Arc<dyn dayu_domain::traits::Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// 创建流动引擎（指定损失模型，用于对照实验）
    pub fn with_loss_model(config: Arc<Config>, loss_model: Arc<dyn LossModel>) -> Self {
        Self {
            config,
            loss_model,
            metrics: None,
            clock: Arc::new(dayu_domain::traits::SystemClock),
        }
    }

    /// 设置指标记录器
    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsRecorder>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// 执行一次流动（单输入源）
    ///
    /// 返回点亮球列表（按到达顺序），供结算引擎使用。
    pub async fn flow(
        &self,
        storage: &dyn StorageBackend,
        embedding: &[f32],
        significance: f64,
    ) -> Result<FlowOutcome, FlowError> {
        self.flow_multi(storage, &[embedding], significance).await
    }

    /// 多输入源流动：用户消息、工具结果、模型自身输出可能各自产生查询嵌入
    ///
    /// 三步（docs/02 多输入源汇合）：
    /// 1. 各源独立选起始球，合并加载子图
    /// 2. 按“扩散区域是否重叠”对源做连通分组——重叠度 = |A∩B|/|A∪B|，
    ///    低于阈值的源各自独立流动，高于阈值的并成一组
    /// 3. 总水量 `W₀ × min(1 + source_bonus×(n−1), source_cap)`，按各源最高余弦占比
    ///    分到组内，组内再按球余弦分到各起始球；同一球被多源命中时受
    ///    `multi_source_merge_cap` 约束
    ///
    /// 单源（`sources.len() == 1`）时放大系数恒为 1，退化为“命中几个球都不改变总注水量”。
    pub async fn flow_multi(
        &self,
        storage: &dyn StorageBackend,
        sources: &[&[f32]],
        significance: f64,
    ) -> Result<FlowOutcome, FlowError> {
        if sources.is_empty() {
            return Err(FlowError::NoStartingBalls);
        }

        // 1. 各源独立选起始球；空源剔除（空源不参与分组，否则会被误判为“不扩散”）
        let mut per_source: Vec<Vec<(BallId, f64)>> = Vec::with_capacity(sources.len());
        for emb in sources {
            per_source.push(self.select_starts(storage, emb).await?);
        }
        let kept: Vec<usize> = (0..sources.len())
            .filter(|i| !per_source[*i].is_empty())
            .collect();
        if kept.is_empty() {
            return Err(FlowError::NoStartingBalls);
        }
        let queries: Vec<&[f32]> = kept.iter().map(|&i| sources[i]).collect();
        let per_source: Vec<Vec<(BallId, f64)>> =
            kept.iter().map(|&i| per_source[i].clone()).collect();

        // 2. 合并加载子图
        let mut all_ids: Vec<BallId> = per_source.iter().flatten().map(|(id, _)| *id).collect();
        all_ids.sort();
        all_ids.dedup();
        let sub = storage
            .load_subgraph(&all_ids, self.config.max_flow_depth)
            .await?;

        // 3. 按扩散区域重叠度分组
        let mut adjacency: HashMap<BallId, Vec<BallId>> = HashMap::new();
        for pipe in &sub.pipes {
            adjacency.entry(pipe.a).or_default().push(pipe.b);
            adjacency.entry(pipe.b).or_default().push(pipe.a);
        }
        let reach: Vec<std::collections::HashSet<BallId>> = per_source
            .iter()
            .map(|starts| {
                starts
                    .iter()
                    .flat_map(|(id, _)| self.reachable_set(&adjacency, *id))
                    .collect()
            })
            .collect();
        let groups = group_by_overlap(&reach, self.config.multi_source_overlap_threshold);

        // 4. 总水量与各源子预算
        let n = queries.len();
        let strong_cue = per_source
            .iter()
            .any(|s| s.iter().any(|(_, c)| *c >= self.config.strong_cue_threshold));
        let cue_multiplier = if strong_cue {
            self.config.strong_cue_multiplier
        } else {
            1.0
        };
        let amplify = if n > 1 {
            (1.0 + self.config.source_bonus * (n as f64 - 1.0)).min(self.config.source_cap)
        } else {
            1.0
        };
        let total_water =
            self.config.total_water_budget * significance * cue_multiplier * amplify;

        let max_cos: Vec<f64> = per_source
            .iter()
            .map(|s| s.iter().map(|(_, c)| c.max(0.0)).fold(0.0, f64::max))
            .collect();
        let cos_sum: f64 = max_cos.iter().sum();
        let source_budget: Vec<f64> = if cos_sum > 0.0 {
            max_cos.iter().map(|c| total_water * c / cos_sum).collect()
        } else {
            vec![total_water / n as f64; n]
        };

        // 5. 每组一次流动
        let mut lit_balls: Vec<LitBall> = Vec::new();
        let mut blocked_pipes: Vec<PipeId> = Vec::new();
        for group in &groups {
            let seeds = self.build_seeds(group, &per_source, &source_budget);
            let (mut lit, mut blocked) = self.propagate(&sub, &seeds, &queries);
            lit_balls.append(&mut lit);
            blocked_pipes.append(&mut blocked);
        }

        // 6. 合并去重：组间可达集不重叠，但实际点亮集仍可能相交（水没走到的地方）
        lit_balls.sort_by_key(|l| (l.ball_id, l.depth));
        let mut deduped: Vec<LitBall> = Vec::with_capacity(lit_balls.len());
        for l in lit_balls {
            match deduped.last_mut() {
                Some(last) if last.ball_id == l.ball_id => {
                    if l.water > last.water {
                        *last = l;
                    }
                }
                _ => deduped.push(l),
            }
        }
        deduped.truncate(self.config.max_lit_ball_count);
        Ok(FlowOutcome {
            lit_balls: deduped,
            blocked_pipes,
        })
    }

    /// 组内起始球种子：各源子预算按球余弦分，同球多源命中累加并受 `merge_cap` 约束
    fn build_seeds(
        &self,
        group: &[usize],
        per_source: &[Vec<(BallId, f64)>],
        source_budget: &[f64],
    ) -> Vec<(BallId, f64)> {
        let mut merged: HashMap<BallId, (f64, f64)> = HashMap::new();
        for &si in group {
            let starts = &per_source[si];
            let budget = source_budget[si];
            let sim_sum: f64 = starts.iter().map(|(_, c)| c.max(0.0)).sum();
            for &(id, cos) in starts {
                let w = if sim_sum > 0.0 {
                    budget * cos.max(0.0) / sim_sum
                } else {
                    budget / starts.len() as f64
                };
                let e = merged.entry(id).or_insert((0.0, 0.0));
                e.0 += w;
                e.1 = e.1.max(w);
            }
        }
        let mut seeds: Vec<(BallId, f64)> = merged
            .into_iter()
            .map(|(id, (sum, max_single))| {
                (
                    id,
                    sum.min(max_single * self.config.multi_source_merge_cap),
                )
            })
            .collect();
        seeds.sort_by_key(|(id, _)| *id);
        seeds
    }

    /// 纯图可达集合（不含水量判断）：从 `start` 出发、`max_flow_depth` 跳内能到达的球
    fn reachable_set(
        &self,
        adjacency: &HashMap<BallId, Vec<BallId>>,
        start: BallId,
    ) -> std::collections::HashSet<BallId> {
        let mut seen = std::collections::HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((start, 0usize));
        while let Some((cur, d)) = queue.pop_front() {
            if !seen.insert(cur) {
                continue;
            }
            if d >= self.config.max_flow_depth {
                continue;
            }
            if let Some(ns) = adjacency.get(&cur) {
                for &nb in ns {
                    queue.push_back((nb, d + 1));
                }
            }
        }
        seen
    }

    /// 选择起始球：嵌入相似度 Top-K
    ///
    /// 两道阈值：
    /// - 绝对：`cos ≥ start_ball_threshold`（挡住完全不相关的球）
    /// - 相对：`cos ≥ 最佳命中 × start_ball_relative_threshold`（挡住“勉强相关”的球）
    ///
    /// 相对阈值是必需的：球数上万时，总会有几个球余弦超过 0.3，
    /// 只靠绝对阈值会让起始球数量随图规模增长，最终淹没整图。
    async fn select_starts(
        &self,
        storage: &dyn StorageBackend,
        embedding: &[f32],
    ) -> Result<Vec<(BallId, f64)>, FlowError> {
        let candidates = storage
            .search_by_embedding(embedding, self.config.start_top_n)
            .await?;

        let best = candidates
            .iter()
            .map(|(_, sim)| *sim as f64)
            .fold(f64::NEG_INFINITY, f64::max);
        let relative_floor = if best.is_finite() {
            best * self.config.start_ball_relative_threshold
        } else {
            f64::INFINITY
        };

        let filtered: Vec<_> = candidates
            .into_iter()
            .filter(|(_, sim)| {
                let s = *sim as f64;
                s >= self.config.start_ball_threshold && s >= relative_floor
            })
            .map(|(id, sim)| (id, sim as f64))
            .collect();

        if tracing::enabled!(tracing::Level::DEBUG) {
            tracing::debug!(
                candidates = filtered.len(),
                threshold = self.config.start_ball_threshold,
                relative_floor,
                best,
                balls = ?filtered
                    .iter()
                    .map(|(id, sim)| format!("{}@{:.3}", id.0, sim))
                    .collect::<Vec<_>>(),
                "起始球选择"
            );
        }

        Ok(filtered)
    }

    /// 把可分配水量分到各条出边。
    ///
    /// `Weighted`：按 `1 + w` 加权——粗管多分、细管少分但**保底非零**。
    /// 若按 `w` 直接分，`w = 0` 的管道分到 0，等于用分配规则把路封死，
    /// 与“阻力不是开关、强线索可突破”直接冲突。
    /// `Equal`：平均分，仅用于对照实验。
    fn split_water(&self, water_available: f64, neighbors: &[(BallId, &Pipe)]) -> Vec<f64> {
        let n = neighbors.len();
        if n == 0 {
            return Vec::new();
        }
        match self.config.split_mode {
            SplitMode::Equal => vec![water_available / n as f64; n],
            SplitMode::Weighted => {
                // 按**有效粗细**（含时间衰减）加权：久未流动的粗管会逐渐变细
                let now = self.clock.now();
                let weights: Vec<f64> = neighbors
                    .iter()
                    .map(|(_, p)| 1.0 + p.effective_thickness(now))
                    .collect();
                let total: f64 = weights.iter().sum();
                weights
                    .iter()
                    .map(|w| water_available * w / total)
                    .collect()
            }
        }
    }

    /// 注水 + BFS 传播（修正后：先扣损、再判断阻力）
    ///
    /// 补充资格只看 `cos(原始查询, 本球)`：球与当前问题相关才释放水。
    /// 不再引入“水源球”——路径上的球本来就由管道语义相连，
    /// 再用起点球做几何平均会双重限制、且让“离起点远”变成“不相关”。
    fn propagate(
        &self,
        sub: &SubGraph,
        seeds: &[(BallId, f64)],
        queries: &[&[f32]],
    ) -> (Vec<LitBall>, Vec<PipeId>) {
        let ball_map: HashMap<BallId, &Ball> =
            sub.balls.iter().map(|b| (b.id, b)).collect();

        // 本次流动的统一时间基准：衰减、分配、阻力共用同一时刻，避免中途漂移
        let now = self.clock.now();
        // 相关性：取该球与**任一输入源**的最大余弦——与任何一个输入相关都算相关。
        // 维度不匹配（切模型）的源被跳过；全部不匹配时视为 0 → 不补充。
        let relatedness = |ball: &Ball| -> f64 {
            queries
                .iter()
                .filter_map(|q| dayu_domain::math::cosine_similarity(q, &ball.embedding))
                .map(|c| f64::from(c.max(0.0)))
                .fold(0.0, f64::max)
        };

        // 构建邻接表
        let mut adjacency: HashMap<BallId, Vec<(BallId, &Pipe)>> = HashMap::new();
        for pipe in &sub.pipes {
            adjacency.entry(pipe.a).or_default().push((pipe.b, pipe));
            adjacency.entry(pipe.b).or_default().push((pipe.a, pipe));
        }

        let mut lit_balls: Vec<LitBall> = Vec::new();
        let mut blocked_pipes: Vec<PipeId> = Vec::new();
        let mut visited: HashMap<BallId, usize> = HashMap::new();
        // 球 → lit_balls 下标，供多源汇合时原地更新水量
        let mut lit_index: HashMap<BallId, usize> = HashMap::new();
        // (当前球, 水量, 深度, 水流源头球)
        let mut queue: VecDeque<(BallId, f64, usize, Option<PipeId>)> = VecDeque::new();

        // 初始注水：水量已由 `flow_multi` 按“总预算 × 源份额 × 球余弦”算好，
        // 这里只负责入队。预算如何分配、多源如何放大与封顶，全部在上层，
        // 传播阶段只做“给多少水、能流多远”。
        for &(start_id, water) in seeds {
            if !ball_map.contains_key(&start_id) {
                continue;
            }
            queue.push_back((start_id, water, 0, None));
        }

        // BFS
        while let Some((current_id, water, depth, via_pipe)) = queue.pop_front() {
            if depth > self.config.max_flow_depth {
                continue;
            }
            if lit_balls.len() >= self.config.max_lit_ball_count {
                break;
            }

            // 一个球只点亮一次：重复点亮会重复累加 supplement 与强化量。
            // 但多源汇合时，后来的水要累加到已点亮的球上——
            // 上限 = 最大单源 × merge_cap（避免线性叠加导致水量爆炸）。
            // 合并后的水不再向下游传播：各源的水已各自沿路径分发过，
            // 这里合并只影响该球自身的相关性强度（结算与注入）。
            if visited.contains_key(&current_id) {
                if let Some(&idx) = lit_index.get(&current_id) {
                    let old = lit_balls[idx].water;
                    let cap = old.max(water) * self.config.multi_source_merge_cap;
                    let merged = (old + water).min(cap);
                    if merged > old {
                        lit_balls[idx].water = merged;
                    }
                }
                continue;
            }
            visited.insert(current_id, depth);
            lit_index.insert(current_id, lit_balls.len());

            let ball = match ball_map.get(&current_id) {
                Some(b) => b,
                None => continue,
            };

            // 补充：只有与「当前输入」相关的球才释放水量（docs/03 §4.3）
            // 大小已含时间衰减，不再额外乘新鲜度
            let rel = relatedness(ball);
            let theta = self.config.supplement_threshold;
            let supplement = if rel > theta {
                // 两段式：阈值以上按超出比例释放（θ 处为 0、rel=1 处满额）。
                // 线性乘积会把 0.31 的勉强相关球也放大成可观补充，
                // 违背“补充是强相关才有的额外推力”这个语义。
                let headroom = ((rel - theta) / (1.0 - theta).max(1e-9)).clamp(0.0, 1.0);
                ball.size_effective(now) * headroom * self.config.supplement_ratio
            } else {
                0.0
            };

            lit_balls.push(LitBall {
                ball_id: current_id,
                water,
                supplement,
                depth,
                via_pipe,
            });

            if depth < self.config.max_flow_depth
                && let Some(neighbors) = adjacency.get(&current_id)
            {
                // 只向**未访问**的出边分配水量：已点亮的球不再分水（水不会逆流回刚来的地方）。
                // 否则中间球每条来路会分走一半水量，每经过一个中间球水量就腾空一半。
                let pending: Vec<(BallId, &Pipe)> = neighbors
                    .iter()
                    .filter(|(nb, _)| !visited.contains_key(nb))
                    .copied()
                    .collect();

                // 分配守恒：补充先计入可分配总量（分配阶段不产生、不消灭水），
                // 再按 split_mode 分到各条待访出边；每条管道用自己那份独立判断。
                let water_available = (water + supplement)
                    .min(water * self.config.max_supplement_ratio);
                let shares = self.split_water(water_available, &pending);

                for (&(neighbor_id, pipe), &share) in pending.iter().zip(&shares) {
                    let w_eff = pipe.effective_thickness(now);

                    // 1. 先算损失（不管是否通过，损失都发生）
                    let loss = self.loss_model.loss(w_eff, &self.config);
                    let water_after_loss = share * (1.0 - loss.get());

                    // 记录指标：按管道粗细的损失率
                    if let Some(ref m) = self.metrics {
                        m.record_histogram(
                            METRIC_FLOW_LOSS_BY_PIPE_WEIGHT,
                            loss.get(),
                            &[("w_eff_bucket", &format!("{:.1}", (w_eff * 10.0).floor() / 10.0))],
                        );
                        m.record_histogram(
                            METRIC_FLOW_SUPPLEMENT_BY_PIPE_WEIGHT,
                            supplement,
                            &[("w_eff_bucket", &format!("{:.1}", (w_eff * 10.0).floor() / 10.0))],
                        );
                    }

                    // 2. 再判断阻力
                    //    被挡过的管道门槛会降低（最多降 50%）——“试多了，路就通了”。
                    //    这是与“强线索一次给足水”互补的另一条路：不靠水量，靠次数。
                    let blocked = pipe.blocked_effective(now, self.config.blocked_decay);
                    let resistance = (1.0 / (1.0 + w_eff))
                        * (1.0
                            - self.config.blocked_discount * blocked.min(self.config.blocked_cap));

                    if water_after_loss > resistance
                        && water_after_loss >= self.config.min_water_threshold
                    {
                        queue.push_back((neighbor_id, water_after_loss, depth + 1, Some(pipe.id)));
                    } else {
                        // 不通过：这份水损失掉，不回收给其他出边（“想不起来就是白费力气”）
                        blocked_pipes.push(pipe.id);
                    }
                }
            }
        }

        // 记录流动结果指标
        if let Some(ref m) = self.metrics {
            m.record_counter(METRIC_FLOW_TRIGGER_RATE, 1, &[("status", "success")]);
            m.record_gauge(METRIC_LIT_BALL_COUNT, lit_balls.len() as f64, &[]);
            m.record_gauge(METRIC_BLOCKED_PIPE_COUNT, blocked_pipes.len() as f64, &[]);
        }

        (lit_balls, blocked_pipes)
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settle::SettleEngine;
    use dayu_storage::testing::{seed, temp_store};
    use dayu_testkit::*;

    /// 本模块绝大多数测试验证的是注水分配 / 损失 / 衰减 / 阻力，
    /// 不希望被“高余弦 → 水量 ×3”的强线索逻辑干扰，
    /// 因此默认关闭强线索（余弦上限为 1，阈值 1.1 等价于关闭）。
    /// 强线索本身由 `strong_cue_multiplies_total_water` 专门验证。
    fn sample_config() -> std::sync::Arc<Config> {
        let mut c = (*dayu_testkit::sample_config()).clone();
        c.strong_cue_threshold = 1.1;
        std::sync::Arc::new(c)
    }

    #[tokio::test]
    async fn flow_engine_chain_propagation() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(5);
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let engine = FlowEngine::new(config);

        let query = vec![0.5; 384];
        let lit = engine.flow(&store, &query, 1.0).await.unwrap();

        assert!(!lit.is_empty());
        // 至少有一个深度0的起始球（不指定具体哪个，因为相似度排序可能不确定）
        assert!(lit.iter().any(|l| l.depth == 0), "should have at least one depth-0 start ball");
    }

    #[tokio::test]
    async fn flow_engine_empty_store_returns_error() {
        let store = temp_store();
        let config = sample_config();
        let engine = FlowEngine::new(config);

        let result = engine.flow(&store, &vec![0.5; 384], 1.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn flow_engine_respects_max_depth() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(10);
        seed(&store, balls, pipes).await;

        let config = Config {
            max_flow_depth: 2,
            ..Config::default()
        };
        let engine = FlowEngine::new(Arc::new(config));

        let query = vec![0.5; 384];
        let lit = engine.flow(&store, &query, 1.0).await.unwrap();

        for l in lit.iter() {
            assert!(l.depth <= 2, "depth {} exceeds max", l.depth);
        }
    }

    #[tokio::test]
    async fn flow_engine_water_decays_with_depth() {
        // 单一起始球的粗管链：验证水量随深度单调递减。
        //
        // 注：这里用 significance = 1.0 而非更大值——排除回边后传播能力已恢复正常，
        // 参数回到语义量“这个片段有多显著”，不再需要放大来补偿分配浪费。
        // 预期值是按 `(1 + w)` 分配 + 先扣损后判阻力手工算出的，不是从实现反填。
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let mut balls = vec![sample_ball(0, 1.0, query.clone())];
        for i in 1..5 {
            // 正交向量：不会被选为起始球，relatedness = 0 → 无补充，纯靠初始水量
            balls.push(sample_ball(i, 1.0, vec![0.0, 1.0, 0.0, 0.0]));
        }
        let pipes: Vec<_> = (0..4).map(|i| sample_pipe(i, i, i + 1, 0.9)).collect();
        seed(&store, balls, pipes).await;

        let engine = FlowEngine::new(sample_config());
        let lit = engine.flow(&store, &query, 1.0).await.unwrap();

        // 逐跳：1.2 → 0.941 → 0.738 → 0.578（下一跳 0.453 < resistance 0.526，停）
        let depths: Vec<usize> = lit.iter().map(|l| l.depth).collect();
        assert_eq!(depths, vec![0, 1, 2, 3], "应点亮 depth 0..3，实际 {depths:?}");

        let waters: Vec<f64> = lit.iter().map(|l| l.water).collect();
        for pair in waters.windows(2) {
            assert!(pair[1] < pair[0], "水量应随深度递减: {waters:?}");
        }
        assert!((waters[0] - 1.0).abs() < 1e-9, "起始球到水量应为基础注水量: {waters:?}");
        // 下一跳水量证明补充已计入可分配总量：1.0 + 0.2 后再扣损
        assert!(
            (waters[1] - 1.2 * 0.784).abs() < 1e-9,
            "第二跳应为 (1.0 + 补充 0.2) × 0.784，实际 {}",
            waters[1]
        );
    }

    #[tokio::test]
    async fn flow_engine_loss_reduces_water() {
        // 用正交 one-hot 向量构造链：仅 ball_0 命中起始阈值，其余靠传播到达
        // （uniform 向量彼此 cos=1，会让所有球都成为起始球，掩盖传播行为）
        let onehot = |i: usize| {
            let mut v = vec![0.0f32; 384];
            v[i] = 1.0;
            v
        };

        let store = temp_store();
        let balls = vec![
            sample_ball(0, 1.0, onehot(0)),
            sample_ball(1, 1.0, onehot(1)),
            sample_ball(2, 1.0, onehot(2)),
        ];
        let pipes = vec![sample_pipe(1, 0, 1, 0.5), sample_pipe(2, 1, 2, 0.5)];
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let expected_base = config.total_water_budget * 1.0;
        let engine = FlowEngine::new(config);

        let lit = engine.flow(&store, &onehot(0), 1.0).await.unwrap();

        // 验证流动发生：至少有两个不同深度的点亮球
        let depths: Vec<usize> = lit.iter().map(|l| l.depth).collect();
        let max_depth = depths.iter().max().copied().unwrap_or(0);
        assert!(max_depth > 0, "flow should reach depth > 0, got max_depth={}", max_depth);
        
        // 验证损失发生：深度0的水量应为基础水量
        let d0: Vec<_> = lit.iter().filter(|l| l.depth == 0).collect();
        assert!(!d0.is_empty(), "should have depth-0 balls");
        assert!((d0[0].water - expected_base).abs() < 1e-9,
            "depth-0 water should be base_water={}, got {}", expected_base, d0[0].water);
    }

    #[tokio::test]
    async fn constant_loss_model_flow() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(3);
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let loss_model = Arc::new(crate::loss::ConstantLossModel);
        let engine = FlowEngine::with_loss_model(config, loss_model);

        let query = vec![0.5; 384];
        let lit = engine.flow(&store, &query, 1.0).await.unwrap();
        assert!(!lit.is_empty(), "constant loss model should still propagate");
    }

    #[tokio::test]
    async fn flow_engine_records_metrics() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(5);
        seed(&store, balls, pipes).await;

        let config = sample_config();
        let recorder = Arc::new(dayu_observability::MockMetricsRecorder::new());
        let engine = FlowEngine::new(config).with_metrics(recorder.clone());

        let query = vec![0.5; 384];
        let _lit = engine.flow(&store, &query, 1.0).await.unwrap();

        // 断言至少记录了 3 种不同指标
        let distinct = recorder.distinct_metric_count();
        assert!(distinct >= 3,
            "should record at least 3 distinct metrics, got {}", distinct);

        // 断言 flow_trigger_rate 被记录
        let triggers = recorder.filter_by_name(dayu_observability::METRIC_FLOW_TRIGGER_RATE);
        assert!(!triggers.is_empty(), "flow_trigger_rate should be recorded");

        // 断言 flow_loss_by_pipe_weight 被记录
        let losses = recorder.filter_by_name(dayu_observability::METRIC_FLOW_LOSS_BY_PIPE_WEIGHT);
        assert!(!losses.is_empty(), "flow_loss_by_pipe_weight should be recorded");

        // 断言 lit_ball_count 被记录
        let counts = recorder.filter_by_name(dayu_observability::METRIC_LIT_BALL_COUNT);
        assert!(!counts.is_empty(), "lit_ball_count should be recorded");
    }

    #[tokio::test]
    async fn flow_engine_respects_max_lit_balls() {
        // 安全阀：球数超限时必须截断，否则大图会爆内存
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(20);
        seed(&store, balls, pipes).await;

        let config = Config {
            max_lit_ball_count: 2,
            ..Config::default()
        };
        let engine = FlowEngine::new(Arc::new(config));

        let lit = engine.flow(&store, &vec![0.5; 384], 1.0).await.unwrap();

        assert!(!lit.is_empty(), "至少应点亮一个球");
        assert!(
            lit.len() <= 2,
            "max_lit_ball_count=2 应截断，实际点亮 {}",
            lit.len()
        );
    }

    /// 补充相关性 = sqrt(cos(水源球, 本球) × cos(query, 本球))
    /// 与水源正交的球即便被水流到达，也不释放补充
    #[tokio::test]
    async fn supplement_is_zero_when_ball_orthogonal_to_origin() {
        let store = temp_store();
        let mut a = vec![0.0f32; 384];
        a[0] = 1.0;
        let mut b = vec![0.0f32; 384];
        b[1] = 1.0;

        seed(&store, 
            vec![sample_ball(0, 1.0, a.clone()), sample_ball(1, 1.0, b)],
            vec![sample_pipe(1, 0, 1, 0.9)],
        ).await;

        // 只让 A 当起始球，保证 B 是被水流带到的
        let config = Config {
            start_top_n: 1,
            ..Config::default()
        };
        let engine = FlowEngine::new(Arc::new(config));

        let lit = engine.flow(&store, &a, 1.0).await.unwrap();

        let b_lit = lit
            .iter()
            .find(|l| l.ball_id == BallId(1))
            .expect("水流应到达 B（阻力允许）");
        assert_eq!(b_lit.supplement, 0.0, "与水源正交的球不应释放补充");
    }

    #[tokio::test]
    async fn supplement_positive_when_ball_aligned_with_origin() {
        let store = temp_store();
        let mut a = vec![0.0f32; 384];
        a[0] = 1.0;
        let mut b = vec![0.0f32; 384];
        b[0] = 0.9;
        b[1] = 0.1;

        seed(&store, 
            vec![sample_ball(0, 1.0, a.clone()), sample_ball(1, 1.0, b)],
            vec![sample_pipe(1, 0, 1, 0.9)],
        ).await;

        let config = Config {
            start_top_n: 1,
            ..Config::default()
        };
        let engine = FlowEngine::new(Arc::new(config));

        let lit = engine.flow(&store, &a, 1.0).await.unwrap();
        let b_lit = lit
            .iter()
            .find(|l| l.ball_id == BallId(1))
            .expect("水流应到达 B");

        assert!(
            b_lit.supplement > 0.0,
            "与水源同向的球应释放补充，实际 {}",
            b_lit.supplement
        );
    }

    #[tokio::test]
    async fn flow_engine_handles_self_loop_and_isolated_ball() {
        // 图结构异常：自环不应导致死循环；孤立球（无管道）不应导致错误
        let store = temp_store();
        let balls = sample_balls(4);
        let pipes = vec![
            sample_pipe(1, 0, 1, 0.5),
            sample_pipe(2, 1, 1, 0.9), // 自环
            sample_pipe(3, 1, 2, 0.5),
        ];
        seed(&store, balls, pipes).await;

        let engine = FlowEngine::new(sample_config());

        let lit = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            engine.flow(&store, &vec![0.5; 384], 1.0),
        )
        .await
        .expect("自环图导致流动超时（疑似死循环）")
        .expect("自环/孤立球不应导致流动失败");

        assert!(!lit.is_empty());

        // 同一球不应被重复点亮
        let total = lit.len();
        let unique: std::collections::HashSet<_> = lit.iter().map(|l| l.ball_id).collect();
        assert_eq!(unique.len(), total, "同一球被重复点亮: {lit:?}");
    }

    // ============================================================
    // 第二层：理念断言
    // ============================================================

    /// “阻力不是开关”：同一条极细管道，常规水量过不去，强线索放大后能过。
    ///
    /// 如果这条测试失败（强线索也过不去），说明分配/阻力规则把路封死了，
    /// 比数值错误更严重——它直接证伪“疏而不堵”。
    #[tokio::test]
    async fn thin_pipe_is_resistance_not_switch() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let near = sample_ball(0, 1.0, query.clone());
        // 正交向量：不会被选为起始球，relatedness = 0 → 无补充
        let far = sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]);
        seed(&store, vec![near, far], vec![sample_pipe(0, 0, 1, 0.02)]).await;

        let engine = FlowEngine::new(sample_config());

        // w=0.02 → resistance≈0.980，loss≈0.357；water=1.0 → 0.643 < 0.980 挡住
        let weak = engine.flow(&store, &query, 1.0).await.unwrap();
        assert_eq!(weak.len(), 1, "常规水量应被细管挡住，只点亮起始球");

        // water=3.0 → 1.930 > 0.980 通过
        let strong = engine.flow(&store, &query, 3.0).await.unwrap();
        assert!(
            strong.iter().any(|l| l.ball_id == BallId(1)),
            "强线索必须能突破同一条细管：阻力不是开关"
        );
    }

    /// “流动只点亮、结算才强化”：流动前后存储字节不变，结算后才变。
    #[tokio::test]
    async fn flow_only_lights_settle_reinforces() {
        let store = temp_store();
        let (balls, pipes) = build_chain_graph(4);
        seed(&store, balls, pipes).await;
        let query = vec![1.0; 384];
        let cfg = sample_config();
        let engine = FlowEngine::new(cfg.clone());

        let sub_before = store.load_subgraph(&[BallId(0)], 5).await.unwrap();
        let sizes_before = sorted_sizes(&sub_before);
        let weights_before = sorted_weights(&sub_before);

        let lit = engine.flow(&store, &query, 1.0).await.unwrap();

        let sub_after = store.load_subgraph(&[BallId(0)], 5).await.unwrap();
        let sizes_after = sorted_sizes(&sub_after);
        let weights_after = sorted_weights(&sub_after);

        assert_eq!(sizes_before, sizes_after, "流动阶段不得写球大小");
        assert_eq!(weights_before, weights_after, "流动阶段不得写管道粗细");

        let settle = SettleEngine::new(cfg);
        settle
            .settle(&store, &lit, &sub_after, &[query.as_slice()], &[])
            .await
            .unwrap();

        let sub_settled = store.load_subgraph(&[BallId(0)], 5).await.unwrap();
        let sizes_settled = sorted_sizes(&sub_settled);
        assert_ne!(sizes_before, sizes_settled, "结算后球大小必须变化");
    }

    /// 按 id 排序后取 (id, 值)，避免依赖子图加载顺序
    fn sorted_sizes(sub: &SubGraph) -> Vec<(u64, f64)> {
        let mut v: Vec<(u64, f64)> = sub.balls.iter().map(|b| (b.id.0, b.ball_stored_size)).collect();
        v.sort_by_key(|(id, _)| *id);
        v
    }

    fn sorted_weights(sub: &SubGraph) -> Vec<(u64, f64)> {
        let mut v: Vec<(u64, f64)> = sub.pipes.iter().map(|p| (p.id.0, p.pipe_base_thickness)).collect();
        v.sort_by_key(|(id, _)| *id);
        v
    }

    // ============================================================
    // 第四层：对照实验
    // ============================================================

    /// 赌注四：管道决定损耗必须产生行为差异。
    ///
    /// 若两种模型点亮集合完全相同，说明“管道决定损耗”在当前参数下没有实际效果
    /// ——它可能被其他机制掩盖了，这个修正是装饰。
    #[tokio::test]
    async fn pipe_weight_loss_changes_lit_set_vs_constant() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        // 星型：ball0 是唯一起始球，两条出边一细一极细
        // 分配后细管拿到约 1.36 的水——恰好落在“Constant 能过、PipeWeight 过不了”的区间
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
        ];
        let pipes = vec![sample_pipe(0, 0, 1, 0.02), sample_pipe(1, 0, 2, 0.001)];
        seed(&store, balls, pipes).await;

        let cfg = sample_config();
        let by_pipe = FlowEngine::new(cfg.clone());
        let by_constant =
            FlowEngine::with_loss_model(cfg.clone(), Arc::new(crate::loss::ConstantLossModel));

        let lit_pipe = by_pipe.flow(&store, &query, 2.5).await.unwrap();
        let lit_const = by_constant.flow(&store, &query, 2.5).await.unwrap();

        let ids = |l: &[LitBall]| l.iter().map(|x| x.ball_id.0).collect::<Vec<_>>();
        assert_ne!(
            ids(&lit_pipe),
            ids(&lit_const),
            "两种损失模型的点亮集合必须不同，否则“管道决定损耗”是装饰"
        );
        assert!(
            lit_pipe.len() < lit_const.len(),
            "细管下 PipeWeight 损耗应大于 Constant，扩散应更短"
        );
    }

    // ============================================================
    // 起始球注水分配
    // ============================================================

    /// 注水守恒：总注水量 W₀ 固定，按余弦占比分到各起始球（docs/03 §5.2）。
    ///
    /// 若每个起始球各拿一份 W₀，命中 k 个球的输入就自动获得 k 倍水量——
    /// 等于让“多义输入”凭空获得更强扩散力，这不是设计意图。
    #[tokio::test]
    async fn starting_water_is_split_by_cosine_across_starts() {
        let store = temp_store();
        let query = vec![1.0, 0.0];
        // 三个孤立球，与 query 余弦分别为 1.0 / 0.8 / 0.6
        let balls = vec![
            sample_ball(0, 1.0, vec![1.0, 0.0]),
            sample_ball(1, 1.0, vec![0.8, 0.6]),
            sample_ball(2, 1.0, vec![0.6, 0.8]),
        ];
        seed(&store, balls, vec![]).await;

        let mut config = (*sample_config()).clone();
        config.start_ball_relative_threshold = 0.0; // 本测试只验注水分配，放开相对阈值
        let total_water = config.total_water_budget; // significance = 1.0
        let engine = FlowEngine::new(std::sync::Arc::new(config));
        let lit = engine.flow(&store, &query, 1.0).await.unwrap();
        assert_eq!(lit.len(), 3, "三个孤立球都应被点亮: {lit:?}");
        let sum: f64 = lit.iter().map(|l| l.water).sum();
        assert!(
            (sum - total_water).abs() < 1e-6,
            "总注水量应恒为 W₀={total_water}，实得 {sum}"
        );

        let water_of = |id: BallId| lit.iter().find(|l| l.ball_id == id).unwrap().water;
        let cos_sum = 1.0 + 0.8 + 0.6;
        for (id, sim) in [(0u64, 1.0), (1, 0.8), (2, 0.6)] {
            let want = total_water * sim / cos_sum;
            let got = water_of(BallId(id));            assert!((got - want).abs() < 1e-6, "球 {id}: got {got}, want {want}");
        }
    }

    /// 反证：命中多个起始球不等于拿到多份水——总注水量与起始球数量无关。
    #[tokio::test]
    async fn starting_water_total_is_independent_of_start_count() {
        async fn total_water_for(n: u64) -> f64 {
            let store = temp_store();
            let query = vec![1.0, 0.0];
            let balls: Vec<_> = (0..n)
                .map(|i| sample_ball(i, 1.0, vec![1.0, 0.1 * i as f32]))
                .collect();
            seed(&store, balls, vec![]).await;
            let engine = FlowEngine::new(sample_config());
            engine
                .flow(&store, &query, 1.0)
                .await
                .unwrap()
                .iter()
                .map(|l| l.water)
                .sum()
        }

        for n in 1..=3 {
            let sum = total_water_for(n).await;
            assert!((sum - 1.0).abs() < 1e-6, "起始球 {n} 个时总注水量 {sum}，应为 1.0");
        }
    }

    // ============================================================
    // 多出边水量分配
    // ============================================================

    fn test_pipes(weights: &[f64]) -> Vec<Pipe> {
        weights
            .iter()
            .enumerate()
            .map(|(i, &w)| sample_pipe(i as u64, 0, i as u64 + 1, w))
            .collect()
    }

    /// 原则一：分配守恒——各出边分到的水量之和等于可分配总量
    #[test]
    fn split_water_is_conservative() {
        let engine = FlowEngine::new(sample_config());
        let pipes = test_pipes(&[0.9, 0.5, 0.1, 0.001]);
        let refs: Vec<(BallId, &Pipe)> = pipes.iter().map(|p| (p.b, p)).collect();

        let shares = engine.split_water(1.5, &refs);
        let sum: f64 = shares.iter().sum();

        assert_eq!(shares.len(), 4);
        assert!((sum - 1.5).abs() < 1e-9, "分配不守恒: sum={sum}");
        assert!(shares.iter().all(|s| *s >= 0.0), "出现负水量: {shares:?}");
    }

    /// 原则二：加权比例等于 `1 + w` 的比例，不是 `w` 的比例
    #[test]
    fn split_water_uses_one_plus_weight() {
        let engine = FlowEngine::new(sample_config());
        let pipes = test_pipes(&[0.9, 0.5, 0.1]);
        let refs: Vec<(BallId, &Pipe)> = pipes.iter().map(|p| (p.b, p)).collect();

        let shares = engine.split_water(4.4, &refs);

        // 权重 1.9 : 1.5 : 1.1，总和 4.5 → 4.4 总量下的精确份额
        let expected = [4.4 * 1.9 / 4.5, 4.4 * 1.5 / 4.5, 4.4 * 1.1 / 4.5];
        for (got, want) in shares.iter().zip(expected) {
            assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
        }

        // 反证：既不是按 w 分，也不是平均分
        let by_w = [4.4 * 0.9 / 1.5, 4.4 * 0.5 / 1.5, 4.4 * 0.1 / 1.5];
        assert!((shares[0] - by_w[0]).abs() > 1e-3, "退化成按 w 分配");
        assert!((shares[0] - shares[2]).abs() > 1e-3, "退化成平均分配");
    }

    /// 原则三：细管保底非零——`w = 0` 的管道仍分到正水量
    #[test]
    fn split_water_gives_zero_weight_pipe_nonzero_share() {
        let engine = FlowEngine::new(sample_config());
        let pipes = test_pipes(&[0.9, 0.0]);
        let refs: Vec<(BallId, &Pipe)> = pipes.iter().map(|p| (p.b, p)).collect();

        let shares = engine.split_water(2.0, &refs);

        assert!(shares[1] > 0.0, "w=0 的管道分到了零水，等于把路封死: {shares:?}");
        assert!(shares[0] > shares[1], "粗管应分到更多水: {shares:?}");
    }

    /// 原则六：`equal` 模式平均分配
    #[test]
    fn split_water_equal_mode_divides_evenly() {
        let mut config = (*sample_config()).clone();
        config.split_mode = SplitMode::Equal;
        let engine = FlowEngine::new(Arc::new(config));

        let pipes = test_pipes(&[0.9, 0.1, 0.001]);
        let refs: Vec<(BallId, &Pipe)> = pipes.iter().map(|p| (p.b, p)).collect();

        let shares = engine.split_water(3.0, &refs);
        assert_eq!(shares, vec![1.0, 1.0, 1.0]);
    }

    /// 原则四：先分配后判断，失败管道的水**不回收**给成功管道。
    ///
    /// 三条出边：粗管、中管能过，细管过不去。
    /// 若实现是“失败后重新分配”，粗管拿到的水会大于分配时那份——本测试会失败。
    #[tokio::test]
    async fn failed_edges_do_not_return_water_to_successful_ones() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
            sample_ball(3, 1.0, vec![0.0, 0.0, 0.0, 1.0]),
        ];
        let pipes = vec![
            sample_pipe(0, 0, 1, 0.9),
            sample_pipe(1, 0, 2, 0.5),
            sample_pipe(2, 0, 3, 0.02),
        ];
        seed(&store, balls, pipes).await;

        let cfg = sample_config();
        let engine = FlowEngine::new(cfg.clone());
        let lit = engine.flow(&store, &query, 3.0).await.unwrap();

        let ids: Vec<u64> = lit.iter().map(|l| l.ball_id.0).collect();
        assert!(ids.contains(&1), "粗管应通过: {ids:?}");
        assert!(ids.contains(&2), "中管应通过: {ids:?}");
        assert!(!ids.contains(&3), "细管应被挡住: {ids:?}");

        // 可分配总量 = water 3.0 + 起始球自身补充 0.2 = 3.2
        // 粗管份额 = 3.2 × 1.9/4.42，扣损 21.6% 后到达 ball1
        let expected = 3.2 * 1.9 / 4.42 * (1.0 - 0.216);
        let actual = lit.iter().find(|l| l.ball_id == BallId(1)).unwrap().water;
        assert!(
            (actual - expected).abs() < 1e-6,
            "粗管拿到的水量 {actual} 应等于分配份额扣损后的 {expected}——失败管道的水不得回流"
        );
    }

    /// 原则五：所有出边都过不去时，水全部耗尽，无任何下游被点亮
    #[tokio::test]
    async fn all_edges_blocked_lights_no_downstream() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
        ];
        let pipes = vec![sample_pipe(0, 0, 1, 0.001), sample_pipe(1, 0, 2, 0.001)];
        seed(&store, balls, pipes).await;

        let lit = FlowEngine::new(sample_config())
            .flow(&store, &query, 1.0)
            .await
            .unwrap();

        assert_eq!(lit.len(), 1, "全部出边被挡时只能点亮起始球: {lit:?}");
        assert_eq!(lit[0].ball_id, BallId(0));
    }

    /// 回边（指向已访问球）不参与分配：来路不消耗前进边的水量。
    ///
    /// 若实现把回边也算进分配，中间球的水量会被腰斩，每经过一个中间球损失一半——
    /// 5 球链在默认参数下将永远出不了第 1 跳。
    #[tokio::test]
    async fn visited_neighbor_does_not_consume_water() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
        ];
        let pipes = vec![sample_pipe(0, 0, 1, 0.5), sample_pipe(1, 1, 2, 0.5)];
        seed(&store, balls, pipes).await;

        let lit = FlowEngine::new(sample_config())
            .flow(&store, &query, 3.0)
            .await
            .unwrap();

        // ball1 的出边：回边（ball0，已访问）+ 前进边（ball2）
        // 前进边拿全额：(3.0 + 0.2) × 0.72 × 0.72
        let expected = 3.2 * 0.72 * 0.72;
        let actual = lit
            .iter()
            .find(|l| l.ball_id == BallId(2))
            .expect("ball2 应被点亮")
            .water;

        assert!(
            (actual - expected).abs() < 1e-6,
            "前进边水量 {actual} 应等于 {expected}——回边不得分走一半"
        );
    }

    /// 死路：球的所有出边都已访问时，水全部损失，不得回流。
    ///
    /// 三角环 0—1—2—0：ball2 的两条出边（0 和 1）都已被访问，
    /// 它必须成为死路——不能因为“没路可走”就把水分回去或重复点亮。
    #[tokio::test]
    async fn ball_with_all_neighbors_visited_is_dead_end() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
        ];
        let pipes = vec![
            sample_pipe(0, 0, 1, 1.0),
            sample_pipe(1, 1, 2, 1.0),
            sample_pipe(2, 2, 0, 1.0),
        ];
        seed(&store, balls, pipes).await;

        let lit = FlowEngine::new(sample_config())
            .flow(&store, &query, 2.0)
            .await
            .unwrap();

        // 三个球各点亮一次，不因环路重复
        let ids: Vec<u64> = lit.iter().map(|l| l.ball_id.0).collect();
        assert_eq!(ids, vec![0, 1, 2], "环图应各点亮一次: {ids:?}");

        // 多源汇合：ball2 先由 ball0 直达 0.88，后由 ball1 汇入 0.704，
        // 累加受上限约束 → min(0.88+0.704, max(0.88,0.704)×1.5) = 1.32
        let w2 = lit.iter().find(|l| l.ball_id == BallId(2)).unwrap().water;
        assert!(
            (w2 - 1.32).abs() < 1e-9,
            "ball2 水量 {w2} 应为多源汇合后的上限值 1.32（最大单源 0.88 × 1.5）"
        );
    }

    /// 原则三（端到端）：分配后细管分到的最少份额，在总水量足够大时仍能通过阻力。
    ///
    /// 这是“分配不封死细管”的理念测试——若细管永远过不去，
    /// “阻力不是开关、强线索可突破”就被分配规则架空了。
    #[tokio::test]
    async fn thinnest_edge_still_breakable_with_enough_water() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
            sample_ball(3, 1.0, vec![0.0, 0.0, 0.0, 1.0]),
        ];
        let pipes = vec![
            sample_pipe(0, 0, 1, 0.9),
            sample_pipe(1, 0, 2, 0.5),
            sample_pipe(2, 0, 3, 0.001), // 几乎为零的管道，分配份额最小
        ];
        seed(&store, balls, pipes).await;
        let engine = FlowEngine::new(sample_config());

        // 中等水量：细管份额 1.18 → 扣损 0.757 < resistance 0.999，过不去
        let mid = engine.flow(&store, &query, 5.0).await.unwrap();
        assert!(
            !mid.iter().any(|l| l.ball_id == BallId(3)),
            "中等水量下最细管道应被挡住: {:?}",
            mid.iter().map(|l| l.ball_id).collect::<Vec<_>>()
        );

        // 强线索：细管份额 4.59 → 扣损 2.94 > 0.999，必须通过
        let strong = engine.flow(&store, &query, 20.0).await.unwrap();
        assert!(
            strong.iter().any(|l| l.ball_id == BallId(3)),
            "强线索下最细管道必须能突破，否则分配规则把路封死了"
        );
    }

    /// 原则六（端到端）：`Weighted` 与 `Equal` 必须产生不同的点亮集合。
    ///
    /// 若两者完全一致，说明分配模式在当前参数下不产生实质影响——
    /// 该结果允许存在，但必须被记录，不能默认“加了这个开关就等于有了效果”。
    #[tokio::test]
    async fn weighted_and_equal_split_produce_different_lit_sets() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, query.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, vec![0.0, 0.0, 1.0, 0.0]),
            sample_ball(3, 1.0, vec![0.0, 0.0, 0.0, 1.0]),
        ];
        let pipes = vec![
            sample_pipe(0, 0, 1, 0.9),
            sample_pipe(1, 0, 2, 0.5),
            sample_pipe(2, 0, 3, 0.02),
        ];
        seed(&store, balls, pipes).await;

        let mut equal_cfg = (*sample_config()).clone();
        equal_cfg.split_mode = SplitMode::Equal;

        let weighted = FlowEngine::new(sample_config())
            .flow(&store, &query, 5.0)
            .await
            .unwrap();
        let equal = FlowEngine::new(Arc::new(equal_cfg))
            .flow(&store, &query, 5.0)
            .await
            .unwrap();

        let ids = |l: &[LitBall]| l.iter().map(|x| x.ball_id.0).collect::<Vec<_>>();
        assert_ne!(
            ids(&weighted),
            ids(&equal),
            "两种分配模式的点亮集合必须不同，否则 split_mode 是装饰"
        );
        // 平均分配下细管分到 1/3（1.73），加权下只分到 1.02/4.42（1.20）
        assert!(
            equal.len() > weighted.len(),
            "平均分配让细管分到更多水，应点亮更多球: weighted={:?} equal={:?}",
            ids(&weighted),
            ids(&equal)
        );
    }

    /// 第二层“补充条件”第三组：同一拓扑、同一相关性，只改球大小。
    ///
    /// 小球 → 补充 < 当跳损失 → 水量衰减；
    /// 大球 → 补充 > 当跳损失 → 水量增长。
    #[tokio::test]
    async fn supplement_beats_loss_only_for_large_ball() {
        async fn flow_with_size(size: f64) -> Vec<(u64, f64)> {
            let store = temp_store();
            let query = vec![1.0, 0.0, 0.0];
            // 球 0 cos=0.9、球 2 cos=0.8（高于补充阈值 0.45）、球 3 cos=0
            let balls = vec![
                sample_ball(0, 1.0, vec![0.9, 0.436, 0.0]),
                sample_ball(2, size, vec![0.8, 0.6, 0.0]),
                sample_ball(3, 1.0, vec![0.0, 0.0, 1.0]),
            ];
            let pipes = vec![sample_pipe(0, 0, 2, 0.5), sample_pipe(1, 2, 3, 0.5)];
            seed(&store, balls, pipes).await;

            // 只让球 0 成为起始球：球 2 够格补充但被 Top-1 挤掉，
            // 它的补充量大小直接决定水流能否继续到球 3
            let mut cfg = (*sample_config()).clone();
            cfg.start_top_n = 1;

            FlowEngine::new(Arc::new(cfg))
                .flow(&store, &query, 1.0)
                .await
                .unwrap()
                .iter()
                .map(|l| (l.ball_id.0, l.water))
                .collect()
        }

        let small = flow_with_size(0.1).await;
        let large = flow_with_size(8.0).await;

        let w = |v: &[(u64, f64)], id: u64| v.iter().find(|(i, _)| *i == id).map(|(_, w)| *w);

        // 小球：补充 0.011 < 当跳损失 0.24，扣损后 0.63 跌破 resistance 0.667 → 流动停止
        assert!(
            w(&small, 3).is_none(),
            "小球补充不足以抵消损失，水流应停止在 ball2: {small:?}"
        );
        assert!(
            w(&small, 2).unwrap() > w(&small, 3).unwrap_or(0.0),
            "小球场景 ball2 自身仍应被点亮: {small:?}"
        );

        // 大球：补充 0.842 > 当跳损失 0.24，扣损后 1.23 > resistance → 继续扩散
        assert!(
            w(&large, 3).unwrap_or(-1.0) > w(&large, 2).unwrap_or(0.0),
            "大球补充超过损失，水量应增长: {large:?}"
        );
    }

    /// 第三层：起始球选择的精确集合。
    ///
    /// 四个球与 query 的余弦分别为 0.9 / 0.7 / 0.5 / 0.2：
    /// 0.2 低于 start_ball_threshold(0.3) 必须被排除，
    /// 其余按 Top-3 全部入选。
    #[tokio::test]
    async fn start_ball_selection_matches_expected_set() {
        let store = temp_store();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let mk = |id: u64, cos: f64| {
            let sin = (1.0 - cos * cos).sqrt() as f32;
            sample_ball(id, 1.0, vec![cos as f32, sin, 0.0, 0.0])
        };
        seed(&store, 
            vec![mk(0, 0.9), mk(1, 0.7), mk(2, 0.5), mk(3, 0.2)],
            vec![],
        ).await;

        let mut config = (*sample_config()).clone();
        config.start_ball_relative_threshold = 0.0; // 本测试只验绝对阈值与 Top-K
        let lit = FlowEngine::new(std::sync::Arc::new(config))
            .flow(&store, &query, 1.0)
            .await
            .unwrap();

        let mut ids: Vec<u64> = lit.iter().map(|l| l.ball_id.0).collect();
        ids.sort();
        assert_eq!(ids, vec![0, 1, 2], "起始球集合应为余弦 0.9/0.7/0.5 的三个，实际 {ids:?}");
        assert!(lit.iter().all(|l| l.depth == 0), "无管道时只应点亮起始球");
    }

    /// 强线索：查询与起始球余弦 ≥ 阈值时，总注水量放大 `strong_cue_multiplier` 倍。
    ///
    /// 判据是“这句话和某颗球像不像”（指代），不是“这句话重不重要”。
    #[tokio::test]
    async fn strong_cue_multiplies_total_water() {
        async fn start_water(threshold: f64) -> f64 {
            let store = temp_store();
            let ball = sample_ball(0, 1.0, vec![0.5; 384]);
            seed(&store, vec![ball], vec![]).await;

            let mut cfg = (*dayu_testkit::sample_config()).clone();
            cfg.strong_cue_threshold = threshold;

            let lit = FlowEngine::new(Arc::new(cfg))
                .flow(&store, &vec![0.5; 384], 1.0)
                .await
                .unwrap();
            // 起始球（depth 0）拿到的水量之和 = 总注水量
            lit.iter().filter(|l| l.depth == 0).map(|l| l.water).sum()
        }

        let off = start_water(1.1).await;
        let on = start_water(0.8).await;
        assert!((off - 1.0).abs() < 1e-9, "未触发强线索时应为 W₀=1，实际 {off}");
        assert!((on - 3.0).abs() < 1e-9, "触发强线索时应为 W₀×3=3，实际 {on}");
    }

    // ============================================================
    // 多源汇合
    // ============================================================

    /// 扩散区域重叠的源并成一组，共享放大后的总预算
    #[tokio::test]
    async fn overlapping_sources_share_one_water_budget() {
        let store = temp_store();
        // 两个源各自命中一个孤立球：都不扩散 → 视为同一片水域
        let qa = vec![1.0, 0.0];
        let qb = vec![0.0, 1.0];
        let balls = vec![
            sample_ball(0, 1.0, qa.clone()),
            sample_ball(1, 1.0, qb.clone()),
        ];
        seed(&store, balls, vec![]).await;

        let lit = FlowEngine::new(sample_config())
            .flow_multi(&store, &[&qa, &qb], 1.0)
            .await
            .unwrap();

        // 两源 → 放大系数 min(1 + 0.2×1, 2.0) = 1.2，同组共享
        let sum: f64 = lit.iter().filter(|l| l.depth == 0).map(|l| l.water).sum();
        assert!(
            (sum - 1.2).abs() < 1e-6,
            "重叠的源应共享一份放大后预算 1.2，实际 {sum}"
        );
    }

    /// 扩散区域不相交的源各自独立流动，预算按源份额分开
    #[tokio::test]
    async fn disjoint_sources_get_independent_water_budgets() {
        let store = temp_store();
        let qa = vec![1.0, 0.0, 0.0, 0.0];
        let qb = vec![0.0, 0.0, 1.0, 0.0];
        let balls = vec![
            sample_ball(0, 1.0, qa.clone()),
            sample_ball(1, 1.0, vec![0.0, 1.0, 0.0, 0.0]),
            sample_ball(2, 1.0, qb.clone()),
            sample_ball(3, 1.0, vec![0.0, 0.0, 0.0, 1.0]),
        ];
        let pipes = vec![sample_pipe(0, 0, 1, 0.9), sample_pipe(1, 2, 3, 0.9)];
        seed(&store, balls, pipes).await;

        let lit = FlowEngine::new(sample_config())
            .flow_multi(&store, &[&qa, &qb], 1.0)
            .await
            .unwrap();

        // 两个不相交区域 → 两组，总预算 1.2 按源份额各 0.6
        let sum: f64 = lit.iter().filter(|l| l.depth == 0).map(|l| l.water).sum();
        assert!(
            (sum - 1.2).abs() < 1e-6,
            "不相交的源总注水应为 1.2（各 0.6），实际 {sum}"
        );
        assert!(lit.iter().any(|l| l.ball_id == BallId(1)), "区域 A 应扩散");
        assert!(lit.iter().any(|l| l.ball_id == BallId(3)), "区域 B 应扩散");
    }

    /// `source_bonus = 0` 时多源严格守恒：总注水等于单源 W₀
    #[tokio::test]
    async fn zero_source_bonus_keeps_total_water_conserved() {
        let store = temp_store();
        let qa = vec![1.0, 0.0];
        let qb = vec![0.0, 1.0];
        seed(&store, 
            vec![
                sample_ball(0, 1.0, qa.clone()),
                sample_ball(1, 1.0, qb.clone()),
            ],
            vec![],
        ).await;

        let mut cfg = (*sample_config()).clone();
        cfg.source_bonus = 0.0;

        let lit = FlowEngine::new(Arc::new(cfg))
            .flow_multi(&store, &[&qa, &qb], 1.0)
            .await
            .unwrap();

        let sum: f64 = lit.iter().filter(|l| l.depth == 0).map(|l| l.water).sum();
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "source_bonus=0 时应严格守恒为 1.0，实际 {sum}"
        );
    }

    /// 强线索只放大总水量，不改变分配比例：两个等高余弦起始球仍各拿一半。
    #[tokio::test]
    async fn strong_cue_preserves_split_ratio() {
        let store = temp_store();
        let balls = vec![
            sample_ball(0, 1.0, vec![0.5; 384]),
            sample_ball(1, 1.0, vec![0.5; 384]),
        ];
        seed(&store, balls, vec![]).await;

        let mut cfg = (*dayu_testkit::sample_config()).clone();
        cfg.strong_cue_threshold = 0.8;

        let lit = FlowEngine::new(Arc::new(cfg))
            .flow(&store, &vec![0.5; 384], 1.0)
            .await
            .unwrap();
        let starts: Vec<f64> = lit.iter().filter(|l| l.depth == 0).map(|l| l.water).collect();
        assert_eq!(starts.len(), 2);
        assert!((starts[0] - 1.5).abs() < 1e-9, "总水 3、两球等权应各 1.5: {starts:?}");
        assert!((starts[1] - 1.5).abs() < 1e-9, "总水 3、两球等权应各 1.5: {starts:?}");
    }

    /// 被挡记账：细管第一次过不去 → 结算后 `blocked_count = 1`；
    /// 连续被挡 5 次后，同样的水量应该能通过（阻力最多降 50%）。
    #[tokio::test]
    async fn blocked_pipe_is_counted_and_softens_resistance() {
        let balls = vec![
            sample_ball(0, 1.0, vec![0.5; 384]),
            sample_ball(1, 1.0, vec![0.5; 384]),
        ];
        let pipes = vec![sample_pipe(0, 0, 1, 0.02)]; // resistance ≈ 0.98
        let store = temp_store();
        seed(&store, balls.clone(), pipes.clone()).await;

        let mut cfg = (*dayu_testkit::sample_config()).clone();
        cfg.strong_cue_threshold = 1.1; // 关闭强线索，只测阻力
        cfg.start_top_n = 1; // 只让球 0 成为起始球，球 1 必须过管道才能到达
        cfg.supplement_threshold = 0.99; // 关闭补充，只测阻力
        let engine = FlowEngine::new(Arc::new(cfg.clone()));
        let settle = SettleEngine::new(Arc::new(cfg));

        let query = vec![0.5; 384];
        let sub = SubGraph { balls: balls.clone(), pipes: pipes.clone() };

        // 第 1 次：细管挡住
        let first = engine.flow(&store, &query, 1.0).await.unwrap();
        assert_eq!(first.lit_balls.len(), 1, "细管应挡住下游: {:?}", first.lit_balls);
        assert_eq!(first.blocked_pipes, vec![PipeId(0)], "应记录被挡管道");

        settle
            .settle(&store, &first.lit_balls, &sub, &[query.as_slice()], &first.blocked_pipes)
            .await
            .unwrap();
        let after_one = store.load_subgraph(&[BallId(0)], 1).await.unwrap();
        assert_eq!(
            after_one.pipes[0].blocked_pass_count_stored, 1,
            "结算后应记一次被挡"
        );

        // 再挡 4 次（共 5 次）→ 阻力降到 0.98 × (1 - 0.1×5) = 0.49
        for _ in 0..4 {
            let sub_now = store.load_subgraph(&[BallId(0)], 1).await.unwrap();
            let out = engine.flow(&store, &query, 1.0).await.unwrap();
            settle
                .settle(&store, &out.lit_balls, &sub_now, &[query.as_slice()], &out.blocked_pipes)
                .await
                .unwrap();
        }

        let last = engine.flow(&store, &query, 1.0).await.unwrap();
        assert_eq!(
            last.lit_balls.len(), 2,
            "被挡 5 次后阻力降 50%，同样水量应能通过: {:?}",
            last.lit_balls
        );
    }
}
