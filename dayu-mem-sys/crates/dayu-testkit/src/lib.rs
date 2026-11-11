//! dayu-testkit: 测试夹具与辅助函数
//!
//! 提供 sample_ball()、sample_pipe()、sample_config() 等夹具函数。
//! 供所有 crate 的测试使用。

use chrono::Utc;
use dayu_domain::types::*;
use dayu_domain::traits::*;

// ============================================================
// 测试目录
// ============================================================

/// 测试产物根目录：`<workspace>/test/out`
///
/// 落在 `out/` 而不是 `test/` 根：`test/out/` 是「一次性产物，整目录可删」的
/// 唯一位置（见 `test/README.md`），已在 .gitignore 里。测试跑出来的 db / 日志
/// 散落在 `test/` 根会破坏这个约定。
pub fn test_root() -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test/out");
    std::fs::create_dir_all(&root).expect("创建 test/out 目录失败");
    root
}

/// 本次测试进程的产物根目录：`test/out/run-<时间戳>-<pid>/`
///
/// 每个测试二进制进程一个目录，进程内所有 `temp_dir` 都落在它下面。
/// **跑完不删**——要能事后翻 db、日志、指标对比行为。
fn run_dir() -> &'static std::path::Path {
    static RUN: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    RUN.get_or_init(|| {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        // 测试二进制名（去掉 cargo 追加的哈希后缀），让目录一眼看出是哪个 crate 产生的
        let suite = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .and_then(|name| name.split('-').next().map(str::to_string))
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "test".to_string());
        let dir = test_root().join(format!("run-{suite}-{stamp}"));
        std::fs::create_dir_all(&dir).expect("创建测试运行目录失败");
        dir
    })
}

/// 在本次运行目录下创建**保留**的测试目录，返回其路径
///
/// 不返回 `TempDir`：产物要在测试结束后仍可查看。
/// 需要清理时手动删 `test/out/run-*`，或跑 `cargo test` 前自行清空。
pub fn temp_dir(prefix: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = run_dir().join(format!("{prefix}{seq}"));
    std::fs::create_dir_all(&dir).expect("创建测试目录失败");
    dir
}

// ============================================================
// 球夹具
// ============================================================

/// 创建一个简单测试球
pub fn sample_ball(id: u64, size: f64, embedding: Vec<f32>) -> Ball {
    Ball {
        id: BallId(id),
        embedding,
        ball_stored_size: size,
        ball_size_ceiling: 10.0,
        content_list: vec![],
        content_list_max: 5,
        last_watered: Utc::now(),
        ball_stability: 7.0,
        lit_count: 0,
        status: BallStatus::Active,
        domain: None,
        created_at: Utc::now(),
        schema_version: 1,
    }
}

/// 创建带内容的测试球
pub fn sample_ball_with_content(id: u64, size: f64, embedding: Vec<f32>, texts: &[&str]) -> Ball {
    let mut ball = sample_ball(id, size, embedding);
    for (i, text) in texts.iter().enumerate() {
        ball.content_list.push(Content {
            id: ContentId(i as u64),
            text: text.to_string(),
            encoding: ContentEncoding::Raw,
            source: TurnId(format!("turn_{}", i)),
            injected_count: 0,
            status: ContentStatus::Active,
        });
    }
    ball
}

/// 创建一批测试球（确定性嵌入）
pub fn sample_balls(count: u64) -> Vec<Ball> {
    (0..count)
        .map(|i| {
            let emb = vec![(i + 1) as f32 / count as f32; 512];
            sample_ball(i, 1.0 + (i as f64 * 0.5), emb)
        })
        .collect()
}

// ============================================================
// 管道夹具
// ============================================================

/// 创建测试管道
pub fn sample_pipe(id: u64, a: u64, b: u64, weight: f64) -> Pipe {
    let (a_id, b_id) = if a < b { (a, b) } else { (b, a) };
    Pipe {
        id: PipeId(id),
        a: BallId(a_id),
        b: BallId(b_id),
        pipe_base_thickness: weight,
        pipe_stability: 3.0,
        last_blocked_at: None,
        blocked_pass_count_stored: 0,
        last_flowed: Utc::now(),
        source: PipeSource::NewBall,
        status: PipeStatus::Active,
    }
}

/// 创建一批管道连接链式球
pub fn sample_chain_pipes(ball_count: u64, weight: f64) -> Vec<Pipe> {
    (0..ball_count - 1)
        .map(|i| sample_pipe(i, i, i + 1, weight))
        .collect()
}

// ============================================================
// 裁决夹具
// ============================================================

/// 创建测试裁决（产生新球）
pub fn sample_verdict_new_ball(significance: f64) -> Verdict {
    Verdict {
        should_judge: true,
        significance,
        offset_direction: "neutral".into(),
        emotion_valence: 0.0,
        emotion_intensity: 0.5,
        hit_dimensions: vec![],
        belongs_to: None,
        belong_confidence: 0.0,
        reason: "new concept".into(),
    }
}

/// 创建测试裁决（归属已有球）
pub fn sample_verdict_belong(belongs_to: u64, confidence: f64) -> Verdict {
    Verdict {
        should_judge: true,
        significance: 0.7,
        offset_direction: "neutral".into(),
        emotion_valence: 0.0,
        emotion_intensity: 0.5,
        hit_dimensions: vec![],
        belongs_to: Some(BallId(belongs_to)),
        belong_confidence: confidence,
        reason: "related to existing".into(),
    }
}

