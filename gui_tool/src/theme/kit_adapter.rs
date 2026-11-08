//! iced-kit 组件适配器
//!
//! 将 iced-kit widget（硬编码 `iced_kit::Theme`）包装为
//! 兼容 `iced::Theme` 的 `Element`，使 iced-kit 组件可嵌入
//! 以 `iced::Theme` 为应用主题的视图。
//!
//! # 原理
//!
//! `Widget` trait 的 `theme` 参数仅用于样式解析。
//! `KitAdapter` 忽略传入的 `&iced::Theme`，改用持有的
//! `iced_kit::Theme` 委托给内部 iced-kit widget。

use iced::advanced::Widget;
use iced::advanced::layout::{self, Layout};
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Operation;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::Shell;
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Vector};

struct KitAdapter<'a, Message: 'a> {
    kit_element: Element<'a, Message, iced_kit::Theme, Renderer>,
    kit_theme: iced_kit::Theme,
}

impl<'a, Message: 'a> Widget<Message, iced::Theme, Renderer>
    for KitAdapter<'a, Message>
{
    fn size(&self) -> Size<Length> {
        self.kit_element.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.kit_element.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.kit_element.as_widget_mut().layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.kit_element.as_widget().draw(
            tree, renderer, &self.kit_theme, style, layout, cursor, viewport,
        );
    }

    fn tag(&self) -> tree::Tag {
        self.kit_element.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.kit_element.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.kit_element.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.kit_element.as_widget().diff(tree);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.kit_element.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, Renderer>> {
        None
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.kit_element.as_widget().mouse_interaction(
            tree, layout, cursor, viewport, renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.kit_element.as_widget_mut().operate(tree, layout, renderer, operation);
    }
}

/// 将 iced-kit `Element` 包装为兼容 `iced::Theme` 的 `Element`。
pub fn kit_element<'a, Message: 'a>(
    element: Element<'a, Message, iced_kit::Theme, Renderer>,
    kit_theme: &iced_kit::Theme,
) -> Element<'a, Message, iced::Theme, Renderer> {
    Element::new(KitAdapter {
        kit_element: element,
        kit_theme: kit_theme.clone(),
    })
}
