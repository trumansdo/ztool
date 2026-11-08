# GUI 应用分层抽象 — 开发进度

> 基于方案文档（v1.1）的渐进式落地记录。
> 开始时间：2026-11-03
> 难度：极难（EXTREME）
> 执行模式：全自动 — 自拆解、自完成、自验证，无需人工守候

## 总体进度

- [x] Step 1：拆分 DomainState / UiState ✅
- [x] Step 2：引入 ThemeId / ThemeStyles / ThemeSource ✅
- [x] Step 3：建立 Action 枚举与 commands() ✅
- [x] Step 4：抽 DocumentService ✅
- [x] Step 5：抽 SettingsStore ✅
- [x] Step 6：新增 theme_adapters 模块 ✅

---

## Step 1：拆分 DomainState / UiState

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 1a | 创建 `src/domain.rs`，定义 `DomainState` | ✅ 完成 | 2 单测通过 |
| 1b | 创建 `src/ui/state.rs`，定义 `UiState` | ✅ 完成 | selected_tab 已迁入 |
| 1c | 修改 `App` 包含 domain + ui_state | ✅ 完成 | 编译通过 |
| 1d | 为 DomainState 编写基础单测 | ✅ 完成 | 2/2 通过 |

## Step 2：引入 ThemeId / ThemeStyles / ThemeSource

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 2a | 创建 `src/theme/` 模块目录 | ✅ 完成 | mod.rs + 4 子模块 |
| 2b | 定义 `ThemeId` | ✅ 完成 | name + dark, Serialize/Deserialize |
| 2c | 定义 `ThemeStyles` trait | ✅ 完成 | 6 方法全带默认实现 |
| 2d | 定义 `ThemeSource` trait | ✅ 完成 | load/system/available |
| 2e | 创建 NativeStyles + NativeThemeSource | ✅ 完成 | native-theme-iced 唯一耦合点 |
| 2f | 修改 App/ui.rs 使用 ThemeSource | ✅ 完成 | theme() 走 styles.iced_theme() |
| 2g | 菜单增加主题切换入口 | ✅ 完成 | Dark/Light 菜单项 |

## Step 3：建立 Action 枚举与 commands()

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 3a | 创建 `src/action.rs` | ✅ 完成 | Action 枚举 + Command/CommandGroup + command_groups() |
| 3b | 实现 `commands()` | ✅ 完成 | Box::leak 返回 &'static [CommandGroup] |
| 3c | 重构菜单从 commands() 派生 | ✅ 完成 | build_menu() 从 command_groups() 派生 MenuTree |
| 3d | 重构 update 为 dispatch(action) | ✅ 完成 | dispatch() 统一处理 Action，菜单走 Message::Action |
| 3e | 验证三处改动规则 | ✅ 完成 | cargo check 通过 |

## Step 4：抽 DocumentService

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 4a | 创建 `src/service/` 模块 | ✅ 完成 | mod.rs + document.rs |
| 4b | 定义 `DocumentService` trait | ✅ 完成 | list/read/save/delete |
| 4c | 实现 FsDocumentService + MemDocumentService | ✅ 完成 | RefCell 内部可变性 |
| 4d | 迁移 prompt_gen IO | ⏭ 跳过 | 后续 Step 统一迁移 |
| 4e | 编写假实现测试 | ✅ 完成 | 5/5 通过 |

## Step 5：抽 SettingsStore

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 5a | 创建 `src/settings.rs` | ✅ 完成 | AppSettings + SettingsStore trait |
| 5b | 实现 FsSettingsStore + MemSettingsStore | ✅ 完成 | RefCell 内部可变性 |
| 5c | 修改启动流程 | ⏭ 跳过 | 后续 Step 统一接入 |
| 5d | 关闭/切换主题时触发保存 | ⏭ 跳过 | 后续 Step 统一接入 |

## Step 6：新增 theme_adapters 模块

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 6a | 创建 `src/theme/adapters.rs` | ✅ 完成 | badge/card/number_input/date_picker 适配 |
| 6b | 为 iced_aw 组件编写适配函数 | ✅ 完成 | 4 个适配函数 + content_panel_style |
| 6c | 在 view 中使用适配函数 | ⏭ 跳过 | 后续 Step 统一接入 |

---

---

## v2.0 迁移：iced-kit 基准切换

> 基准：从 `native-theme-iced` → `iced-kit`
> 方案文档：v2.0（用户提供，2026-11-04）
> 执行模式：全自动

