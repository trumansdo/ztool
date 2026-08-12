//! LSP 类型定义 —— lsp-types 重导出 + jdtls 私有协议扩展
//!
//! 标准 LSP 类型直接使用 lsp_types crate，本模块只补充非标准扩展。

pub use lsp_types::*;

use serde::Serialize;

// ─── jdtls 私有协议扩展 ───────────────────────────────

/// java/buildWorkspace 请求参数
/// jdtls 要求 [bool] 数组格式 (vscode-java #1929 兼容)
#[derive(Debug, Serialize)]
pub struct JavaBuildWorkspaceParams {
    pub force_rebuild: bool,
}

impl From<JavaBuildWorkspaceParams> for Vec<bool> {
    fn from(p: JavaBuildWorkspaceParams) -> Self {
        vec![p.force_rebuild]
    }
}

/// BuildWorkspaceStatus 枚举映射
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStatus {
    Failed = 0,
    Succeeded = 1,
    WithError = 2,
    Cancelled = 3,
}

impl BuildStatus {
    pub fn from_ordinal(n: u64) -> Self {
        match n {
            0 => Self::Failed,
            1 => Self::Succeeded,
            2 => Self::WithError,
            3 => Self::Cancelled,
            _ => Self::Failed,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failed => "FAILED",
            Self::Succeeded => "SUCCEED",
            Self::WithError => "WITH_ERROR",
            Self::Cancelled => "CANCELLED",
        }
    }
}
