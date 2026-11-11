//! 命名空间注册表：一个 namespace 一个独立的 TriviumDB 文件
//!
//! 为什么按 namespace 分库而不是分表/加前缀：
//! - `reset` 退化为删文件，无状态残留，评测问与问之间不可能互相污染
//! - 评测时 namespace 数 = 样本数（100–500），单库几百 KB 到几 MB，磁盘压力可接受
//!
//! 并发模型：评测 harness 用 4 个 worker 并行跑不同问题，每个 worker 独占一个
//! namespace。因此注册表只需要保证「取状态」这一步互斥，取到 `Arc<NamespaceState>`
//! 之后的所有操作都在锁外进行——不同 namespace 之间完全不争用。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use dayu_core::DayuCore;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::types::{BallId, Config};
use dayu_storage::TriviumStorage;

/// 单个 namespace 的运行时状态
pub struct NamespaceState {
    pub core: DayuCore,
    pub storage: TriviumStorage,
    /// 每个 session 最后写入的球：用于把同一会话的相邻回合连成管道
    last_ball: Mutex<HashMap<String, BallId>>,
}

impl NamespaceState {
    /// 取出并清空该 session 的上一个球
    pub fn take_last_ball(&self, session_id: &str) -> Option<BallId> {
        self.last_ball
            .lock()
            .ok()
            .and_then(|mut m| m.remove(session_id))
    }

    pub fn set_last_ball(&self, session_id: &str, id: BallId) {
        if let Ok(mut m) = self.last_ball.lock() {
            m.insert(session_id.to_string(), id);
        }
    }
}

/// 嵌入缓存上限：超过后不再写入新条目（只命中已有的）
///
/// 评测会为每个问题重建整个 haystack，同一批 turn 被重复嵌入 N 次；
/// 缓存把 N 次远程调用压成 1 次。上限存在的意义只是防止异常输入把内存打爆。
const EMBED_CACHE_MAX: usize = 200_000;

pub struct Registry {
    config: Arc<Config>,
    backend: Arc<dyn EmbeddingBackend>,
    store_dir: PathBuf,
    dim: usize,
    /// `None` = 不连管道（纯余弦 Top-N 对照组）
    link_weight: Option<f64>,
    namespaces: RwLock<HashMap<String, Arc<NamespaceState>>>,
    embed_cache: Mutex<HashMap<String, Arc<Vec<f32>>>>,
}

