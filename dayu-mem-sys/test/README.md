# test/ —— 测试产物与脚本

## 目录约定

```
test/
├── README.md              本文件
├── corpus.cjs             语料本体（15 领域 × 25 条 + 30 条查询），纯数据
│
├── config/                配置：base 写全量，各后端只写差异
│   ├── base.toml          唯一完整配置
│   ├── onnx.toml          本地 ONNX（pizza-embed-zh-small-v1，512 维）
│   ├── http.toml          远端 HTTP（Qwen3-Embedding-4B，2560 维）含密钥，不入库
│   ├── http.example.toml  同上模板，密钥为占位符
│   └── lms.toml           本机 LM Studio（Qwen3-Embedding-0.6B，1024 维）
│
├── lib/                   公共逻辑
│   ├── paths.cjs          路径约定（data / out / cache / 运行目录）
│   ├── config.cjs         TOML 分层合并
│   ├── corpus.cjs         语料展开 + 建图规则
│   ├── stats.cjs          cos / 分位数 / 符号检验
│   └── harness.cjs        napi 加载、初始化、向量缓存、批量建图、meta 落盘
│
├── bin/                   冒烟测试（有断言，可进 CI）
│   ├── smoke.cjs          长期库全链路冒烟（data/main.tdb）
│   ├── scale.cjs          规模阶梯冒烟
│   └── probe.cjs          嵌入后端连通性
│
├── lab/                   参数标定与对照实验（产出数字，无断言，不进 CI）
│   ├── relevance.cjs      语义级相关率（建库 + 查询）
│   ├── relevance-rate.cjs 相关率离线分析（读运行目录）
│   ├── relevance-compare.cjs  跨模型对照（离线）
│   ├── relevance-scale.cjs    跨规模对照（离线）
│   └── dedupe-threshold.cjs   起始球去重阈值校准
│
├── data/                  【长期资产】跨运行存活，不自动清理
│   ├── main.tdb           长期全链路库
│   ├── main.history.jsonl 每轮冒烟的统计快照（一行一轮）
│   ├── cache/<profile>.json  向量缓存（按模型隔离）
│   ├── snapshots/         长期库备份
│   └── logs/              长期库的日志与指标
│
├── memory-eval/           【评测资产】公开基准（LoCoMo / LongMemEval）
│   ├── README.md          目录约定 + 运行器用法
│   ├── run-eval.cjs       一次性运行器（起服务 + 跑 mceval + 归档产物）
│   └── <时间戳>-<adapter>-<dataset>[-<tag>]/
│       ├── config.toml    本次运行的完整配置
│       ├── serve.log      dayu-serve 日志
│       ├── store/         各 namespace 的库
│       ├── <adapter>_*.json  mceval 逐题结果
│       └── meta.json      参数 / git / 耗时 / 产物清单
│
└── out/                   【一次性产物】整目录可删
    ├── <时间戳>-<脚本>-<profile>-<参数>/
    │   ├── meta.json      配置快照 + git commit + 参数 + 耗时
    │   ├── result.json    结果
    │   ├── db/            该次运行的库
    │   └── logs/          该次的日志与指标
    └── legacy-20261110/   重构前的历史产物（可直接删）
```

## 测试分层

从「离代码多近」到「离真实使用多近」，五层各管一件事，**不重复断言同一个东西**。

| 层 | 位置 | 用什么跑 | 关注点 | 数量 |
|:--|:--|:--|:--|--:|
| 1 单元 | `crates/*/src/**/*.rs` 的 `mod tests` | `cargo test` | 单个函数/分支，可访问私有 | ~184 |
| 2 单模块契约 | `crates/*/tests/contract.rs` | `cargo test` | 只能用 `pub` API，断对外承诺 | 18 |
| 3 聚合用例 | `crates/dayu-core/tests/e2e/` | `cargo test --features onnx` | 跨 crate 串起来的一条完整链路 | 6 |
| 4 冒烟 | `test/bin/*.cjs` | `node` | 真实 napi + 真实嵌入，有断言可 CI | 3 |
| 5 基准 | `test/memory-eval/` | `node test/memory-eval/run-eval.cjs` | 公开数据集上的检索质量 | 1 |

### 为什么 1 和 2 要分开

`src/` 里的测试能访问私有函数，改实现时它跟着一起改，因此**它证明不了对外承诺**。
`tests/` 是独立 crate，编译器强制只看 `pub`——写在那里的断言，调用方真的可以依赖。

这条边界会抓出真问题。比如 `dayu-storage` 的契约测试发现
`load_subgraph` 会把跨出深度边界的边一起返回（子图不闭合）：两个现有消费者都做了
防御跳过，所以之前没暴露，但任何新消费者只要假设「子图闭合」就会读到不存在的球。

### 为什么 4 和 5 要分开

`bin/` 有断言（失败退出码非 0），`lab/` 没有（只有数字）。见下面第 4 条规则。
`memory-eval/` 又是一类：跑一次几十分钟，结果要留作横向对比的基准。

## 五条规则

**1. `data/` / `memory-eval/` / `out/` 三者是硬边界**

| | `data/` | `memory-eval/` | `out/` |
|:--|:--|:--|:--|
| 跨运行 | 有价值 | 有价值（一次跑几十分钟） | 无价值 |
| 删除后果 | 丢长期数据 | 丢评测对比基准 | 零损失 |
| 清理 | 手动 + `snapshots/` 保护 | 手动按目录删 | `rm -rf test/out` |
| 产生者 | `bin/smoke.cjs` | `memory-eval/run-eval.cjs` | `bin/*.cjs`、`cargo test` |

