以下按“必要 → 重要 → 可选”三级给出 UML 图集，全部用 Mermaid 语法，可直接粘贴到支持 Mermaid 的编辑器渲染。

---

## 一、用例图（必要）

回答“系统对外提供什么能力、谁在用”。

```mermaid
flowchart LR
    User([用户])
    Host[宿主 dsh / pi]
    Model[主模型]
    CLI[离线巩固 CLI]

    subgraph Dayu[大禹水利记忆网络]
        UC1[recall 召回并注入]
        UC2[archive_turn 归档本轮]
        UC3[judge_turn 起念判别]
        UC4[correct_memory 纠正]
        UC5[consolidate 离线巩固]
        UC6[diagnose 诊断与调参建议]
    end

    User --> Host
    Host --> UC1
    Host --> UC2
    Host --> UC3
    Host --> UC4
    Model -.注入后消费.-> UC1
    CLI --> UC5
    CLI --> UC6
    UC1 -.旁路观测.-> UC6
```

**为什么必要**：`recall`/`archive_turn` 是每轮对话的同步路径，`judge_turn` 是异步旁路，`correct_memory` 是用户显式操作，`consolidate`/`diagnose` 是离线路径。这六条用例划清了在线与离线、同步与异步的边界。

---

## 二、组件图（必要）

回答“代码怎么分层、依赖方向是否单向”。

```mermaid
flowchart TD
    subgraph Domain[dayu-domain 零依赖]
        D1[类型 newtype]
        D2[核心 trait]
        D3[错误枚举]
    end

    subgraph Infra[基础设施实现]
        S[dayu-storage<br/>TriviumDB]
        E[dayu-embedding<br/>ONNX / Mock]
        J[dayu-judge<br/>EmbeddingRule / Mock / 预留 DecisionModel]
    end

    subgraph Core[dayu-flow]
        F1[FlowEngine]
        F2[SettleEngine]
        F3[LossModel / SupplementStrategy / StrengthenStrategy]
    end

    subgraph Assembly[dayu-core 唯一组装点]
        C[DayuCore]
    end

    subgraph Boundary[边界]
        N[dayu-napi<br/>DTO 翻译]
        CLI[dayu-cli<br/>离线巩固]
    end

    D1 --> F1
    D2 --> S
    D2 --> E
    D2 --> J
    F1 --> C
    F2 --> C
    S --> C
    E --> C
    J --> C
    C --> N
    C --> CLI
```

**为什么必要**：依赖方向必须单向——`dayu-domain` 零依赖，`dayu-flow`/`dayu-judge` 只依赖 trait 不依赖实现，`dayu-core` 是唯一知道所有实现的地方，`dayu-napi` 只做 DTO 翻译。这张图是“模块可独立测试、可替换”的编译期依据。

---

## 三、类图（必要，核心领域模型）

回答“球、管道、内容、判定、后端抽象之间是什么关系”。

```mermaid
classDiagram
    class Ball {
        +BallId id
        +f64 size_stored
        +f64 size_max
        +f64 stability_S
        +Embedding embedding
        +ContentId[] content_list
        +DateTime last_watered
        +BallStatus status
        +Domain domain
        +size_effective() f64
    }

    class Pipe {
        +PipeId id
        +BallId a
        +BallId b
        +f64 w
        +u32 blocked_count
        +DateTime last_flowed
        +PipeStatus status
        +w_effective(now) f64
    }

    class Content {
        +ContentId id
        +BallId ball_id
        +bytes text
        +Encoding encoding
        +u32 injected_count
        +ContentStatus status
    }

    class Verdict {
        +bool should_judge
        +Relevance significance
        +Option~BallId~ belongs_to
        +Relevance belong_confidence
        +Option~f64~ emotion_valence
        +Option~f64~ emotion_intensity
        +Option~String~ offset_direction
        +Option~Vec~ hit_dimensions
        +Option~String~ reason
        +VerdictProducer producer
    }

    class JudgeBackend {
        <<trait>>
        +judge(input) Verdict
        +producer() VerdictProducer
        +capabilities() JudgeCapabilities
    }

    class StorageBackend {
        <<trait>>
        +get_ball(id) Ball
        +load_subgraph(starts, depth) SubGraph
        +search_by_embedding(query, k) BallIdScore[]
        +begin_transaction() StorageTx
    }

    class EmbeddingBackend {
        <<trait>>
        +embed_batch(texts) Vec~Vec~f32~~
        +dimension() usize
    }

    class Clock {
        <<trait>>
        +now() DateTime
    }

    class LossModel {
        <<trait>>
        +loss(w_eff, cfg) LossRate
    }

    Ball "1" --> "*" Content : 内容列表上限可配
    Ball "1" -- "*" Pipe : 两端
    JudgeBackend ..> Verdict : 产出
    StorageBackend ..> Ball : 读写
    StorageBackend ..> Pipe : 读写
    EmbeddingBackend ..> Ball : 提供嵌入
    Clock ..> Ball : 时间源
    LossModel ..> Pipe : 按 w_eff 算损失
```

