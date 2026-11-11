//! dayu-storage 对外契约（crate 级集成测试）
//!
//! `StorageBackend` / `StorageTx` 是流动与结算唯一依赖的存储面。它们的行为
//! 若和实现细节耦合（比如「upsert 会不会覆盖 embedding」「rollback 到底回滚
//! 了什么」），上层就只能在单元测试里靠读实现来推断。
//!
//! 本文件只依赖 `pub` API + 真实 `TriviumStorage`，每条断言都是
//! **上层可以依赖的承诺**。

use dayu_domain::traits::StorageBackend;
use dayu_domain::types::*;
use dayu_storage::TriviumStorage;

/// 临时库：`TempDir` 必须活到测试结束，它一 Drop 文件就没了
struct Store {
    _dir: tempfile::TempDir,
    db: TriviumStorage,
}

impl std::ops::Deref for Store {
    type Target = TriviumStorage;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

fn new_store(dim: usize) -> Store {
    let dir = tempfile::tempdir().expect("[TEST] 建临时目录失败");
    let path = dir.path().join("test.tdb");
    let db = TriviumStorage::with_dim(path.to_str().expect("[TEST] 路径非 UTF-8"), dim)
        .expect("[TEST] 打开 TriviumDB 失败");
    Store { _dir: dir, db }
}

async fn seed(store: &Store, balls: Vec<Ball>, pipes: Vec<Pipe>) {
    let mut tx = store.begin_transaction().await.unwrap();
    for b in &balls {
        tx.upsert_ball(b).await.unwrap();
    }
    for p in &pipes {
        tx.upsert_pipe(p).await.unwrap();
    }
    tx.commit().await.unwrap();
}

fn ball(id: u64, size: f64, dim: usize) -> Ball {
    let mut b = dayu_testkit::sample_ball(id, size, vec![0.5; dim]);
    b.content_list = vec![Content {
        id: ContentId(7),
        text: format!("球{id}的内容"),
        encoding: ContentEncoding::Raw,
        source: TurnId(format!("session-{id}/turn-3")),
        injected_count: 2,
        status: ContentStatus::Active,
    }];
    b
}

#[tokio::test]
async fn allocated_ball_ids_are_monotonic_and_unique() {
    let store = new_store(8);
    let mut seen = std::collections::HashSet::new();
    let mut prev = 0u64;
    for _ in 0..50 {
        let id = store.allocate_ball_id().await.unwrap();
        assert!(id.0 > prev, "ID 必须严格递增：{} 之后是 {}", prev, id.0);
        assert!(seen.insert(id.0), "ID 不得重复：{}", id.0);
        prev = id.0;
    }
}

#[tokio::test]
async fn upsert_then_get_roundtrips_every_field() {
    let store = new_store(8);
    let original = ball(1, 3.5, 8);
    seed(&store, vec![original.clone()], vec![]).await;

    let got = store.get_ball(BallId(1)).await.unwrap().expect("球必须存在");

    assert_eq!(got.id, original.id);
    assert!((got.ball_stored_size - 3.5).abs() < 1e-9, "size 必须原样读回");
    assert_eq!(got.embedding, original.embedding, "向量必须原样读回");
    assert_eq!(got.content_list.len(), 1);
    let c = &got.content_list[0];
    assert_eq!(c.text, "球1的内容");
    assert_eq!(c.source, TurnId("session-1/turn-3".to_string()), "来源回合必须持久化");
    assert_eq!(c.injected_count, 2, "注入计数必须持久化");
    assert_eq!(c.status, ContentStatus::Active);
    assert_eq!(got.status, BallStatus::Active);
}

#[tokio::test]
async fn get_balls_matches_individual_gets() {
    let store = new_store(8);
    let balls: Vec<Ball> = (1..=5).map(|i| ball(i, i as f64, 8)).collect();
    seed(&store, balls.clone(), vec![]).await;

    let ids: Vec<BallId> = balls.iter().map(|b| b.id).collect();
    let batch = store.get_balls(&ids).await.unwrap();
    assert_eq!(batch.len(), 5);

    for b in &batch {
        let single = store.get_ball(b.id).await.unwrap().unwrap();
        assert_eq!(single.ball_stored_size, b.ball_stored_size);
        assert_eq!(single.embedding, b.embedding);
    }

    // 不存在的 ID 必须被静默跳过，而不是报错或返回占位
    let missing = store.get_balls(&[BallId(999)]).await.unwrap();
    assert!(missing.is_empty(), "不存在的球应被跳过：{missing:?}");
}

#[tokio::test]
async fn load_subgraph_respects_depth() {
    let store = new_store(8);
    // 链：1 - 2 - 3 - 4，粗细 0.5
    let balls: Vec<Ball> = (1..=4).map(|i| ball(i, 1.0, 8)).collect();
    let pipes = vec![
        dayu_testkit::sample_pipe(1, 1, 2, 0.5),
        dayu_testkit::sample_pipe(2, 2, 3, 0.5),
        dayu_testkit::sample_pipe(3, 3, 4, 0.5),
    ];
    seed(&store, balls, pipes).await;

    let depth1 = store.load_subgraph(&[BallId(1)], 1).await.unwrap();
    let ids1: std::collections::HashSet<u64> = depth1.balls.iter().map(|b| b.id.0).collect();
    assert_eq!(ids1, [1, 2].into_iter().collect(), "depth=1 只到直接邻居");

    let depth2 = store.load_subgraph(&[BallId(1)], 2).await.unwrap();
    let ids2: std::collections::HashSet<u64> = depth2.balls.iter().map(|b| b.id.0).collect();
    assert_eq!(ids2, [1, 2, 3].into_iter().collect(), "depth=2 到两跳");

    let depth9 = store.load_subgraph(&[BallId(1)], 9).await.unwrap();
    assert_eq!(depth9.balls.len(), 4, "深度超过链路长度时返回整条链");

    // 子图里的管道两端必须都在子图内，否则上层会拿到悬空边
    for p in &depth2.pipes {
        assert!(
            ids2.contains(&p.a.0) && ids2.contains(&p.b.0),
            "子图管道端点必须在球集合内：{p:?}"
        );
    }
}

#[tokio::test]
async fn increment_ball_size_materializes_decay() {
    let store = new_store(8);
    let mut original = ball(1, 5.0, 8);
    // 让球看起来是 30 天前浇的水：不物化的话 size 会从 5.0 继续累加
    original.last_watered = chrono::Utc::now() - chrono::Duration::days(30);
    original.ball_stability = 7.0;
    seed(&store, vec![original], vec![]).await;

    let now = chrono::Utc::now();
    let mut tx = store.begin_transaction().await.unwrap();
    tx.increment_ball_size(BallId(1), 1.0, now).await.unwrap();
    tx.commit().await.unwrap();

    let after = store.get_ball(BallId(1)).await.unwrap().unwrap();
    assert!(
        after.ball_stored_size < 5.0,
        "衰减必须物化：30 天后 5.0 + 1.0 不应得到 ≥6.0，实际 {}",
        after.ball_stored_size
    );
    assert!(
        after.last_watered > now - chrono::Duration::seconds(5),
        "浇水时间必须刷新"
    );
}

#[tokio::test]
async fn rollback_discards_all_writes() {
    let store = new_store(8);
    seed(&store, vec![ball(1, 1.0, 8)], vec![]).await;

    let mut tx = store.begin_transaction().await.unwrap();
    tx.upsert_ball(&ball(2, 9.0, 8)).await.unwrap();
    tx.update_ball_status(BallId(1), BallStatus::Archived).await.unwrap();
    tx.rollback().await.unwrap();

    assert!(
        store.get_ball(BallId(2)).await.unwrap().is_none(),
        "回滚后新球不得存在"
    );
    assert_eq!(
        store.get_ball(BallId(1)).await.unwrap().unwrap().status,
        BallStatus::Active,
        "回滚后状态必须复原"
    );
}

#[tokio::test]
async fn list_balls_filters_by_status() {
    let store = new_store(8);
    seed(&store, (1..=3).map(|i| ball(i, 1.0, 8)).collect(), vec![]).await;

    let mut tx = store.begin_transaction().await.unwrap();
    tx.update_ball_status(BallId(2), BallStatus::Archived).await.unwrap();
    tx.commit().await.unwrap();

    let active = store.list_balls(Some(BallStatus::Active)).await.unwrap();
    let archived = store.list_balls(Some(BallStatus::Archived)).await.unwrap();
    let all = store.list_balls(None).await.unwrap();

    assert_eq!(active.len(), 2);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].id, BallId(2));
    assert_eq!(all.len(), 3, "None 表示全部，含归档");
}

