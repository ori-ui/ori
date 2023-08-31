use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    theme::{container, style},
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// A container view.
#[derive(Rebuild)]
pub struct Container<V> {
    /// The content.
    pub content: Content<V>,
    /// The space available to the content.
    ///
    /// By default, the content is given [`Space::UNBOUNDED`].
    #[rebuild(layout)]
    pub space: Space,
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
}

impl<V> Container<V> {
    /// Create a new [`Container`].
    pub fn new(content: V) -> Self {
        Self {
            content: Content::new(content),
            space: Space::default(),
            background: style(container::BACKGROUND),
            border_radius: style(container::BORDER_RADIUS),
            border_width: style(container::BORDER_WIDTH),
            border_color: style(container::BORDER_COLOR),
        }
    }

    /// Set the size.
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.space = Space::from_size(size.into());
        self
    }

    /// Set the width.
    pub fn width(mut self, width: f32) -> Self {
        self.space.min.width = width;
        self.space.max.width = width;
        self
    }

    /// Set the height.
    pub fn height(mut self, height: f32) -> Self {
        self.space.min.height = height;
        self.space.max.height = height;
        self
    }

    /// Set the minimum width.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.space.min.width = min_width;
        self
    }

    /// Set the minimum height.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.space.min.height = min_height;
        self
    }

    /// Set the maximum width.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.space.max.width = max_width;
        self
    }

    /// Set the maximum height.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.space.max.height = max_height;
        self
    }

    /// Set the background color.
    pub fn background(mut self, background: impl Into<Color>) -> Self {
        self.background = background.into();
        self
    }

    /// Set the border radius.
    pub fn border_radius(mut self, border_radius: impl Into<BorderRadius>) -> Self {
        self.border_radius = border_radius.into();
        self
    }

    /// Set the border width.
    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = border_width.into();
        self
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

    /// Set the border color.
    pub fn border_color(mut self, border_color: impl Into<Color>) -> Self {
        self.border_color = border_color.into();
        self
    }
}

impl<T, V: View<T>> View<T> for Container<V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.request_rebuild();
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
        let space = self.space.constrain(space);
        self.content.layout(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.draw_quad(
            cx.rect(),
            self.background,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        self.content.draw(state, cx, data, canvas);
    }
}

/// Create a new [`Container`].
pub fn container<V>(content: V) -> Container<V> {
    Container::new(content)
}

/// Create a new [`Container`] with a fixed size.
pub fn size<V>(size: impl Into<Size>, content: V) -> Container<V> {
    Container {
        space: Space::from_size(size.into()),
        ..Container::new(content)
    }
}

/// Create a new [`Container`] with a fixed width.
pub fn width<V>(width: f32, content: V) -> Container<V> {
    Container::new(content).width(width)
}

/// Create a new [`Container`] with a fixed height.
pub fn height<V>(height: f32, content: V) -> Container<V> {
    Container::new(content).height(height)
}

/// Create a new [`Container`] with a minimum width.
pub fn min_width<V>(min_width: f32, content: V) -> Container<V> {
    Container::new(content).min_width(min_width)
}

/// Create a new [`Container`] with a minimum height.
pub fn min_height<V>(min_height: f32, content: V) -> Container<V> {
    Container::new(content).min_height(min_height)
}

/// Create a new [`Container`] with a maximum width.
pub fn max_width<V>(max_width: f32, content: V) -> Container<V> {
    Container::new(content).max_width(max_width)
}

/// Create a new [`Container`] with a maximum height.
pub fn max_height<V>(max_height: f32, content: V) -> Container<V> {
    Container::new(content).max_height(max_height)
}
