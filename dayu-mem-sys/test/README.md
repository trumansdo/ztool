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
├── bin/                   可执行脚本
│   ├── smoke.cjs          长期库全链路冒烟（data/main.tdb）
│   ├── scale.cjs          规模阶梯 / 参数扫描
│   ├── relevance.cjs      语义级相关率（建库 + 查询）
│   ├── relevance-rate.cjs 相关率离线分析（读运行目录）
│   ├── relevance-compare.cjs  跨模型对照（离线）
│   ├── relevance-scale.cjs    跨规模对照（离线）
│   ├── dedupe-threshold.cjs   起始球去重阈值校准
│   └── probe.cjs          嵌入后端连通性
│
├── data/                  【长期资产】跨运行存活，不自动清理
│   ├── main.tdb           长期全链路库
│   ├── main.history.jsonl 每轮冒烟的统计快照（一行一轮）
│   ├── cache/<profile>.json  向量缓存（按模型隔离）
│   ├── snapshots/         长期库备份
│   └── logs/              长期库的日志与指标
│
└── out/                   【一次性产物】整目录可删
    ├── <时间戳>-<脚本>-<profile>-<参数>/
    │   ├── meta.json      配置快照 + git commit + 参数 + 耗时
    │   ├── result.json    结果
    │   ├── db/            该次运行的库
    │   └── logs/          该次的日志与指标
    └── legacy-20261110/   重构前的历史产物（可直接删）
```

## 四条规则

**1. `data/` 与 `out/` 是硬边界**

| | `data/` | `out/` |
|:--|:--|:--|
| 跨运行 | 有价值 | 无价值 |
| 删除后果 | 丢长期数据 | 零损失 |
| 清理 | 手动 + `snapshots/` 保护 | `rm -rf test/out` |

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

**4. 配置分层 + 逻辑抽公共**

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
node test/bin/relevance.cjs --profile http --balls 5000 --budget 10 --top 5

# 离线分析（读最近一次运行）
node test/bin/relevance-rate.cjs
node test/bin/relevance-scale.cjs          # 扫描 out/ 下所有 relevance 运行
node test/bin/relevance-compare.cjs        # 自动取各 profile 最近一次
```

## 清理

```bash
rm -rf test/out                    # 所有一次性产物
rm -rf test/out/legacy-20261110    # 重构前的历史归档
```

`test/data/` 不要随手删——向量缓存重算要花 API 钱或本地算力，`main.tdb` 与 `main.history.jsonl` 是长期回归的基线。

## 已知限制

- 球 id 从 1 开始顺序分配这一假设成立的前提是「每次运行都是全新的库」，离线分析脚本（`relevance-rate.cjs`、`relevance-compare.cjs`）依赖它。
- 语料是 375 条基础句铺到 N 个球，N 越大重复越多，因此 `precision@K` / `recall@K` 只能同规模比，不能跨规模比（跨规模用同领域率与余弦裕度）。
