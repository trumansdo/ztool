# 大禹记忆网络 · pi-agent 扩展

在 pi-agent 中集成长期记忆能力：自动召回相关记忆、不可见注入上下文、轮次归档。

## 安装

```bash
# 1. 构建 napi 原生模块
cd crates/dayu-napi && npm run build

# 2. 在 pi 配置中注册扩展
```

pi 配置示例（`.pi/config.yaml` 或 `~/.pi/config.yaml`）：

```yaml
extensions:
  dayu-mem-sys:
    path: ./extensions/pi-agent/src/index.ts
    config:
      recallEnabled: true
      archiveEnabled: true
      maxInjectionBlocks: 5
      injectionMode: hidden
      minRelevanceScore: 0.3
      injectionTokenBudget: 500
```

## 配置项

| 字段 | 类型 | 默认值 | 说明 |
|:--|:--|:--|:--|
| `recallEnabled` | boolean | `true` | 是否启用记忆召回注入 |
| `archiveEnabled` | boolean | `true` | 是否启用轮次归档 |
| `maxInjectionBlocks` | number | `5` | 最大注入块数 |
| `injectionMode` | `"hidden"` \| `"visible"` | `"hidden"` | hidden=不可见注入（不破坏前缀缓存），visible=可见注入 |
| `minRelevanceScore` | number | `0.3` | 最小相关性阈值（0-1），低于此值不注入 |
| `injectionTokenBudget` | number | `500` | 注入块 token 预算上限 |

## 钩子说明

| 钩子 | 触发时机 | 行为 |
|:--|:--|:--|
| `session_start` | 会话创建/切换 | 设置会话名称标记，初始化指标 |
| `before_agent_start` | Agent 每次启动前 | 召回相关记忆 → 过滤 → 不可见注入 |
| `turn_end` | 每轮对话结束 | 归档用户消息 + assistant 回复 |
| `session_before_compact` | 上下文压缩前 | 输出指标，确保数据持久化 |

## 调试命令

在 pi-agent 中可用：

- `dayu:stats` — 查看扩展指标和 napi 状态
- `dayu:reload` — 热重载配置

## 降级策略

napi 原生模块不可用时，扩展自动降级为 noop 模式：
- `recall()` → 返回空数组（不注入）
- `archiveTurn()` → 跳过（不归档）
- 不影响 pi-agent 正常运行
