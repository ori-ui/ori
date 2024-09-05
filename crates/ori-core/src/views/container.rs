use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color, Curve, FillRule, Mask},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    view::{Pod, State, View},
};

/// Create a new [`Container`].
pub fn container<V>(content: V) -> Container<V> {
    Container::new(content)
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
pub fn background<V>(background: Color, content: V) -> Container<V> {
    Container::new(content).background(background)
}

/// The style of a [`Container`].
#[derive(Clone, Debug)]
pub struct ContainerStyle {
    /// The background color.
    pub background: Color,

    /// The border radius.
    pub border_radius: BorderRadius,

    /// The border width.
    pub border_width: BorderWidth,

    /// The border color.
    pub border_color: Color,
}

impl Style for ContainerStyle {
    fn styled(style: &Styles) -> Self {
        Self {
            background: style.palette().surface,
            border_radius: BorderRadius::all(0.0),
            border_width: BorderWidth::all(0.0),
            border_color: style.palette().outline,
        }
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

impl<V> Container<V> {
    /// Create a new [`Container`].
    pub fn new(content: V) -> Self {
        Self::styled(content, style())
    }

    /// Create a new [`Container`] with a style.
    pub fn styled(content: V, style: ContainerStyle) -> Self {
        Self {
            content: Pod::new(content),
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
            mask: false,
        }
    }

    /// Set the border width of the top edge.
    pub fn border_top(mut self, width: f32) -> Self {
        self.border_width.top = width;
        self
    }

    /// Set the border width of the right edge.
    pub fn border_right(mut self, width: f32) -> Self {
        self.border_width.right = width;
        self
    }

    /// Set the border width of the bottom edge.
    pub fn border_bottom(mut self, width: f32) -> Self {
        self.border_width.bottom = width;
        self
    }

    /// Set the border width of the left edge.
    pub fn border_left(mut self, width: f32) -> Self {
        self.border_width.left = width;
        self
    }
}

impl<T, V: View<T>> View<T> for Container<V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.quad(
            cx.rect(),
            self.background,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        match self.mask {
            true => {
                let mut mask = Curve::new();
                mask.push_rect_with_radius(cx.rect(), self.border_radius);

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
