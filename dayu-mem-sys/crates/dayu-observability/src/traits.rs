//! 指标记录器 trait

/// 指标记录器 trait：可替换，支持 Mock / Prometheus / 文件等多种后端
pub trait MetricsRecorder: Send + Sync {
    /// 记录计数器（单调递增）
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// 记录瞬时值
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// 记录分布样本
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}
