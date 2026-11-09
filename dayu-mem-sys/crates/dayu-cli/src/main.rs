//! dayu-cli: 离线巩固 CLI
//!
//! 子命令：consolidate / merge / split / archive / decay-pipes / diagnose / reembed。
//! 不要求实时，可 CLI 化隔离复杂度。
//!
//! 所有子命令都需要嵌入后端（归档扫描要算 `size_effective`，诊断要算相似度），
//! 而系统已无 mock 回退，因此 `onnx` 是默认 feature。

#[cfg(not(any(feature = "onnx", feature = "http", feature = "test-support")))]
compile_error!(
    "dayu-cli 至少需要一个嵌入后端 feature：onnx / http / test-support"
);

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// 大禹水利记忆网络 · 离线巩固工具
#[derive(Parser)]
#[command(name = "dayu")]
#[command(version = "0.1.0")]
#[command(about = "大禹水利记忆网络离线巩固与诊断工具")]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    /// 存储文件路径（.tdb）
    #[arg(short, long, default_value = "store.tdb")]
    store: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行摘要巩固：对活跃球生成摘要
    Summaries {
        /// 每次运行最大球数
        #[arg(long, default_value = "100")]
        max_balls: usize,
    },
    /// 运行抽象巩固：从球中提取抽象概念
    Abstractions {
        #[arg(long, default_value = "50")]
        max_balls: usize,
    },
    /// 合并相似球
    Merge {
        /// 相似度阈值（余弦）
        #[arg(long, default_value = "0.85")]
        threshold: f64,
        /// 最大合并数
        #[arg(long, default_value = "20")]
        max_merges: usize,
    },
    /// 拆分过大的球
    Split {
        /// 触发拆分的球大小阈值
        #[arg(long, default_value = "50.0")]
        size_threshold: f64,
    },
    /// 归档不活跃球
    ///
    /// 判定：`距 last_watered > inactive_days` 且 `size_effective < archive_size_threshold`。
    /// 另有容量兜底：活跃球超过 `max_balls` 时按有效大小升序强制归档。
    /// 不传参数则用配置文件里的 `[archive]` 段。
    ///
    /// 归档球仍可被检索到，命中即翻回 Active（无临时态）。
    Archive {
        /// 不活跃天数阈值（不传则用配置）
        #[arg(long)]
        inactive_days: Option<f64>,
        /// 活跃球数上限（不传则用配置）
        #[arg(long)]
        max_balls: Option<usize>,
    },
    /// 管道衰减：降低长期未使用管道的权重
    DecayPipes {
        /// 衰减因子（0-1）
        #[arg(long, default_value = "0.95")]
        factor: f64,
        /// 未使用天数阈值
        #[arg(long, default_value = "14")]
        unused_days: u32,
    },
    /// 运行诊断：输出结构化诊断信号
    Diagnose {
        /// 输出格式
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// 重建向量：用当前模型重算维度不匹配的球的 embedding
    ///
    /// # 背景
    ///
    /// 球一旦建立，其 `embedding` 就绑定在生成它的模型上：不同维度（乃至同维不同模型）
    /// 的向量空间不可比。检索层已对维度不匹配的球「告警 + 跳过」，本命令负责把它们
    /// 真正迁移到新模型的向量空间。
    ///
    /// # 实现步骤
    ///
    /// 1. 读 config，构建 `OrtEmbeddingBackend`（维度由模型输出自动检测，不依赖配置）
    /// 2. 打开 `TriviumStorage`，遍历 Active 球
    /// 3. 跳过 `ball.embedding.len() == backend.dimension()` 的球（已在新空间）
    /// 4. 对不匹配的球：取 `content_list` 原文 → 按列表顺序拼接 → `embed()` → 替换 `embedding`
    /// 5. 只改 `embedding` 字段，`size` / `pipes` / `content_list` 均不动
    /// 6. 单事务批量提交；`--dry-run` 只统计不写回
    ///
    /// # 待定
    ///
    /// - 拼接 vs 逐条 embed 加权平均：当前按「顺序拼接」实现，语义简单可预测
    /// - 空 `content_list` 的球无法重算，计入 `failed` 并保留旧向量
    /// - 同维换模型无法靠维度区分，需后续给 Ball 加 `embedding_model` 指纹
    ///
    /// # 输出
    ///
    /// `total / skipped / reembedded / failed`
    Reembed {
        /// 只统计不写回
        #[arg(long)]
        dry_run: bool,
        /// 每次处理最大球数（0 = 不限）
        #[arg(long, default_value = "0")]
        max_balls: usize,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "dayu=info".into()),
        )
        .init();

    let cli = Cli::parse();

    tracing::info!(config = %cli.config.display(), store = %cli.store.display(), "dayu-cli starting");

    match cli.command {
        Commands::Summaries { max_balls } => {
            println!("[consolidate summaries] max_balls={max_balls}");
            println!("状态：骨架就位，等待 TriviumDB 存储接入后实现");
        }
        Commands::Abstractions { max_balls } => {
            println!("[consolidate abstractions] max_balls={max_balls}");
            println!("状态：骨架就位，等待 TriviumDB 存储接入后实现");
        }
        Commands::Merge { threshold, max_merges } => {
            println!("[merge] threshold={threshold} max_merges={max_merges}");
            println!("状态：骨架就位，等待 TriviumDB 存储接入后实现");
        }
        Commands::Split { size_threshold } => {
            println!("[split] size_threshold={size_threshold}");
            println!("状态：骨架就位，等待 TriviumDB 存储接入后实现");
        }
        Commands::Archive { inactive_days, max_balls } => {
            let mut config = (*dayu_config::ConfigManager::from_file_or_default(&cli.config)?
                .snapshot())
                .clone();
            if let Some(d) = inactive_days {
                config.archive_inactive_days = d;
            }
            if let Some(n) = max_balls {
                config.max_active_balls = n;
            }
            let (days, cap) = (config.archive_inactive_days, config.max_active_balls);
            // 嵌入后端由 config.embedding_backend 决定（onnx / http / test）
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let backend = rt
                .block_on(dayu_core::load_embedding_backend(&config))
                .map_err(|e| anyhow::anyhow!("嵌入后端加载失败: {e}"))?;
            let core = dayu_core::DayuCore::new(std::sync::Arc::new(config), backend)?;
            let store = dayu_storage::TriviumStorage::new(
                cli.store.to_str().ok_or_else(|| anyhow::anyhow!("非法存储路径"))?,
            )?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let report = rt
                .block_on(core.archive_sweep(&store))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!(
                "[archive] archived={} (inactive_days={}, max_active_balls={})",
                report.archived, days, cap
            );
        }
        Commands::DecayPipes { factor, unused_days } => {
            println!("[decay-pipes] factor={factor} unused_days={unused_days}");
            println!("状态：骨架就位，等待 TriviumDB 存储接入后实现");
        }
        Commands::Diagnose { format } => {
            println!("[diagnose] format={format}");
            // 输出示例诊断信号
            let diagnostics = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "signals": [
                    {
                        "type": "loss_dominant",
                        "ratio": 0.0,
                        "threshold": 0.6,
                        "recommendation": "loss_dominant ratio below threshold, no action needed"
                    },
                    {
                        "type": "supplement_insufficient",
                        "ratio": 0.0,
                        "threshold": 0.5,
                        "recommendation": "supplement ratio below threshold, no action needed"
                    }
                ],
                "status": "skeleton_ready"
            });
            println!("{}", serde_json::to_string_pretty(&diagnostics).unwrap());
        }
        Commands::Reembed { dry_run, max_balls } => {
            // TODO(reembed)：方案见上方 Commands::Reembed 文档注释
            //   1. config → OrtEmbeddingBackend::load（维度自动检测）
            //   2. TriviumStorage::open(store) → 遍历 Active 球（max_balls 限流，0 = 不限）
            //   3. ball.embedding.len() == backend.dimension() → skipped
            //   4. 否则 content_list 顺序拼接文本 → embed → 替换 embedding
            //   5. dry_run 时只累加计数；否则单事务批量 upsert_ball 提交
            //   6. 输出 total / skipped / reembedded / failed
            println!("[reembed] dry_run={dry_run} max_balls={max_balls}");
            println!("状态：骨架就位，等待 TriviumDB 存储接入后实现");
        }
    };
    Ok(())
}
