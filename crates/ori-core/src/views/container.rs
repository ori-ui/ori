use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color, Curve, FillRule, Mask},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    view::{Pod, PodState, View},
};

/// Create a new [`Container`].
pub fn container<V>(view: V) -> Container<V> {
    Container::new(view)
}

/// Create a new [`Container`] with background.
///
/// # Examples
/// ```
/// # use ori_core::{canvas::Color, view::*, views::*};
/// pub fn ui<T>(_data: T) -> impl View<T> {
///     background(Color::RED, text("Hello, World!"))
/// }
/// ````
pub fn background<V>(background: impl Into<Color>, view: V) -> Container<V> {
    Container::new(view).background(background)
}

/// The style of a container.
#[derive(Clone, Rebuild)]
pub struct ContainerStyle {
    /// The background color.
    #[rebuild(draw)]
    pub background: Color,

    /// The border radius.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,

    /// The border width.
    #[rebuild(draw)]
    pub border_width: BorderWidth,

    /// The border color.
    #[rebuild(draw)]
    pub border_color: Color,

    /// Whether to mask the content.
    #[rebuild(draw)]
    pub mask: bool,
}

impl Style for ContainerStyle {
    fn builder() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| ContainerStyle {
            background: theme.surface(0),
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(0.0),
            border_color: theme.outline,
            mask: false,
        })
    }
}

/// A container view.
#[example(name = "container", width = 400, height = 300)]
#[derive(Build, Rebuild)]
pub struct Container<V> {
    /// The content.
    #[build(ignore)]
    pub content: Pod<V>,

    /// The background color.
    pub background: Option<Color>,

    /// The border radius.
    pub border_radius: Option<BorderRadius>,

    /// The border width.
    pub border_width: Option<BorderWidth>,

    /// The border color.
    pub border_color: Option<Color>,

    /// Whether to mask the content.
    pub mask: Option<bool>,
}

impl<V> Container<V> {
    /// Create a new [`Container`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            background: None,
            border_radius: None,
            border_width: None,
            border_color: None,
            mask: None,
        }
    }
}

impl<V> Stylable for Container<V> {
    type Style = ContainerStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        ContainerStyle {
            background: self.background.unwrap_or(style.background),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            border_width: self.border_width.unwrap_or(style.border_width),
            border_color: self.border_color.unwrap_or(style.border_color),
            mask: self.mask.unwrap_or(style.mask),
        }
    }
}

impl<T, V: View<T>> View<T> for Container<V> {
    type State = (ContainerStyle, PodState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let style = self.style(cx.style());
        (style, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (style, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        self.rebuild_style(cx, style);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (_, state): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        self.content.event(state, cx, data, event)
    }

    fn layout(
        &mut self,
        (_, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, (style, state): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.quad(
            cx.rect(),
            style.background,
            style.border_radius,
            style.border_width,
            style.border_color,
        );

        match style.mask {
            true => {
                let mut mask = Curve::new();
                mask.push_rect_with_radius(cx.rect(), style.border_radius);

                cx.masked(Mask::new(mask, FillRule::NonZero), |cx| {
                    self.content.draw(state, cx, data);
                });
            }
            false => {
                self.content.draw(state, cx, data);
            }
        }
    }
}
