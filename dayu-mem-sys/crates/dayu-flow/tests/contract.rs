//! dayu-flow 对外契约（crate 级集成测试）
//!
//! 与 `src/` 内的单元测试分工：
//!
//! | | `src/*.rs` 的 `mod tests` | 本文件 |
//! |:--|:--|:--|
//! | 可见性 | 能访问私有函数 | **只能用 `pub` API** |
//! | 关注点 | 单个函数/分支对不对 | 对外承诺的行为对不对 |
//! | 依赖 | 内存桩 `SubGraph` | 真实 `TriviumStorage` |
//!
//! 这里断言的每一条都是**调用方可以依赖的承诺**，改实现时若破坏它们，
//! 应该先讨论语义再改测试——而不是顺手改断言。

use std::sync::Arc;

use dayu_domain::error::{FlowError, SettleError};
use dayu_domain::traits::StorageBackend;
use dayu_domain::types::*;
use dayu_flow::{FlowEngine, SettleEngine};
use dayu_storage::testing::{seed, temp_store};

/// 三球链：0 —w=0.9— 1 —w=0.9— 2，向量两两正交
///
/// 正交是为了让「起始球选择」只命中球 0（余弦 1.0），其余为 0，
/// 低于默认 `start_ball_threshold = 0.3`。这样后续点亮只可能来自扩散。
fn orthogonal_chain() -> (Vec<Ball>, Vec<Pipe>, Vec<f32>) {
    let dim = 8;
    let axis = |i: usize| {
        let mut v = vec![0.0f32; dim];
        v[i] = 1.0;
        v
    };
    let balls = vec![
        dayu_testkit::sample_ball(0, 1.0, axis(0)),
        dayu_testkit::sample_ball(1, 1.0, axis(1)),
        dayu_testkit::sample_ball(2, 1.0, axis(2)),
    ];
    let pipes = vec![
        dayu_testkit::sample_pipe(0, 0, 1, 0.9),
        dayu_testkit::sample_pipe(1, 1, 2, 0.9),
    ];
    (balls, pipes, axis(0))
}

#[tokio::test]
async fn flow_lights_reachable_balls_without_writing() {
    let (balls, pipes, query) = orthogonal_chain();
    let store = temp_store();
    seed(&store, balls.clone(), pipes).await;

    let before: Vec<f64> = {
        let mut v = Vec::new();
        for i in 0..3u64 {
            v.push(store.get_ball(BallId(i)).await.unwrap().unwrap().ball_stored_size);
        }
        v
    };

    let engine = FlowEngine::new(dayu_testkit::sample_config());
    let outcome = engine.flow(&store, &query, 1.0).await.unwrap();

    let lit: Vec<u64> = outcome.lit_balls.iter().map(|b| b.ball_id.0).collect();
    assert_eq!(lit.first(), Some(&0), "起始球必须是被点亮的第一个");
    assert!(lit.contains(&1), "厚管道（w=0.9）必须能把水送过一跳：{lit:?}");
    assert!(lit.contains(&2), "水应继续扩散到第二跳：{lit:?}");

    // 流动是纯读：不写库，球大小一分不变（写入只发生在 settle）
    let after: Vec<f64> = {
        let mut v = Vec::new();
        for i in 0..3u64 {
            v.push(store.get_ball(BallId(i)).await.unwrap().unwrap().ball_stored_size);
        }
        v
    };
    assert_eq!(before, after, "flow 不得修改球状态");
}

#[tokio::test]
async fn flow_on_empty_store_reports_no_starting_balls() {
    let store = temp_store();
    store.ensure(8); // 先固定维度，避免维度不符先于「空库」报错
    let engine = FlowEngine::new(dayu_testkit::sample_config());
    let err = engine.flow(&store, &[1.0; 8], 1.0).await.unwrap_err();
    assert!(
        matches!(err, FlowError::NoStartingBalls),
        "空库必须报 NoStartingBalls，实际：{err:?}"
    );
}

#[tokio::test]
async fn flow_rejects_dimension_mismatch() {
    let (balls, pipes, _) = orthogonal_chain();
    let store = temp_store();
    seed(&store, balls, pipes).await; // 库维度 = 8

    let engine = FlowEngine::new(dayu_testkit::sample_config());
    let err = engine.flow(&store, &[1.0; 16], 1.0).await.unwrap_err();
    assert!(
        matches!(err, FlowError::Storage(_)),
        "维度不符必须报存储错误而不是静默返回空结果，实际：{err:?}"
    );
}

#[tokio::test]
async fn flow_multi_with_single_source_equals_flow() {
    let (balls, pipes, query) = orthogonal_chain();
    let store = temp_store();
    seed(&store, balls, pipes).await;

    let engine = FlowEngine::new(dayu_testkit::sample_config());
    let single = engine.flow(&store, &query, 1.0).await.unwrap();
    let multi = engine.flow_multi(&store, &[&query], 1.0).await.unwrap();

    // 单源时源放大系数恒为 1，两条路径必须完全等价
    let a: Vec<(u64, usize)> = single.lit_balls.iter().map(|b| (b.ball_id.0, b.depth)).collect();
    let b: Vec<(u64, usize)> = multi.lit_balls.iter().map(|b| (b.ball_id.0, b.depth)).collect();
    assert_eq!(a, b, "flow 与 flow_multi 单源必须等价");
}

