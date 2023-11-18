use crate::{
    canvas::{Background, BorderRadius, BorderWidth, BoxShadow, Canvas, Color},
    event::{Event, PointerEvent},
    layout::{Size, Space, Vector},
    rebuild::Rebuild,
    theme::{container, style},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

/// A container view.
#[derive(Rebuild)]
pub struct Container<V> {
    /// The content.
    pub content: Pod<V>,
    /// The background color.
    #[rebuild(draw)]
    pub background: Background,
    /// The border radius.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,
    /// The border width.
    #[rebuild(draw)]
    pub border_width: BorderWidth,
    /// The border color.
    #[rebuild(draw)]
    pub border_color: Color,
    /// The shadow.
    #[rebuild(draw)]
    pub shadow: BoxShadow,
}

impl<V> Container<V> {
    /// Create a new [`Container`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            background: style(container::BACKGROUND),
            border_radius: style(container::BORDER_RADIUS),
            border_width: style(container::BORDER_WIDTH),
            border_color: style(container::BORDER_COLOR),
            shadow: BoxShadow::default(),
        }
    }

    /// Set the background color.
    pub fn background(mut self, background: impl Into<Background>) -> Self {
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

    /// Set the shadow.
    pub fn shadow(mut self, shadow: impl Into<BoxShadow>) -> Self {
        self.shadow = shadow.into();
        self
    }

    /// Set the shadow color.
    pub fn shadow_color(mut self, color: impl Into<Color>) -> Self {
        self.shadow.color = color.into();
        self
    }

    /// Set the shadow blur.
    pub fn shadow_blur(mut self, blur: f32) -> Self {
        self.shadow.blur = blur;
        self
    }

    /// Set the shadow spread.
    pub fn shadow_spread(mut self, spread: f32) -> Self {
        self.shadow.spread = spread;
        self
    }

    /// Set the shadow offset.
    pub fn shadow_offset(mut self, offset: impl Into<Vector>) -> Self {
        self.shadow.offset = offset.into();
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

        if let Some(pointer) = event.get::<PointerEvent>() {
            if cx.is_hot() && pointer.is_move() {
                event.handle();
            }
        }
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

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.draw(self.shadow.mesh(cx.rect(), self.border_radius));

        canvas.view(cx.id());
        canvas.draw_quad(
            cx.rect(),
            self.background.clone(),
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
