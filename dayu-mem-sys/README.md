# 大禹水利记忆网络 (Dayu Memory System)

基于水利隐喻的 AI 长期记忆系统——以"水流—管道—水球"模型实现记忆的存储、流动、结算与注入。

## 模块架构

```
┌──────────────────────────────────────────────────────────────────┐
│                         dayu-core                                 │
│                   统一入口 · 生命周期管理                            │
│         DayuCore::new(config) → recall / archive / flow           │
└──────┬───────┬───────┬───────┬───────┬───────┬──────────────────┘
       │       │       │       │       │       │
       ▼       ▼       ▼       ▼       ▼       ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│dayu-flow │ │dayu-judge│ │dayu-embed│ │dayu-storage│ │dayu-obs  │
│ 流动引擎  │ │ 判别引擎  │ │ 嵌入引擎  │ │ 存储引擎  │ │ 可观测性  │
│          │ │          │ │          │ │          │ │          │
│FlowEngine│ │JudgeEngi│ │Embedding │ │TriviumStore│ │MetricsRec│
│SettleEngi│ │MockJudge │ │MockEmbed │ │TriviumStorage│ │MockMetri│
│LossModel │ │OrtJudge │ │FixedEmbed│ │          │ │Diagnostic│
│          │ │          │ │OrtEmbed  │ │          │ │          │
└────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘
     │            │            │            │            │
     └────────────┴────────────┴────────────┴────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │ dayu-domain  │
                    │  领域类型层   │
                    │              │
                    │ Ball · Pipe  │
                    │ FlowEngine   │
                    │ StorageBack  │
                    │ JudgeBackend │
                    │ EmbeddingBa  │
                    │ MetricsRec   │
                    │ Config       │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ dayu-config  │
                    │  配置系统     │
                    │              │
                    │ TOML 解析    │
                    │ arc-swap     │
                    │ 热更新       │
                    │ 校验         │
                    └──────────────┘
```

## 依赖方向（单向，零循环）

```
dayu-domain       ← 零依赖（纯 trait + newtype）
    ↑
dayu-config       ← dayu-domain
    ↑
dayu-flow         ← dayu-domain + dayu-config
dayu-judge        ← dayu-domain + dayu-config
dayu-embedding    ← dayu-domain + dayu-config
dayu-storage      ← dayu-domain
dayu-observability← dayu-domain
    ↑
dayu-core         ← 聚合以上全部 + dayu-testkit(dev)
    ↑
dayu-napi         ← dayu-core
dayu-cli          ← dayu-core
```

## 数据流

```
用户消息 ──→ DayuCore.recall()
                │
                ├──→ EmbeddingBackend.encode(text) → 嵌入向量
                ├──→ StorageBackend.query(embedding) → 候选球
                ├──→ FlowEngine.flow(store, query) → 点亮球列表
                ├──→ JudgeEngine.judge(lit_balls) → 注入块
                └──→ 返回 InjectionBlock[]

用户消息 ──→ DayuCore.archive_turn()
                │
                ├──→ EmbeddingBackend.encode(text)
                ├──→ StorageBackend.insert(ball)
                ├──→ FlowEngine.flow(store, query) → 点亮
                ├──→ SettleEngine.settle(lit_balls) → 强化/衰减/新建
                └──→ StorageBackend.commit()
```

## 目录结构