impl Registry {
    pub fn new(
        config: Arc<Config>,
        backend: Arc<dyn EmbeddingBackend>,
        store_dir: PathBuf,
        link_weight: Option<f64>,
    ) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&store_dir)?;
        let dim = backend.dimension();
        Ok(Self {
            config,
            backend,
            store_dir,
            dim,
            link_weight,
            namespaces: RwLock::new(HashMap::new()),
            embed_cache: Mutex::new(HashMap::new()),
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn link_weight(&self) -> Option<f64> {
        self.link_weight
    }

    /// 取（必要时创建）namespace 的状态
    pub async fn state(&self, namespace: &str) -> anyhow::Result<Arc<NamespaceState>> {
        if let Some(st) = self
            .namespaces
            .read()
            .map_err(|_| anyhow::anyhow!("namespace 读锁被污染"))?
            .get(namespace)
            .cloned()
        {
            return Ok(st);
        }

        // 双检：并发创建同一 namespace 时只有一个真正 open
        let mut guard = self
            .namespaces
            .write()
            .map_err(|_| anyhow::anyhow!("namespace 写锁被污染"))?;
        if let Some(st) = guard.get(namespace).cloned() {
            return Ok(st);
        }

        let path = self.db_path(namespace);
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("store 路径非 UTF-8: {}", path.display()))?;

        let storage = TriviumStorage::with_dim(path_str, self.dim)
            .map_err(|e| anyhow::anyhow!("打开 namespace 库失败（{namespace}）：{e}"))?;
        let core = DayuCore::new(Arc::clone(&self.config), Arc::clone(&self.backend))
            .map_err(|e| anyhow::anyhow!("构造 DayuCore 失败（{namespace}）：{e}"))?;

        let st = Arc::new(NamespaceState {
            core,
            storage,
            last_ball: Mutex::new(HashMap::new()),
        });
        guard.insert(namespace.to_string(), Arc::clone(&st));
        Ok(st)
    }

    /// 清空 namespace：先从注册表移除（Drop 释放文件锁），再删文件
    pub fn reset(&self, namespace: &str) -> anyhow::Result<()> {
        let removed = self
            .namespaces
            .write()
            .map_err(|_| anyhow::anyhow!("namespace 写锁被污染"))?
            .remove(namespace);

        // 必须显式 drop：Arc 释放 → TriviumStorage Drop → Database 关闭 → 锁文件释放
        drop(removed);

        self.delete_db_files(namespace)
    }

    /// 向量：带内容缓存（相同文本只嵌入一次）
    pub async fn embed_cached(&self, text: &str) -> anyhow::Result<Arc<Vec<f32>>> {
        if let Some(v) = self
            .embed_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("嵌入缓存锁被污染"))?
            .get(text)
            .cloned()
        {
            return Ok(v);
        }

        let vec = Arc::new(
            self.backend
                .embed(text)
                .await
                .map_err(|e| anyhow::anyhow!("嵌入失败：{e}"))?,
        );

        let mut cache = self
            .embed_cache
            .lock()
            .map_err(|_| anyhow::anyhow!("嵌入缓存锁被污染"))?;
        if cache.len() < EMBED_CACHE_MAX {
            cache.insert(text.to_string(), Arc::clone(&vec));
        }
        Ok(vec)
    }

    fn db_path(&self, namespace: &str) -> PathBuf {
        self.store_dir.join(format!("{}.tdb", sanitize(namespace)))
    }

    /// 删除 namespace 的全部分片文件
    ///
    /// Windows 上文件句柄释放到真正可删之间有微小延迟，故带重试。
    fn delete_db_files(&self, namespace: &str) -> anyhow::Result<()> {
        let prefix = format!("{}.tdb", sanitize(namespace));
        let entries = match std::fs::read_dir(&self.store_dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(anyhow::anyhow!("读 store 目录失败：{e}")),
        };

        let targets: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .map(|e| e.path())
            .collect();

        for path in targets {
            remove_with_retry(&path)?;
        }
        Ok(())
    }
}

fn remove_with_retry(path: &Path) -> anyhow::Result<()> {
    const ATTEMPTS: usize = 20;
    let mut last: Option<std::io::Error> = None;
    for i in 0..ATTEMPTS {
        match std::fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                last = Some(e);
                std::thread::sleep(std::time::Duration::from_millis(10 * (i as u64 + 1)));
            }
        }
    }
    Err(anyhow::anyhow!(
        "删除 {} 失败（文件可能仍被占用）：{}",
        path.display(),
        last.map(|e| e.to_string()).unwrap_or_default()
    ))
}

/// namespace 只允许安全字符，防止 `../` 之类穿越到 store 目录外
fn sanitize(namespace: &str) -> String {
    namespace
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// 来源回合编码：写进 `Content.source: TurnId`，随球落库
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SourceTurn {
    pub session_id: String,
    pub turn_idx: i64,
    #[serde(default)]
    pub session_idx: i64,
}

impl SourceTurn {
    pub fn encode(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn decode(raw: &str) -> Option<Self> {
        if raw.is_empty() {
            return None;
        }
        serde_json::from_str(raw).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_blocks_path_traversal() {
        assert_eq!(sanitize("../etc/passwd"), "___etc_passwd");
        assert_eq!(sanitize("mceval-a1b2c3"), "mceval-a1b2c3");
    }

    #[test]
    fn source_turn_roundtrip() {
        let s = SourceTurn {
            session_id: "s|1".into(),
            turn_idx: 7,
            session_idx: 2,
        };
        let back = SourceTurn::decode(&s.encode()).unwrap();
        assert_eq!(back.session_id, "s|1", "分隔符不应参与编码");
        assert_eq!(back.turn_idx, 7);
        assert_eq!(back.session_idx, 2);
        assert!(SourceTurn::decode("").is_none());
    }
}