#[tokio::test]
async fn settle_rejects_empty_lit_balls() {
    let store = temp_store();
    let engine = SettleEngine::new(dayu_testkit::sample_config());
    let sub = SubGraph {
        balls: vec![],
        pipes: vec![],
    };
    let err = engine
        .settle(&store, &[], &sub, &[&[1.0; 8]], &[])
        .await
        .unwrap_err();
    assert!(
        matches!(err, SettleError::NoLitBalls),
        "无点亮球必须报 NoLitBalls，实际：{err:?}"
    );
}

#[tokio::test]
async fn settle_grows_ball_and_thickens_travelled_pipe() {
    let (balls, pipes, query) = orthogonal_chain();
    let store = temp_store();
    seed(&store, balls, pipes).await;

    let config = dayu_testkit::sample_config();
    let flow = FlowEngine::new(Arc::clone(&config));
    let settle = SettleEngine::new(Arc::clone(&config));

    let outcome = flow.flow(&store, &query, 1.0).await.unwrap();
    let sub = store
        .load_subgraph(&[BallId(0)], config.max_flow_depth)
        .await
        .unwrap();

    let size_before = store.get_ball(BallId(0)).await.unwrap().unwrap().ball_stored_size;
    let pipe_before = store
        .list_pipes()
        .await
        .unwrap()
        .into_iter()
        .find(|p| p.a == BallId(0) && p.b == BallId(1))
        .expect("0↔1 管道应存在")
        .pipe_base_thickness;

    let result = settle
        .settle(&store, &outcome.lit_balls, &sub, &[&query], &outcome.blocked_pipes)
        .await
        .unwrap();

    assert!(result.reinforced_balls >= 1, "起始球必须拿到强化");

    let size_after = store.get_ball(BallId(0)).await.unwrap().unwrap().ball_stored_size;
    assert!(size_after > size_before, "强化后球应变大：{size_before} → {size_after}");

    let pipe_after = store
        .list_pipes()
        .await
        .unwrap()
        .into_iter()
        .find(|p| p.a == BallId(0) && p.b == BallId(1))
        .expect("0↔1 管道应存在")
        .pipe_base_thickness;
    assert!(
        pipe_after > pipe_before,
        "被水走通的管道应变粗：{pipe_before} → {pipe_after}"
    );
}

#[tokio::test]
async fn settle_never_exceeds_token_budget() {
    let (balls, pipes, query) = orthogonal_chain();
    let store = temp_store();
    seed(&store, balls, pipes).await;

    // 给球塞点内容，让 token 预算真正起作用
    let mut tx = store.begin_transaction().await.unwrap();
    for i in 0..3u64 {
        let mut ball = store.get_ball(BallId(i)).await.unwrap().unwrap();
        ball.content_list = vec![Content {
            id: ContentId(0),
            text: "这是一段用来占 token 的内容".repeat(20),
            encoding: ContentEncoding::Raw,
            source: TurnId(format!("t{i}")),
            injected_count: 0,
            status: ContentStatus::Active,
        }];
        tx.upsert_ball(&ball).await.unwrap();
    }
    tx.commit().await.unwrap();

    let mut config = (*dayu_testkit::sample_config()).clone();
    config.token_budget = 32;
    let config = Arc::new(config);

    let flow = FlowEngine::new(Arc::clone(&config));
    let settle = SettleEngine::new(Arc::clone(&config));

    let outcome = flow.flow(&store, &query, 1.0).await.unwrap();
    let sub = store.load_subgraph(&[BallId(0)], config.max_flow_depth).await.unwrap();
    let result = settle
        .settle(&store, &outcome.lit_balls, &sub, &[&query], &outcome.blocked_pipes)
        .await
        .unwrap();

    assert!(
        result.total_tokens <= config.token_budget,
        "注入 token {} 超出预算 {}",
        result.total_tokens,
        config.token_budget
    );
    assert!(
        result.injection_blocks.len() <= config.max_balls_per_inject,
        "注入块数 {} 超出上限 {}",
        result.injection_blocks.len(),
        config.max_balls_per_inject
    );
}

#[tokio::test]
async fn settle_only_injects_lit_balls() {
    let (balls, pipes, query) = orthogonal_chain();
    let store = temp_store();
    seed(&store, balls, pipes).await;

    let config = dayu_testkit::sample_config();
    let flow = FlowEngine::new(Arc::clone(&config));
    let settle = SettleEngine::new(Arc::clone(&config));

    let outcome = flow.flow(&store, &query, 1.0).await.unwrap();
    let sub = store.load_subgraph(&[BallId(0)], config.max_flow_depth).await.unwrap();
    let result = settle
        .settle(&store, &outcome.lit_balls, &sub, &[&query], &outcome.blocked_pipes)
        .await
        .unwrap();

    let lit: std::collections::HashSet<BallId> =
        outcome.lit_balls.iter().map(|b| b.ball_id).collect();
    for block in &result.injection_blocks {
        for id in &block.ball_ids {
            assert!(lit.contains(id), "注入块里出现了没被点亮的球：{id:?}");
        }
    }
}
