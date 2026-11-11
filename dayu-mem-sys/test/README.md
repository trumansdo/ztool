# test/ —— 测试产物与脚本

## 目录约定

```
test/
├── README.md              本文件
├── corpus.cjs             人工语料本体（15 领域 × 25 条 + 30 条查询），纯数据，入库
├── gen-dataset.cjs        数据集生成器（人工语料 → datasets/*.json）
├── migrate-cache.cjs      一次性迁移旧 JSON 缓存 → JSONL
│
├── datasets/              【派生产物】不入库，由 gen-dataset.cjs 生成
│   └── main.json          落地数据集（记忆 + 查询），改语料后需重跑生成器
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
│   ├── corpus.cjs         语料展开 + 建图规则（只被 gen-dataset 用）
│   ├── dataset.cjs        数据集加载 + sourceHash 校验 + 取前缀
│   ├── cache.cjs          向量缓存（JSONL，append-only，身份校验）
│   ├── stats.cjs          cos / 分位数 / 符号检验
│   └── harness.cjs        napi 加载、初始化、向量缓存入口、批量建图、meta 落盘
│
├── bin/                   冒烟测试（有断言，可进 CI）
│   ├── smoke.cjs          长期库全链路冒烟（data/main.tdb）
│   └── probe.cjs          部署连通性（napi 加载 / 配置解析 / 库 open / 一次完整链路）
│
├── lab/                   参数标定与对照实验（产出数字，无断言，不进 CI）
│   ├── scale.cjs          规模 × 参数矩阵测量
│   ├── relevance.cjs      语义级相关率（建库 + 查询）
│   ├── relevance-rate.cjs 相关率离线分析（读运行目录）
│   ├── relevance-compare.cjs  跨模型对照（离线）
│   ├── relevance-scale.cjs    跨规模对照（离线）
│   └── dedupe-threshold.cjs   起始球去重阈值校准
│
├── data/                  【长期资产】跨运行存活，不自动清理
│   ├── main.tdb           长期全链路库
│   ├── main.history.jsonl 每轮冒烟的统计快照（一行一轮）
│   ├── cache/<key>.jsonl  向量缓存（键 = 后端-模型-维度-数据集-来源哈希）
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
合计 214 个可枚举用例（`cargo test --all-features -- --list`），其中 19 个带 `#[ignore]`（需真模型或远程服务），默认 `cargo test --workspace` 跑 195 个全绿。

| 层 | 位置 | 用什么跑 | 关注点 | 数量 |
|:--|:--|:--|:--|--:|
| 1 单元 | `crates/*/src/**/*.rs` 的 `mod tests` | `cargo test` | 单个函数/分支，可访问私有 | 185 |
| 2 单模块契约 | `crates/*/tests/contract.rs` | `cargo test` | 只能用 `pub` API，断对外承诺 | 18 |
| 3 聚合用例 | `crates/dayu-core/tests/{e2e/,http_e2e.rs}` | `cargo test --features onnx` / `--features http` | 跨 crate 串起来的一条完整链路 | 11（6 用例 + 5 HTTP） |
| 4 冒烟 | `test/bin/*.cjs` | `node` | 真实 napi + 真实嵌入，有断言可 CI | 2 脚本 / 11 断言 |
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

## 八条规则

**1. `data/` / `memory-eval/` / `out/` 三者是硬边界**

| | `data/` | `memory-eval/` | `out/` |
|:--|:--|:--|:--|
| 跨运行 | 有价值 | 有价值（一次跑几十分钟） | 无价值 |
| 删除后果 | 丢长期数据 | 丢评测对比基准 | 零损失 |
| 清理 | 手动 + `snapshots/` 保护 | 手动按目录删 | `rm -rf test/out` |
| 产生者 | `bin/smoke.cjs` | `memory-eval/run-eval.cjs` | `bin/*.cjs`、`lab/*.cjs`、`cargo test` |

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

`relevance*.cjs` 与 `scale.cjs` 的 `process.exit(1)` 只出现在 `.catch()` 里——崩了才退出，
跑完无论数字多离谱都是 0。把它们和 `smoke.cjs` / `probe.cjs` 混在一个目录，
会让「跑测试」和「做实验」的维护方式互相污染。

`probe.cjs` 曾经也在这个边界上（只打印、无断言）。它被留下并补上 8 条断言，理由是：
它是唯一验证「napi 能加载 / 配置能解析 / 库能 open / 一次 recall 能跑通」的脚本，
这些是部署问题而不是性能问题，必须有人守。`scale.cjs` 相反——产出是 30 行数字，
没有 pass/fail 可言，所以搬去了 `lab/`。

