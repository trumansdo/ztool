# 主题层 v3.0 — 令牌抽象接口版

> **核心定位**：令牌统一 + 来源可插拔。策略/工厂/适配器是实现手段，不是目的。

## 1. 一句话目的

在 `iced::Theme` 和各设计系统之间，放一个统一的令牌层。所有设计参数——颜色、字体、间距、圆角——都收敛到这一层。各库有就提取，没有就按该库的设计语言补全。上层代码只认这个令牌层，不认具体是 iced-kit、shadcn 还是 fluent。

## 2. 接口的四重角色

不是刻意套四个模式，而是这四件事天然需要四个不同的东西：

| 角色 | 承担什么 | 项目中的实现 |
|:---|:---|:---|
| **数据载体** | 统一令牌模型 | `Tokens`（颜色/字体/间距/圆角/边框） |
| **提供者接口** | "给定一个设计系统，产出 Tokens 和 iced::Theme" | `ThemeStyles`（`tokens()` + `iced_theme()`） |
| **工厂** | 按 `ThemeId` 创建具体提供者 | `ThemeSource` |
| **适配器** | 把 Tokens 翻译成外部库需要的类型 | `iced_theme()`（→ `iced::Theme`）+ `theme_adapters`（→ `iced_aw::Appearance` 等） |

策略模式只是"提供者接口"这一格的表现——`ThemeStyles` 有多个实现，上层不关心是哪一个。但整个接口的价值，在"数据载体"和"适配器"这两格：**令牌统一了，适配器才能只写一次；来源可插拔了，换设计系统才不用动上层。**

## 3. 核心验证判据

> **新增一个设计系统（比如 fluent），只写一个 `XxxStyles` + 一个 `XxxThemeSource`，`Tokens` 字段够用就不用改，`theme_adapters` 一行不改，`App` 一行不改。**

如果满足，说明令牌层真的把"各库缺什么"隔离住了；如果不满足（比如 fluent 有 iced-kit 没有的令牌维度，`Tokens` 装不下），那就扩展 `Tokens` 字段——**扩展的是数据契约，不是上层逻辑**。这才是"隔离各库令牌要素缺失"的正确含义：缺失在 `from_xxx` 里补，不在上层补。

## 4. 架构图

```
┌─────────────────────────────────────────────────────────┐
│  App / Features（上层代码）                               │
│  只认: &dyn ThemeStyles                                  │
│  调用: styles.tokens() → &Tokens                         │
│        styles.iced_theme() → iced::Theme                 │
│        adapters::badge_style(styles) → iced_aw Style     │
└────────────┬────────────────────────────────────────────┘
             │ Box<dyn ThemeStyles>
             │
     ┌───────┴────────┐
     │  ThemeStyles   │  ← 提供者接口（策略）
     │  tokens()      │
     │  iced_theme()  │
     └───┬────────┬───┘
         │        │
    ┌────▼───┐ ┌──▼──────────┐
    │KitStyles│ │NativeStyles │  ← 具体提供者
    │from_kit │ │from_native  │
    └────┬───┘ └──┬──────────┘
         │        │
    ┌────▼───┐ ┌──▼──────────┐
    │iced-kit│ │iced::Theme  │  ← 外部设计系统
    └────────┘ └─────────────┘
```

## 5. Tokens — 唯一数据契约

```rust
pub struct Tokens {
    // 颜色
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub border: Color,

    // 间距
    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,

    // 圆角
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
    pub radius_full: f32,

    // 字体
    pub font_body: FontLevel,
    pub font_title: FontLevel,
    pub font_caption: FontLevel,
    pub font_mono: FontLevel,

    // 边框
    pub border_width: f32,
}
```

## 6. ThemeStyles — 提供者接口

