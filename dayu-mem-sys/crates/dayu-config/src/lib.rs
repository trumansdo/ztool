//! dayu-config: 配置加载、热更新、版本化
//!
//! TOML 解析 → ConfigSnapshot → arc-swap 热更新。
//! 所有系数、阈值、预算可配置。

use arc_swap::ArcSwap;
use dayu_domain::types::Config;
use std::path::Path;
use std::sync::Arc;

// ============================================================
// 错误类型
// ============================================================

/// 配置错误
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML 解析错误: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("配置校验失败: {0}")]
    Validation(String),
}

// ============================================================
// ConfigManager
// ============================================================

/// 配置管理器：持有热更新配置
pub struct ConfigManager {
    inner: ArcSwap<Config>,
}

impl ConfigManager {
    /// 用默认配置创建
    pub fn with_default() -> Self {
        Self {
            inner: ArcSwap::from(Arc::new(Config::default())),
        }
    }

    /// 从 TOML 文件加载配置，失败时回退到默认值
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        validate_config(&config)?;
        Ok(Self {
            inner: ArcSwap::from(Arc::new(config)),
        })
    }

    /// 从 TOML 文件加载，合并默认值（文件中缺失的字段用默认值）
    pub fn from_file_or_default(path: &Path) -> Result<Self, ConfigError> {
        match Self::from_file(path) {
            Ok(mgr) => Ok(mgr),
            Err(_) => {
                tracing::warn!("配置加载失败，使用默认配置: {}", path.display());
                Ok(Self::with_default())
            }
        }
    }

    /// 获取当前配置快照
    pub fn snapshot(&self) -> Arc<Config> {
        self.inner.load_full()
    }

    /// 热更新：从文件重新加载配置
    pub fn reload(&self, path: &Path) -> Result<Arc<Config>, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        validate_config(&config)?;
        let arc = Arc::new(config);
        self.inner.store(Arc::clone(&arc));
        Ok(arc)
    }

    /// 热更新：直接设置配置
    pub fn update(&self, config: Config) -> Result<Arc<Config>, ConfigError> {
        validate_config(&config)?;
        let arc = Arc::new(config);
        self.inner.store(Arc::clone(&arc));
        Ok(arc)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::with_default()
    }
}

// ============================================================
// 配置校验
// ============================================================

