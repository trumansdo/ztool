//! AI提示词生成器 —— JSON 文件持久化
//!
//! 使用单文件 JSON 存储所有提示词模板，文件位于 ~/.agents/data 目录。

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use super::model::{PromptStore, PromptTemplate};

/// 默认存储文件名
const STORE_FILE: &str = "prompt_templates.json";

/// 获取存储目录路径（~/.agents/data）
fn store_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".agents").join("data")
}

/// 获取存储文件完整路径
fn store_path() -> PathBuf {
    store_dir().join(STORE_FILE)
}

/// 从文件加载提示词存储
pub fn load() -> Result<PromptStore> {
    let path = store_path();
    if !path.exists() {
        return Ok(PromptStore::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("无法读取存储文件: {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(PromptStore::default());
    }
    serde_json::from_str(&content)
        .with_context(|| format!("存储文件 JSON 解析失败: {}", path.display()))
}

/// 保存提示词存储到文件
pub fn save(store: &PromptStore) -> Result<()> {
    let dir = store_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("无法创建存储目录: {}", dir.display()))?;
    let path = store_path();
    let content = serde_json::to_string_pretty(store)
        .context("序列化提示词数据失败")?;
    fs::write(&path, content)
        .with_context(|| format!("无法写入存储文件: {}", path.display()))
}

/// 按 ID 查找提示词
pub fn find_by_id<'a>(store: &'a PromptStore, id: &str) -> Option<&'a PromptTemplate> {
    store.prompts.iter().find(|p| p.id == id)
}

/// 按 ID 查找提示词（可变引用）
pub fn find_by_id_mut<'a>(store: &'a mut PromptStore, id: &str) -> Option<&'a mut PromptTemplate> {
    store.prompts.iter_mut().find(|p| p.id == id)
}

/// 删除指定 ID 的提示词
pub fn delete_by_id(store: &mut PromptStore, id: &str) -> bool {
    let len_before = store.prompts.len();
    store.prompts.retain(|p| p.id != id);
    store.prompts.len() < len_before
}
