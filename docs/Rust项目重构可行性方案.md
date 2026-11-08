# Rust 项目重构可行性文档（AI Agent 执行版）

> **文档性质**：给本地 AI agent 阅读与执行的可行性方案，不是可直接编译的代码。
> **前提**：项目已完成主题抽象层的重构，现有代码可运行。本方案的目标是在**不破坏现有功能**的前提下，按四项重构目标系统性提升代码质量。
> **Agent 执行要求**：落笔前必须通读现有项目结构、`Cargo.toml`、所有 `unsafe` 块、模块组织方式。本方案的伪代码只表达契约与判据，具体类型名、模块路径以项目实际为准。


## 零、Agent 操作约束（最高优先级）

### 0.1 必须先分解 todo 目标列表

写代码之前，agent **必须**先把第六节的执行阶段展开成可勾选列表，每项标注验收条件，输出给用户确认。执行中逐项更新，**禁止跳过或合并**。

### 0.2 执行 bash 命令必须设置超时

| 命令类型 | 建议超时 |
| :--- | :--- |
| `cargo check`（首次或大改后） | 300–600 秒 |
| `cargo check`（增量） / `cargo clippy` | 120–180 秒 |
| `cargo build` / `cargo run` | 300 秒 |
| `cargo geiger` | 120–300 秒 |
| `cargo fix --edition` | 300 秒 |
| 普通 shell 命令 | 30–60 秒 |

**优先 `cargo check` 替代 `cargo build`**；**优先单 crate（`-p <crate>`）**。超时后先判断是"仍在编译"还是"真卡住"，不盲目重试。

### 0.3 重构的硬性安全边界

- **禁止一次性重写整个项目**。所有改动必须可增量编译、可独立测试。
- **禁止为追求"最佳风格"而改变对外行为**。每个重构步骤前后，现有功能必须通过。
- **禁止在没有测试覆盖的情况下大规模修改核心逻辑**。若目标模块缺少测试，先补测试，再重构。
- `unsafe` 相关改动**必须附带 `// SAFETY:` 注释**，说明安全不变式。


## 一、重构目标与验收总览

| # | 目标 | 核心判据 | 验证手段 |
| :-- | :--- | :--- | :--- |
| **1** | 最佳 Rust 风格，`unsafe` ≤ 0.5%，坏味道报告 | `unsafe` 行占比 ≤ 0.5%；Clippy 无 `correctness` / `suspicious` 级别告警；坏味道清单完整 | `cargo geiger` + `cargo clippy` + 人工审计 |
| **2** | 设计模式可重写性 + 人类可读性评估 | 每个功能点有"当前模式 / 候选模式 / 可读性结论"三栏记录 | 逐模块分析 + 模式对照表 |
| **3** | 2024 edition 推荐目录结构与模块化 | 无 `mod.rs`；按领域/功能组织；`lib.rs` / `main.rs` 职责清晰 | 目录审计 + `cargo check` |
| **4** | 最新 edition 特性评估 | 列出可利用的 2024 特性与迁移成本 | `cargo fix --edition` + 手动审计 |


## 二、目标 1：Rust 风格与 `unsafe` 约束

### 2.1 `unsafe` ≤ 0.5% 的判定方式

`cargo geiger` 统计的是 `unsafe` 函数、表达式、impl、trait、方法的数量，并支持 `ratio` 输出用于 CI 阈值。**"0.5%"应明确口径**：

- **推荐口径**：`unsafe` 代码行数 / 项目自身总代码行数（不含依赖），≤ 0.5%。
- **Agent 落地方式**：运行 `cargo geiger --output-format ratio`，记录项目 crate 的 ratio；若工具输出口径与上述不同，**以工具口径为准并记录偏差**。
- **注意**：`cargo geiger` 也统计依赖树的 `unsafe`。**判据只针对本项目自身代码**，依赖的 `unsafe` 单独报告，不计入 0.5%。

### 2.2 `unsafe` 最小化规则（Agent 必须遵守）

Rust 2024 edition 通过 `unsafe_op_in_unsafe_fn` lint **强制**：即使在 `unsafe fn` 内部，每个 `unsafe` 操作也需要自己的显式 `unsafe {}` 块。这意味着"整个函数标 unsafe"的旧写法在 2024 下不再成立。