/// 校验配置合法性
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    if config.max_flow_depth == 0 {
        return Err(ConfigError::Validation(
            "max_flow_depth 必须 > 0".into(),
        ));
    }
    if config.max_lit_ball_count == 0 {
        return Err(ConfigError::Validation(
            "max_lit_ball_count 必须 > 0".into(),
        ));
    }
    if config.start_top_n == 0 {
        return Err(ConfigError::Validation(
            "start_top_n 必须 > 0".into(),
        ));
    }
    if config.start_ball_threshold < 0.0 || config.start_ball_threshold > 1.0 {
        return Err(ConfigError::Validation(format!(
            "start_ball_threshold 必须在 [0,1] 范围内，当前: {}",
            config.start_ball_threshold
        )));
    }
    if config.start_ball_relative_threshold <= 0.0 || config.start_ball_relative_threshold > 1.0 {
        return Err(ConfigError::Validation(format!(
            "start_ball_relative_threshold 必须在 (0,1] 范围内，当前: {}",
            config.start_ball_relative_threshold
        )));
    }
    if config.start_dedupe_threshold < 0.0 || config.start_dedupe_threshold > 1.0 {
        return Err(ConfigError::Validation(format!(
            "start_dedupe_threshold 必须在 [0,1] 范围内（0 = 关闭去重），当前: {}",
            config.start_dedupe_threshold
        )));
    }
    if config.total_water_budget <= 0.0 {
        return Err(ConfigError::Validation(
            "total_water_budget 必须 > 0".into(),
        ));
    }
    if config.supplement_ratio < 0.0 || config.supplement_ratio > 1.0 {
        return Err(ConfigError::Validation(format!(
            "supplement_ratio 必须在 [0,1] 范围内，当前: {}",
            config.supplement_ratio
        )));
    }
    if config.base_loss <= 0.0 || config.base_loss >= 1.0 {
        return Err(ConfigError::Validation(format!(
            "base_loss 必须在 (0,1) 范围内，当前: {}",
            config.base_loss
        )));
    }
    if config.loss_pipe_factor < 0.0 {
        return Err(ConfigError::Validation(format!(
            "loss_pipe_factor 必须 >= 0，当前: {}",
            config.loss_pipe_factor
        )));
    }
    if config.loss_max <= 0.0 || config.loss_max >= 1.0 {
        return Err(ConfigError::Validation(format!(
            "loss_max 必须在 (0,1) 范围内，当前: {}",
            config.loss_max
        )));
    }
    // base_loss + base_loss × loss_pipe_factor 必须 < 1.0，否则 loss_max 形同虚设
    if config.base_loss * (1.0 + config.loss_pipe_factor) >= 1.0 {
        return Err(ConfigError::Validation(format!(
            "base_loss × (1 + loss_pipe_factor) = {} 必须 < 1.0",
            config.base_loss * (1.0 + config.loss_pipe_factor)
        )));
    }
    if config.min_water_threshold < 0.0 {
        return Err(ConfigError::Validation(
            "min_water_threshold 必须 >= 0".into(),
        ));
    }
    if !(0.0..=1.0).contains(&config.strengthen_eligibility_threshold) {
        return Err(ConfigError::Validation(
            "strengthen_eligibility_threshold 必须在 [0, 1] 内（语义相似度阈值）".into(),
        ));
    }
    if !(0.0..=1.0).contains(&config.min_strengthen_value) {
        return Err(ConfigError::Validation(
            "min_strengthen_value 必须在 [0, 1] 内（相对 base_reinforcement 的比例）".into(),
        ));
    }
    if config.base_reinforcement < 0.0 {
        return Err(ConfigError::Validation(
            "base_reinforcement 必须 >= 0".into(),
        ));
    }
    if config.supplement_reinforce_ratio < 0.0 {
        return Err(ConfigError::Validation(
            "supplement_reinforce_ratio 必须 >= 0".into(),
        ));
    }
    if config.min_reinforcement_threshold < 0.0 {
        return Err(ConfigError::Validation(
            "min_reinforcement_threshold 必须 >= 0".into(),
        ));
    }
    if config.pipe_strengthen_factor < 0.0 {
        return Err(ConfigError::Validation(
            "pipe_strengthen_factor 必须 >= 0".into(),
        ));
    }
    if config.min_pipe_reinforce_threshold < 0.0 {
        return Err(ConfigError::Validation(
            "min_pipe_reinforce_threshold 必须 >= 0".into(),
        ));
    }
    if config.token_budget == 0 {
        return Err(ConfigError::Validation(
            "token_budget 必须 > 0".into(),
        ));
    }
    if config.max_balls_per_inject == 0 {
        return Err(ConfigError::Validation(
            "max_balls_per_inject 必须 > 0".into(),
        ));
    }
    if config.render_clear_threshold < 0.0 || config.render_clear_threshold > 1.0 {
        return Err(ConfigError::Validation(format!(
            "render_clear_threshold 必须在 [0,1] 范围内，当前: {}",
            config.render_clear_threshold
        )));
    }
    if config.impression_threshold < 0.0 || config.impression_threshold > 1.0 {
        return Err(ConfigError::Validation(format!(
            "impression_threshold 必须在 [0,1] 范围内，当前: {}",
            config.impression_threshold
        )));
    }
    if config.impression_threshold > config.render_clear_threshold {
        return Err(ConfigError::Validation(format!(
            "impression_threshold ({}) 必须 <= render_clear_threshold ({})",
            config.impression_threshold, config.render_clear_threshold
        )));
    }
    // 渲染四档必须单调：印象球下限 ≤ 浅印象 ≤ 中印象 ≤ 深印象 ≤ 清晰
    if !(config.impression_threshold <= config.render_impression_threshold
        && config.render_impression_threshold <= config.render_deep_impression_threshold
        && config.render_deep_impression_threshold <= config.render_clear_threshold)
    {
        return Err(ConfigError::Validation(format!(
            "渲染档位必须单调递增：impression({}) ≤ render_impression({}) ≤ render_deep({}) ≤ clear({})",
            config.impression_threshold,
            config.render_impression_threshold,
            config.render_deep_impression_threshold,
            config.render_clear_threshold
        )));
    }
    // 补充阈值必须高于起始阈值，否则“够格补充的球”早就被选成起始球了
    if config.supplement_threshold <= config.start_ball_threshold
        || config.supplement_threshold >= 1.0
    {
        return Err(ConfigError::Validation(format!(
            "supplement_threshold ({}) 必须在 (start_ball_threshold={}, 1.0) 内",
            config.supplement_threshold, config.start_ball_threshold
        )));
    }
    if !(0.0..=0.3).contains(&config.flow_context_blend_alpha) {
        return Err(ConfigError::Validation(format!(
            "flow_context_blend_alpha 必须在 [0, 0.3] 内（原始查询至少占 70%），当前: {}",
            config.flow_context_blend_alpha
        )));
    }
    if config.sort_exponent_b < 0.0 {
        return Err(ConfigError::Validation(
            "sort_exponent_b 必须 >= 0".into(),
        ));
    }
    if config.tokens_per_size <= 0.0 {
        return Err(ConfigError::Validation(
            "tokens_per_size 必须 > 0".into(),
        ));
    }
    if config.size_saturation_gamma < 0.0 {
        return Err(ConfigError::Validation(
            "size_saturation_gamma 必须 >= 0".into(),
        ));
    }
    Ok(())
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 合法 TOML 内容
    const VALID_TOML: &str = r#"