**为什么必要**：这张图固定了三个关键契约——`size_stored` 与 `size_effective()` 的分离（新鲜度只算一次）、`Verdict` 的必填/可选字段分组、四个核心 trait 的方法集。后面所有实现都以此为契约。

---

## 四、时序图：recall（必要，在线同步路径）

回答“一轮对话开始时，注入是怎么同步完成的”。

```mermaid
sequenceDiagram
    participant TS as TS 适配层
    participant Core as DayuCore
    participant Emb as EmbeddingBackend
    participant Store as StorageBackend
    participant Flow as FlowEngine
    participant Settle as SettleEngine

    TS->>Core: recall(query, token_budget)
    Core->>Emb: embed(query)
    Emb-->>Core: query_embedding
    Core->>Store: search_by_embedding + load_subgraph
    Store-->>Core: 子图快照 + 版本号 V_start
    Core->>Flow: run(子图, 起始球, 水量)
    Note over Flow: 纯内存：注水→分配→逐跳判断→点亮
    Flow-->>Settle: 点亮集合 + 路径集合
    Settle->>Settle: 路径相关性→强化值→注入排序
    Settle->>Store: BEGIN IMMEDIATE + 版本校验
    alt 版本一致
        Store-->>Settle: 直接写入增量
    else 版本冲突
        Settle->>Store: 重读涉及球/管道，重算增量
        Store-->>Settle: 写入重算增量
    end
    Settle-->>Core: 注入块（完整球，不切割）
    Core-->>TS: Vec~InjectionBlock~
    TS->>TS: 拼装尾部 ephemeral 消息
```

**为什么必要**：它把“流动只点亮、结算才写入、结算即注入决策、版本冲突重算”四个关键约束画在了同一条时序上。

---

## 五、时序图：judge_turn（必要，异步旁路）

回答“判别器怎么在不阻塞主对话的前提下产生新球”。

```mermaid
sequenceDiagram
    participant Host as 宿主 turn_end
    participant TS as TS 适配层
    participant Core as DayuCore
    participant Judge as JudgeEngine
    participant BE as JudgeBackend
    participant Store as StorageBackend

    Host->>TS: turn 结束事件
    TS->>Core: judge_turn(候选片段, 角色状态, 最近窗口)
    Core->>Judge: run(candidates)
    loop 每个候选片段（全量，不做语义粗筛）
        Judge->>BE: judge(input)
        BE-->>Judge: Verdict
        alt 解析失败
            Judge->>Judge: 丢弃，不重试
        else should_judge 且显著性 ≥ 阈值
            Judge->>Store: 查兜底合并（余弦 > 0.95）
            alt 命中已有球
                Store-->>Judge: 追加内容 + 强化已有球
            else 新球
                Judge->>Store: 事务：建球+建内容+建初始管道
            end
        end
    end
    Core-->>TS: 完成（不阻塞主对话）
```

**为什么必要**：它固定了“判别异步、失败不重试、兜底合并先于建新球、新球事务原子”四个契约。

---

## 六、状态图：球与管道的生命周期（必要）

回答“球和管道在什么条件下转换状态，什么时候归档、什么时候可唤醒”。

```mermaid
stateDiagram-v2
    [*] --> active: 新球创建 / 管道创建
    active --> contested: 隐式纠正 / 离线矛盾检测
    contested --> active: 离线巩固确认正确
    contested --> archived: 用户显式纠正 / 离线确认错误
    active --> archived: size_effective<阈值 且 超90天未注水
    archived --> active: 被检索命中（≥ start_ball_threshold）
    archived --> deleted: 用户显式“永远忘掉” + 二次确认
    deleted --> [*]

    note right of archived
        不删除，只休眠
        管道跟随球一起归档
    end note

    note right of deleted
        唯一不可逆状态
        需显式确认
    end note
```

**为什么必要**：这张图是“疏而不堵”在状态层面的直接表达——`archived` 可被强线索唤醒，只有 `deleted` 不可逆。

---

## 七、活动图：流动与结算（必要，算法主流程）

回答“水从起始球到注入块，经过了哪些判定”。

