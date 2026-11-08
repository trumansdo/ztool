//! 走 tracing 的指标实现
//!
//! 指标事件统一打到 `target = "dayu::metric"`，由 `logging::init` 路由到
//! `logs/metrics.jsonl`，与人类可读日志物理分离。

use crate::logging::METRIC_TARGET;
use crate::traits::MetricsRecorder;

/// 把指标写入 tracing 管道
///
/// 每行一条 JSON，字段固定为 `kind` / `name` / `value` / `labels`，
/// 便于 `jq -r 'select(.fields.name=="lit_ball_count") | .fields.value'` 聚合。
pub struct TracingMetricsRecorder;

/// 标签序列化包装：输出 `{"k":"v"}` 而非 Rust 的 `[("k", "v")]`
struct Labels<'a>(&'a [(&'a str, &'a str)]);

impl std::fmt::Display for Labels<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{")?;
        for (i, (k, v)) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{:?}:{:?}", k, v)?;
        }
        f.write_str("}")
    }
}

impl MetricsRecorder for TracingMetricsRecorder {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        tracing::info!(
            target: METRIC_TARGET,
            kind = "counter",
            name,
            value,
            labels = %Labels(labels),
        );
    }

    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        tracing::info!(
            target: METRIC_TARGET,
            kind = "gauge",
            name,
            value,
            labels = %Labels(labels),
        );
    }

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        tracing::info!(
            target: METRIC_TARGET,
            kind = "histogram",
            name,
            value,
            labels = %Labels(labels),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_render_as_json_object() {
        let labels = [("source", "recall"), ("tier", "strengthened")];
        assert_eq!(
            Labels(&labels).to_string(),
            r#"{"source":"recall","tier":"strengthened"}"#
        );
        assert_eq!(Labels(&[]).to_string(), "{}");
    }
}
