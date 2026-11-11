//! `dayu-serve`：把大禹记忆引擎包成 HTTP 服务，供外部评测 harness 调用
//!
//! 存在的理由：napi 绑定与 Node 运行时强耦合（需 `cargo build` 产出 .node、
//! 再 `require`），评测框架跨语言/跨机器时不方便。HTTP 是更窄的契约面。
//!
//! 启动成功后向 stdout 打印一行 `LISTENING <port>` 并立即 flush，
//! harness 靠这一行拿到端口（`--listen 127.0.0.1:0` 由内核分配，避免端口冲突）。

mod api;
mod state;

use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use dayu_config::ConfigManager;
use tracing_subscriber::EnvFilter;

use crate::state::Registry;

#[derive(Parser, Debug)]
#[command(name = "dayu-serve", about = "大禹记忆网络 HTTP 服务")]
struct Args {
    /// 配置文件路径（toml）
    #[arg(long, default_value = "config/default.toml")]
    config: PathBuf,

    /// 每个 namespace 的数据库文件存放目录
    #[arg(long, default_value = "out/serve-store")]
    store_dir: PathBuf,

    /// 监听地址，端口 0 表示由内核分配
    #[arg(long, default_value = "127.0.0.1:0")]
    listen: String,

    /// 同会话相邻回合的管道粗细；`off` 关闭连边（纯余弦 Top-N 对照组）
    #[arg(long, default_value = "0.5")]
    link_weight: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    let link_weight = parse_link_weight(&args.link_weight)?;

    let manager = ConfigManager::from_file(&args.config)
        .with_context(|| format!("加载配置失败：{}", args.config.display()))?;
    let config = manager.snapshot();

    let backend = dayu_core::load_embedding_backend(&config)
        .await
        .context("加载嵌入后端失败")?;
    tracing::info!(
        "嵌入后端就绪：dim={} model={}",
        backend.dimension(),
        config.embedding_api_model
    );

    let reg = Arc::new(Registry::new(
        config,
        backend,
        args.store_dir.clone(),
        link_weight,
    )?);
    tracing::info!(
        "服务就绪：dim={} store_dir={} link_weight={:?}",
        reg.dim(),
        args.store_dir.display(),
        link_weight
    );

    let addr: SocketAddr = args
        .listen
        .parse()
        .with_context(|| format!("非法监听地址：{}", args.listen))?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("绑定 {addr} 失败"))?;
    let port = listener.local_addr()?.port();

    // harness 靠这一行同步就绪状态：必须在 serve 之前 flush
    {
        let mut out = std::io::stdout().lock();
        writeln!(out, "LISTENING {port}")?;
        out.flush()?;
    }

    axum::serve(listener, api::router(reg))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP 服务异常退出")?;

    Ok(())
}

fn parse_link_weight(raw: &str) -> anyhow::Result<Option<f64>> {
    let s = raw.trim();
    if s.eq_ignore_ascii_case("off") || s.eq_ignore_ascii_case("none") || s == "0" {
        return Ok(None);
    }
    let w: f64 = s
        .parse()
        .with_context(|| format!("link_weight 非法：{raw}（取值 0–1 或 off）"))?;
    if !(0.0..=1.0).contains(&w) {
        anyhow::bail!("link_weight 必须在 0–1 之间，实际 {w}");
    }
    Ok(Some(w))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("收到中断信号，正在退出");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_weight_accepts_off_and_range() {
        assert_eq!(parse_link_weight("off").unwrap(), None);
        assert_eq!(parse_link_weight("0").unwrap(), None);
        assert_eq!(parse_link_weight("0.5").unwrap(), Some(0.5));
        assert!(parse_link_weight("1.5").is_err());
        assert!(parse_link_weight("abc").is_err());
    }
}