/// 创建测试裁决（不起念）
pub fn sample_verdict_skip() -> Verdict {
    Verdict {
        should_judge: false,
        significance: 0.1,
        offset_direction: "neutral".into(),
        emotion_valence: 0.0,
        emotion_intensity: 0.0,
        hit_dimensions: vec![],
        belongs_to: None,
        belong_confidence: 0.0,
        reason: "insignificant".into(),
    }
}

// ============================================================
// 配置夹具
// ============================================================

/// 默认测试配置（所有系数宽松，便于测试流动）
pub fn sample_config() -> std::sync::Arc<dayu_domain::types::Config> {
    std::sync::Arc::new(dayu_domain::types::Config::default())
}

// ============================================================
// 子图夹具
// ============================================================

/// 构建链式子图：ball_0 — pipe_0 — ball_1 — pipe_1 — ... — ball_{n-1}
pub fn build_chain_graph(n: u64) -> (Vec<Ball>, Vec<Pipe>) {
    let balls = sample_balls(n);
    let pipes = sample_chain_pipes(n, 0.5);
    (balls, pipes)
}

/// 构建细管道链：w=0.1，用于测试损失放大效应
pub fn build_thin_pipe_chain(n: u64) -> (Vec<Ball>, Vec<Pipe>) {
    let balls = sample_balls(n);
    let pipes = sample_chain_pipes(n, 0.1);
    (balls, pipes)
}

/// 构建粗管道链：w=0.9，用于测试损失最小化
pub fn build_thick_pipe_chain(n: u64) -> (Vec<Ball>, Vec<Pipe>) {
    let balls = sample_balls(n);
    let pipes = sample_chain_pipes(n, 0.9);
    (balls, pipes)
}

/// 构建星型子图：中心球连接所有外围球
pub fn build_star_graph(center_id: u64, satellite_count: u64) -> (Vec<Ball>, Vec<Pipe>) {
    let mut balls = vec![sample_ball(center_id, 5.0, vec![0.5; 512])];
    let mut pipes = Vec::new();
    for i in 0..satellite_count {
        let sid = center_id + 1 + i;
        balls.push(sample_ball(sid, 1.0, vec![0.1 * (i + 1) as f32; 512]));
        pipes.push(sample_pipe(i, center_id, sid, 0.3));
    }
    (balls, pipes)
}

// ============================================================
// Mock 实现
// ============================================================

/// Mock 时钟：固定时间
pub struct MockClock(pub chrono::DateTime<Utc>);

impl Clock for MockClock {
    fn now(&self) -> chrono::DateTime<Utc> {
        self.0
    }
}



// ============================================================
// 断言辅助
// ============================================================

/// 断言球大小在结算后符合预期
pub fn assert_ball_size(ball: &Ball, expected_min: f64, expected_max: f64) {
    assert!(
        ball.ball_stored_size >= expected_min && ball.ball_stored_size <= expected_max,
        "ball {} size {} not in [{}, {}]",
        ball.id.0, ball.ball_stored_size, expected_min, expected_max
    );
}

/// 断言管道粗细在结算后符合预期
pub fn assert_pipe_weight(pipe: &Pipe, expected_min: f64, expected_max: f64) {
    assert!(
        pipe.pipe_base_thickness >= expected_min && pipe.pipe_base_thickness <= expected_max,
        "pipe {} weight {} not in [{}, {}]",
        pipe.id.0, pipe.pipe_base_thickness, expected_min, expected_max
    );
}

// ============================================================
// 判别器夹具
// ============================================================

/// 创建测试用 JudgeInput
pub fn sample_judge_input(text: &str, embedding: Vec<f32>) -> JudgeInput {
    JudgeInput {
        candidate_text: text.to_string(),
        candidate_embedding: embedding,
        recent_window: vec!["上一句话".to_string()],
        candidate_archetypes: vec![(BallId(1), 0.5)],
    }
}

/// 创建测试用 Verdict
pub fn sample_verdict(significance: f64, belongs_to: Option<BallId>, belong_confidence: f64) -> Verdict {
    Verdict {
        should_judge: significance > 0.0,
        significance,
        offset_direction: String::new(),
        emotion_valence: 0.0,
        emotion_intensity: 0.0,
        hit_dimensions: vec![],
        belongs_to,
        belong_confidence,
        reason: "test verdict".to_string(),
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_ball_has_correct_id() {
        let ball = sample_ball(42, 3.0, vec![0.1; 512]);
        assert_eq!(ball.id, BallId(42));
        assert_eq!(ball.ball_stored_size, 3.0);
        assert_eq!(ball.status, BallStatus::Active);
    }

    #[test]
    fn sample_pipe_is_undirected_order() {
        let pipe = sample_pipe(1, 10, 5, 0.5);
        assert_eq!(pipe.a, BallId(5));
        assert_eq!(pipe.b, BallId(10));
        assert_eq!(pipe.pipe_base_thickness, 0.5);
    }

    #[test]
    fn build_chain_graph_correct_size() {
        let (balls, pipes) = build_chain_graph(5);
        assert_eq!(balls.len(), 5);
        assert_eq!(pipes.len(), 4);
    }
}
