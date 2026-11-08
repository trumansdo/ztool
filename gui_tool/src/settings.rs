//! 设置持久化模块
//!
//! 定义应用设置的加载/保存接口，支持文件系统和内存两种后端。
//! 用于主题偏好、窗口状态等跨会话持久化。

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

use crate::theme::ThemeId;

// ============================================================================
// AppSettings — 可持久化的设置数据
// ============================================================================

/// 应用设置（可序列化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 当前主题 ID
    pub theme_id: ThemeId,
    /// 窗口宽度（像素）
    pub window_width: f32,
    /// 窗口高度（像素）
    pub window_height: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_id: ThemeId::dark(),
            window_width: 1000.0,
            window_height: 700.0,
        }
    }
}

// ============================================================================
// SettingsStore trait
// ============================================================================

/// 设置存储 — 加载/保存应用设置
pub trait SettingsStore {
    /// 加载设置，不存在时返回默认值
    fn load(&self) -> Result<AppSettings, String>;

    /// 保存设置
    fn save(&self, settings: &AppSettings) -> Result<(), String>;
}

// ============================================================================
// FsSettingsStore — 文件系统后端
// ============================================================================

/// 基于 JSON 文件的设置存储
///
/// 存储路径：`<config_dir>/settings.json`
pub struct FsSettingsStore {
    path: PathBuf,
}

impl FsSettingsStore {
    /// 创建文件系统设置存储
    ///
    /// 自动创建父目录。
    pub fn new(path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建设置目录失败: {}", e))?;
        }
        Ok(Self { path })
    }
}

impl SettingsStore for FsSettingsStore {
    fn load(&self) -> Result<AppSettings, String> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }
        let content =
            fs::read_to_string(&self.path).map_err(|e| format!("读取设置文件失败: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析设置 JSON 失败: {}", e))
    }

    fn save(&self, settings: &AppSettings) -> Result<(), String> {
        let content =
            serde_json::to_string_pretty(settings).map_err(|e| format!("序列化设置失败: {}", e))?;
        fs::write(&self.path, content).map_err(|e| format!("写入设置文件失败: {}", e))
    }
}

// ============================================================================
// MemSettingsStore — 内存后端（测试用）
// ============================================================================

/// 基于内存的设置存储
pub struct MemSettingsStore {
    store: RefCell<Option<AppSettings>>,
}

impl Default for MemSettingsStore {
    fn default() -> Self {
        Self {
            store: RefCell::new(None),
        }
    }
}

impl MemSettingsStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SettingsStore for MemSettingsStore {
    fn load(&self) -> Result<AppSettings, String> {
        Ok(self
            .store
            .borrow()
            .clone()
            .unwrap_or_else(AppSettings::default))
    }

    fn save(&self, settings: &AppSettings) -> Result<(), String> {
        *self.store.borrow_mut() = Some(settings.clone());
        Ok(())
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_default_on_first_load() {
        let store = MemSettingsStore::new();
        let settings = store.load().unwrap();
        assert_eq!(settings.theme_id, ThemeId::dark());
    }

    #[test]
    fn mem_save_and_load() {
        let store = MemSettingsStore::new();
        let mut settings = store.load().unwrap();
        settings.theme_id = ThemeId::light();
        store.save(&settings).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.theme_id, ThemeId::light());
    }

    #[test]
    fn mem_persists_across_loads() {
        let store = MemSettingsStore::new();
        let s1 = store.load().unwrap();
        assert_eq!(s1.window_width, 1000.0);

        let mut s2 = s1.clone();
        s2.window_width = 1200.0;
        store.save(&s2).unwrap();

        let s3 = store.load().unwrap();
        assert_eq!(s3.window_width, 1200.0);
    }
}