具体规则：

1. **`unsafe` 块只包裹真正需要不安全的操作**，周围的算术、断言、循环保持在安全区域。
2. **优先安全包装**：一个小的安全函数包裹一个小的 `unsafe {}`，优于把整个函数暴露为 `unsafe fn`。
3. **每个 `unsafe` 块配 `// SAFETY:` 注释**，说明哪个前置条件保证了安全性。
4. 多个连续操作如果共享**完全相同的前置条件**，可以合并为一个块，但仍应是最小必要范围。

### 2.3 坏味道报告（Agent 必须产出）

使用 `cargo clippy --all-targets --all-features`，Clippy 提供 800+ lint，分为 `correctness`（默认 deny）、`suspicious` / `style` / `complexity` / `perf`（默认 warn）、`pedantic` / `restriction` / `nursery`（默认 allow）。

**报告必须包含**：

- 按 lint 类别统计的告警数量。
- `pedantic` 级别的告警清单（虽默认 allow，但对"最佳风格"目标有价值）。
- 人工识别的、Clippy 不覆盖的坏味道，例如：`unwrap()` 在非测试代码中的滥用、过度 `.clone()`、`Rc/Arc/RefCell` 不必要的共享、模块命名为 `utils` / `common` / `misc`。
- 每条坏味道标注：位置、类别、建议修复方式、是否自动可修（`cargo clippy --fix`）。


## 三、目标 2：设计模式的 Rust 化重写与可读性评估

### 3.1 核心原则：经典模式多数不需要"照搬"

Rust 社区的一致结论是：**许多经典 GoF 模式在 Rust 中是不必要的**。YAGNI 是设计模式章节的开篇原则；Strategy 就是 trait，`Option::map` 本身就是 Strategy。因此，评估的重点不是"能不能套模式"，而是"**用 Rust 惯用法重写后是否更可读**"。

### 3.2 Agent 必须逐功能点产出的评估表

| 字段 | 内容要求 |
| :--- | :--- |
| **功能点** | 模块/函数/类型名 |
| **当前实现方式** | 描述现有的组织方式（如手写状态机、重复的构造函数、`Box<dyn Trait>` 动态分发等） |
| **候选 Rust 模式** | 从下方清单中选择，注明"适用 / 不适用 / 有条件适用" |
| **可读性结论** | 重写后人类阅读成本是降低、持平还是升高；给出理由 |
| **建议** | 重写 / 保持现状 / 先补测试再重写 |

### 3.3 候选 Rust 模式清单与可读性判据

| 模式 | 何时值得用 | 可读性风险 |
| :--- | :--- | :--- |
| **Newtype** | 需要类型安全、防止参数混淆 | 低；但过多的 newtype 包裹会增加解包噪音 |
| **Builder** | 复杂配置、大量可选参数 | 中；简单类型不需要 Builder，`Default` + 结构体更新语法更简洁 |
| **RAII guard** | 资源清理、锁、临时状态回滚 | 低；比手动 `finally` 更清晰，但 `Drop` 不可依赖于关键逻辑 |
| **Trait object（`Box<dyn Trait>`）** | 需要运行时多态、异构集合 | 中；会增加间接层，**优先泛型 + `impl Trait`**，除非动态分发是必要的 |
| **泛型 + trait bounds** | 编译期多态、零成本抽象 | 低到中；`where` 子句在复杂时比内联 bound 更可读 |
| **状态机（enum + 关联数据）** | 非法状态不应可表示 | 低；Rust 的 enum 天然适合，优于"struct + 多个 Option 字段" |
| **`mem::take` / `mem::replace`** | 从 `&mut` 中取出字段而不克隆 | 低；比 `.clone()` 更清晰且高效 |

**可读性总判据（Agent 必须使用）** ：

> 如果一个模式让**调用方**写更少代码、让**实现方**的意图更明显、且不引入额外的 trait/泛型层级，则可读性提升。反之，如果只是为了"用模式"而增加间接层，应拒绝。

### 3.4 与现有 `ThemeStyles` 策略模式的关系

