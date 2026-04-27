//! # UI 组件库展示视图
//!
//! 展示 iced 和 iced_aw 库中各组件的用法，是项目中最大的视图文件。
//!
//! ## Rust 概念 — 函数分解
//! 主 view 函数按 sub-tab 分发到各 `view_*` 私有函数。
//! 每个 `view_*` 不接收 `&UiLibs` 也能独立测试和修改。
//! 这种「分而治之」的模块设计是 Rust 工程化的常见模式。
//!
//! ## Rust 概念 — `Element<'static, Msg>`
//! `'static` 生命周期表示元素不包含借用引用（或引用永久有效）。
//! 大部分简单 UI 元素不需要借用运行时数据，可以直接 'static。

use crate::features::theme;
use crate::features::ui_libs::update::ComponentTab;
use crate::ui::widgets::{toast, Layered};
use iced::widget::{button, column, container, row, text, toggler, scrollable};
use iced::Element;

use super::Msg;
use super::UiLibs;
use toast::{ToastLevel, ToastPosition, view_toasts};

/// 主视图：渲染 sub-tab 切换栏 + 对应的组件展示内容
///
/// ## Rust 概念 — 返回值中的 Vec<Layered>
/// 除了主内容 Element，还返回 Toasts 作为叠加层列表。
/// 当前 Toast 展示页面的 Toasts 叠加在整个组件示例页面上。
///
/// # 参数
/// - `libs`: UI 组件库状态（包含当前选中的 tab、点击计数等）
///
/// # 返回值
/// (主要内容, Toast 叠加层列表)
pub fn view(libs: &UiLibs) -> (Element<'_, Msg>, Vec<Layered<'_, Msg>>) {
    // Sub-tab 切换栏 — 13 个横向排列的按钮
    // `tab_button()` 是下方定义的私有函数，接受标签名、tab 枚举和当前选中项
    let tabs = row![
        tab_button("Badge", ComponentTab::Badge, libs.selected_tab),
        tab_button("Card", ComponentTab::Card, libs.selected_tab),
        tab_button("Button", ComponentTab::Button, libs.selected_tab),
        tab_button("Toggle", ComponentTab::Toggle, libs.selected_tab),
        tab_button("Separator", ComponentTab::Separator, libs.selected_tab),
        tab_button("Tab", ComponentTab::Tab, libs.selected_tab),
        tab_button("NumberInput", ComponentTab::NumberInput, libs.selected_tab),
        tab_button("Spinner", ComponentTab::Spinner, libs.selected_tab),
        tab_button("Wrap", ComponentTab::Wrap, libs.selected_tab),
        tab_button("Split", ComponentTab::Split, libs.selected_tab),
        tab_button("Toast", ComponentTab::Toast, libs.selected_tab),
        tab_button("Color", ComponentTab::ColorPicker, libs.selected_tab),
        tab_button("Date", ComponentTab::DatePicker, libs.selected_tab),
    ]
    .spacing(theme::size(0.3).0 as u32);

    // match 分发：根据选中的 tab 调用对应的 view_* 私有函数
    // Rust 的 match 是表达式，返回值可以绑定到变量
    let content = match libs.selected_tab {
        ComponentTab::Badge => view_badge(),
        ComponentTab::Card => view_card(libs),
        ComponentTab::Button => view_button(libs),
        ComponentTab::Toggle => view_toggle(libs),
        ComponentTab::Separator => view_separator(),
        ComponentTab::Tab => view_tab(libs),
        ComponentTab::NumberInput => view_number_input(libs),
        ComponentTab::Spinner => view_spinner(),
        ComponentTab::Wrap => view_wrap(),
        ComponentTab::Split => view_split(),
        ComponentTab::Toast => view_toast(),
        ComponentTab::ColorPicker => view_color_picker(libs),
        ComponentTab::DatePicker => view_date_picker(),
    };

    // 外层容器：标题 + 横向滚动 tab 栏 + 内容区
    let content = container(
        column![
            text("iced UI 组件示例").size(theme::font(1.2)),
            text("").size(theme::font(0.5)),
            // scrollable + horizontal — 允许横向滚动（tab 太多时）
            scrollable(row![tabs]).horizontal(),
            text("").size(theme::font(0.5)),
            content,
        ]
        .spacing(theme::size(0.5).0 as u32)
        .padding(theme::padding(1.0)),
    )
    .padding(theme::padding(1.0))
    .into();

    // view_toasts 将 UiLibs 中的 toasts 向量转换为 Layered 叠加层列表
    let overlays = view_toasts(&libs.toasts, Msg::AutoDismiss);
    (content, overlays)
}