**5. 配置分层 + 逻辑抽公共**

改语料只改 `corpus.cjs`，改展开/建图只改 `lib/corpus.cjs`，改数据集格式只改 `gen-dataset.cjs`，改缓存格式只改 `lib/cache.cjs`。`bin/`、`lab/` 里的脚本只写自己关心的事——它们不再 `require('corpus.cjs')`，只通过 `lib/dataset.cjs` 取数据。

**6. 数据从落地文件读，不现算**

`test/datasets/main.json` 是 `gen-dataset.cjs` 从人工语料生成的落地数据集（10 万条记忆 + 30 条查询，10.9 MB）。脚本读它、取前缀，而不是每次自己 `expandCorpus`：

```bash
node test/gen-dataset.cjs main 100000    # 生成（改语料后必须重跑）
```

数据集头部记着 `sourceHash`——`corpus.cjs` + `lib/corpus.cjs` + `gen-dataset.cjs` 三个文件的 sha1。加载时比对，不一致直接报错：

```
数据集已过期：
  文件  sourceHash=374a412c（id=main，记忆 100000 条）
  当前  sourceHash=8a1b3f02（corpus.cjs / lib/corpus.cjs / gen-dataset.cjs 有改动）
```

不做「文件不存在就自动生成」：静默生成会让「改完语料忘了重跑」变成「测试用着旧数据但你以为是新的」。

生成器不含任何随机数，同参数重跑逐字节相同（`sha256sum` 可验证）。**要做另一个数据集，就改人工语料或另写一份 `corpus.cjs`，然后重跑生成器**——不要试图给生成器加随机种子，那会让「同一个数据集」失去意义。

**7. 向量缓存的身份必须自证**

缓存文件名 = `后端-模型-维度-数据集-来源哈希.jsonl`，且文件第 1 行写同样的身份：

```
http-qwen-qwen3-embedding-4b-2560-main-374a412c.jsonl
onnx-pizza-embed-zh-small-v1-512-main-374a412c.jsonl
```

加载时逐字段比对，不一致就抛错而不是静默重算（重算要花钱或算力，静默换掉测量条件更糟）。`preprocess` / `normalize` 也记在头里：以后给 query 加 instruction 前缀或关掉 L2 归一化，旧缓存会自动失效。

为什么不是一个大 JSON：旧实现每批（`chunk=64`）都把整个对象 `JSON.stringify` 重写一遍，7554 条要写 ~28 GB，10 万条要写 ~3.9 TB。现在是 append-only，只写新增的那批；中途失败也只丢最后一批。

旧格式迁移：

```bash
node test/migrate-cache.cjs           # 预览
node test/migrate-cache.cjs --write   # 执行（旧文件改名为 *.migrated-<时间戳>）
```

**8. 磁盘缓存只在测试工具里，核心路径不许有**

`crates/**` 每次都必须真的调嵌入模型。`dayu-serve` 里有一个 `EMBED_CACHE_MAX` 的进程内 Map，那是内存记忆化（进程退出即消失、不读写文件），不是磁盘缓存。

CI 有一条 `No Disk Cache In Core` 守着：`crates/**/*.rs` 出现 `CACHE_DIR` / `cache_path` / `cachePath` / `test/data/cache` 就红。

## 构建

```bash
# 远端 HTTP 嵌入
cargo build -p dayu-napi --release --no-default-features --features http

# 本地 ONNX
cargo build -p dayu-napi --release --no-default-features --features onnx

cp target/release/dayu_napi.dll test/dayu_test.node
```

本地 ONNX 还需要 ONNX Runtime 动态库；未设置 `ORT_DYLIB_PATH` 时 `lib/harness.cjs` 自动指向 `models/onnxruntime/lib/onnxruntime.dll`。

Cargo 侧同理：Rust 测试里由 `dayu_testkit::ensure_ort_dylib()` 在首次 `ort` 调用前补上同一个路径（条件是用户没自己设 `ORT_DYLIB_PATH`、且仓库里确实有这个 dll）。**必须在 `OnceLock`/`LazyLock` 初始化的最前面调用**——`ort` 只在建 Session 时读一次环境变量。

不加这一步会踩一个很难看的坑：`ort` 找不到 `ORT_DYLIB_PATH` 就退回系统 `PATH`，而系统里常有一个更旧的 `onnxruntime.dll`（本机是 1.17.1），报错是

