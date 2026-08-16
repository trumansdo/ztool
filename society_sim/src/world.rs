//! World — 世界容器
//!
//! 环形地图，持有所有 Grid、Org、Citizen。
//! 全局时钟驱动模拟推进。

use crate::citizen::Citizen;
use crate::grid::Grid;
use crate::org::Org;

/// 世界容器
#[derive(Debug, Clone)]
pub struct World {
    /// 全局时钟（tick=天）
    pub tick: u64,
    /// 地图宽度（Grid 列数）
    pub width: u32,
    /// 地图高度（Grid 行数）
    pub height: u32,
    /// 所有空间网格
    pub grids: Vec<Grid>,
    /// 所有组织
    pub orgs: Vec<Org>,
    /// 所有公民（SoA 布局，对象池管理）
    pub citizens: Vec<Citizen>,
    /// 随机种子
    pub seed: u64,
}
