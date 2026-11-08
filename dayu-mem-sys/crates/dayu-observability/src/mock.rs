//! Mock 指标记录器：内存 push，零性能影响，用于测试断言

use std::sync::Mutex;
use crate::traits::MetricsRecorder;

/// 单条指标记录
#[derive(Debug, Clone)]
pub enum MetricEntry {
    Counter {
        name: String,
        value: u64,
        labels: Vec<(String, String)>,
    },
    Gauge {
        name: String,
        value: f64,
        labels: Vec<(String, String)>,
    },
    Histogram {
        name: String,
        value: f64,
        labels: Vec<(String, String)>,
    },
}

impl MetricEntry {
    pub fn name(&self) -> &str {
        match self {
            MetricEntry::Counter { name, .. } => name,
            MetricEntry::Gauge { name, .. } => name,
            MetricEntry::Histogram { name, .. } => name,
        }
    }

    pub fn labels(&self) -> &[(String, String)] {
        match self {
            MetricEntry::Counter { labels, .. } => labels,
            MetricEntry::Gauge { labels, .. } => labels,
            MetricEntry::Histogram { labels, .. } => labels,
        }
    }

    /// 查找标签值
    pub fn label_value(&self, key: &str) -> Option<&str> {
        self.labels().iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
}

/// Mock 指标记录器
pub struct MockMetricsRecorder {
    metrics: Mutex<Vec<MetricEntry>>,
}

impl MockMetricsRecorder {
    pub fn new() -> Self {
        Self {
            metrics: Mutex::new(Vec::new()),
        }
    }

    /// 取出所有已记录指标（消耗性读取）
    pub fn drain(&self) -> Vec<MetricEntry> {
        let mut guard = self.metrics.lock().unwrap();
        std::mem::take(&mut *guard)
    }

    /// 按名称过滤指标
    pub fn filter_by_name(&self, name: &str) -> Vec<MetricEntry> {
        let guard = self.metrics.lock().unwrap();
        guard
            .iter()
            .filter(|e| e.name() == name)
            .cloned()
            .collect()
    }

    /// 统计不同指标名称的数量
    pub fn distinct_metric_count(&self) -> usize {
        let guard = self.metrics.lock().unwrap();
        let mut names: Vec<&str> = guard.iter().map(|e| e.name()).collect();
        names.sort();
        names.dedup();
        names.len()
    }

    /// 总记录条数
    pub fn total_count(&self) -> usize {
        self.metrics.lock().unwrap().len()
    }
}

impl Default for MockMetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRecorder for MockMetricsRecorder {
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let mut guard = self.metrics.lock().unwrap();
        guard.push(MetricEntry::Counter {
            name: name.to_string(),
            value,
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        });
    }

    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let mut guard = self.metrics.lock().unwrap();
        guard.push(MetricEntry::Gauge {
            name: name.to_string(),
            value,
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        });
    }

    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let mut guard = self.metrics.lock().unwrap();
        guard.push(MetricEntry::Histogram {
            name: name.to_string(),
            value,
            labels: labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        });
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_recorder_stores_metrics() {
        let recorder = MockMetricsRecorder::new();
        recorder.record_counter("test_counter", 42, &[("status", "ok")]);
        recorder.record_gauge("test_gauge", 3.5, &[]);
        recorder.record_histogram("test_histogram", 0.5, &[("bucket", "a")]);

        assert_eq!(recorder.total_count(), 3);
        assert_eq!(recorder.distinct_metric_count(), 3);
    }

    #[test]
    fn mock_recorder_filter_by_name() {
        let recorder = MockMetricsRecorder::new();
        recorder.record_counter("a", 1, &[]);
        recorder.record_counter("a", 2, &[]);
        recorder.record_counter("b", 3, &[]);

        let a_metrics = recorder.filter_by_name("a");
        assert_eq!(a_metrics.len(), 2);
    }

    #[test]
    fn mock_recorder_drain_clears() {
        let recorder = MockMetricsRecorder::new();
        recorder.record_counter("x", 1, &[]);
        assert_eq!(recorder.total_count(), 1);

        let drained = recorder.drain();
        assert_eq!(drained.len(), 1);
        assert_eq!(recorder.total_count(), 0);
    }

    #[test]
    fn metric_entry_label_value() {
        let recorder = MockMetricsRecorder::new();
        recorder.record_gauge("m", 1.0, &[("dominant", "loss"), ("source", "flow")]);

        let entries = recorder.drain();
        assert_eq!(entries[0].label_value("dominant"), Some("loss"));
        assert_eq!(entries[0].label_value("source"), Some("flow"));
        assert_eq!(entries[0].label_value("nonexistent"), None);
    }
}