```mermaid
flowchart TD
    A[当前输入嵌入] --> B[起始球选择：余弦 Top-N，过滤阈值]
    B --> C{有球超阈值?}
    C -- 否 --> Z[本轮不注水，返回空]
    C -- 是 --> D[按余弦占比分配水量，强线索放大]
    D --> E[当前球：计算可分配水量 = 流入 + 补充]
    E --> F[按 1+w 加权分配到各出边]
    F --> G[每条出边：先扣损 loss w_eff]
    G --> H{水量 > 阻力?}
    H -- 是 --> I[管道 blocked_count 不增，目标球点亮]
    H -- 否 --> J[blocked_count+1，损失不回收]
    I --> K{触达安全阀?}
    K -- 深度/球数到顶 --> L[该方向停止]
    K -- 否 --> E
    J --> M{还有其他出边?}
    M -- 是 --> G
    M -- 否 --> N[该球所有方向耗尽]
    L --> O[流动结束，点亮集合固定]
    N --> O
    O --> P[结算：流动加权语境，α≤0.3]
    P --> Q[路径相关性 × 长度折扣]
    Q --> R[强化球 / 最低强化球划分]
    R --> S[多路径取最大值，原子写入]
    S --> T[注入排序：相关性 × size_effective]
    T --> U[按预算装完整球，不切割]
    U --> V[渲染分档文本，尾部注入]
```

**为什么必要**：它把“先分配后判断、损失不回收、1+w 加权、安全阀、不切割完整球”全部画在一条流程上，是流动与结算的唯一权威流程图。

---

## 八、部署图（必要）

回答“dsh、pi、napi、TriviumDB、ONNX 模型在物理上怎么放”。

```mermaid
flowchart TB
    subgraph DshProc[dsh 进程]
        DshPlugin[TS 薄适配层<br/>session/event + agent/pre-step]
        DshNode[dayu-napi.node]
    end

    subgraph PiProc[pi 进程]
        PiPlugin[TS 薄适配层<br/>before_agent_start + turn_end]
        PiNode[dayu-napi.node]
    end

    subgraph Shared[共享本地资源]
        TDB[(store.tdb<br/>TriviumDB WAL)]
        ModelFast[models/fast<br/>pizza ONNX ~91MB]
        ModelBal[models/balanced<br/>可选 ONNX]
        Backup[(backup/ 快照)]
    end

    subgraph Offline[离线进程]
        CLI[dayu-cli]
    end

    DshPlugin --> DshNode
    PiPlugin --> PiNode
    DshNode --> TDB
    PiNode --> TDB
    DshNode --> ModelFast
    PiNode --> ModelFast
    DshNode -.可选.-> ModelBal
    CLI --> TDB
    CLI --> Backup
    TDB -.定期 VACUUM INTO.-> Backup

    note1[可选：压缩插件适配器<br/>仅当宿主装了压缩插件时注册<br/>核心不依赖它]
```

**为什么必要**：它说明“单文件 TriviumDB + 进程级文件锁”如何支撑 dsh 和 pi 共享同一份记忆，以及 ONNX 模型作为数据文件、离线 CLI 作为独立进程的部署形态。

---

## 九、可选：配置热更新时序（重要但非核心）

```mermaid
sequenceDiagram
    participant Admin as 管理入口
    participant Cfg as ConfigSnapshot
    participant Flow as FlowEngine
    participant Log as param_change_log

    Admin->>Cfg: 更新参数（受 min/max 边界约束）
    Cfg->>Cfg: 校验边界，生成新 config_version
    Cfg->>Log: 审计：old/new/reason/trigger_signal
    Cfg-->>Flow: arc-swap 新快照（不重启）
    Flow->>Flow: 下一次调用入口读新快照
    Note over Flow: 结算事务内使用同一快照<br/>不出现流动旧配置、结算新配置
```

---

### 必要 vs 可选汇总

| 图 | 级别 | 理由 |
|---|---|---|
| 用例图 | **必要** | 划清在线/离线、同步/异步边界 |
| 组件图 | **必要** | 依赖方向是独立测试和可替换的前提 |
| 类图 | **必要** | 固定 `size_stored`/`size_effective`、`Verdict` 分组、四个核心 trait |
| recall 时序 | **必要** | 固定“流动只点亮、结算即注入、版本冲突重算” |
| judge_turn 时序 | **必要** | 固定“异步旁路、失败不重试、兜底合并先于建球” |
| 状态图 | **必要** | “疏而不堵”在状态层的直接表达 |
| 活动图 | **必要** | 流动与结算的唯一权威流程 |
| 部署图 | **必要** | 多客户端共享单文件、模型作为数据文件、CLI 独立进程 |
| 配置热更新时序 | 可选 | 重要但属于运维层，不改变核心行为 |
