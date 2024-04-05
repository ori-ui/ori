use smol_str::SmolStr;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    event::{AnimationFrame, Event, PointerMoved},
    layout::{Affine, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{style, Styled, Styles},
    text::{FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAttributes, TextBuffer},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// The style of a tooltip.
#[derive(Clone, Debug)]
pub struct TooltipStyle {
    /// The padding of the tooltip.
    pub padding: Padding,
    /// The color of the text.
    pub color: Color,
    /// The background color of the text.
    pub background: Color,
    /// The border radius of the text.
    pub border_radius: BorderRadius,
    /// The border width of the text.
    pub border_width: BorderWidth,
    /// The border color of the text.
    pub border_color: Color,
}

impl Styled for TooltipStyle {
    fn from_style(style: &Styles) -> Self {
        Self {
            padding: Padding::from([8.0, 4.0]),
            color: style.palette().text(),
            background: style.palette().secondary(),
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(1.0),
            border_color: style.palette().secondary_dark(),
        }
    }
}

/// Create a new [`Tooltip`] view.
pub fn tooltip<V>(alt: impl Into<SmolStr>, content: V) -> Tooltip<V> {
    Tooltip::new(alt, content)
}

/// A view that displays some text when the content is hovered.
#[derive(Rebuild)]
pub struct Tooltip<V> {
    /// The content to display.
    pub content: V,
    /// The text to display.
    #[rebuild(layout)]
    pub text: SmolStr,
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

impl<V> Tooltip<V> {
    /// Create a new tooltip view.
    pub fn new(text: impl Into<SmolStr>, content: V) -> Self {
        let style = style::<TooltipStyle>();

        Self {
            content,
            text: text.into(),
            padding: style.padding,
            color: style.color,
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer) {
        buffer.set_text(
            fonts,
            &self.text,
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
pub struct TooltipState {
    pub buffer: TextBuffer,
    pub timer: f32,
    pub position: Point,
}

impl<T, V: View<T>> View<T> for Tooltip<V> {
    type State = (TooltipState, V::State);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut state = TooltipState {
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

        if self.text != old.text || self.color != old.color {
            self.set_attributes(cx.fonts(), &mut state.buffer);
        }

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

            if cx.is_hot() || cx.has_hot() {
                state.position = moved.position;
                cx.request_animation_frame();
                event.handle();
            }
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if cx.is_hot() || cx.has_hot() && state.timer < 1.0 {
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
        state.buffer.set_bounds(cx.fonts(), Size::UNBOUNDED);
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

        let mut layer = canvas.layer();
        layer.transform = Affine::IDENTITY;
        layer.translate(Vector::from(state.position + offset));
        layer.depth += 1000.0;
        layer.clip = Rect::min_size(Point::ZERO, cx.window().size());

        layer.draw_quad(
            Rect::min_size(Point::ZERO, size),
            self.background.fade(alpha),
            self.border_radius,
            self.border_width,
            self.border_color.fade(alpha),
        );

        layer.translate(self.padding.offset());

        let mesh = cx.rasterize_text(&state.buffer);
        layer.draw_pixel_perfect(mesh);
    }
}
