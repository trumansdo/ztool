//! 系统配置

use serde::{Deserialize, Serialize};
use super::enums::{InjectionMode, SplitMode};

/// 系统配置（流动 + 结算 + 注入参数）
///
/// `#[serde(default)]`：任何缺失字段均回落到 `Default`，
/// 新增配置项不会让旧配置文件加载失败。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // --- 流动参数 ---
    /// 最大流动深度（跳数）
    pub max_flow_depth: usize,
    /// 最大点亮球数（安全阀）
    pub max_lit_ball_count: usize,
    /// 起始球最大数量
    pub start_top_n: usize,
    /// 起始球相似度阈值
    pub start_ball_threshold: f64,
    /// 起始球相似度下限（相对值）：`cos ≥ 最佳命中 × 此值` 才可作为起始球
    ///
    /// 绝对阈值 `start_ball_threshold` 在球数上万时失效——总会有几个球
    /// 余弦超过 0.3，导致起始球数量与图规模正相关。相对阈值让起始球
    /// 数量只取决于「与最佳命中的接近程度」，与图规模无关。
    pub start_ball_relative_threshold: f64,
    /// 起始球去重阈值：候选与已选起始球的余弦 ≥ 此值时视为同一条记忆，跳过
    ///
    /// 为什么需要：`start_top_n` 的预算会被「同一句话的不同写法」吃光。
    /// 实测（Qwen3-Embedding-4B）：加口语前缀的副本 0.969，口语/书面同义 0.976，
    /// 而**语义反转**（喜欢/不喜欢）也有 0.828 —— 余弦分辨不了否定。
    /// 所以阈值必须卡在 0.95 附近：只去掉肉眼可辨的同一句，同义改写交给离线合并。
    ///
    /// `0.0` 关闭去重。
    pub start_dedupe_threshold: f64,
    /// 基础注水量
    ///
    /// 注意：注水量守恒——总量固定按余弦分给各起始球。
    /// 起始球去重后球数减少，单个起始球分到的水会变多，扩散会变强，
    /// 因此调整 `start_dedupe_threshold` 时需同步重调此值。
    pub total_water_budget: f64,
    /// 球自身补充比例
    pub supplement_ratio: f64,
    /// 补充上限：补充量 ≤ 扣损前水量 × 此值
    pub max_supplement_ratio: f64,
    /// 补充资格阈值：`cos(原始查询, 球)` 超过此值才释放补充（流动阶段逐跳用原始查询嵌入）
    pub supplement_threshold: f64,
    /// 多出边水量分配模式：`Weighted`（按 `1 + w` 加权，默认）/ `Equal`（平均，仅对照实验）
    pub split_mode: SplitMode,
    /// 路径长度折扣：每跳衰减比例
    pub discount_per_hop: f64,
    /// 路径长度折扣下限（最远路径也保留此比例）
    pub min_discount: f64,
    /// 强线索阈值（docs/03 §5.4）
    ///
    /// **尚未接入**：需要“用户明确指向某段记忆”这一独立信号，
    /// 当前 API 只有 `significance`（记忆价值），语义不同，不能复用。
    /// 待判别器/上层提供线索信号后再接入 `propagate` 的注水段。
    pub strong_cue_threshold: f64,
    /// 强线索放大倍数（同上，尚未接入）
    pub strong_cue_multiplier: f64,
    /// 注入排序指数 a：`score = sim^a × (size_eff/ceiling)^b`（docs/03 §7.4）
    pub sort_exponent_a: f64,
    /// 注入排序指数 b：默认 0（只看相关性，大小已通过补充影响流动）
    pub sort_exponent_b: f64,
    /// 饱和增长指数 gamma：增量 × (1 - size/ceiling)^gamma（docs/03 §2.4）
    pub size_saturation_gamma: f64,
    /// 被挡一次，下次有效阻力降低的比例（docs/03 §3.5）
    pub blocked_discount: f64,
    /// 被挡次数的计入上限（最多降低 50% 阻力）
    pub blocked_cap: f64,
    /// 被挡次数衰减时间常数（天）：长期不再被挡则逐渐遗忘
    pub blocked_decay: f64,
    /// 球稳定度上限 S_max（天）：避免老记忆永不衰减
    pub ball_stability_max: f64,
    /// 球稳定度增长系数 k：`S += (S_max - S) × delta / (delta + k)`（docs/03 §4.6）
    ///
    /// k 越大增长越慢。没有真实数据时取保守值 1.0。
    pub ball_stability_k: f64,
    /// token 估算：每多少个非中文字符计 1 token
    pub chars_per_token: usize,
    /// Token 粗估系数：`est_tokens = ball_stored_size × tokens_per_size`（docs/03 §7.3）
    ///
    /// 只用于注入前的快速预筛（避免渲染全部内容再估算），
    /// 实际 token 计数仍按字符估算，更准。
    pub tokens_per_size: f64,
    /// 管道粗细上限（强化饱和值）
    pub pipe_thickness_ceiling: f64,
    /// 最小水量阈值（低于此值停止传播）
    pub min_water_threshold: f64,

    // --- 损失参数（管道双重角色：阻力+损耗） ---
    /// 粗管基准损失率（w_eff=1 时），默认 0.2
    pub base_loss: f64,
    /// 细管额外损耗系数，默认 0.8
    pub loss_pipe_factor: f64,
    /// 损失率上限，默认 0.8
    pub loss_max: f64,

    // --- 结算参数 ---
    /// 结算加权语境系数 α：`语境 = (1-α)×原始查询 + α×路径球水量加权平均`
    ///
    /// 上限 0.3（原始查询至少占 70%）；`0.0` 退化为原始查询、可作对照基线。
    /// 只影响结算语义相似度，不影响起始球选择与流动逐跳判定。
    pub flow_context_blend_alpha: f64,
    /// 强化球下限：语义相似度达到此值才判为“强化球”
    ///
    /// 作用在 `semantic_sim`（语义相似度）上，**不含跳数折扣**——
    /// 资格由语义决定，路径长度只影响强化幅度（docs/03 §7.1）。
    /// 介于 `impression_threshold` 与此值之间的为**印象球**（拿最低强化值但参与注入）。
    pub strengthen_eligibility_threshold: f64,
    /// 最低强化球强化值，**相对 `base_reinforcement` 的比例**（默认 0.05）
    ///
    /// 必须是相对量：绝对量会与达标球的最小增量（`base × 阈值 × min_discount`）
    /// 量级倒挂——`base_reinforcement = 0.1` 时，最深处的强相关球只能拿到 0.03，
    /// 而路过的弱相关球拿 0.05，弱相关反而被强化得更多。
    pub min_strengthen_value: f64,
    /// 基础强化量
    pub base_reinforcement: f64,
    /// 补充强化比例
    pub supplement_reinforce_ratio: f64,
    /// 最小强化阈值
    pub min_reinforcement_threshold: f64,
    /// 基础管道强化量
    pub pipe_strengthen_factor: f64,
    /// 最小管道强化阈值
    pub min_pipe_reinforce_threshold: f64,

    // --- 嵌入参数 ---
    /// 嵌入后端选择：`"onnx"` / `"http"` / `"test"`
    ///
    /// 不再有隐式回退：选中的后端不可用就直接报错。
    pub embedding_backend: String,
    /// 期望的嵌入向量维度
    ///
    /// - `None`（默认）：以模型实际输出维度为准，不校验（推荐）
    /// - `Some(n)`：锁定维度，后端不匹配时报错
    pub embedding_dim: Option<usize>,
    /// 嵌入模型路径（ONNX 文件，`onnx` 后端用）
    pub embedding_model_path: String,
    /// 嵌入分词器路径（tokenizer.json，`onnx` 后端用）
    pub embedding_tokenizer_path: String,
    /// HTTP 嵌入服务地址（`http` 后端用，不含尾部 `/embeddings`）
    pub embedding_api_base: String,
    /// HTTP 嵌入服务密钥（`http` 后端用）
    pub embedding_api_key: String,
    /// HTTP 嵌入模型名（`http` 后端用，如 `Qwen/Qwen3-Embedding-4B`）
    pub embedding_api_model: String,
    /// 注入预算模式
    pub injection_mode: InjectionMode,
    /// Token 预算上限
    pub token_budget: usize,
    /// 球数上限
    pub max_balls_per_inject: usize,
    /// 清晰档下限（`sim ≥` 此值为清晰记忆）
    pub render_clear_threshold: f64,
    /// 印象档·深下限（0.6–0.7：有比较深的印象）
    pub render_deep_impression_threshold: f64,
    /// 印象档·中下限（0.5–0.6：有印象）
    pub render_impression_threshold: f64,
    /// 印象球下限（低于它属最低强化球、**不注入**）
    pub impression_threshold: f64,

    // --- 归档参数 ---
    /// 归档：超过这么多天未被注水才可能归档
    pub archive_inactive_days: f64,
    /// 归档：有效大小低于此值才可能归档
    pub archive_size_threshold: f64,
    /// 活跃球数上限：超过则按有效大小升序强制归档
    pub max_active_balls: usize,

    // --- 多源汇合参数 ---
    /// 起始球分组重叠阈值：两源扩散区域的 Jaccard 重叠度 ≥ 此值则并为一组
    ///
    /// 重叠度 = |A∩B| / |A∪B|，A/B 为各自在子图上的可达球集合（不涉及水量）。
    /// 阈值过低会把不相干的区域并成一组，过高则永远不合并。
    pub multi_source_overlap_threshold: f64,
    /// 同球多源命中时的水量上限系数：`min(各源之和, 最大单源 × 此值)`
    ///
    /// 不设上限则多源命中会让水量爆炸；上限系数保证“多源确实更强，但不是线性叠加”。
    pub multi_source_merge_cap: f64,
    /// 多输入源的总水量放大系数：`W₀ × min(1 + source_bonus × (n−1), source_cap)`
    ///
    /// 多个线索共同指向同一批球是强信号，值得比单源略强，但不该线性叠加。
    /// `0.0` 关闭放大（严格守恒），用于对照实验。
    pub source_bonus: f64,
    /// 多输入源总水量的放大上限
    pub source_cap: f64,

    // --- 存储参数 ---
    /// 数据库文件路径
    ///
    /// 优先级：`init(configToml, dbPath)` 的 `dbPath` 参数 > 本字段 > `./dayu-mem.tdb`。
    /// 测试环境统一指向 `test/` 下，避免污染仓库根。
    pub db_path: Option<String>,

    // --- 可观测性参数 ---
    /// 日志与指标输出目录（相对进程工作目录或绝对路径）
    pub log_dir: String,
    /// 控制台与主日志级别：`trace` / `debug` / `info` / `warn` / `error`
    pub log_level: String,
    /// `dayu.log` 保留天数（按天轮转）
    pub log_retention_days: usize,
    /// 是否启用指标采集（`metrics.jsonl`）
    pub metrics_enabled: bool,
    /// `metrics.jsonl` 保留天数
    pub metrics_retention_days: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_flow_depth: 5,
            max_lit_ball_count: 50,
            start_top_n: 3,
            start_ball_threshold: 0.3,
            start_ball_relative_threshold: 0.85,
            start_dedupe_threshold: 0.95,
            total_water_budget: 1.0,
            supplement_ratio: 0.2,
            max_supplement_ratio: 2.0,
            supplement_threshold: 0.45,
            split_mode: SplitMode::Weighted,
            discount_per_hop: 0.1,
            min_discount: 0.5,
            strong_cue_threshold: 0.8,
            strong_cue_multiplier: 3.0,
            sort_exponent_a: 1.0,
            sort_exponent_b: 0.0,
            size_saturation_gamma: 1.0,
            blocked_discount: 0.1,
            blocked_cap: 5.0,
            blocked_decay: 7.0,
            ball_stability_max: 30.0,
            ball_stability_k: 1.0,
            chars_per_token: 4,
            tokens_per_size: 20.0,
            pipe_thickness_ceiling: 1.0,
            min_water_threshold: 0.01,
            base_loss: 0.2,
            loss_pipe_factor: 0.8,
            loss_max: 0.8,
            flow_context_blend_alpha: 0.2,
            strengthen_eligibility_threshold: 0.7,
            min_strengthen_value: 0.05,
            base_reinforcement: 0.1,
            supplement_reinforce_ratio: 0.5,
            min_reinforcement_threshold: 0.001,
            pipe_strengthen_factor: 0.05,
            min_pipe_reinforce_threshold: 0.01,
            injection_mode: InjectionMode::Token,
            embedding_backend: "onnx".to_string(),
            embedding_dim: None,
            embedding_model_path: String::new(),
            embedding_tokenizer_path: String::new(),
            embedding_api_base: String::new(),
            embedding_api_key: String::new(),
            embedding_api_model: String::new(),
            token_budget: 2048,
            max_balls_per_inject: 10,
            render_clear_threshold: 0.7,
            render_deep_impression_threshold: 0.6,
            render_impression_threshold: 0.5,
            impression_threshold: 0.3,
            archive_inactive_days: 90.0,
            archive_size_threshold: 0.2,
            max_active_balls: 10_000,
            multi_source_overlap_threshold: 0.3,
            multi_source_merge_cap: 1.5,
            source_bonus: 0.2,
            source_cap: 2.0,
            db_path: None,
            log_dir: "logs".to_string(),
            log_level: "info".to_string(),
            log_retention_days: 7,
            metrics_enabled: true,
            metrics_retention_days: 90,
        }
    }
}
