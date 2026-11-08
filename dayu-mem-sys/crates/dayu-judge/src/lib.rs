//! dayu-judge: 判别器抽象与后端实现
//!
//! 异步旁路：判定是否产生新球、归属已有球。
//!
//! 核心组件：
//! - `mock` — MockJudgeBackend：可编程 verdict 队列，用于测试
//! - `engine` — JudgeEngine：阈值过滤 → 灰区判断 → 归属检查 → 兜底创建
//! - `ort` — OrtJudgeBackend：ONNX Runtime 判别模型（需 `onnx` feature）

mod mock;
mod engine;
mod ort;

pub use mock::MockJudgeBackend;
pub use engine::{JudgeEngine, VerdictPostProcessor};
#[cfg(feature = "onnx")]
pub use ort::OrtJudgeBackend;
pub use ort::DEFAULT_JUDGE_PROMPT_TEMPLATE;