```
ort 2.0.0-rc.15 is not compatible with the ONNX Runtime binary found at `onnxruntime.dll`;
expected version >= '1.31.x', but got '1.17.1'
```

——报错指向裸文件名 `onnxruntime.dll`，看不出是找错了文件，会误以为是版本装错。

模型目录 `models/infinilabs` 在 Windows 上必须是**目录联接（junction）**而不是 cygwin 符号链接：

```powershell
New-Item -ItemType Junction -Path 'models\infinilabs' -Target 'E:\dev_data\huggingface_models\infinilabs'
```

cygwin 的 `ln -s` 生成的是带 `/cygdrive/...` 目标的链接，Rust（原生 Windows 二进制）`Path::exists()` 直接返回 false。这个坑曾经让所有 ONNX 用例「静默跳过」而看着全绿，所以现在改成**硬失败**：

`dayu_testkit::require_onnx_model_paths()` 找不到模型就 panic，并打印处理指引（junction 命令 + 环境变量 + 「不想跑就别开 `--features onnx`」）。缺模型 = 红，不再有「跳过」这条路径。

历史判据留作参考：修之前 `cargo test -p dayu-core --features onnx --test e2e` 显示 `finished in 0.00s` 就是模型没被找到（真跑要 0.5s 以上）。

## 常用命令

```bash
# 数据集（改语料后必须重跑；产物不入库）
node test/gen-dataset.cjs main 100000

# 向量缓存迁移（旧 JSON → JSONL，一次性）
node test/migrate-cache.cjs
node test/migrate-cache.cjs --write

# 部署连通性（8 条断言，失败退出码非 0）
node test/bin/probe.cjs --profile http
node test/bin/probe.cjs --profile onnx

# 长期库全链路冒烟（追加一轮，3 条不变量）
node test/bin/smoke.cjs --profile onnx
node test/bin/smoke.cjs --profile onnx --reset   # 重建（先备份到 data/snapshots/）

# 规模 × 参数矩阵（测量，无断言）
node test/lab/scale.cjs --profile http --ladder
node test/lab/scale.cjs --profile http --balls 5000 --budgets 1.0,2.5,10

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

`test/datasets/` 可以随手删（派生物），删了重跑 `node test/gen-dataset.cjs` 即可，向量缓存仍然命中（`sourceHash` 没变）。

## 什么入库、什么不入库

| 类型 | 入库 | 例 |
|:--|:--|:--|
| 脚本 | ✅ | `bin/*.cjs`、`lab/*.cjs`、`lib/*.cjs`、`memory-eval/run-eval.cjs` |
| 配置（无密钥） | ✅ | `config/base.toml`、`lms.toml`、`onnx.toml`、`http.example.toml` |
| 语料 | ✅ | `corpus.cjs`、`gen-dataset.cjs` |
| 数据集 | ❌ | `datasets/*.json`（派生物，由生成器重建） |
| 文档 | ✅ | `README.md`、`PROGRESS.md`、`memory-eval/README.md` |
| 评测报告 | ✅ | `memory-eval/*/meta.json`、`memory-eval/*/*.json`、`data/main.history.jsonl` |
| 含密钥的真实配置 | ❌ | `config/http.toml` |
| 数据库二进制与向量缓存 | ❌ | `*.tdb`、`*.tdb.*`、`data/cache/*.jsonl`、`*.migrated-*` |
| 日志与指标 | ❌ | `logs/`、`*.log` |
| 构建产物 | ❌ | `*.node`、`*.dll`、`*.pdb` |
| 一次性运行目录 | ❌ | `out/`、`memory-eval/*/store/` |

规则在仓库根的 `.gitignore`。两个容易踩的坑：

- `test/data/` 不能整个忽略，否则 `!test/data/main.history.jsonl` 这类例外规则失效（git 不会进入被忽略的目录）——所以写成 `test/data/*` + 白名单。
- `logs/` 是目录模式，挡不住 `serve.log` 这种同级单文件，得单独列。

## 已知限制

- 球 id 从 1 开始顺序分配这一假设成立的前提是「每次运行都是全新的库」，离线分析脚本（`relevance-rate.cjs`、`relevance-compare.cjs`）依赖它。
- 语料是 375 条基础句铺到 N 个球，N 越大重复越多，因此 `precision@K` / `recall@K` 只能同规模比，不能跨规模比（跨规模用同领域率与余弦裕度）。
