//! dayu-observability: 指标、日志、诊断规则
//!
//! 指标矩阵：流动层 / 管道层 / 结算层 / 注入层 / 存储层。
//!
//! - `traits` — MetricsRecorder trait（可替换后端）
//! - `mock` — MockMetricsRecorder（内存 push，测试断言）
//! - `tracing_recorder` — 生产实现，指标落 `logs/metrics.jsonl`
//! - `logging` — 日志与指标落盘初始化（按天轮转、自动清理）
//! - `diagnostics` — 诊断规则（纯函数，无副作用）

mod traits;
mod mock;
mod tracing_recorder;
mod logging;
mod diagnostics;

pub use traits::MetricsRecorder;
pub use mock::{MockMetricsRecorder, MetricEntry};
pub use tracing_recorder::TracingMetricsRecorder;
pub use logging::{init, InitError, ObservabilityGuard, METRIC_TARGET};
pub use diagnostics::{Diagnosis, DiagnosisSeverity, diagnose_loss_dominant, diagnose_supplement_weak, diagnose_loss_saturation};

// ============================================================
// 指标名称常量
// ============================================================

/// 流动触发率：每次 recall 是否成功触发流动
pub const METRIC_FLOW_TRIGGER_RATE: &str = "flow_trigger_rate";

/// 流动损失按管道粗细分桶
pub const METRIC_FLOW_LOSS_BY_PIPE_WEIGHT: &str = "flow_loss_by_pipe_weight";

/// 流动补充按管道粗细分桶
pub const METRIC_FLOW_SUPPLEMENT_BY_PIPE_WEIGHT: &str = "flow_supplement_by_pipe_weight";

/// 路径补充损失比（带 dominant 标签：loss | supplement）
pub const METRIC_PATH_SUPPLEMENT_LOSS_RATIO: &str = "path_supplement_loss_ratio";

/// 注入 token 使用量
pub const METRIC_INJECTION_TOKEN_USED: &str = "injection_token_used";

/// 点亮球数量
pub const METRIC_LIT_BALL_COUNT: &str = "lit_ball_count";

/// 结算强化球数量
pub const METRIC_REINFORCED_BALL_COUNT: &str = "reinforced_ball_count";

/// 结算强化管道数量
pub const METRIC_REINFORCED_PIPE_COUNT: &str = "reinforced_pipe_count";

/// 单轮 recall 耗时（毫秒）
pub const METRIC_RECALL_ELAPSED_MS: &str = "recall_elapsed_ms";