```
dayu-mem-sys/
├── crates/
│   ├── dayu-domain/       # 领域类型：trait + newtype + 错误
│   │   └── src/
│   │       ├── types/     # BallId, PipeId, ContentId, TurnId, Config, LossRate
│   │       ├── error.rs   # DomainError, FlowError, StorageError, ValueError
│   │       ├── traits.rs  # FlowEngine, StorageBackend, JudgeBackend, EmbeddingBackend
│   │       └── lib.rs
│   ├── dayu-config/       # 配置系统：TOML + arc-swap + 热更新 + 校验
│   ├── dayu-flow/         # 流动引擎 + 结算引擎 + 损失模型
│   │   └── src/
│   │       ├── engine.rs  # FlowEngine: 逐跳流动，先扣损再判断
│   │       ├── settle.rs  # SettleEngine: 结算 + 路径折扣 + 注入渲染
│   │       ├── loss.rs    # LossModel trait + PipeWeightLossModel + ConstantLossModel
│   │       └── lib.rs
│   ├── dayu-judge/        # 判别引擎：Mock + ONNX 后端
│   │   └── src/
│   │       ├── mock.rs    # MockJudgeBackend
│   │       ├── ort.rs     # OrtJudgeBackend (feature=onnx)
│   │       └── lib.rs
│   ├── dayu-embedding/    # 嵌入引擎：Mock + Fixed + ONNX 后端
│   │   └── src/
│   │       ├── mock.rs    # MockEmbeddingBackend
│   │       ├── fixed.rs   # FixedEmbeddingBackend
│   │       ├── ort.rs     # OrtEmbeddingBackend (feature=onnx)
│   │       └── lib.rs
│   ├── dayu-storage/      # 存储引擎：Memory + Trivium(JSON文件)
│   │   └── src/
│   │       ├── trivium.rs # TriviumStorage (TriviumDB 0.9.0)
│   │       ├── trivium.rs # TriviumStorage (JSON文件过渡方案)
│   │       └── lib.rs
│   ├── dayu-observability/# 可观测性：MetricsRecorder + Mock + 诊断规则
│   │   └── src/
│   │       ├── mock.rs    # MockMetricsRecorder
│   │       └── lib.rs
│   ├── dayu-core/         # 统一入口：DayuCore + 维度校验 + 端到端
│   ├── dayu-napi/         # napi-rs 绑定：Node.js 原生模块
│   ├── dayu-cli/          # CLI 工具：命令行交互
│   └── dayu-testkit/      # 测试夹具：共享测试工具
├── extensions/
│   ├── pi-agent/          # pi-agent 适配层
│   │   └── index.ts       # before_agent_start / turn_end / session_before_compact
│   └── dsh/               # dsh 插件
│       └── index.ts       # onSessionEvent / onAgentPreStep
├── models/                # 模型文件（junction → D:\program_data\hugging_face）
├── scripts/               # 工具脚本
│   └── download_embedding_model.ps1
├── docs/                  # 设计文档
│   ├── 00-对话.md
│   ├── 01-概念确认条目表.md
│   ├── 02-概念确认条目说明.md
│   ├── 03-大禹水利记忆系统-设计说明文档.md
│   ├── 04-技术方案.md
│   ├── 05-依赖与技术选型.md
│   ├── 06-实施规划文档.md
│   ├── 07-测试方案原则.md
│   └── 08-设计图uml图集.md
├── PROGRESS.md            # 实施进度追踪
├── Cargo.toml             # workspace 根配置
└── README.md
```

## 核心概念

| 概念 | 隐喻 | 对应 |
|:--|:--|:--|
| **水球 (Ball)** | 记忆单元 | 一条对话片段、一个知识点 |
| **管道 (Pipe)** | 记忆关联 | 两个水球之间的语义连接 |
| **水流 (Flow)** | 记忆检索 | 从查询出发，沿管道逐跳流动 |
| **点亮 (Lit)** | 记忆命中 | 水流经过的水球被"点亮" |
| **结算 (Settle)** | 记忆巩固 | 强化有效路径，衰减无效路径 |
| **注入 (Inject)** | 记忆呈现 | 将点亮的水球注入 AI 上下文 |
| **损失 (Loss)** | 记忆衰减 | 管道双重角色：阻力 + 损耗 |

## 快速开始

```bash
# 编译
cargo build --workspace

# 测试
cargo test --workspace

# 运行 CLI
cargo run -p dayu-cli -- --config config.toml

# 下载嵌入模型（首次）
.\scripts\download_embedding_model.ps1
```

## 测试覆盖

**113 个测试全部通过**，覆盖全部 9 个 crate：

| Crate | 测试数 | 覆盖内容 |
|:--|:--|:--|
| dayu-config | 12 | TOML 解析、热更新、校验 |
| dayu-core | 13 | 统一入口、维度校验、端到端 |
| dayu-domain | 11 | newtype、Config、错误 |
| dayu-embedding | 7 | Mock + Fixed 后端 |
| dayu-flow | 22 | 流动引擎、结算引擎、损失模型 |
| dayu-judge | 13 | Mock + ONNX 判别 |
| dayu-napi | 2 | Node.js 绑定 |
| dayu-observability | 18 | 指标记录、诊断规则 |
| dayu-storage | 10 | Memory + Trivium 存储 |
| dayu-testkit | 5 | 共享测试夹具 |