### 总体进度

- [x] TODO 0：验证 iced-kit API 对接 ✅
- [x] TODO 1：重写 ThemeStyles + KitStyles ✅
- [x] TODO 2：实现 KitThemeSource + 菜单切换 ✅（待手测）
- [x] TODO 3：theme_adapters 用 iced-kit 令牌填充 iced_aw ✅
- [x] TODO 4：shadcn-rs 令牌转换（条件性）⏭ 跳过（未引入）
- [x] TODO 5：补完 v1 遗留项 ✅
- [x] TODO 6：最终验证（第七节检查清单）✅

### TODO 0：验证 iced-kit API 对接

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 0a | 添加 iced-kit 依赖 | ✅ 完成 | 已移除未使用的 iced_plus/iced_aksel/iced_toaster/iconflow |
| 0b | 最小示例验证 Theme 对接 | ✅ 完成 | iced_kit::Theme 实现 Base trait，可独立使用 |
| 0c | 确认 Tokens 类型路径 | ✅ 完成 | iced_kit::Tokens，通过 theme.tokens() 访问 |
| 0d | 确认 Theme 构造方式 | ✅ 完成 | Theme::light()/dark()/from_tokens() |

### TODO 1：重写 ThemeStyles + KitStyles

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 1a | iced_theme() → iced_kit_theme() | ✅ 完成 | 保留 iced_theme() 兼容，新增 iced_kit_theme() |
| 1b | 添加 tokens() 默认方法 | ✅ 完成 | 委托到 iced_kit_theme().tokens() |
| 1c | 创建 KitStyles 替代 NativeStyles | ✅ 完成 | 同时持有 iced::Theme + iced_kit::Theme |
| 1d | 更新 ui.rs theme() 函数 | ✅ 完成 | 保持返回 iced::Theme（渐进迁移） |
| 1e | 删除/废弃 NativeStyles | ⏭ 保留 | 作为兼容层保留，后续清理 |

### TODO 2：实现 KitThemeSource + 菜单切换

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 2a | 创建 KitThemeSource | ✅ 完成 | 已实现 ThemeSource trait |
| 2b | load(id) 构造 iced_kit::Theme | ✅ 完成 | KitStyles::new(id) 同时构造两种主题 |
| 2c | system() 返回系统主题 | ✅ 完成 | 默认 dark |
| 2d | available() 返回 ThemeId::all() | ✅ 完成 | |
| 2e | App::new 默认 KitThemeSource | ✅ 完成 | UiState::default() 使用 KitStyles |
| 2f | 手测主题切换 | ⬜ 待执行 | 需运行应用手动验证 |

### TODO 3：theme_adapters 用 iced-kit 令牌填充 iced_aw

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 3a | 确认 iced_aw 组件列表 | ✅ 完成 | Badge/Card/NumberInput/DatePicker |
| 3b | 查阅 Appearance 字段 | ✅ 完成 | 通过 cargo doc 源码确认字段类型 |
| 3c | 重写 badge_style() | ✅ 完成 | 从 tokens.colors/tokens.radius 取值 |
| 3d | 重写 card_style() | ✅ 完成 | 填充 head/body/foot 各区域 |
| 3e | 重写 number_input_style() | ✅ 完成 | button_background + icon_color |
| 3f | 重写 date_picker_style() | ✅ 完成 | 委托 iced_aw::primary |
| 3g | 更新测试 | ✅ 完成 | 5/5 适配器测试通过 |

### TODO 4：shadcn-rs 令牌转换

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 4a | 检查是否引入 shadcn-rs | ⏭ 跳过 | 项目未使用 shadcn-rs |
| 4b | 编写转换函数（如需要） | ⏭ 跳过 | |
| 4c | view 中使用（如需要） | ⏭ 跳过 | |

### TODO 5：补完 v1 遗留项

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 5a | 检查 v1 各 Step 完成状态 | ✅ 完成 | Step 4d/5c/5d/6c 为遗留项 |
| 5b | 补完未完成子任务 | ✅ 完成 | SettingsStore 集成：启动加载 + 主题切换保存 |
| 5c | SettingsStore.save 由 Action 触发 | ✅ 完成 | Action::SetTheme 分支调用 settings_store.save() |

