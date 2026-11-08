//! 标识符 newtype
//!
//! 每种 ID 独立类型，消灭混用。

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// 球（记忆节点）唯一标识
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct BallId(pub u64);

/// 管道（边）唯一标识
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct PipeId(pub u64);

/// 内容条目唯一标识
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ContentId(pub u64);

/// 对话回合唯一标识
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct TurnId(pub String);

/// 配置版本号
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ConfigVersion(pub u64);

/// 一次查询的唯一标识
///
/// 由本系统生成，用于串联 `judge_turn` → `recall` → `archive_turn` 全链路日志。
/// 调用方（插件/CLI）只负责透传，不自行构造语义。
///
/// 格式：`{毫秒时间戳:013x}{进程内序号:04x}{进程号低16位:04x}`，共 21 字符。
/// 字典序即时间序，便于日志按轮次排序。
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct QueryId(pub String);

impl QueryId {
    /// 生成新的查询标识
    pub fn new() -> Self {
        static SEQ: AtomicU32 = AtomicU32::new(0);
        let ms = chrono::Utc::now().timestamp_millis().max(0) as u64;
        let seq = SEQ.fetch_add(1, Ordering::Relaxed) & 0xFFFF;
        let pid = std::process::id() & 0xFFFF;
        Self(format!("{ms:013x}{seq:04x}{pid:04x}"))
    }

    /// 复用调用方传入的标识（空串视为未提供）
    pub fn from_input(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some(s) if !s.is_empty() => Self(s.to_string()),
            _ => Self::new(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for QueryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_id_is_unique_and_sortable() {
        let a = QueryId::new();
        let b = QueryId::new();
        assert_ne!(a, b, "同一进程内连续生成不得重复");
        assert_eq!(a.as_str().len(), 21, "固定 21 字符");
        assert!(a < b, "字典序应等于生成顺序");
    }

    #[test]
    fn query_id_reuses_non_empty_input() {
        assert_eq!(QueryId::from_input(Some("given")).as_str(), "given");
        assert_eq!(QueryId::from_input(Some("  ")).as_str().len(), 21);
        assert_eq!(QueryId::from_input(None).as_str().len(), 21);
    }
}