你的 `ThemeStyles` 已经是 Rust 惯用的策略模式（trait + 多实现 + `Box<dyn>` 持有）。**不需要把它"重写成经典策略类图"**。评估时重点看：`Box<dyn ThemeStyles>` 是否引入了不必要的不动态分发；如果调用点可以用泛型替代，可以评估，但**不要为了消除 `dyn` 而破坏"换主题只换实现"的简洁性**。


## 四、目标 3：2024 edition 目录结构与模块化

### 4.1 模块文件布局

2024 edition 的推荐做法是 **`foo.rs` + `foo/bar.rs`**，而不是旧的 `foo/mod.rs`。Agent 必须：

- **新代码禁止使用 `mod.rs`**。
- 现有 `mod.rs` 逐步迁移为 `module.rs` + `module/` 目录结构。
- 每个子模块在父模块中显式声明（`mod foo;`），通过路径引用，而不是依赖文件系统隐含关系。

### 4.2 按领域/功能组织，而非按类型

**禁止** `utils/`、`common/`、`misc/`、`types/`、`traits/`、`enums/` 这类按"文件类型"划分的目录。应按**领域**或**功能**组织：

```
src/
├── main.rs / lib.rs
├── theme/          # 主题抽象：tokens、ThemeStyles、ThemeSource
├── domain/         # 领域状态，无 iced 依赖
├── ui/             # view、组件、theme_adapters
├── services/       # DocumentService、SettingsStore 等 IO 抽象
└── actions/        # Action、Command
```

每个子域是独立文件夹，边界清晰；依赖方向应为 `ui → theme/domain → services`，**领域层不反向依赖 UI**。

### 4.3 `lib.rs` / `main.rs` 职责

- `main.rs` 只做启动、组装 `App`、调用 iced 运行入口，不包含业务逻辑。
- 若项目同时产出库和可执行文件，核心逻辑放在 `lib.rs` 中，`main.rs` 做薄壳。
- 单元测试与代码同文件（`#[cfg(test)]`），集成测试放在 `tests/`。

### 4.4 可见性最小化

每个模块的公开接口应尽可能小。用 `pub(crate)` 替代不必要的 `pub`；跨 crate 的公共 API 才用 `pub`。Agent 审计时，对每个 `pub` 项检查是否真的需要对外暴露。


## 五、目标 4：2024 edition 特性评估

### 5.1 必须先做的迁移动作

- **在旧 edition 下运行 `cargo fix --edition`**，审查 diff，再修改 `Cargo.toml` 的 `edition = "2024"`。
- 2024 隐含 **resolver v3**（Rust-version aware），Agent 需确认 workspace 依赖解析行为符合预期。

### 5.2 必须人工审计的破坏性变更

`cargo fix` 只能做机械修复，**不能证明 FFI 签名、符号导出、裸指针操作的安全性**。Agent 必须逐项审计：

| 变更 | 对项目的潜在影响 | Agent 动作 |
| :--- | :--- | :--- |
| **RPIT lifetime capture** | `impl Trait` 返回值现在隐式捕获所有作用域内的泛型/生命周期参数，可能改变借用检查行为 | 检查所有返回 `impl Trait` 的函数，确认生命周期捕获是否符合预期；需要时用 `use<..>` 精确控制 |
| **`if let` / tail expression 临时作用域收窄** | 临时值可能比旧 edition 更早 drop，可能改变锁、资源、借用行为的时机 | 审计依赖临时值生命周期的代码，确认更早 drop 是期望行为 |
| **`unsafe extern` 块** | `extern` 块现在必须写 `unsafe extern` | 机械迁移 + 确认 FFI 边界安全 |
| **`unsafe` 属性** | `no_mangle` / `export_name` / `link_section` 必须包在 `#[unsafe(...)]` 中 | 机械迁移 |
| **`static mut` 引用** | 对 `static mut` 的引用被限制 | 审计所有 `static mut`，考虑用 `OnceLock` / `Mutex` / `Atomic` 替代 |
| **match ergonomics 保留** | 某些模式组合被禁止，为未来改进让路 | 按编译错误逐一修复，通常无行为变化 |
| **`gen` 关键字保留** | 名为 `gen` 的标识符需改为 `r#gen` | 机械迁移 |

