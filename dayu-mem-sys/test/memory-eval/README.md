# test/memory-eval —— 公开基准评测产物

LoCoMo / LongMemEval 的评测产物全部落在这里，**与 dayu-mem-sys 自身的测试产物
（`test/out/`）完全隔离**，互不干扰、互不清理。

## 目录约定

```
test/memory-eval/
├── README.md                       本文件（入库）
├── run-eval.cjs                    运行器（入库）
└── <YYYYMMDD-HHMM>-<adapter>-<dataset>[-<split>][-<tag>]/
    ├── config.toml                 本次运行实际使用的完整配置
    ├── serve.log                   dayu-serve 的 stdout + stderr
    ├── store/                      各 namespace 的 TriviumDB 文件
    ├── <adapter>_<ts>_n<N>.json    mceval 逐题结果
    └── meta.json                   参数 / git 提交 / 耗时 / 产物清单
```

一次运行 = 一个自描述目录，整目录可删、可打包、可对比。

## 用法

```bash
# dayu（需要先 cargo build -p dayu-serve --release）
node test/memory-eval/run-eval.cjs --dataset locomo --sample 100 --stratified --seed 0

# 基线（不需要起服务）
node test/memory-eval/run-eval.cjs --adapter bm25 --dataset locomo --sample 100 --stratified

# 管道对照
node test/memory-eval/run-eval.cjs --dataset locomo --sample 100 --stratified --link-weight off --tag nolink

# LongMemEval
node test/memory-eval/run-eval.cjs --dataset longmemeval --split oracle --sample 50 --seed 0
```

| 选项 | 说明 |
|:--|:--|
| `--adapter` | mceval adapter，默认 `dayu`；`bm25` / `dense` / `hybrid-rrf` 为基线 |
| `--config` | 嵌入后端配置名，对应 `test/config/<name>.toml`，默认 `lms` |
| `--dataset` | `locomo` / `longmemeval` |
| `--split` | 仅 longmemeval：`oracle` / `s` / `m` |
| `--sample` | 题目数，**必须**配 `--seed` 或 `--stratified` |
| `--workers` | 并行题目数，默认 4 |
| `--top-k` | 同时设 `start_top_n` 与 `max_balls_per_inject`，默认 10 |
| `--link-weight` | 同会话相邻回合管道粗细，`off` = 纯余弦对照 |
| `--tag` | 目录后缀，区分对照实验 |
| `--eval-repo` | memory-core-eval 仓库根（默认 `MCEVAL_REPO` 或 `D:/code_space/other/memory-core-eval`） |

## 依赖

| 依赖 | 说明 |
|:--|:--|
| `dayu-serve` | `cargo build -p dayu-serve --release` |
| `mceval` | memory-core-eval 仓库，`pip install -e .` 或直接 `python -m mceval.cli` |
| 数据集 | 拷进 `~/.cache/memory-core-eval/`，见 `docs/12` |
| 嵌入后端 | `--config lms` 需要本机 LM Studio 在 1234 端口跑嵌入模型 |

## 为什么单独开一个目录

`test/out/` 是 `cargo test` 的一次性产物，删了不损失信息；
`test/memory-eval/` 是**评测资产**——一次跑几十分钟，结果要留着横向对比。
两者生命周期不同，混在一起会导致 `rm -rf test/out` 顺手删掉评测结果，
或者评测的几百 MB 库文件拖慢日常测试。

## 结果怎么读

`meta.json` 里的 `result_files` 指向本次的逐题结果。字段含义：

| 字段 | 含义 |
|:--|:--|
| `recall` | `{1: bool, 5: bool, 10: bool}`，前 k 条里是否命中任一 gold 会话 |
| `n_indexed` | 本题灌进去的回合数 |
| `n_retrieved` | 服务实际返回条数（可能小于 top_k，见 `docs/12` §7） |
| `retrieved_sessions` | 返回顺序的会话 ID，用于复盘排序 |
| `evidence_sessions` | gold 会话 ID |
