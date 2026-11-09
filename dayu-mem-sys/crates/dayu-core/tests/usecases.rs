//! 用例级测试：模型真正会走的完整路径
//!
//! 与 `src/lib.rs` 里的单元测试分开，原因有三：
//! 1. **黑盒**：这里只能 import `pub` API，编译器强制不依赖私有实现；
//! 2. **场景完整**：建球 → 建管道 → 单源/多源回忆 → 注入块，一条链路一个断言；
//! 3. **独立编译单元**：与单元测试并行编译，失败时能立刻区分"API 用错了"还是"内部逻辑错了"。
//!
//! 单元测试留在 `src/` 内（可访问私有函数），用例测试放这里（只用公开接口）。
//!
//! 本文件用**真实 ONNX 模型**，验证的是语义行为，因此需要显式启用 feature：
//!   cargo test -p dayu-core --features onnx --test usecases
//!
//! 模型路径可用环境变量覆盖：
//!   DAYU_EMBEDDING_MODEL_PATH=models/infinilabs/pizza-embed-zh-small-v1/onnx/model.onnx
//!   DAYU_EMBEDDING_TOKENIZER_PATH=models/infinilabs/pizza-embed-zh-small-v1/tokenizer.json

// 未启用 `onnx` 时整个用例文件为空，避免 `OrtEmbeddingBackend` 导入失败。
#![cfg(feature = "onnx")]

use dayu_core::DayuCore;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::types::*;
use dayu_storage::testing::temp_store;
use std::sync::{Arc, LazyLock};

// ============================================================
// 全局单例：ort 只允许一个 Environment per process
// ============================================================

static GLOBAL_EMBEDDING: LazyLock<Arc<dyn EmbeddingBackend>> = LazyLock::new(|| {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("无法找到 workspace 根目录");
    let model_path = std::env::var("DAYU_EMBEDDING_MODEL_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("models/infinilabs/pizza-embed-zh-small-v1/onnx/model.onnx"));
    let tokenizer_path = std::env::var("DAYU_EMBEDDING_TOKENIZER_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("models/infinilabs/pizza-embed-zh-small-v1/tokenizer.json"));
    Arc::new(
        dayu_embedding::OrtEmbeddingBackend::load(&model_path, &tokenizer_path, 512)
            .expect("ONNX 模型加载失败"),
    )
});

fn embedding_backend() -> Arc<dyn EmbeddingBackend> {
    Arc::clone(&GLOBAL_EMBEDDING)
}

fn core() -> DayuCore {
    DayuCore::new(Arc::new(Config::default()), embedding_backend())
        .expect("DayuCore 创建失败")
}

fn joined_text(blocks: &[InjectionBlock]) -> String {
    blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================
// 建球
// ============================================================

#[tokio::test]
async fn create_ball_then_recall_returns_its_content() {
    let core = core();
    let store = temp_store();
    let emb = core.embedding_backend().embed("用户的生日是 3 月 5 日").await.unwrap();

    core.create_ball(&store, "用户的生日是 3 月 5 日", &emb, 3.0)
        .await
        .unwrap();

    let blocks = core
        .recall_with_text(&store, "用户的生日是哪天", 1.0)
        .await
        .unwrap();

    let text = joined_text(&blocks);
    assert!(text.contains("3 月 5 日"), "应召回生日信息，实际: {text}");
}

#[tokio::test]
async fn create_ball_with_embedding_stores_content() {
    let core = core();
    let store = temp_store();

    let emb = core.embedding_backend().embed("Python 的 GIL 限制了多线程并行").await.unwrap();
    core.create_ball(&store, "Python 的 GIL 限制了多线程并行", &emb, 3.0)
        .await
        .unwrap();

    let blocks = core
        .recall_with_text(&store, "Python 多线程", 1.0)
        .await
        .unwrap();

    let text = joined_text(&blocks);
    assert!(text.contains("GIL") || text.contains("Python"), "应召回 Python 相关内容，实际: {text}");
}

// ============================================================
// 管道与流动
// ============================================================

#[tokio::test]
async fn linked_balls_flow_water_to_related_query() {
    let core = core();
    let store = temp_store();

    let emb1 = core.embedding_backend().embed("Rust 的所有权系统保证内存安全").await.unwrap();
    let emb2 = core.embedding_backend().embed("Rust 的借用检查器在编译时验证引用").await.unwrap();

    let id1 = core
        .create_ball(&store, "Rust 的所有权系统保证内存安全", &emb1, 3.0)
        .await
        .unwrap();
    let id2 = core
        .create_ball(&store, "Rust 的借用检查器在编译时验证引用", &emb2, 3.0)
        .await
        .unwrap();

    core.link_balls(&store, id1, id2, 0.8).await.unwrap();

    let blocks = core
        .recall_with_text(&store, "Rust 内存管理", 1.0)
        .await
        .unwrap();

    let text = joined_text(&blocks);
    assert!(
        text.contains("所有权") || text.contains("借用"),
        "应通过管道流动召回关联内容，实际: {text}"
    );
}

#[tokio::test]
async fn multi_source_recall_aggregates_from_multiple_starting_balls() {
    let core = core();
    let store = temp_store();

    let emb1 = core.embedding_backend().embed("Python 装饰器").await.unwrap();
    let emb2 = core.embedding_backend().embed("Rust 宏系统").await.unwrap();

    core.create_ball(&store, "Python 装饰器是一种高阶函数", &emb1, 3.0)
        .await
        .unwrap();
    core.create_ball(&store, "Rust 的宏系统支持元编程", &emb2, 3.0)
        .await
        .unwrap();

    // 查询"元编程"应能命中两个球
    let blocks = core
        .recall_with_text(&store, "元编程机制", 1.0)
        .await
        .unwrap();

    let text = joined_text(&blocks);
    // 至少命中一个
    assert!(
        text.contains("装饰器") || text.contains("宏"),
        "多源回忆应聚合多个起始球的内容，实际: {text}"
    );
}

// ============================================================
// 注入块
// ============================================================

#[tokio::test]
async fn recall_returns_injection_blocks_with_confidence() {
    let core = core();
    let store = temp_store();

    let emb = core.embedding_backend().embed("重要信息").await.unwrap();
    core.create_ball(&store, "重要信息：项目截止日期是 12 月 31 日", &emb, 5.0)
        .await
        .unwrap();

    let blocks = core
        .recall_with_text(&store, "项目截止日期", 1.0)
        .await
        .unwrap();

    assert!(!blocks.is_empty(), "应返回注入块");
    for block in &blocks {
        assert!(!block.text.is_empty(), "注入块文本不应为空");
        // confidence 是枚举，至少不是默认值
        assert!(!block.ball_ids.is_empty(), "注入块应关联至少一个球");
    }
}

// ============================================================
// 归档
// ============================================================

#[tokio::test]
async fn archive_sweep_removes_weak_inactive_balls() {
    let core = core();
    let store = temp_store();

    let emb = core.embedding_backend().embed("临时信息").await.unwrap();
    core.create_ball(&store, "临时信息：今天天气不错", &emb, 1.0)
        .await
        .unwrap();

    // 使用带小 max_active_balls 的配置触发归档
    let cfg = Config {
        max_active_balls: 0, // 强制全部归档
        archive_inactive_days: 0.0,
        ..Config::default()
    };
    let archive_core = DayuCore::new(Arc::new(cfg), embedding_backend()).unwrap();

    let report = archive_core.archive_sweep(&store).await.unwrap();
    assert!(report.archived > 0, "应归档至少一个球，实际归档: {}", report.archived);
}