评测产物单独开目录，是为了让 `rm -rf test/out`（日常测试清理）不会顺手删掉
几十分钟才跑出来的一次基准结果；反过来，评测的几百 MB 库也不会拖慢日常测试。

**2. 一次运行 = 一个自描述目录**

目录名即实验标签，`meta.json` 记下配置、git commit、参数与耗时。想知道某个数字是怎么跑出来的，看那一个目录就够了。

**3. 长期库与规模测试分开**

| | `data/main.tdb` | `out/*/db` |
|:--|:--|:--|
| 用途 | 模拟真实用户长期使用 | 单次实验的干净环境 |
| 写入 | 每轮追加一点 | 每次重建 |
| 断言 | 统计不变量（球数单调、无孤儿管道） | 参数扫描的精确数值 |
| 规模 | 几百~几千球 | 5 万~10 万球 |

共用一个库会让两件事都坏掉：长期库反复写入后球数增长、管道稠密，规模测试数值不可复现；反过来十万球库塞进长期库，冒烟慢到没法每轮跑。

**4. `bin/` 与 `lab/` 按「有没有断言」分**

| | `bin/` | `lab/` |
|:--|:--|:--|
| 跑完有 pass/fail | 有 | 没有，只有数字 |
| 能进 CI | 能（失败即红） | 不能（跑几十分钟、永远绿） |
| 数字含义 | 断言阈值 | 记录，用于横向对比 |
| 什么时候删 | 跟着代码改 | 几个月后还要按同一组参数重跑 |
| 产物 | `out/`，可删 | 对照结论，要留 |

`relevance*.cjs` 的 `process.exit(1)` 只出现在 `.catch()` 里——崩了才退出。
把它们和 `smoke.cjs` 混在一个目录，会让「跑测试」和「做实验」的维护方式互相污染。

**5. 配置分层 + 逻辑抽公共**

改语料只改 `corpus.cjs`，改展开/建图只改 `lib/corpus.cjs`，改缓存格式只改 `lib/harness.cjs`。`bin/` 里的脚本只写自己关心的事。

## 构建

```bash
# 远端 HTTP 嵌入
cargo build -p dayu-napi --release --no-default-features --features http

# 本地 ONNX
cargo build -p dayu-napi --release --no-default-features --features onnx

cp target/release/dayu_napi.dll test/dayu_test.node
```

本地 ONNX 还需要 ONNX Runtime 动态库；未设置 `ORT_DYLIB_PATH` 时 `lib/harness.cjs` 自动指向 `models/onnxruntime/lib/onnxruntime.dll`。

## 常用命令

```bash
# 连通性
node test/bin/probe.cjs --profile http

# 长期库全链路冒烟（追加一轮）
node test/bin/smoke.cjs --profile onnx
node test/bin/smoke.cjs --profile onnx --reset   # 重建（先备份到 data/snapshots/）

# 规模阶梯
node test/bin/scale.cjs --profile http --ladder
node test/bin/scale.cjs --profile http --balls 5000 --budgets 1.0,2.5,10

# 语义相关率
node test/lab/relevance.cjs --profile http --balls 5000 --budget 10 --top 5

# 离线分析（读最近一次运行）
node test/lab/relevance-rate.cjs
node test/lab/relevance-scale.cjs          # 扫描 out/ 下所有 relevance 运行
node test/lab/relevance-compare.cjs        # 自动取各 profile 最近一次
```

## 清理

```bash
rm -rf test/out                    # 所有一次性产物
rm -rf test/out/legacy-20261110    # 重构前的历史归档
rm -rf test/memory-eval/2026*      # 某次或全部评测运行
```

`test/data/` 不要随手删——向量缓存重算要花 API 钱或本地算力，`main.tdb` 与 `main.history.jsonl` 是长期回归的基线。

## 什么入库、什么不入库

| 类型 | 入库 | 例 |
|:--|:--|:--|
| 脚本 | ✅ | `bin/*.cjs`、`lib/*.cjs`、`memory-eval/run-eval.cjs` |
| 配置（无密钥） | ✅ | `config/base.toml`、`lms.toml`、`onnx.toml`、`http.example.toml` |
| 语料 | ✅ | `corpus.cjs` |
| 文档 | ✅ | `README.md`、`PROGRESS.md`、`memory-eval/README.md` |
| 评测报告 | ✅ | `memory-eval/*/meta.json`、`memory-eval/*/*.json`、`data/main.history.jsonl` |
| 含密钥的真实配置 | ❌ | `config/http.toml` |
| 数据库二进制 | ❌ | `*.tdb`、`*.tdb.*`、`data/cache/*.json` |
| 日志与指标 | ❌ | `logs/`、`*.log` |
| 构建产物 | ❌ | `*.node`、`*.dll`、`*.pdb` |
| 一次性运行目录 | ❌ | `out/`、`memory-eval/*/store/` |

规则在仓库根的 `.gitignore`。两个容易踩的坑：

- `test/data/` 不能整个忽略，否则 `!test/data/main.history.jsonl` 这类例外规则失效（git 不会进入被忽略的目录）——所以写成 `test/data/*` + 白名单。
- `logs/` 是目录模式，挡不住 `serve.log` 这种同级单文件，得单独列。

## 已知限制

- 球 id 从 1 开始顺序分配这一假设成立的前提是「每次运行都是全新的库」，离线分析脚本（`relevance-rate.cjs`、`relevance-compare.cjs`）依赖它。
- 语料是 375 条基础句铺到 N 个球，N 越大重复越多，因此 `precision@K` / `recall@K` 只能同规模比，不能跨规模比（跨规模用同领域率与余弦裕度）。
