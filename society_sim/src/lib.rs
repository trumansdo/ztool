//! 社会演化模拟引擎 — 核心数据结构
//!
//! 基于 T/V 值动态流转原则：
//! - T 值（真生命）：生存力，{current, max, regen}，最终输出
//! - V 值（虚生命）：社会地位/权力指标，事件/行为的结果，作为修正系数反作用于 T_Regen
//! - 事件是唯一驱动力，修改 T/V 的流速和存量
//!
//! 设计原则：无大脑（阈值触发涌现）、事件即一切、T 值静态基底、涌现优于复现
//!
//! 阶级/制度不是数据结构——阶级由 V 值连续函数动态体现，制度只是事件配置的标签。

pub mod types;
pub mod citizen;
pub mod org;
pub mod event;
pub mod world;
pub mod grid;
pub mod blackboard;
pub mod snapshot;
pub mod config;

pub use citizen::Citizen;
pub use org::Org;
pub use event::Event;
pub use world::World;
pub use grid::Grid;
pub use blackboard::{Blackboard, EventSlot};
pub use snapshot::{Snapshot, GridSummary, OrgStat};
pub use config::EventConfig;
pub use types::{
    VRange, EventSource, EventScope, MutationRule,
    CalcType, AttrTarget, Modifier, Relation,
};
