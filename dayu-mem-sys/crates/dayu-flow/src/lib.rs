//! dayu-flow: 流动与结算引擎
//!
//! 流动：起始球选择 → 注水 → 逐跳传播（纯内存，只点亮）
//! 结算：路径相关性 → 强化写入 → 注入排序 → 渲染

pub mod engine;
pub mod loss;
pub mod settle;

pub use engine::FlowEngine;
pub use loss::{flow_loss, ConstantLossModel, LossModel, PipeWeightLossModel};
pub use settle::{SettleEngine, SettleResult};