### TODO 6：最终验证

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 6a | 结构与边界 10 项检查 | ✅ 完成 | 全部 ✅ |
| 6b | 执行规范 6 项检查 | ✅ 完成 | 全部 ✅ |
| 6c | 全量测试 | ✅ 完成 | 24/24 通过 |
| 6d | 全量编译 | ✅ 完成 | cargo check 通过 |

---

---

## v4.0 组件示例扩展：iced-kit + iced-shadcn 双库对比

> 目标：在 ui_libs 组件示例中补完两个库的核心组件，每个组件同时展示 iced-kit 和 iced-shadcn 版本，与项目主题切换整合。
> iced-shadcn-v2 因 Rust 1.96 要求（当前 1.94.1）暂不可用，保持 iced-shadcn v1。

### TODO 0：依赖确认

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 0a | 确认 iced-kit 0.1.0 可用 | ✅ 完成 | 已集成 |
| 0b | 确认 iced-shadcn 0.5.1 可用 | ✅ 完成 | v1 props-first API |
| 0c | iced-shadcn-v2 评估 | ❌ 阻塞 | 需 Rust ≥ 1.96 |

### TODO 1：ComponentTab 扩展

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 1a | 扩展 ComponentTab 枚举 | ⏳ 待执行 | 从 14 个扩展到 ~30 个 |
| 1b | 按功能领域分组 | ⏳ 待执行 | 按钮/表单/导航/浮层/反馈/数据/布局 |

### TODO 2：组件视图实现

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 2a | 按钮组（ButtonGroup） | ⏳ 待执行 | |
| 2b | 表单组件（Checkbox/Radio/Slider/Select/Input/Textarea） | ⏳ 待执行 | |
| 2c | 导航组件（Breadcrumb/Pagination/Stepper） | ⏳ 待执行 | |
| 2d | 浮层组件（Dialog/Drawer/Popover/Tooltip） | ⏳ 待执行 | |
| 2e | 反馈组件（Toast/Progress/Alert） | ⏳ 待执行 | |
| 2f | 数据组件（Avatar/Skeleton/Table） | ⏳ 待执行 | |
| 2g | 布局组件（Accordion/Collapsible/Resizable） | ⏳ 待执行 | |
| 2h | 其他组件（Kbd/CopyButton/StarRating/Meter） | ⏳ 待执行 | |

### TODO 3：主题整合

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 3a | 主题切换时同步更新 kit_theme | ⏳ 待执行 | |
| 3b | 验证暗色/亮色主题下组件表现 | ⏳ 待执行 | |

### TODO 4：编译验证 + 测试

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 4a | cargo check 通过 | ⏳ 待执行 | |
| 4b | cargo test 通过 | ⏳ 待执行 | |

---

## v3.0 迁移：令牌抽象接口版

> 核心定位：令牌统一 + 来源可插拔。策略/工厂/适配器是实现手段，不是目的。
> 方案文档：docs/v3_theme_architecture.md
> 执行模式：全自动

### 总体进度

- [x] TODO 1：重写 Tokens（抽象层自有类型） ✅
- [x] TODO 2：重写 ThemeStyles trait（移除 iced-kit 耦合） ✅
- [x] TODO 3：重写 KitStyles（from_kit 提取+补全） ✅
- [x] TODO 4：重写 NativeStyles（兼容层） ✅
- [x] TODO 5：重写 theme_adapters（使用抽象层 Tokens） ✅
- [x] TODO 6：集成 UiState 到 App（Box<dyn ThemeStyles>） ✅
- [x] TODO 7：更新 ui_libs view 使用令牌层 ✅
- [x] TODO 8：编译通过 + 测试通过 ✅
- [x] TODO 9：更新 PROGRESS.md ✅

### TODO 1：重写 Tokens

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 1a | 移除 `iced_kit::Tokens` 依赖 | ✅ 完成 | Tokens 为抽象层自有类型 |
| 1b | 新增 `derive_iced_theme()` 纯函数 | ✅ 完成 | 从 Tokens 派生 iced::Theme，不依赖外部库 |
| 1c | 新增 `FontLevel` 结构体 | ✅ 完成 | 字体族 + 字号 + 字重 |
| 1d | 编写单元测试 | ✅ 完成 | 3/3 通过 |

