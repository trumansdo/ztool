//! 流动引擎性能基线
//!
//! 运行：cargo bench -p dayu-flow
//!
//! 基线目标（阶段 2/3 验证标准）：
//! - 单次流动（50 球子图、5 跳）：<1ms
//! - 单次结算 + 注入：<2ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dayu_domain::traits::*;
use dayu_domain::types::*;
use dayu_flow::engine::FlowEngine;
use dayu_flow::settle::SettleEngine;
use dayu_flow::loss::PipeWeightLossModel;
use dayu_storage::testing::{seed, temp_store};
use dayu_testkit::*;
use std::sync::Arc;

/// 构造 50 球链图用于性能测试
async fn build_bench_graph(store: &dayu_storage::testing::TestStore, n: usize) {
    let mut balls = Vec::new();
    let mut pipes = Vec::new();
    for i in 0..n as u64 {
        balls.push(sample_ball_with_content(
            i, 1.0 + i as f64 * 0.1,
            vec![(i + 1) as f32 / n as f32; 384],
            &["bench content"],
        ));
        if i > 0 {
            pipes.push(sample_pipe(i - 1, i - 1, i, 0.5));
        }
    }
    seed(store, balls, pipes).await;
}

fn bench_flow_50_balls_5_hops(c: &mut Criterion) {
    let store = temp_store();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(build_bench_graph(&store, 50));

    let config = sample_config();
    let loss_model = Arc::new(PipeWeightLossModel);
    let engine = FlowEngine::with_loss_model(config, loss_model);

    let query = vec![0.5; 384];

    c.bench_function("flow_50_balls_5_hops", |b| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        b.iter(|| {
            rt.block_on(async {
                let _ = engine.flow(black_box(&store), black_box(&query), black_box(1.0)).await;
            });
        })
    });
}

fn bench_settle_and_inject(c: &mut Criterion) {
    let store = temp_store();
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(build_bench_graph(&store, 50));

    let config = sample_config();
    let loss_model = Arc::new(PipeWeightLossModel);
    let flow_engine = FlowEngine::with_loss_model(Arc::clone(&config), loss_model);
    let settle_engine = SettleEngine::new(Arc::clone(&config));

    let query = vec![0.5; 384];

    // 先执行一次流动获取点亮结果
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let lit_balls = rt.block_on(async {
        flow_engine.flow(&store, &query, 1.0).await.unwrap()
    });
    let ball_ids: Vec<BallId> = lit_balls.iter().map(|l| l.ball_id).collect();
    let sub = rt.block_on(async {
        store.load_subgraph(&ball_ids, 5).await.unwrap()
    });

    c.bench_function("settle_and_inject", |b| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        b.iter(|| {
            rt.block_on(async {
                let _ = settle_engine.settle(
                    black_box(&store),
                    black_box(&lit_balls),
                    black_box(&sub),
                    black_box(&[query.as_slice()]),
                    black_box(&[]),
                ).await;
            });
        })
    });
}

criterion_group!(benches, bench_flow_50_balls_5_hops, bench_settle_and_inject);
criterion_main!(benches);
