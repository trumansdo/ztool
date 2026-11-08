//! 用例级测试：模型真正会走的完整路径
//!
//! 与 `src/lib.rs` 里的单元测试分开，原因有三：
//! 1. **黑盒**：这里只能 import `pub` API，编译器强制不依赖私有实现；
//! 2. **场景完整**：建球 → 建管道 → 单源/多源回忆 → 注入块，一条链路一个断言；
//! 3. **独立编译单元**：与单元测试并行编译，失败时能立刻区分"API 用错了"还是"内部逻辑错了"。
//!
//! 单元测试留在 `src/` 内（可访问私有函数），用例测试放这里（只用公开接口）。

use dayu_core::DayuCore;
use dayu_domain::types::*;
use dayu_flow::FlowEngine;
use dayu_storage::testing::temp_store;
use std::sync::Arc;

// ============================================================
// 夹具
// ============================================================

/// 第 `axis` 维为 1 的单位向量
fn axis_vec(axis: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; 384];
    v[axis] = 1.0;
    v
}

/// 与 `axis` 维夹角余弦为 `cos` 的向量（另一维取 sqrt(1-cos²)）
fn tilted_vec(axis: usize, cos: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 384];
    v[axis] = cos;
    v[axis + 1] = (1.0 - cos * cos).sqrt();
    v
}

fn joined_text(blocks: &[InjectionBlock]) -> String {
    blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn core() -> DayuCore {
    DayuCore::new(Arc::new(Config::default()))
}

// ============================================================
// 建球
// ============================================================

#[tokio::test]
async fn create_ball_then_recall_returns_its_content() {
    let core = core();
    let store = temp_store();
    let emb = axis_vec(0);

    let id = core
        .create_ball(&store, "用户的生日是 3 月 5 日", &emb, 3.0)
        .await
        .unwrap();

    let blocks = core.recall(&store, &emb, 1.0).await.unwrap();
    assert!(!blocks.is_empty(), "同向查询必须召回刚建的球");

    let text = joined_text(&blocks);
    assert!(text.contains("生日"), "注入内容应含球内容，实际: {text}");
    assert!(
        blocks.iter().any(|b| b.ball_ids.contains(&id)),
        "注入块的 ball_ids 应含 {id:?}"
    );
    assert_eq!(
        blocks[0].confidence,
        InjectionConfidence::Clear,
        "完全同向应判 clear"
    );
}

#[tokio::test]
async fn create_ball_content_is_readable_by_get_ball() {
    let core = core();
    let store = temp_store();
    let id = core
        .create_ball(&store, "要能被读回来的内容", &axis_vec(2), 1.5)
        .await
        .unwrap();

    let ball = core.get_ball(&store, id).await.unwrap().expect("球应存在");
    assert_eq!(ball.content_list.len(), 1);
    assert_eq!(ball.content_list[0].text, "要能被读回来的内容");
    assert!((ball.ball_stored_size - 1.5).abs() < 1e-9);
    assert_eq!(ball.status, BallStatus::Active);
}

// ============================================================
// 单源回忆
// ============================================================

#[tokio::test]
async fn recall_does_not_leak_unrelated_ball() {
    let core = core();
    let store = temp_store();
    core.create_ball(&store, "A 的内容：数据库密码在 vault", &axis_vec(0), 3.0)
        .await
        .unwrap();
    core.create_ball(&store, "B 的内容：今晚吃火锅", &axis_vec(1), 3.0)
        .await
        .unwrap();

    let blocks = core.recall(&store, &axis_vec(0), 1.0).await.unwrap();
    let text = joined_text(&blocks);
    assert!(text.contains("vault"), "相关内容应被注入，实际: {text}");
    assert!(
        !text.contains("火锅"),
        "正交且无管道相连的球不应被注入，实际: {text}"
    );
}

/// 管道传播：A 被注水后，水经粗管到达 B（B 与查询正交，只能靠管道点亮）。
///
/// 这里用 `FlowEngine` 而不是 `recall`——`recall` 之后还有注入过滤，
/// 正交球即便被点亮也不会进注入块，测不出"水有没有流过去"。
#[tokio::test]
async fn pipe_makes_neighbor_lit_by_flow() {
    let core = core();
    let store = temp_store();
    let a = core
        .create_ball(&store, "起始：项目用 Rust 写", &axis_vec(0), 3.0)
        .await
        .unwrap();
    let b = core
        .create_ball(&store, "邻居：CI 用 GitHub Actions", &axis_vec(1), 3.0)
        .await
        .unwrap();
    core.link_balls(&store, a, b, 0.9).await.unwrap();

    let engine = FlowEngine::new(Arc::new(Config::default()));
    let lit = engine.flow(&store, &axis_vec(0), 2.0).await.unwrap();

    assert!(
        lit.iter().any(|l| l.ball_id == a && l.depth == 0),
        "起始球应以 depth 0 被点亮，实际 {:?}",
        lit.iter().map(|l| (l.ball_id, l.depth)).collect::<Vec<_>>()
    );
    assert!(
        lit.iter().any(|l| l.ball_id == b && l.depth == 1),
        "邻居应经管道以 depth 1 被点亮，实际 {:?}",
        lit.iter().map(|l| (l.ball_id, l.depth)).collect::<Vec<_>>()
    );
}

// ============================================================
// 多源回忆
// ============================================================

#[tokio::test]
async fn recall_multi_merges_same_ball_from_two_sources() {
    let core = core();
    let store = temp_store();
    let emb = axis_vec(0);
    let id = core
        .create_ball(&store, "被两个源同时命中的球", &emb, 3.0)
        .await
        .unwrap();

    let blocks = core
        .recall_multi(&store, &[&emb, &emb], 1.0)
        .await
        .unwrap();

    let hits: Vec<_> = blocks
        .iter()
        .filter(|b| b.ball_ids.contains(&id))
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "同一个球只能出现在一个注入块里，实际 {} 个",
        hits.len()
    );
}

