//! 文档服务抽象
//!
//! 定义文档级 CRUD 接口，支持文件系统和内存两种后端。
//! 用于 prompt_gen 模板存储等场景。

use serde::{de::DeserializeOwned, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// DocumentService trait
// ============================================================================

/// 文档服务 — 按 ID 管理可序列化文档
///
/// 泛型 `D` 为文档类型，必须可序列化/反序列化。
pub trait DocumentService<D> {
    /// 列出所有文档 ID
    fn list(&self) -> Result<Vec<String>, String>;

    /// 按 ID 读取文档
    fn read(&self, id: &str) -> Result<D, String>;

    /// 保存文档（新建或覆盖）
    fn save(&self, id: &str, doc: &D) -> Result<(), String>;

    /// 删除文档
    fn delete(&self, id: &str) -> Result<(), String>;
}

// ============================================================================
// FsDocumentService — 文件系统后端
// ============================================================================

/// 基于文件系统的文档服务
///
/// 每个文档存储为 `<store_dir>/<id>.json`。
pub struct FsDocumentService {
    store_dir: PathBuf,
}

impl FsDocumentService {
    /// 创建文件系统文档服务
    ///
    /// 若目录不存在则自动创建。
    pub fn new(store_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&store_dir).map_err(|e| format!("创建存储目录失败: {}", e))?;
        Ok(Self { store_dir })
    }

    fn file_path(&self, id: &str) -> PathBuf {
        self.store_dir.join(format!("{}.json", id))
    }
}

impl<D> DocumentService<D> for FsDocumentService
where
    D: Serialize + DeserializeOwned,
{
    fn list(&self) -> Result<Vec<String>, String> {
        let mut ids = Vec::new();
        let entries = fs::read_dir(&self.store_dir).map_err(|e| format!("读取目录失败: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }
        Ok(ids)
    }

    fn read(&self, id: &str) -> Result<D, String> {
        let path = self.file_path(id);
        let content = fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))
    }

    fn save(&self, id: &str, doc: &D) -> Result<(), String> {
        let path = self.file_path(id);
        let content =
            serde_json::to_string_pretty(doc).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let path = self.file_path(id);
        fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))
    }
}

// ============================================================================
// MemDocumentService — 内存后端（测试用）
// ============================================================================

/// 基于内存 HashMap 的文档服务
///
/// 用于单元测试，不持久化。
pub struct MemDocumentService<D> {
    store: RefCell<HashMap<String, D>>,
}

impl<D> Default for MemDocumentService<D> {
    fn default() -> Self {
        Self {
            store: RefCell::new(HashMap::new()),
        }
    }
}

impl<D> MemDocumentService<D> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> DocumentService<D> for MemDocumentService<D>
where
    D: Clone + Serialize + DeserializeOwned,
{
    fn list(&self) -> Result<Vec<String>, String> {
        let mut ids: Vec<String> = self.store.borrow().keys().cloned().collect();
        ids.sort();
        Ok(ids)
    }

    fn read(&self, id: &str) -> Result<D, String> {
        self.store
            .borrow()
            .get(id)
            .cloned()
            .ok_or_else(|| format!("文档不存在: {}", id))
    }

    fn save(&self, id: &str, doc: &D) -> Result<(), String> {
        self.store
            .borrow_mut()
            .insert(id.to_string(), doc.clone());
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        self.store.borrow_mut().remove(id);
        Ok(())
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestDoc {
        title: String,
        content: String,
    }

    #[test]
    fn mem_list_empty() {
        let svc = MemDocumentService::<TestDoc>::new();
        assert_eq!(svc.list().unwrap(), Vec::<String>::new());
    }

    #[test]
    fn mem_save_and_read() {
        let svc = MemDocumentService::new();
        let doc = TestDoc {
            title: "测试".into(),
            content: "内容".into(),
        };
        svc.save("doc1", &doc).unwrap();
        let read: TestDoc = svc.read("doc1").unwrap();
        assert_eq!(read.title, "测试");
        assert_eq!(read.content, "内容");
    }

    #[test]
    fn mem_list_after_save() {
        let svc = MemDocumentService::new();
        svc.save(
            "a",
            &TestDoc {
                title: "A".into(),
                content: "".into(),
            },
        )
        .unwrap();
        svc.save(
            "b",
            &TestDoc {
                title: "B".into(),
                content: "".into(),
            },
        )
        .unwrap();
        assert_eq!(svc.list().unwrap(), vec!["a", "b"]);
    }

    #[test]
    fn mem_delete() {
        let svc = MemDocumentService::new();
        svc.save(
            "x",
            &TestDoc {
                title: "X".into(),
                content: "".into(),
            },
        )
        .unwrap();
        svc.delete("x").unwrap();
        assert!(svc.read("x").is_err());
    }

    #[test]
    fn mem_read_missing() {
        let svc = MemDocumentService::<TestDoc>::new();
        assert!(svc.read("nonexistent").is_err());
    }
}
