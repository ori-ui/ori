use glam::Vec2;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    event::{Event, HotChanged, PointerEvent},
    layout::{Padding, Rect, Size, Space},
    rebuild::Rebuild,
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Glyphs, TextAlign, TextSection, TextWrap,
    },
    theme::{alt, pt, style, text},
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// Create a new [`Alt`] view.
pub fn alt<V>(alt: impl ToString, content: V) -> Alt<V> {
    Alt::new(alt, content)
}

/// A view that displays some text when the content is hovered.
#[derive(Rebuild)]
pub struct Alt<V> {
    /// The content to display.
    pub content: Content<V>,
    /// The alternative text to display.
    #[rebuild(layout)]
    pub alt: String,
    /// The padding of the text.
    #[rebuild(draw)]
    pub padding: Padding,
    /// The color of text.
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
            content: Content::new(content),
            alt: alt.to_string(),
            color: style(text::COLOR),
            padding: style(alt::PADDING),
            background: style(alt::BACKGROUND),
            border_radius: style(alt::BORDER_RADIUS),
            border_width: style(alt::BORDER_WIDTH),
            border_color: style(alt::BORDER_COLOR),
        }
    }
}

#[doc(hidden)]
pub struct AltState {
    pub glyphs: Option<Glyphs>,
    pub timer: f32,
    pub position: Vec2,
}

impl<T, V: View<T>> View<T> for Alt<V> {
    type State = (AltState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = AltState {
            glyphs: None,
            timer: 0.0,
            position: Vec2::ZERO,
        };

        (state, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);

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

        if event.is::<HotChanged>() {
            state.timer = -f32::INFINITY;
            cx.request_draw();
        }

        if let Some(pointer) = event.get::<PointerEvent>() {
            let local = cx.local(pointer.position);

            if cx.is_hot() && pointer.is_move() {
                state.position = local;
                cx.request_draw();
            }
        }
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let text = TextSection {
            text: &self.alt,
            font_size: pt(14.0),
            font_family: FontFamily::SansSerif,
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: self.color,
            v_align: TextAlign::Start,
            h_align: TextAlign::Start,
            line_height: 1.0,
            wrap: TextWrap::Word,
            bounds: cx.window().size() - self.padding.size(),
        };

        state.glyphs = cx.layout_text(&text);

        self.content.layout(content, cx, data, space)
    }

    fn draw(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(content, cx, data, canvas);

        if cx.is_hot() && state.timer < 0.0 {
            state.timer += cx.dt();
            cx.request_draw();
        }

        if state.timer == -f32::INFINITY {
            state.timer = -1.0;
        }

        let Some(ref glyphs) = state.glyphs else { return };

        if cx.is_hot() && state.timer >= 0.0 {
            let size = glyphs.size() + self.padding.size();
            let offset = Vec2::new(-size.width / 2.0, pt(20.0));
            let text_rect = Rect::min_size(
                state.position + offset + self.padding.offset(),
                glyphs.size(),
            );

            let mut layer = canvas.layer();
            layer.depth += 1000.0;

            layer.draw_quad(
                Rect::min_size(state.position + offset, size),
                self.background,
                self.border_radius,
                self.border_width,
                self.border_color,
            );

            if let Some(mesh) = cx.text_mesh(glyphs, text_rect) {
                layer.draw_pixel_perfect(mesh);
            }
        }
    }
}