#[tokio::test]
async fn pipe_endpoints_are_normalized() {
    let store = new_store(8);
    seed(
        &store,
        vec![ball(1, 1.0, 8), ball(2, 1.0, 8)],
        // 故意反着传：a > b
        vec![dayu_testkit::sample_pipe(1, 2, 1, 0.5)],
    )
    .await;

    let pipes = store.list_pipes().await.unwrap();
    assert_eq!(pipes.len(), 1);
    assert!(
        pipes[0].a < pipes[0].b,
        "管道端点必须规范化成 a<b，实际 a={:?} b={:?}",
        pipes[0].a,
        pipes[0].b
    );
}

#[tokio::test]
async fn search_by_embedding_returns_nearest_first() {
    let store = new_store(8);
    let dim = 8;
    let mut balls = Vec::new();
    for i in 0..4usize {
        let mut v = vec![0.0f32; dim];
        v[i] = 1.0;
        balls.push(dayu_testkit::sample_ball(i as u64, 1.0, v));
    }
    seed(&store, balls, vec![]).await;

    // 查询贴近轴 2
    let mut q = vec![0.0f32; dim];
    q[2] = 1.0;
    let hits = store.search_by_embedding(&q, 2).await.unwrap();

    assert_eq!(hits.len(), 2, "top_k 必须被遵守");
    assert_eq!(hits[0].0, BallId(2), "最近的必须排第一");
    assert!(
        hits[0].1 > hits[1].1,
        "相似度必须降序：{:?}",
        hits.iter().map(|(id, s)| (id.0, s)).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn active_ball_count_ignores_archived() {
    let store = new_store(8);
    seed(&store, (1..=4).map(|i| ball(i, 1.0, 8)).collect(), vec![]).await;
    assert_eq!(store.active_ball_count().await.unwrap(), 4);

    let mut tx = store.begin_transaction().await.unwrap();
    tx.update_ball_status(BallId(1), BallStatus::Archived).await.unwrap();
    tx.commit().await.unwrap();

    assert_eq!(store.active_ball_count().await.unwrap(), 3);
}
