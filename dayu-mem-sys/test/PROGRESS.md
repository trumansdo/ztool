# 测试进度与剩余工作

> 更新时间：2026-11-10 00:30
> 当前状态：HTTP 嵌入后端 + 真实语料 + 五十至十万球规模冒烟全部跑通

---

## 一、总体进度

| 阶段 | 状态 | 产物 |
|:--|:--:|:--|
| 单元测试（workspace，默认 feature） | ✅ 142 通过 | `cargo test --workspace` |
| clippy | ✅ 0 警告 | `cargo clippy --workspace --all-targets` |
| 移除 mock 后端（改用显式注入） | ✅ | `DayuCore::new(config, backend)` |
| HTTP 嵌入后端（OpenAI 兼容） | ✅ | `crates/dayu-embedding/src/http.rs` |
| 配置驱动后端选择 | ✅ | `embedding_backend = onnx / http / test` |
| 连通性验证 | ✅ | `test/probe.cjs` |
| 真实语料（15 领域 × 25 条） | ✅ | `test/corpus.cjs` |
| 规模冒烟 50 / 500 / 5 000 / 50 000 / 100 000 | ✅ | `docs/10`、`test/scale-http-result.json` |
| 参数扫描（budget / start_top_n） | ✅ | `docs/10` §4 |
| ANN 索引（检索延迟） | ❌ 未做 | 见 §五 |

---

## 二、已完成：规模冒烟（HTTP 嵌入，真实语料）

参数：`budget = 10`、`start_top_n = 5`、`significance = 1.0`、`Qwen/Qwen3-Embedding-4B`（2560 维）

| 球数 | 管道 | 点亮 | 被挡 | 强化 | 注入 | token | 延迟 | 命中率 | DB |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 50 | 102 | 5.93 | 1.9 | 0.6 | 5.9 | 126 | 198 ms | 0.554 | 0.6 MB |
| 500 | 1 013 | 12.1 | 19.7 | 2.2 | 12.1 | 257 | 255 ms | 0.961 | 6.6 MB |
| 5 000 | 10 125 | 12.7 | 30.2 | 4.9 | 12.7 | 305 | 492 ms | 0.987 | 65.9 MB |
| 50 000 | 101 250 | 13.77 | 31.9 | 5.2 | 13.77 | 334 | 3 430 ms | 0.988 | 656 MB |
| 100 000 | 202 500 | 13.8 | 32.0 | 5.2 | 13.8 | 334 | 6 502 ms | 0.988 | 1 313 MB |

**结论**：5000 球后行为完全饱和（点亮/注入/token/命中率不变），唯一随规模恶化的是检索延迟（线性）。

---

## 三、已完成：参数扫描（5000 球）

### budget

| budget | 点亮 | 被挡 | token | 命中率 |
|--:|--:|--:|--:|--:|
| 2.5 | **5.00** | 20.2 | 121 | 0.967 |
| 5 | 8.83 | 26.8 | 212 | 0.981 |
| **10** | **12.7** | **30.2** | **305** | **0.987** |
| 20 | 29.63 | 44.8 | 707 | 0.972 |
| 40 | 39.6 | 46.6 | 942 | 0.971 |
| 80 | 46.9 | 35.5 | 1 113 | 0.943 |

`budget = 2.5` 时点亮数 == 起始球数，**水一条管道都没通过**。扩散阈值约 5，最佳 10。

### start_top_n（budget=10）

| top_n | 点亮 | 被挡 | 强化 | token | 命中率 |
|--:|--:|--:|--:|--:|--:|
| 3 | 15.43 | 26.7 | 3.0 | 365 | 0.976 |
| **5** | **12.7** | **30.2** | **4.9** | **305** | **0.987** |
| 10 | 17.73 | 53.4 | 9.4 | 433 | 0.977 |
| 20 | 17.77 | 68.5 | 13.5 | 435 | 0.927 |

---

## 四、本轮代码改动

| 文件 | 改动 |
|:--|:--|
| `crates/dayu-domain/src/types/config.rs` | 新增 `embedding_backend` / `embedding_api_base` / `embedding_api_key` / `embedding_api_model` |
| `crates/dayu-core/src/lib.rs` | 新增 `load_embedding_backend(config)` 工厂；`embed_batch`；`create_balls_batch`；`link_balls_batch`；`check_embedding_dim` |
| `crates/dayu-embedding/src/http.rs` | 新增 `probe_dimension`（启动时自动探测维度） |
| `crates/dayu-napi/src/lib.rs` | init 改用工厂 + 临时 runtime；新增 `embedBatch` / `createBallsBatch` / `linkBallsBatch` |
| `crates/dayu-napi/Cargo.toml` | 新增 `http` feature；`test-support` 透传 `dayu-core/test-support` |
| `crates/dayu-cli/*` | 改用工厂；新增 `http` / `test-support` feature |
| `crates/dayu-core/Cargo.toml` | dev-dependency 加 `dayu-embedding/test-support`；新增 `test-support` feature |
| `crates/dayu-core/tests/{usecases,http_e2e}.rs` | 加 `#![cfg(feature = ...)]`，未启用对应 feature 时整文件为空 |
| `crates/dayu-core/src/lib.rs` tests | `archive_test_core` 改用确定性后端，不再依赖 ONNX |
| 删除 `crates/dayu-storage/src/memory.rs` | 死代码（未接入模块树），且「留内存的没意义」 |

---

## 五、剩余工作

### P0：默认值

| # | 任务 | 依据 |
|:--:|:--|:--|
| 1 | `total_water_budget` 1.0 → **10.0** | `docs/10` §4.1，1.0/2.5 下扩散失效 |
| 2 | `start_top_n` 3 → **5** | `docs/10` §4.2，5 命中率最高 |
| 3 | 同步 `docs/03` §17 配置总表与默认 toml | 文档一致性 |

### P1：检索延迟

| # | 任务 | 说明 |
|:--:|:--|:--|
| 4 | 评估 TriviumDB 的 ANN 能力 | 当前 `search` 是全量扫描，100k 球 6.5s |
| 5 | 接入 HNSW / IVF 或分层检索 | 目标：100k 球 < 200 ms |
| 6 | 考虑降维（2560 → 1024） | 存储从 1.3 GB 降到 0.5 GB |

### P2：补充验证

| # | 任务 |
|:--:|:--|
| 7 | 5 万 / 10 万球下 `significance` 的影响 |
| 8 | 多源查询（`recallMulti`）在大规模下的表现 |
| 9 | 归档扫描在 10 万球下的耗时 |
| 10 | 管道被挡衰减（`blocked` 随时间的软化）在规模下的效果 |

### P3：工程

| # | 任务 |
|:--:|:--|
| 11 | 批量接口的性能剖析（100k 建球 70 s、连边 150 s） |
| 12 | 嵌入缓存持久化到正式模块（当前在测试脚本里） |

---

## 六、复现

```bash
cargo build -p dayu-napi --release --no-default-features --features http
cp target/release/dayu_napi.dll test/dayu_test.node
cp test/config-http.example.toml test/config-http.toml   # 填入 api key
node test/probe.cjs
node test/scale-http.cjs --ladder --budget 10 --top 5
```

`test/` 下只有脚本、语料、示例配置入库；db / 日志 / 真实配置（含密钥）不入库。
