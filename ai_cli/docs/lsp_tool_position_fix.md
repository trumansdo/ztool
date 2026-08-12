# lsp_tool 光标位置偏移修复记录

> 记录时间：2026-08-13  
> 项目：ztool/ai_cli  
> 工具：lsp_tool —— LSP 代码检索 CLI 工具  
> 影响范围：lsp 模块所有光标位置敏感命令（hover / def / refs / comp / signature / action / rename）

---

## 目录

1. [一、问题背景](#一问题背景)
2. [二、根因分析](#二根因分析)
3. [三、修复方案](#三修复方案)
4. [四、修改清单](#四修改清单)
5. [五、验证结果](#五验证结果)
6. [六、已知限制](#六已知限制)

---

## 一、问题背景

使用 `lsp_tool --server javac-lsp` 在 zero-flow 项目上测试时发现：

| 命令 | 表现 | 期望 |
|:--|:--|:--|
| hover | 声明处/调用处全部"无 hover 信息"，偶发错误返回 `StopException` 文档 | 返回光标处符号签名与 Javadoc |
| def | "未找到定义"，或错误跳转到无关位置 | 精确跳转到目标定义（含重载解析） |
| comp | 仅返回 Java 关键字（package/import/class…） | 返回上下文感知的成员补全 |
| refs | 结果位置整体偏移 | 返回精确的引用列表 |
| symbols | 正常 | 正常 |

进一步发现：`new ArrayListZeroFlow<>`（第 43 行）的 def 请求，在 **jdtls 与 javac-lsp 两个服务器上返回完全相同的错误结果** `ZeroFlow.java:50:18`（第 44 行的 `ZeroFlow` 类型定义）。不同服务器返回相同错误 → 问题定位在**客户端工具层**，而非语言服务器。

## 二、根因分析

**LSP 协议 Position 为 0-based，而 CLI 参数按 1-based（AI 习惯）传入，请求端缺失 1-based → 0-based 转换。**

代码证据：

- `src/lsp/output.rs` 输出端已有补偿：`format_locations` 中 `sl + 1, sc + 1`（0-based → 1-based 显示）✅
- `src/lsp/commands/*.rs` 请求端直接构造 `Position { line, character }` 原样发送，**无任何 -1 转换** ❌

因此所有光标命令的请求位置整体偏移 (+1, +1)，落在目标位置下方一行的同一列：

- 请求 `line=43, char=10`（`new ArrayListZeroFlow<>`）→ 服务器收到 0-based(43,10) = 实际 1-based(44,11) → 命中第 44 行 `ZeroFlow<List<Integer>>` 的 `ZeroFlow` → def 错误跳转到 `ZeroFlow.java:50:18` ✅ 与实测完全吻合
- 请求 `line=61, char=17`（`stop()` 方法名）→ 服务器收到 0-based(61,17) = 1-based(62,18)（空行）→ javac-lsp 空行 hover 匹配到下方 `throw StopException.INSTANCE;` → 错误返回 StopException 文档 ✅ 与实测完全吻合
- comp 偏移后落在空行 → javac 编译器返回顶层关键字补全（默认分支）✅ 与实测完全吻合

## 三、修复方案

**在库层（commands 层）统一转换**，而非 CLI 层，原因：

1. `lsp_tool.rs` 的方案C回退逻辑（`extract_word_at` / `fallback_open_related`）基于**文本级 1-based 行号**（grep 语义），若在 CLI 层转换会破坏该逻辑
2. commands 层是 LSP 请求的唯一出口，7 组命令 9 处 Position 构造点集中，单一转换点最可靠

修复方式：`src/lsp/client.rs` 新增辅助函数：

```rust
/// 将 1-based 行列 (AI/CLI 习惯) 转换为 LSP 协议要求的 0-based Position。
pub fn lsp_position(line: u32, character: u32) -> Position {
    Position {
        line: line.saturating_sub(1),
        character: character.saturating_sub(1),
    }
}
```

`saturating_sub(1)` 保证输入 0 时输出 0（LSP 最小合法值），不 panic。

## 四、修改清单

| 文件 | 修改内容 |
|:--|:--|
| `src/lsp/client.rs` | 新增 `lsp_position()` 辅助函数（1-based → 0-based） |
| `src/lsp/commands/hover.rs` | `position: Position { line, character }` → `lsp_position(line, character)` |
| `src/lsp/commands/definition.rs` | 同上 |
| `src/lsp/commands/references.rs` | 同上 |
| `src/lsp/commands/completion.rs` | 同上 |
| `src/lsp/commands/signature_help.rs` | 同上 |
| `src/lsp/commands/code_action.rs` | `range.start` / `range.end` 两处 → `lsp_position(...)` |
| `src/lsp/commands/rename.rs` | `prepareRename` + `rename` 两处 → `lsp_position(...)` |

共 8 个源文件，9 处 Position 构造点替换。`lsp_tool.rs` 未改动（fallback 文本逻辑天然正确）。

## 五、验证结果

使用 `lsp_tool --server javac-lsp --classpath <target/classes> --project-dir <zero-flow>` 实测：

| 命令 | 验证锚点 | 修复前 | 修复后 |
|:--|:--|:--|:--|
| def | `new ArrayListZeroFlow<>` (SeqTest.java:43) | 错误跳转 `ZeroFlow.java:50:18` | ✅ 精确跳转 `ArrayListZeroFlow.java:12:14`（类声明） |
| def | `of(Arrays.asList(ts))` (ZeroFlow.java:534) | 未找到定义 | ✅ 精确解析重载 `of(Iterable)` → `ZeroFlow.java:544:26` |
| hover | `.map(...)` 调用点 (SeqTest.java:45) | 无信息 | ✅ `SizedZeroFlow<E> map(Function<T,E>)` 签名+@inheritDoc |
| hover | `of(...)` 调用点 (ZeroFlow.java:534) | 无信息 | ✅ `of(Iterable)` 完整签名+Javadoc |
| comp | `.map(` 调用处 (SeqTest.java:45) | 仅 Java 关键字 | ✅ map/mapNotNull/mapToInt 等 7 个上下文成员补全 |
| refs | `ZeroFlow.of(...)` 调用点 (SeqTest.java:59) | 位置偏移 | ✅ 跨文件 15+ 处引用（方案C 自动打开关联文件生效） |
| symbols | ZeroFlow.java | 正常 | ✅ 回归正常 |

构建：`cargo build`（debug）+ `cargo build --release` 均零错误零警告。

## 六、已知限制

1. **javac-lsp hover 仅支持调用点/引用处**：声明处（类声明、方法定义处）返回空，属上游 `FindHoverElement` 行为（基于 IdentifierTree / MemberSelectTree），非本工具 bug
2. **classpath 为空时 mvn 生成可能卡死**：`project_classpath` 未配置时 javac-lsp 自动触发 `mvn dependency:build-classpath`，无三方依赖项目（如 zero-flow）会长时间卡住；使用显式 `--classpath target/classes` 可绕过，建议将项目 classpath 写入 `binconfig.toml` 的 `[lsp.javac-lsp.vars].project_classpath`
3. **jdtls 配置路径顺带修复**：`binconfig.toml` 中 jdtls 的 java/launcher/configuration 路径由失效的 `D:\program\...` 修正为实际存在的 `D:\dev_program\...`（jdk-21.0.1、jdt-language-server-1.61.0）