```rust
pub trait ThemeStyles: Debug + Send + Sync {
    /// 设计令牌访问（必须实现）
    fn tokens(&self) -> &Tokens;

    /// 从自己的 Tokens 派生 iced::Theme（必须实现）
    fn iced_theme(&self) -> iced::Theme;

    // 内置控件样式（默认实现，策略可按需覆盖）
    fn button_primary(&self) -> iced::theme::Button { ... }
    fn button_secondary(&self) -> iced::theme::Button { ... }
    fn text_input(&self) -> iced::theme::TextInput { ... }
    fn container(&self) -> iced::theme::Container { ... }
}
```

## 7. ThemeSource — 工厂接口

```rust
pub trait ThemeSource {
    fn load(&self, id: &ThemeId) -> Result<Box<dyn ThemeStyles>, String>;
    fn system(&self) -> Result<Box<dyn ThemeStyles>, String>;
    fn available(&self) -> Vec<ThemeId>;
}
```

## 8. KitStyles::from_kit — 提取 + 补全

```rust
impl KitStyles {
    pub fn from_kit(kit: &iced_kit::Theme) -> Self {
        let kt = kit.tokens();
        // 提取：iced-kit 有的字段直接映射
        // 补全：iced-kit 没有的字段按其设计节奏推导
        //   - surface_raised ← colors.secondary
        //   - spacing_* ← kt.spacing.* (u16 → f32)
        //   - radius_* ← kt.radius.* (u16 → f32)
        //   - font_* ← kt.typography.*
        //   - border_width ← 1.0 (iced-kit 无此概念)
    }
}
```

## 9. derive_iced_theme — 共享派生函数

```rust
/// 从 Tokens 派生 iced::Theme（纯函数，不依赖任何外部库）
pub fn derive_iced_theme(tokens: &Tokens) -> iced::Theme {
    iced::Theme::custom(
        "ztool".into(),
        iced::theme::Palette {
            background: tokens.background,
            text: tokens.text_primary,
            primary: tokens.accent,
            success: tokens.success,
            danger: tokens.danger,
            warning: tokens.warning,
        },
    )
}
```

## 10. theme_adapters — 适配器层

每个适配函数签名统一：`(&dyn ThemeStyles, ...) → ComponentStyle`。

所有视觉参数从 `styles.tokens()` 派生，不依赖第三方库的默认主题。

```rust
pub fn badge_style(styles: &dyn ThemeStyles) -> iced_aw::style::badge::Style { ... }
pub fn card_style(styles: &dyn ThemeStyles) -> iced_aw::style::card::Style { ... }
pub fn number_input_style(styles: &dyn ThemeStyles) -> iced_aw::style::number_input::Style { ... }
pub fn date_picker_style(styles: &dyn ThemeStyles, status: Status) -> iced_aw::style::date_picker::Style { ... }
pub fn content_panel_style(styles: &dyn ThemeStyles) -> iced::widget::container::Style { ... }
```

## 11. 迁移步骤

| # | 步骤 | 状态 |
|:--|:--|:--|
| 1 | 重写 `Tokens`（抽象层自有类型，不依赖 iced-kit） | ✅ |
| 2 | 重写 `ThemeStyles` trait（移除 iced-kit 耦合） | ✅ |
| 3 | 重写 `KitStyles`（`from_kit` 提取+补全） | ✅ |
| 4 | 重写 `NativeStyles`（兼容层） | ✅ |
| 5 | 重写 `theme_adapters`（使用抽象层 Tokens） | ✅ |
| 6 | 集成 `UiState` 到 `App`（`Box<dyn ThemeStyles>`） | ⬜ |
| 7 | 更新所有 feature view 使用 `styles.tokens()` | ⬜ |
| 8 | 编译通过 + 测试通过 | ⬜ |
| 9 | 更新 PROGRESS.md | ⬜ |

## 12. 验证要点

- [ ] 新增设计系统（如 fluent）只需写 `XxxStyles` + `XxxThemeSource`
- [ ] `Tokens` 字段够用则不改
- [ ] `theme_adapters` 一行不改
- [ ] `App` 一行不改
- [ ] 所有 feature 模块不直接引用 `iced_kit::Theme`
- [ ] 所有 feature 模块不直接引用 `iced_kit::Tokens`
