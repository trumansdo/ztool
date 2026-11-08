//! 日志与指标落盘初始化
//!
//! 三路输出，各自独立：
//! - **控制台**：人类可读，级别由 `Config::log_level` 决定
//! - **`logs/dayu.log.<date>`**：JSON 全量，按天轮转，保留 `log_retention_days` 天
//! - **`logs/metrics.jsonl.<date>`**：仅 `target = "dayu::metric"` 的事件，按天轮转
//!
//! 指标与日志分流的意义：日志是给人 `rg` 的证据链，指标是给 `jq`/pandas 聚合的数值流，
//! 两者保留周期与采样策略不同，混在一个文件里会让两边都难用。

use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

/// 指标事件的 tracing target，用于与普通日志分流
pub const METRIC_TARGET: &str = "dayu::metric";

/// 初始化失败原因
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    /// 日志目录创建或文件打开失败
    #[error("日志目录初始化失败: {0}")]
    Io(#[from] std::io::Error),
    /// 轮转文件追加器初始化失败
    #[error("日志文件轮转初始化失败: {0}")]
    Appender(#[from] tracing_appender::rolling::InitError),
    /// 全局 subscriber 已被其他代码设置（重复初始化）
    #[error("全局日志订阅器已被初始化，跳过重复设置")]
    AlreadyInitialized,
}

/// 可观测性守卫
///
/// 持有 `WorkerGuard` 以保证非阻塞写入线程在进程退出前把缓冲刷到磁盘。
/// **必须在 `main` / `init` 作用域内长期存活**，提前 drop 会丢尾部日志。
pub struct ObservabilityGuard {
    _log_guard: WorkerGuard,
    _metrics_guard: Option<WorkerGuard>,
}

/// 按配置初始化日志与指标输出
///
/// 重复调用返回 `AlreadyInitialized`，调用方应视为「已初始化」而非致命错误。
pub fn init(log_dir: &str, log_level: &str, log_retention_days: usize, metrics_enabled: bool, metrics_retention_days: usize) -> Result<ObservabilityGuard, InitError> {
    let dir = Path::new(log_dir);
    std::fs::create_dir_all(dir)?;

    let level = log_level.trim().to_ascii_lowercase();
    // 用户没写定向规则时，压掉底层存储引擎的刷屏日志（每次 flush 都打 INFO）
    let directive = if level.contains('=') {
        level
    } else {
        format!("{level},triviumdb=warn")
    };
    let filter = EnvFilter::try_new(&directive).unwrap_or_else(|_| EnvFilter::new("info"));

    // --- 主日志文件：JSON，排除指标事件 ---
    let log_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("dayu")
        .filename_suffix("log")
        .max_log_files(log_retention_days.max(1))
        .build(dir)?;
    let (log_writer, log_guard) = tracing_appender::non_blocking(log_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(log_writer)
        .with_ansi(false)
        .with_filter(filter_fn(|meta| meta.target() != METRIC_TARGET))
        .with_filter(filter.clone());

    // --- 控制台：人类可读 ---
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(filter);

    // --- 指标文件：只收指标事件 ---
    let (metrics_layer, metrics_guard) = if metrics_enabled {
        let appender = RollingFileAppender::builder()
            .rotation(Rotation::DAILY)
            .filename_prefix("metrics")
            .filename_suffix("jsonl")
            .max_log_files(metrics_retention_days.max(1))
            .build(dir)?;
        let (writer, guard) = tracing_appender::non_blocking(appender);
        // span 事件必须放行：`fmt` 层靠 `on_new_span` 收集字段，
        // 否则指标事件里的 `qid` 等 span 上下文会全部丢失。
        let layer = tracing_subscriber::fmt::layer()
            .json()
            .with_writer(writer)
            .with_ansi(false)
            .with_filter(filter_fn(|meta| {
                meta.is_span() || meta.target() == METRIC_TARGET
            }));
        (Some(layer), Some(guard))
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(metrics_layer)
        .try_init()
        .map_err(|_| InitError::AlreadyInitialized)?;

    Ok(ObservabilityGuard {
        _log_guard: log_guard,
        _metrics_guard: metrics_guard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证三路分流：主日志不含指标事件，指标文件只含指标事件
    #[test]
    fn init_splits_log_and_metrics_files() {
        let dir = dayu_testkit::temp_dir("obs-");
        let guard = init(dir.to_str().unwrap(), "info", 3, true, 3).expect("init");

        tracing::info!(qid = "q-test", "plain log line");
        tracing::info!(
            target: METRIC_TARGET,
            kind = "gauge",
            name = "lit_ball_count",
            value = 7,
            labels = "{}",
        );
        drop(guard); // 刷盘并回收写线程

        let read_all = |prefix: &str| -> String {
            let mut out = String::new();
            for entry in std::fs::read_dir(&dir).expect("read_dir") {
                let path = entry.expect("entry").path();
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
                {
                    out.push_str(&std::fs::read_to_string(&path).expect("read file"));
                }
            }
            out
        };

        let logs = read_all("dayu");
        let metrics = read_all("metrics");

        assert!(logs.contains("plain log line"), "主日志应含普通日志: {logs}");
        assert!(
            !logs.contains("lit_ball_count"),
            "主日志不应含指标事件: {logs}"
        );
        assert!(
            metrics.contains("lit_ball_count"),
            "指标文件应含指标事件: {metrics}"
        );
        assert!(
            !metrics.contains("plain log line"),
            "指标文件不应含普通日志: {metrics}"
        );
    }
}