### 5.3 可主动利用的 2024 特性

- **`Future` / `IntoFuture` 进入 prelude**：异步代码可能受益，但需注意方法调用歧义。
- **Boxed slice `.into_iter()` 返回 owned 值**：若项目依赖旧行为（返回引用），需显式调整。
- **更严格的 `unsafe` 规则**：与目标 1 的 `unsafe` 最小化天然协同，迁移过程本身就是一次 `unsafe` 审计。


## 六、执行阶段与 Agent 检查清单

### 阶段 1：基线审计（只读，不改代码）

- [ ] 运行 `cargo geiger --output-format ratio`，记录项目自身 `unsafe` 占比和依赖 `unsafe` 概况。
- [ ] 运行 `cargo clippy --all-targets --all-features`，按类别统计告警，产出坏味道初版清单。
- [ ] 审计目录结构：列出所有 `mod.rs`、`utils`/`common`/`misc` 命名、按类型划分的目录。
- [ ] 列出所有 `unsafe` 块及其位置，标注是否已有 `// SAFETY:` 注释。

**验收**：产出三份基线报告（`unsafe` 占比、坏味道清单、目录结构问题清单）。

### 阶段 2：2024 edition 迁移

- [ ] 在旧 edition 下 `cargo fix --edition`，审查 diff。
- [ ] 人工审计第五节的破坏性变更（RPIT、临时作用域、`unsafe extern`、`static mut`）。
- [ ] 修改 `Cargo.toml` 为 `edition = "2024"`，`cargo check` 通过。
- [ ] 按 2.2 规则重构 `unsafe` 块（最小化、`// SAFETY:`、安全包装）。

**验收**：`cargo check` 通过；`unsafe` 占比 ≤ 0.5%；每个 `unsafe` 块有 `// SAFETY:`。

### 阶段 3：模块与目录重构

- [ ] 按 4.2 的领域/功能原则重组目录，移除 `mod.rs` 和 `utils` 类命名。
- [ ] 收敛 `pub` 可见性，`main.rs` 瘦身。
- [ ] 每完成一个模块迁移，`cargo check -p <crate>` 通过。

**验收**：目录符合 4.1–4.3；`cargo check` 通过；现有功能手动验证通过。

### 阶段 4：设计模式评估与选择性重写

- [ ] 逐模块填写 3.2 的评估表。
- [ ] 对"建议重写"的项，**先补测试**，再按 Rust 惯用法重写。
- [ ] 对 `ThemeStyles` 策略层，评估是否有不必要的 `Box<dyn>` 或可以简化的泛型，但**不破坏策略模式的可替换性**。

**验收**：评估表完整；重写项有测试覆盖；可读性结论有明确理由。

### 阶段 5：坏味道清理与最终报告

- [ ] 用 `cargo clippy --fix` 自动修复可修项，人工处理不可修项。
- [ ] 对 `unwrap()`、过度 `.clone()`、不必要的 `Rc/Arc/RefCell` 做定向审计。
- [ ] 产出最终报告：`unsafe` 前后对比、Clippy 告警前后对比、目录结构变更清单、设计模式评估结论、2024 特性利用清单。

**验收**：报告完整；`unsafe` ≤ 0.5%；`cargo clippy` 无 `correctness` 级别告警。


## 七、边界声明

- **不追求"零 `unsafe`"**：0.5% 是硬上限，但若某个 `unsafe` 是 FFI/平台绑定所必需且已安全封装，保留是合理的。
- **不追求"消除所有经典设计模式"**：策略、Builder、Newtype 等在 Rust 中有正当用途；判据是**可读性**，不是"是否像 GoF"。
- **不为了 2024 特性而使用 2024 特性**：每个特性必须有明确的收益或迁移必要性。
- **不在缺少测试时大规模重写核心逻辑**：先补测试，再重构。
- **不一次性合并多个阶段**：每个阶段独立编译、独立验证。


**文档版本**：v1.0。Agent 若发现方案与现有代码结构冲突，**以现有代码为准并记录偏差**，不要为迁就文档而破坏已有正确实现。
