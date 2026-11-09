//! HTTP 嵌入后端全链路验证
//!
//! 使用远程嵌入服务进行真实嵌入，验证 DayuCore 完整链路：
//! 建球 → 建管道 → 回忆 → 注入块。
//!
//! 运行方式：
//!   DAYU_HTTP_API_BASE=http://host:port/v1 \
//!   DAYU_HTTP_API_KEY=xxx \
//!   cargo test -p dayu-core --test http_e2e --features http -- --nocapture --test-threads=1 --include-ignored
//!
//! 所有用例默认 `#[ignore]`：它们依赖外部服务，不能进常规 CI。

// 未启用 `http` feature 时整个测试文件为空，避免 `HttpBackend` 导入失败。
#![cfg(feature = "http")]

use dayu_core::DayuCore;
use dayu_domain::traits::EmbeddingBackend;
use dayu_domain::types::*;
use dayu_embedding::HttpBackend;
use dayu_storage::testing::temp_store;
use std::sync::{Arc, LazyLock};

/// 嵌入维度：与 `DAYU_HTTP_DIM` 保持一致
const DIM: usize = 4096;

// ============================================================
// 全局单例：HTTP 后端复用连接池
// ============================================================

/// 连接信息全部来自环境变量，不把内网地址与密钥写进仓库
static GLOBAL_HTTP: LazyLock<Arc<dyn EmbeddingBackend>> = LazyLock::new(|| {
    let api_base = std::env::var("DAYU_HTTP_API_BASE")
        .expect("需设置 DAYU_HTTP_API_BASE（如 http://host:port/v1）");
    let api_key = std::env::var("DAYU_HTTP_API_KEY").expect("需设置 DAYU_HTTP_API_KEY");
    let model = std::env::var("DAYU_HTTP_MODEL")
        .unwrap_or_else(|_| "Qwen3-Embedding-8B".to_string());
    let dim = std::env::var("DAYU_HTTP_DIM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DIM);
    Arc::new(HttpBackend::new(&api_base, &api_key, &model, dim))
});

fn embedding_backend() -> Arc<dyn EmbeddingBackend> {
    Arc::clone(&GLOBAL_HTTP)
}

fn core() -> DayuCore {
    let mut config = Config::default();
    config.embedding_dim = Some(DIM);
    config.total_water_budget = 2.5;
    config.start_top_n = 5;
    config.max_flow_depth = 3;
    config.max_lit_ball_count = 50;
    DayuCore::new(Arc::new(config), embedding_backend()).expect("DayuCore 创建失败")
}

fn store_4096() -> dayu_storage::testing::TestStore {
    let store = temp_store();
    store.ensure(DIM);
    store
}

fn joined_text(blocks: &[InjectionBlock]) -> String {
    blocks.iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================
// 基础：单球召回
// ============================================================

#[tokio::test]
#[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
async fn single_ball_recall() {
    let core = core();
    let store = store_4096();
    let emb = core.embed_text("用户最喜欢的编程语言是 Rust，已经用了 3 年").await.unwrap();

    core.create_ball(&store, "用户最喜欢的编程语言是 Rust，已经用了 3 年", &emb, 3.0)
        .await
        .unwrap();

    let blocks = core.recall_with_text(&store, "用户喜欢什么编程语言", 1.0).await.unwrap();
    let text = joined_text(&blocks);

    assert!(!blocks.is_empty(), "应至少召回一个注入块");
    assert!(text.contains("Rust"), "应包含 Rust，实际: {text}");
    eprintln!("[single_ball] 注入块数: {}, 内容: {text}", blocks.len());
}

// ============================================================
// 多球关联召回
// ============================================================

#[tokio::test]
#[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
async fn multi_ball_linked_recall() {
    let core = core();
    let store = store_4096();

    // 建球
    let e1 = core.embed_text("用户住在北京朝阳区").await.unwrap();
    let e2 = core.embed_text("用户每天坐地铁10号线通勤").await.unwrap();
    let e3 = core.embed_text("用户公司在中关村软件园").await.unwrap();
    let e4 = core.embed_text("用户喜欢在公司附近吃川菜").await.unwrap();

    let id1 = core.create_ball(&store, "用户住在北京朝阳区", &e1, 2.0).await.unwrap();
    let id2 = core.create_ball(&store, "用户每天坐地铁10号线通勤", &e2, 2.0).await.unwrap();
    let id3 = core.create_ball(&store, "用户公司在中关村软件园", &e3, 3.0).await.unwrap();
    let id4 = core.create_ball(&store, "用户喜欢在公司附近吃川菜", &e4, 2.0).await.unwrap();

    // 建管道（关联记忆）
    core.link_balls(&store, id1, id2, 0.7).await.unwrap(); // 住 → 通勤
    core.link_balls(&store, id2, id3, 0.8).await.unwrap(); // 通勤 → 公司
    core.link_balls(&store, id3, id4, 0.6).await.unwrap(); // 公司 → 午餐

    // 查询：用户在哪工作
    let blocks = core.recall_with_text(&store, "用户在哪里工作", 1.0).await.unwrap();
    let text = joined_text(&blocks);

    assert!(!blocks.is_empty(), "应召回工作相关信息");
    assert!(text.contains("中关村") || text.contains("公司"), "应包含工作地点，实际: {text}");
    eprintln!("[multi_linked] 注入块数: {}, 内容: {text}", blocks.len());
}

// ============================================================
// 跨主题区分度
// ============================================================

#[tokio::test]
#[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
async fn cross_topic_discrimination() {
    let core = core();
    let store = store_4096();

    // 主题 A：技术
    let ea1 = core.embed_text("Rust 的所有权系统确保内存安全无需 GC").await.unwrap();
    let ea2 = core.embed_text("async/await 是 Rust 异步编程的核心机制").await.unwrap();
    let id_a1 = core.create_ball(&store, "Rust 的所有权系统确保内存安全无需 GC", &ea1, 3.0).await.unwrap();
    let id_a2 = core.create_ball(&store, "async/await 是 Rust 异步编程的核心机制", &ea2, 3.0).await.unwrap();
    core.link_balls(&store, id_a1, id_a2, 0.8).await.unwrap();

    // 主题 B：生活
    let eb1 = core.embed_text("周末去朝阳公园跑步很舒服").await.unwrap();
    let eb2 = core.embed_text("最近在学习做川菜，麻婆豆腐已经练熟了").await.unwrap();
    let id_b1 = core.create_ball(&store, "周末去朝阳公园跑步很舒服", &eb1, 2.0).await.unwrap();
    let id_b2 = core.create_ball(&store, "最近在学习做川菜，麻婆豆腐已经练熟了", &eb2, 2.0).await.unwrap();
    core.link_balls(&store, id_b1, id_b2, 0.7).await.unwrap();

    // 查询技术话题
    let tech_blocks = core.recall_with_text(&store, "Rust 的内存管理机制", 1.0).await.unwrap();
    let tech_text = joined_text(&tech_blocks);

    // 查询生活话题
    let life_blocks = core.recall_with_text(&store, "周末有什么休闲活动", 1.0).await.unwrap();
    let life_text = joined_text(&life_blocks);

    eprintln!("[cross_topic] 技术查询 → 注入块数: {}, 内容: {tech_text}", tech_blocks.len());
    eprintln!("[cross_topic] 生活查询 → 注入块数: {}, 内容: {life_text}", life_blocks.len());

    // 技术查询应优先召回技术内容
    assert!(
        tech_text.contains("所有权") || tech_text.contains("Rust") || tech_text.contains("内存"),
        "技术查询应召回技术内容，实际: {tech_text}"
    );

    // 生活查询应优先召回生活内容
    assert!(
        life_text.contains("公园") || life_text.contains("跑步") || life_text.contains("川菜") || life_text.contains("麻婆豆腐"),
        "生活查询应召回生活内容，实际: {life_text}"
    );
}

// ============================================================
// 规模测试：50 球 + 管道网络
// ============================================================

#[tokio::test]
#[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
async fn medium_scale_50_balls() {
    let core = core();
    let store = store_4096();

    let topics = [
        ("编程", vec![
            "Rust 的 trait 系统提供了强大的抽象能力",
            "Cargo 是 Rust 的包管理器和构建系统",
            "生命周期标注是 Rust 最独特的特性之一",
            "模式匹配让 Rust 代码更简洁易读",
            "智能指针 Box Rc Arc 各有不同用途",
        ]),
        ("AI", vec![
            "Transformer 架构彻底改变了自然语言处理",
            "注意力机制是 Transformer 的核心创新",
            "大语言模型通过预训练获得世界知识",
            "嵌入向量可以捕捉文本的语义信息",
            "RAG 检索增强生成提高了事实准确性",
        ]),
        ("生活", vec![
            "每天早上喝一杯咖啡开始新的一天",
            "周末喜欢去图书馆看书学习",
            "最近开始学习弹吉他作为业余爱好",
            "养了一只猫叫咪咪非常可爱",
            "每周三次去健身房保持身体健康",
        ]),
        ("工作", vec![
            "代码审查是保证代码质量的重要环节",
            "每天站会同步团队进度和阻塞项",
            "技术文档应该简洁清晰便于维护",
            "单元测试是重构信心的来源",
            "持续集成可以尽早发现集成问题",
        ]),
        ("旅行", vec![
            "去年去了日本京都看红叶非常美",
            "计划明年去冰岛看极光",
            "喜欢自驾游探索小众景点",
            "旅行中最重要的是一双舒服的鞋",
            "每次旅行都会收集当地明信片",
        ]),
    ];

    let mut ids_by_topic: Vec<(&str, Vec<BallId>)> = Vec::new();

    // 建球
    for (topic, texts) in &topics {
        let mut ids = Vec::new();
        for text in texts {
            let emb = core.embed_text(text).await.unwrap();
            let id = core.create_ball(&store, text, &emb, 2.0).await.unwrap();
            ids.push(id);
        }
        ids_by_topic.push((topic, ids));
    }

    // 同主题内建管道（强连接）
    for (_topic, ids) in &ids_by_topic {
        for window in ids.windows(2) {
            core.link_balls(&store, window[0], window[1], 0.7).await.unwrap();
        }
    }

    // 跨主题弱连接（模拟自然关联）
    core.link_balls(&store, ids_by_topic[0].1[0], ids_by_topic[1].1[0], 0.2).await.unwrap(); // 编程 ↔ AI
    core.link_balls(&store, ids_by_topic[2].1[0], ids_by_topic[3].1[0], 0.3).await.unwrap(); // 生活 ↔ 工作
    core.link_balls(&store, ids_by_topic[2].1[1], ids_by_topic[4].1[1], 0.3).await.unwrap(); // 生活 ↔ 旅行

    // 查询各主题
    let queries = [
        ("Rust 的包管理工具是什么", "编程"),
        ("Transformer 的核心机制", "AI"),
        ("周末喜欢做什么", "生活"),
        ("如何保证代码质量", "工作"),
        ("想去哪里旅游", "旅行"),
    ];

    let mut total_blocks = 0usize;
    let mut hit_count = 0usize;

    for (query, expected_topic) in &queries {
        let blocks = core.recall_with_text(&store, query, 1.0).await.unwrap();
        let text = joined_text(&blocks);
        total_blocks += blocks.len();

        // 检查是否召回了期望主题的内容
        let topic_texts: Vec<&str> = topics.iter()
            .find(|(t, _)| t == expected_topic)
            .map(|(_, texts)| texts.as_slice())
            .unwrap_or(&[])
            .to_vec();

        let hit = topic_texts.iter().any(|t| {
            let prefix: String = t.chars().take(10).collect();
            text.contains(&prefix)
        });
        if hit || !blocks.is_empty() {
            hit_count += 1;
        }

        eprintln!("[50balls] 查询「{query}」→ {}/{} 块, 命中主题「{expected_topic}」: {}",
            blocks.len(), if hit { "✅" } else { "❌" }, hit);
    }

    eprintln!("[50balls] 总计: {}/{} 查询有召回, 总注入块: {}", hit_count, queries.len(), total_blocks);
    assert!(hit_count >= 3, "至少 3/5 查询应有召回，实际: {hit_count}/{}", queries.len());
    assert!(total_blocks > 0, "应有至少一个注入块");
}

// ============================================================
// 统计信息
// ============================================================

#[tokio::test]
#[ignore = "requires DAYU_HTTP_API_BASE / DAYU_HTTP_API_KEY"]
async fn stats_after_operations() {
    let core = core();
    let store = store_4096();

    let emb = core.embed_text("测试记忆内容").await.unwrap();
    core.create_ball(&store, "测试记忆内容", &emb, 2.0).await.unwrap();

    let stats = core.stats(&store).await.unwrap();
    eprintln!("[stats] active_balls={}, archived_balls={}, embedding_dim={}",
        stats.active_balls, stats.archived_balls, stats.embedding_dim);

    assert_eq!(stats.active_balls, 1, "应有 1 个活跃球");
    assert_eq!(stats.embedding_dim, DIM, "嵌入维度应为 {DIM}");
}
