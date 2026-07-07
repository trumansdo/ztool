//! Toast 消息通知组件
//!
//! 在屏幕上显示临时浮动通知，支持 4 种级别 × 2 个位置（右下/右上）的组合。
//! 基于 iced 原生 Widget + Overlay 实现，定时器融入 iced 更新循环，不依赖外部 tokio。

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Operation, Tree};
use iced::advanced::{Clipboard, Shell, Widget};
use iced::mouse;
use iced::time::{self, Duration, Instant};
use iced::widget::{button, container, row, text};
use iced::{
    Alignment, Background, Border, Color, Element, Event, Length, Point, Rectangle, Renderer, Size,
    Theme, Vector,
};
use iced::window;

pub const DEFAULT_TIMEOUT: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    fn border_color(&self) -> Color {
        match self {
            ToastLevel::Info => Color::from_rgb(0.35, 0.55, 0.85),
            ToastLevel::Success => Color::from_rgb(0.25, 0.70, 0.35),
            ToastLevel::Warning => Color::from_rgb(0.80, 0.65, 0.15),
            ToastLevel::Error => Color::from_rgb(0.80, 0.25, 0.25),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    #[default]
    BottomRight,
    TopRight,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub level: ToastLevel,
    pub text: String,
    pub position: ToastPosition,
}

impl Default for Toast {
    fn default() -> Self {
        Self {
            level: ToastLevel::default(),
            text: String::new(),
            position: ToastPosition::default(),
        }
    }
}

pub struct Manager<'a, Message> {
    content: Element<'a, Message>,
    toasts: Vec<Element<'a, Message>>,
    positions: Vec<ToastPosition>,
    timeout_secs: u64,
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Manager<'a, Message>
where
    Message: 'a + Clone,
{
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        let positions: Vec<ToastPosition> = toasts.iter().map(|t| t.position).collect();

        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                let border_color = toast.level.border_color();

                let close_btn: Element<'a, Message> = button(text("×").size(9))
                    .on_press((on_close)(index))
                    .padding([0, 3])
                    .style(|_: &Theme, status: button::Status| {
                        let bg = match status {
                            button::Status::Hovered => Color::from_rgb(0.3, 0.3, 0.35),
                            button::Status::Pressed => Color::from_rgb(0.4, 0.4, 0.45),
                            _ => Color::TRANSPARENT,
                        };
                        button::Style {
                            background: Some(Background::Color(bg)),
                            border: Border {
                                color: Color::TRANSPARENT,
                                width: 0.0,
                                radius: 3.0.into(),
                            },
                            text_color: Color::from_rgb(0.55, 0.55, 0.6),
                            ..Default::default()
                        }
                    })
                    .into();

                container(
                    row![
                        text(toast.text.as_str()).size(12).width(Length::Fill),
                        close_btn,
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .padding([6, 8])
                .width(Length::Fixed(220.0))
                .style(move |_: &Theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        Self {
            content: content.into(),
            toasts,
            positions,
            timeout_secs: DEFAULT_TIMEOUT,
            on_close: Box::new(on_close),
        }
    }

    pub fn timeout(self, seconds: u64) -> Self {
        Self {
            timeout_secs: seconds,
            ..self
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Manager<'_, Message>
where
    Message: Clone,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn tag(&self) -> widget::tree::Tag {
        struct Marker;
        widget::tree::Tag::of::<Marker>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.toasts.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            (old, new) if old > new => {
                instants.truncate(new);
            }
            (old, new) if old < new => {
                instants.extend(std::iter::repeat_n(Some(Instant::now()), new - old));
            }
            _ => {}
        }

        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toasts.iter())
                .collect::<Vec<_>>(),
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            _clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

        let (content_state, toasts_state) = tree.children.split_at_mut(1);

        let content = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let toasts = (!self.toasts.is_empty()).then(|| {
            overlay::Element::new(Box::new(Overlay {
                position: layout.bounds().position() + translation,
                viewport: *viewport,
                toasts: &mut self.toasts,
                positions: &self.positions,
                trees: toasts_state,
                instants,
                on_close: &self.on_close,
                timeout_secs: self.timeout_secs,
            }))
        });

        let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

struct Overlay<'a, 'b, Message> {
    position: Point,
    viewport: Rectangle,
    toasts: &'b mut [Element<'a, Message>],
    positions: &'b [ToastPosition],
    trees: &'b mut [Tree],
    instants: &'b mut [Option<Instant>],
    on_close: &'b dyn Fn(usize) -> Message,
    timeout_secs: u64,
}

impl<Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'_, '_, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        let mut top_nodes = Vec::new();
        let mut bottom_nodes = Vec::new();

        for (i, toast) in self.toasts.iter_mut().enumerate() {
            let node = toast.as_widget_mut().layout(&mut self.trees[i], renderer, &limits);
            let size = node.bounds().size();

            let x = self.viewport.x + self.viewport.width - size.width - 12.0;
            let y_offset = match self.positions.get(i).copied().unwrap_or_default() {
                ToastPosition::TopRight => {
                    let prev_height: f32 = top_nodes.iter().map(|n: &layout::Node| n.bounds().height + 8.0).sum::<f32>();
                    self.position.y + 8.0 + prev_height
                }
                ToastPosition::BottomRight => {
                    let prev_height: f32 = bottom_nodes.iter().map(|n: &layout::Node| n.bounds().height + 8.0).sum::<f32>();
                    self.position.y + bounds.height - size.height - 8.0 - prev_height
                }
            };

            let positioned = node.move_to(Point::new(x, y_offset));
            match self.positions.get(i).copied().unwrap_or_default() {
                ToastPosition::TopRight => top_nodes.push(positioned),
                ToastPosition::BottomRight => bottom_nodes.push(positioned),
            }
        }

        let mut children = top_nodes;
        children.extend(bottom_nodes);
        layout::Node::with_children(bounds, children)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            self.instants
                .iter_mut()
                .enumerate()
                .for_each(|(index, maybe_instant)| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        let remaining =
                            time::seconds(self.timeout_secs).saturating_sub(instant.elapsed());

                        if remaining == Duration::ZERO {
                            maybe_instant.take();
                            shell.publish((self.on_close)(index));
                        } else {
                            shell.request_redraw_at(*now + remaining);
                        }
                    }
                });
        }

        let viewport = layout.bounds();

        for (((child, state), child_layout), instant) in self
            .toasts
            .iter_mut()
            .zip(self.trees.iter_mut())
            .zip(layout.children())
            .zip(self.instants.iter_mut())
        {
            let mut local_messages = vec![];
            let mut local_shell = Shell::new(&mut local_messages);
            let mut local_clipboard = iced::advanced::clipboard::Null;

            child.as_widget_mut().update(
                state,
                event,
                child_layout,
                cursor,
                renderer,
                &mut local_clipboard,
                &mut local_shell,
                &viewport,
            );

            if !local_shell.is_empty() {
                instant.take();
            }

            shell.merge(local_shell, std::convert::identity);
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        for ((child, tree), layout) in self
            .toasts
            .iter()
            .zip(self.trees.iter())
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.toasts
                .iter_mut()
                .zip(self.trees.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child.as_widget_mut().operate(state, layout, renderer, operation);
                });
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.trees.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, &self.viewport, renderer)
                    .max(if cursor.is_over(layout.bounds()) {
                        mouse::Interaction::Idle
                    } else {
                        Default::default()
                    })
            })
            .max()
            .unwrap_or_default()
    }
}

impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(manager: Manager<'a, Message>) -> Self {
        Element::new(manager)
    }
}
