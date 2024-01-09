use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    event::{AnimationFrame, Event, PointerMoved},
    layout::{Affine, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    text::{FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAttributes, TextBuffer},
    theme::{alt, style, text},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Alt`] view.
pub fn alt<V>(alt: impl ToString, content: V) -> Alt<V> {
    Alt::new(alt, content)
}

/// A view that displays some text when the content is hovered.
#[derive(Rebuild)]
pub struct Alt<V> {
    /// The content to display.
    pub content: V,
    /// The alternative text to display.
    #[rebuild(layout)]
    pub alt: String,
    /// The padding of the text.
    #[rebuild(draw)]
    pub padding: Padding,
    /// The color of text.
    #[rebuild(draw)]
    pub color: Color,
    /// The background color of the text.
    #[rebuild(draw)]
    pub background: Color,
    /// The border radius of the text.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,
    /// The border width of the text.
    #[rebuild(draw)]
    pub border_width: BorderWidth,
    /// The border color of the text.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<V> Alt<V> {
    /// Create a new alt view.
    pub fn new(alt: impl ToString, content: V) -> Self {
        Self {
            content,
            alt: alt.to_string(),
            color: style(text::COLOR),
            padding: style(alt::PADDING),
            background: style(alt::BACKGROUND),
            border_radius: style(alt::BORDER_RADIUS),
            border_width: style(alt::BORDER_WIDTH),
            border_color: style(alt::BORDER_COLOR),
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer) {
        buffer.set_text(
            fonts,
            &self.alt,
            TextAttributes {
                family: FontFamily::SansSerif,
                weight: FontWeight::NORMAL,
                stretch: FontStretch::Normal,
                style: FontStyle::Normal,
                color: self.color,
            },
        );
    }
}

#[doc(hidden)]
pub struct AltState {
    pub buffer: TextBuffer,
    pub timer: f32,
    pub position: Point,
}

impl<T, V: View<T>> View<T> for Alt<V> {
    type State = (AltState, V::State);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut state = AltState {
            buffer: TextBuffer::new(cx.fonts(), 12.0, 1.0),
            timer: 0.0,
            position: Point::ZERO,
        };

        self.set_attributes(cx.fonts(), &mut state.buffer);

        (state, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        self.set_attributes(cx.fonts(), &mut state.buffer);

        (self.content).rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        self.content.event(content, cx, data, event);

        if let Some(moved) = event.get::<PointerMoved>() {
            state.timer = 0.0;

            if cx.is_hot() {
                state.position = moved.position;
                cx.request_animation_frame();
                event.handle();
            }
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if cx.is_hot() && state.timer < 1.0 {
                state.timer += dt * 2.0;

                cx.request_animation_frame();
            }

            state.timer = f32::clamp(state.timer, 0.0, 1.0);

            cx.request_draw();
        }
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let bounds = cx.window().size() - self.padding.size();
        state.buffer.set_bounds(cx.fonts(), bounds);

        self.content.layout(content, cx, data, space)
    }

    fn draw(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        // we need to set the view to be enable hit testing
        canvas.set_view(cx.id());
        self.content.draw(content, cx, data, canvas);

        let alpha = f32::clamp(state.timer * 10.0 - 9.0, 0.0, 1.0);

        if alpha <= 0.0 {
            return;
        }

        let size = state.buffer.size() + self.padding.size();
        let offset = Vector::new(-size.width / 2.0, 20.0);
        let text_rect = Rect::min_size(
            state.position + offset + self.padding.offset(),
            state.buffer.size(),
        );

        let mut layer = canvas.layer();
        layer.transform = Affine::IDENTITY;
        layer.depth += 1000.0;
        layer.clip = Rect::min_size(Point::ZERO, cx.window().size());

        layer.draw_quad(
            Rect::min_size(state.position + offset, size),
            self.background.fade(alpha),
            self.border_radius,
            self.border_width,
            self.border_color.fade(alpha),
        );

        let mesh = cx.rasterize_text(&state.buffer, text_rect);
        layer.draw(mesh);
    }
}