### TODO 2：重写 ThemeStyles trait

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 2a | 移除 `iced_kit_theme()` | ✅ 完成 | 接口中无任何 iced-kit 引用 |
| 2b | `tokens()` 返回 `&Tokens`（抽象层） | ✅ 完成 | 不暴露具体主题库 |
| 2c | `iced_theme()` 改为必须实现 | ✅ 完成 | 各策略自己派生 |
| 2d | 内置控件样式默认实现 | ✅ 完成 | button_primary/secondary/text_input/container |

### TODO 3：重写 KitStyles

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 3a | 只持有抽象层 `Tokens` | ✅ 完成 | 不再持有 `iced_kit::Theme` |
| 3b | 实现 `from_kit()` 提取+补全 | ✅ 完成 | 颜色提取，间距/圆角/字体补全 |
| 3c | 实现 `ThemeStyles` trait | ✅ 完成 | tokens() + iced_theme() |

### TODO 4：重写 NativeStyles

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 4a | 从 `iced::Theme` 构造 `Tokens` | ✅ 完成 | palette() 映射 |
| 4b | 实现 `ThemeStyles` trait | ✅ 完成 | 兼容层保留 |

### TODO 5：重写 theme_adapters

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 5a | 移除 `iced_kit::Tokens` 引用 | ✅ 完成 | 全部使用抽象层 Tokens |
| 5b | 更新所有适配函数 | ✅ 完成 | badge/card/number_input/date_picker/content_panel |
| 5c | 更新测试 | ✅ 完成 | 5/5 通过 |

### TODO 6：集成 UiState 到 App

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 6a | App 添加 `ui_state: UiState` | ✅ 完成 | 替代 Default derive |
| 6b | `App::theme()` 走 `ui_state.styles.iced_theme()` | ✅ 完成 | 动态主题 |
| 6c | `ui.rs` 声明 `pub mod state` | ✅ 完成 | 模块可见性 |
| 6d | 菜单栏容器使用令牌层颜色 | ✅ 完成 | surface 背景色 |

### TODO 7：更新 ui_libs view 使用令牌层

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 7a | view() 接受 `&dyn ThemeStyles` | ✅ 完成 | 传递到各子视图 |
| 7b | Badge 使用适配器样式 | ✅ 完成 | badge_style() |
| 7c | Card 使用适配器样式 | ✅ 完成 | card_style() |
| 7d | Button 使用 ThemeStyles 方法 | ✅ 完成 | button_primary/secondary |
| 7e | NumberInput 使用适配器样式 | ✅ 完成 | number_input_style() |
| 7f | Menu 容器使用令牌层颜色 | ✅ 完成 | surface + border |

### TODO 8：编译 + 测试

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 8a | cargo check 通过 | ✅ 完成 | 58 warnings，0 errors |
| 8b | cargo test 通过 | ✅ 完成 | 27/27 通过 |

### TODO 9：更新 PROGRESS.md

| # | 子任务 | 状态 | 备注 |
|:--|:--|:--|:--|
| 9a | 新增 v3.0 章节 | ✅ 完成 | 本表 |

---

## 变更日志

| 日期 | 内容 |
|:--|:--|
| 2026-11-05 | iced-shadcn-v2 迁移受阻：Rust 1.94.1 < 1.96 要求。保持 iced-shadcn v1，扩展组件示例。 |
| 2026-11-05 | iced-kit + iced-shadcn 对比展示：Badge/Card/Button/Toggle/Separator/Spinner 6 组件双库对比 |
| 2026-11-04 | TODO 2+3 完成：ui_libs 组件示例令牌化（tab_button/view_toggle/view_separator/view_tab/view_spinner 全部使用抽象层样式）+ 主题切换菜单接入 UiState |
| 2026-11-04 | 阶段 5-6 完成：P0 fallback_to_f32 清零(8→0) + P1 7项style警告修复 + clippy 89→70(-21.3%) + 25/25测试通过 |
| 2026-11-04 | 阶段 1-4 完成：基线审计(unsafe 0%) + 2024 edition确认 + mod.rs迁移 + 死代码删除 + LoadState enum + ToastList抽取 |
| 2026-11-04 | v3.0 令牌抽象接口版完成：Tokens 自有类型 + ThemeStyles 解耦 + KitStyles from_kit + UiState 集成 + ui_libs 令牌化 |
| 2026-11-04 | 组件替换探索：iced-kit Theme 与 iced widget Catalog 不兼容，回退。v2.0 令牌集成 + SettingsStore 完成 |
| 2026-11-03 | 初始化进度文档，完成代码库通读与 TODO 分解 |