/// 渲染 sub-tab 切换按钮
///
/// ## Rust 概念 — `'static` 生命周期
/// `&'static str` — 字符串字面量「编译时就确定了」，存储在程序的只读数据段，
/// 生命周期为 'static（整个程序运行期间有效）。
///
/// ## Rust 概念 — 比较枚举
/// `selected == tab` — 因为 ComponentTab 实现了 PartialEq 和 Copy，
/// 可以直接用 == 比较。is_selected 用于切换按钮样式。
///
/// ## Rust 概念 — 闭包 vs 函数指针
/// `.on_press(Msg::TabSelected(tab))` — TabSelected 是一个枚举变体构造器，
/// 它本身就是一个函数（Fn(ComponentTab) -> Msg），可以直接传给 on_press。
fn tab_button(label: &'static str, tab: ComponentTab, selected: ComponentTab) -> Element<'static, Msg> {
    let is_selected = selected == tab;
    button(label)
        .on_press(Msg::TabSelected(tab))
        .padding(theme::padding2(0.25, 0.4))
        // button::primary — 蓝色高亮（选中态）
        // button::secondary — 灰色（未选中态）
        .style(if is_selected {
            button::primary
        } else {
            button::secondary
        })
        .into()
}

/// Badge 徽章组件展示
///
/// ## Rust 概念 — 第三方 crate 导入
/// `use iced_aw::widget::Badge;` — iced_aw (iced Additional Widgets) 提供了
/// 原生 iced 不包含的扩展组件。Badge 是彩色标签，常用来显示计数或状态。
fn view_badge() -> Element<'static, Msg> {
    use iced_aw::widget::Badge;

    let badges = row![
        Badge::new("Primary"),
        Badge::new("Success"),
        Badge::new("Danger"),
        Badge::new("Warning"),
    ]
    .spacing(10);

    column![
        text("Badge 徽章组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        badges,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Card 卡片组件展示
///
/// ## Rust 概念 — Builder 模式
/// `Card::new(...).foot(...)` — 链式调用返回 Self，逐步构建组件。
/// iced 大量使用 Builder 模式，避免构造函数参数过多。
///
/// ## Rust 概念 — format! 中的字段访问
/// `format!("...点击次数: {}", libs.click_count)` — click_count 是 u32，
/// format! 自动调用其 Display trait 实现来转换为字符串。
fn view_card(libs: &UiLibs) -> Element<'static, Msg> {
    use iced_aw::widget::Card;

    let card = Card::new(
        text("卡片标题").size(18),
        text(format!("这是卡片内容，点击次数: {}", libs.click_count)).size(14),
    )
    .foot(  // 卡片底部区域
        button("点击我")
            .on_press(Msg::ButtonPressed)
            .padding(theme::padding2(0.3, 0.8)),
    );

    column![
        text("Card 卡片组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        card,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Button 按钮组件展示
///
/// ## Rust 概念 — 多行 row! 宏
/// row! 可以包含多行表达式，每行以逗号结尾。
/// 如下面的 row! 包含 3 个不同类型样式的按钮。
fn view_button(libs: &UiLibs) -> Element<'static, Msg> {
    let buttons = column![
        text("普通按钮").size(theme::font(0.9)),
        row![
            button("Primary").style(button::primary),
            button("Secondary").style(button::secondary),
            button("Danger").style(button::danger),
        ]
        .spacing(10),
        text("").size(theme::font(0.3)),
        text(format!("点击次数: {}", libs.click_count)).size(theme::font(0.9)),
    ]
    .spacing(theme::size(0.3).0 as u32);

    column![
        text("Button 按钮组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        buttons,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Toggle 开关组件展示
///
/// ## Rust 概念 — 响应式 UI
/// `toggler(libs.toggle_value)` — 传入当前状态值。
/// `.on_toggle(Msg::ToggleChanged)` — 状态变化时的回调。
/// iced 的 Elm Architecture 中，UI 永远是「状态的函数」。
fn view_toggle(libs: &UiLibs) -> Element<'static, Msg> {
    let toggle = toggler(libs.toggle_value)
        .on_toggle(Msg::ToggleChanged);

    // if/else 表达式用于根据布尔值选择不同的文字
    let status = if libs.toggle_value { "开启" } else { "关闭" };

    column![
        text("Toggle 开关组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        toggle,
        text("").size(theme::font(0.3)),
        text(format!("状态: {}", status)).size(theme::font(0.9)),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Separator 分隔线组件展示
///
/// ## Rust 概念 — `use iced::widget::rule`
/// rule 是 iced 内置的横线/竖线组件，作为视觉分隔符。
/// `rule::horizontal(30)` — 宽度 30px 的水平线。
fn view_separator() -> Element<'static, Msg> {
    use iced::widget::rule;

    column![
        text("Separator 分隔线组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("上面的内容"),
        rule::horizontal(30),
        text("下面的内容"),
        rule::horizontal(30),
        text("再下面的内容"),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Tab 标签页组件展示（模拟标签页交互）
fn view_tab(libs: &UiLibs) -> Element<'static, Msg> {
    let tabs = row![
        button("标签1")
            .on_press(Msg::TabSelected(ComponentTab::Tab))
            .style(button::primary),
        button("标签2")
            .on_press(Msg::TabSelected(ComponentTab::Tab)),
        button("标签3")
            .on_press(Msg::TabSelected(ComponentTab::Tab)),
    ]
    .spacing(5);

    let tab_content = column![
        text("Tab 标签页组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text(format!("当前点击次数: {}", libs.click_count)),
        text("这是标签页的内容区域"),
    ]
    .spacing(theme::size(0.3).0 as u32);

    column![
        text("Tab 标签页组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        tabs,
        text("").size(theme::font(0.3)),
        container(tab_content)
            .padding(10),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// NumberInput 数字输入组件展示
///
/// ## Rust 概念 — 类型转换 `as f64`
/// `libs.number_value as f64` — i32 转 f64（64 位浮点数）。
/// `as` 是 Rust 的类型转换关键字（不同于 C 的 `(type)value`）。
/// NumberInput 需要 f64 类型，而内部状态用 i32 更合适。
///
/// ## Rust 概念 — 闭包 `|value| Msg::NumberChanged(value as i32)`
/// 闭包（匿名函数）接收 NumberInput 给出的 f64，转回 i32 发送消息。
/// 竖线 | | 中声明参数，花括号中写函数体。
///
/// ## Rust 概念 — 范围语法 `-100.0..=100.0`
/// `..=` 表示包含上界的范围（inclusive range），`..` 不含上界。
/// 这里限制数字输入范围为 -100 到 100。
fn view_number_input(libs: &UiLibs) -> Element<'static, Msg> {
    use iced_aw::widget::NumberInput;

    let float_value = libs.number_value as f64;

    column![
        text("NumberInput 数字输入组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        row![
            NumberInput::new(&float_value, -100.0..=100.0, |value| Msg::NumberChanged(value as i32)),
            text(format!("当前值: {}", libs.number_value)).size(theme::font(0.9)),
        ]
        .spacing(10),
        text("").size(theme::font(0.3)),
        row![
            button("+1").on_press(Msg::NumberChanged(libs.number_value + 1)),
            button("-1").on_press(Msg::NumberChanged(libs.number_value - 1)),
            button("重置").on_press(Msg::NumberChanged(0)),
        ]
        .spacing(10),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Spinner 加载动画组件展示
fn view_spinner() -> Element<'static, Msg> {
    use iced_aw::widget::Spinner;

    column![
        text("Spinner 加载动画组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        Spinner::new(),  // 默认旋转动画，无需任何配置
        text("").size(theme::font(0.3)),
        text("加载中...").size(theme::font(0.9)),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Wrap 自动换行组件展示
///
/// ## Rust 概念 — `.into()` 类型转换
/// `button("按钮 1").into()` — 将 button 转换为 Element。
/// Wrap::with_elements 需要 `Vec<Element>`，所以每个按钮用 .into() 转换。
/// `.into()` 调用 From trait 的实现，编译器自动推断目标类型。
fn view_wrap() -> Element<'static, Msg> {
    use iced_aw::widget::Wrap;

    let items = Wrap::with_elements(vec![
        button("按钮 1").into(),
        button("按钮 2").into(),
        button("按钮 3").into(),
        button("按钮 4").into(),
        button("按钮 5").into(),
        button("按钮 6").into(),
        button("按钮 7").into(),
        button("按钮 8").into(),
    ]);

    column![
        text("Wrap 自动换行组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("下面的按钮会自动换行:"),
        text("").size(theme::font(0.3)),
        items,
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Split 分屏组件展示
///
/// ## Rust 概念 — 第三方 crate `iced_split`
/// 使用 `iced_split::Split` 组件，可以拖拽分割线调整左右面板比例。
/// `Split::new(left, right, 0.3)` — 第三个参数 0.3 表示左侧初始占比 30%。
fn view_split() -> Element<'static, Msg> {
    use iced_split::Split;

    let left_content = column![
        text("左侧面板").size(theme::font(0.9)),
        button("左侧按钮 1"),
        button("左侧按钮 2"),
    ]
    .spacing(5);

    let right_content = column![
        text("右侧面板").size(theme::font(0.9)),
        button("右侧按钮 1"),
        button("右侧按钮 2"),
    ]
    .spacing(5);

    let split = Split::new(
        left_content,
        right_content,
        0.3,   // 左侧初始占比 30%
    );

    column![
        text("Split 分屏组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("拖动分割线可调整左右比例:"),
        text("").size(theme::font(0.3)),
        container(split)
            .height(iced::Length::Fixed(200.0))
            .width(iced::Length::Fill),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// Toast 提示消息组件展示
///
/// ## Rust 概念 — 数组 vs 向量
/// `let positions = [(ToastPosition::TopLeft, "左上"), ...];`
/// 方括号 `[...]` 创建的是**固定大小数组**（编译期确定长度），
/// 而 `vec![...]` 创建的是**动态向量**（堆分配，运行时可变）。
/// 这里 9 个元素固定不变，用数组更高效（栈分配）。
///
/// ## Rust 概念 — 嵌套 for 循环
/// 两个 for 循环的乘积：9 个位置 × 4 个级别 = 36 个按钮。
/// `for (pos, pos_name) in &positions` — 遍历数组引用，`&` 避免消耗所有权。
///
/// ## Rust 概念 — button vs text 元素构造
/// `button(text(format!("{} {}", level_name, pos_name)).size(12))` —
/// button() 内嵌 text()，text() 内嵌 format!。这是 iced 的组件组合模式。
fn view_toast() -> Element<'static, Msg> {
    // 9 个位置 × 4 个级别 = 共 36 种组合
    let positions = [
        (ToastPosition::TopLeft, "左上"),
        (ToastPosition::TopCenter, "中上"),
        (ToastPosition::TopRight, "右上"),
        (ToastPosition::CenterLeft, "左中"),
        (ToastPosition::Center, "居中"),
        (ToastPosition::CenterRight, "右中"),
        (ToastPosition::BottomLeft, "左下"),
        (ToastPosition::BottomCenter, "中下"),
        (ToastPosition::BottomRight, "右下"),
    ];

    // Toast 的四种级别（对应四种颜色）
    let levels = [
        (ToastLevel::Info, "Info"),
        (ToastLevel::Success, "Success"),
        (ToastLevel::Warning, "Warning"),
        (ToastLevel::Error, "Error"),
    ];

    let mut rows = Vec::new();
    for (pos, pos_name) in &positions {
        let mut btns = Vec::new();
        for (level, level_name) in &levels {
            btns.push(
                button(text(format!("{} {}", level_name, pos_name)).size(12))
                    .on_press(Msg::ToastShow(*level, format!("{}提示 - {}", level_name, pos_name), *pos))
                    .padding([4, 8])
                    .style(button::secondary)
                    .into()
            );
        }
        // row(btns) — 非宏版本的 row 函数，接受 Vec<Element>
        rows.push(row(btns).spacing(4).into());
    }

    // column(rows) — 非宏版本的 column 函数
    column![
        text("Toast 提示消息组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("点击按钮显示 Toast，支持9个位置").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        column(rows).spacing(4),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// ColorPicker 颜色选择器组件展示
fn view_color_picker(libs: &UiLibs) -> Element<'static, Msg> {
    let color_display = container(
        text("颜色预览")
    )
    .width(iced::Length::Fixed(80.0))
    .height(iced::Length::Fixed(50.0));

    column![
        text("ColorPicker 颜色选择器组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        row![
            button("选择颜色")
                .on_press(Msg::TabSelected(ComponentTab::ColorPicker)),
            color_display,
            column![
                text("当前颜色:").size(theme::font(0.9)),
                // Debug 格式打印颜色值
                text(format!("RGB: {:?}", libs.selected_color)),
            ]
            .spacing(5),
        ]
        .spacing(20),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}

/// DatePicker 日期选择器组件展示
///
/// ## Rust 概念 — `use iced_aw::date_picker::Date`
/// iced_aw 的 Date 类型用于表示日期，Date::today() 获取当天日期。
/// 日期格式化：`{}/{}/{}` 年/月/日。
fn view_date_picker() -> Element<'static, Msg> {
    use iced_aw::date_picker::Date;

    let today = Date::today();
    let date_str = format!("{}/{}/{}", today.year, today.month, today.day);

    column![
        text("DatePicker 日期选择器组件").size(theme::font(1.0)),
        text("").size(theme::font(0.3)),
        text("点击下方按钮可打开日期选择器").size(theme::font(0.8)),
        text("").size(theme::font(0.3)),
        row![
            button("选择日期")
                .on_press(Msg::TabSelected(ComponentTab::DatePicker)),
            text(format!("当前日期: {}", date_str)),
        ]
        .spacing(10),
        text("").size(theme::font(0.3)),
        // 功能尚未完全实现的状态说明
        text("(需要状态管理，实际使用需要配合 show_picker 和 date 状态)")
            .size(theme::font(0.7)),
    ]
    .spacing(theme::size(0.3).0 as u32)
    .into()
}