#[tokio::test]
async fn recall_multi_returns_balls_from_both_sources() {
    let core = core();
    let store = temp_store();
    let qa = axis_vec(0);
    let qb = axis_vec(1);
    core.create_ball(&store, "源 A 命中的记忆", &qa, 3.0)
        .await
        .unwrap();
    core.create_ball(&store, "源 B 命中的记忆", &qb, 3.0)
        .await
        .unwrap();

    let blocks = core
        .recall_multi(&store, &[&qa, &qb], 1.0)
        .await
        .unwrap();
    let text = joined_text(&blocks);
    assert!(text.contains("源 A"), "第一个源的结果应出现，实际: {text}");
    assert!(text.contains("源 B"), "第二个源的结果应出现，实际: {text}");
}

/// 多源相关性逐球取 max：球 B 只与**第二个**源相关，也必须被注入。
///
/// 若实现只拿第一个源算相关性，B 的 sim 恒为 0，直接被注入阈值过滤掉。
#[tokio::test]
async fn recall_multi_uses_max_relevance_across_sources() {
    let core = core();
    let store = temp_store();
    let qa = axis_vec(0);
    let qb = axis_vec(1);
    // B 只与 qb 同向，与 qa 正交
    core.create_ball(&store, "只跟第二个源相关的记忆", &qb, 3.0)
        .await
        .unwrap();

    let blocks = core
        .recall_multi(&store, &[&qa, &qb], 1.0)
        .await
        .unwrap();
    let text = joined_text(&blocks);
    assert!(
        text.contains("只跟第二个源相关"),
        "第二个源的相关性必须参与结算，实际: {text}"
    );
}

// ============================================================
// 倾斜相关性：验证阈值分级
// ============================================================

#[tokio::test]
async fn recall_confidence_reflects_similarity() {
    let core = core();
    let store = temp_store();
    // 与查询余弦 0.9 的球
    core.create_ball(&store, "高度相关的记忆", &tilted_vec(0, 0.9), 3.0)
        .await
        .unwrap();

    let blocks = core.recall(&store, &axis_vec(0), 1.0).await.unwrap();
    assert!(!blocks.is_empty(), "0.9 相似度应被注入");
    assert_eq!(
        blocks[0].confidence,
        InjectionConfidence::Clear,
        "0.9 ≥ render_clear_threshold 应判 clear"
    );
}