max_flow_depth = 5
max_lit_ball_count = 50
start_top_n = 3
start_ball_threshold = 0.3
start_ball_relative_threshold = 0.85
# 起始球去重：与已选起始球余弦 >= 此值则跳过（0 = 关闭）
# 实测副本 0.969、口语/书面同义 0.976，但语义反转（喜欢/不喜欢）也有 0.828，
# 因此只能卡在 0.95 附近：只去肉眼可辨的同一句，同义改写交给离线合并
start_dedupe_threshold = 0.95
total_water_budget = 1.0
supplement_ratio = 0.2
base_loss = 0.2
loss_pipe_factor = 0.8
loss_max = 0.8
min_water_threshold = 0.01
strengthen_eligibility_threshold = 0.6
min_strengthen_value = 0.05
base_reinforcement = 0.1
supplement_reinforce_ratio = 0.5
min_reinforcement_threshold = 0.001
pipe_strengthen_factor = 0.05
min_pipe_reinforce_threshold = 0.01
injection_mode = "Token"
supplement_threshold = 0.45
split_mode = "Weighted"
max_supplement_ratio = 2.0
discount_per_hop = 0.1
min_discount = 0.5
chars_per_token = 4
pipe_thickness_ceiling = 1.0
# embedding_dim 省略时以模型实际输出为准
embedding_model_path = "models/infinilabs/pizza-embed-zh-small-v1/onnx/model.onnx"
embedding_tokenizer_path = "models/infinilabs/pizza-embed-zh-small-v1/tokenizer.json"
token_budget = 2048
max_balls_per_inject = 10
render_clear_threshold = 0.7
impression_threshold = 0.3
"#;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn parse_valid_toml() {
        let config: Config = toml::from_str(VALID_TOML).unwrap();
        assert_eq!(config.max_flow_depth, 5);
        assert_eq!(config.token_budget, 2048);
    }

    #[test]
    fn config_manager_from_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), VALID_TOML).unwrap();
        let mgr = ConfigManager::from_file(tmp.path()).unwrap();
        let snap = mgr.snapshot();
        assert_eq!(snap.max_flow_depth, 5);
    }

    #[test]
    fn config_manager_default_fallback() {
        let mgr = ConfigManager::from_file_or_default(Path::new("/nonexistent/config.toml"))
            .unwrap();
        let snap = mgr.snapshot();
        assert_eq!(snap.max_flow_depth, Config::default().max_flow_depth);
    }

    #[test]
    fn hot_reload_updates_snapshot() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), VALID_TOML).unwrap();
        let mgr = ConfigManager::from_file(tmp.path()).unwrap();

        let snap1 = mgr.snapshot();
        assert_eq!(snap1.max_flow_depth, 5);

        // 修改文件
        let modified = VALID_TOML.replace("max_flow_depth = 5", "max_flow_depth = 3");
        std::fs::write(tmp.path(), modified).unwrap();
        mgr.reload(tmp.path()).unwrap();

        let snap2 = mgr.snapshot();
        assert_eq!(snap2.max_flow_depth, 3);
    }

    #[test]
    fn hot_reload_rejects_invalid() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), VALID_TOML).unwrap();
        let mgr = ConfigManager::from_file(tmp.path()).unwrap();

        // 写入非法配置
        let invalid = VALID_TOML.replace("max_flow_depth = 5", "max_flow_depth = 0");
        std::fs::write(tmp.path(), invalid).unwrap();
        let result = mgr.reload(tmp.path());
        assert!(result.is_err());

        // 快照不变
        let snap = mgr.snapshot();
        assert_eq!(snap.max_flow_depth, 5);
    }

    #[test]
    fn validate_rejects_zero_max_flow_depth() {
        let config = Config {
            max_flow_depth: 0,
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_invalid_threshold() {
        let config = Config {
            start_ball_threshold: 1.5,
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_impression_gt_clear() {
        let config = Config {
            impression_threshold: 0.9,
            render_clear_threshold: 0.5,
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_invalid_loss_config() {
        let config = Config {
            base_loss: 0.9,
            loss_pipe_factor: 1.0,
            ..Config::default()
        };
        // base_loss * (1 + loss_pipe_factor) = 0.9 * 2.0 = 1.8 >= 1.0
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_zero_injection_budget() {
        let config = Config {
            token_budget: 0,
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn config_manager_update() {
        let mgr = ConfigManager::with_default();
        let new_config = Config {
            max_flow_depth: 7,
            ..Config::default()
        };
        mgr.update(new_config).unwrap();
        assert_eq!(mgr.snapshot().max_flow_depth, 7);
    }
}
