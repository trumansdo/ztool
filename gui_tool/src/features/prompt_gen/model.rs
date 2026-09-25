//! AI提示词生成器 —— 数据模型
//!
//! 定义提示词模板的数据结构：UUID标识、标题、用户变量段、固定模板。

use serde::{Deserialize, Serialize};

/// 提示词模板条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// UUID 唯一标识
    pub id: String,
    /// 标题描述
    pub title: String,
    /// 用户定义的变量段列表
    pub user_sections: Vec<UserSection>,
    /// 固定内容模板（含 ${var} 占位符）
    pub fixed_template: String,
    /// 创建时间（ISO 8601）
    pub created_at: String,
    /// 更新时间（ISO 8601）
    pub updated_at: String,
}

/// 用户定义的变量段
///
/// 格式：`#变量名\n内容`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSection {
    /// 变量名（不含 # 前缀）
    pub var_name: String,
    /// 变量内容
    pub content: String,
}

/// 提示词存储文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptStore {
    /// 所有提示词模板
    pub prompts: Vec<PromptTemplate>,
}

impl PromptTemplate {
    /// 创建新的提示词模板
    pub fn new(title: String) -> Self {
        let now = chrono_now();
        Self {
            id: uuid_v4(),
            title,
            user_sections: Vec::new(),
            fixed_template: String::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 更新修改时间
    pub fn touch(&mut self) {
        self.updated_at = chrono_now();
    }
}

/// 生成 UUID v4（简化版，基于随机数）
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // 用时间戳 + 简单哈希生成伪 UUID
    let a = (nanos >> 96) as u32;
    let b = ((nanos >> 64) & 0xFFFF) as u16;
    let c = ((nanos >> 48) & 0x0FFF) as u16 | 0x4000;
    let d = ((nanos >> 32) & 0x3FFF) as u16 | 0x8000;
    let e = ((nanos >> 16) & 0xFFFF) as u16;
    let f = (nanos & 0xFFFF_FFFF_FFFF) as u64;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:04x}{:012x}",
        a, b, c, d, e, f
    )
}

/// 获取当前时间字符串（ISO 8601 简化格式）
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // 简单转换为日期时间字符串
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // 从 Unix epoch (1970-01-01) 计算年月日
    let (year, month, day) = days_to_ymd(days as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// 将 Unix epoch 以来的天数转换为年月日
fn days_to_ymd(mut days: i64) -> (i64, u32, u32) {
    days += 719468; // 偏移到 0000-03-01
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}
