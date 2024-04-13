use ori_macro::example;
use smol_str::SmolStr;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event},
    layout::{Affine, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    view::View,
};

/// Create a new [`Tooltip`] view.
pub fn tooltip<V>(content: V, text: impl Into<SmolStr>) -> Tooltip<V> {
    Tooltip::new(content, text)
}

/// The style of a tooltip.
#[derive(Clone, Debug)]
pub struct TooltipStyle {
    /// The delay before the tooltip is displayed.
    pub delay: f32,
    /// The padding of the tooltip.
    pub padding: Padding,
    /// The font size of the text.
    pub font_size: f32,
    /// The font family of the text.
    pub font_family: FontFamily,
    /// The font weight of the text.
    pub font_weight: FontWeight,
    /// The font stretch of the text.
    pub font_stretch: FontStretch,
    /// The font style of the text.
    pub font_style: FontStyle,
    /// The color of the text.
    pub color: Color,
    /// The horizontal alignment of the text.
    pub align: TextAlign,
    /// The line height of the text.
    pub line_height: f32,
    /// The text wrap of the text.
    pub wrap: TextWrap,
    /// The background color of the text.
    pub background: Color,
    /// The border radius of the text.
    pub border_radius: BorderRadius,
    /// The border width of the text.
    pub border_width: BorderWidth,
    /// The border color of the text.
    pub border_color: Color,
}

impl Style for TooltipStyle {
    fn style(style: &Styles) -> Self {
        Self {
            delay: 0.2,
            padding: Padding::from([8.0, 4.0]),
            font_size: 12.0,
            font_family: FontFamily::SansSerif,
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: style.palette().text(),
            align: TextAlign::Start,
            line_height: 1.3,
            wrap: TextWrap::Word,
            background: style.palette().secondary(),
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(1.0),
            border_color: style.palette().secondary_dark(),
        }
    }
}

/// A view that displays some text when the content is hovered.
#[example(name = "tooltip", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Tooltip<V> {
    /// The content to display.
    pub content: V,

    /// The text to display.
    #[rebuild(layout)]
    pub text: SmolStr,

    /// The delay before the tooltip is displayed.
    pub delay: f32,

    /// The padding of the text.
    #[rebuild(layout)]
    pub padding: Padding,

    /// The font size of the text.
    #[rebuild(layout)]
    pub font_size: f32,

    /// The font family of the text.
    #[rebuild(layout)]
    pub font_family: FontFamily,

    /// The font weight of the text.
    #[rebuild(layout)]
    pub font_weight: FontWeight,

    /// The font stretch of the text.
    #[rebuild(layout)]
    pub font_stretch: FontStretch,

    /// The font style of the text.
    #[rebuild(layout)]
    pub font_style: FontStyle,

    /// The color of text.
    #[rebuild(draw)]
    pub color: Color,

    /// The horizontal alignment of the text.
    #[rebuild(layout)]
    pub align: TextAlign,

    /// The line height of the text.
    #[rebuild(layout)]
    pub line_height: f32,

    /// The text wrap of the text.
    #[rebuild(layout)]
    pub wrap: TextWrap,

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
    pub fn new(content: V, text: impl Into<SmolStr>) -> Self {
        Self::styled(content, text, style())
    }

    /// Create a new tooltip view with a style.
    pub fn styled(content: V, text: impl Into<SmolStr>, style: TooltipStyle) -> Self {
        Self {
            content,
            text: text.into(),
            delay: style.delay,
            padding: style.padding,
            font_size: style.font_size,
            font_family: style.font_family,
            font_weight: style.font_weight,
            font_stretch: style.font_stretch,
            font_style: style.font_style,
            color: style.color,
            align: style.align,
            line_height: style.line_height,
            wrap: style.wrap,
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer) {
        buffer.set_wrap(fonts, self.wrap);
        buffer.set_align(self.align);
        buffer.set_text(
            fonts,
            &self.text,
            TextAttributes {
                family: self.font_family.clone(),
                weight: self.font_weight,
                stretch: self.font_stretch,
                style: self.font_style,
                color: self.color,
            },
        );
    }
}

#[doc(hidden)]
pub struct TooltipState {
    pub window_size: Size,
    pub buffer: TextBuffer,
    pub timer: f32,
    pub position: Point,
}

impl<T, V: View<T>> View<T> for Tooltip<V> {
    type State = (TooltipState, V::State);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut state = TooltipState {
            window_size: cx.window().size(),
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

        if self.wrap != old.wrap {
            state.buffer.set_wrap(cx.fonts(), self.wrap);
        }

        if self.align != old.align {
            state.buffer.set_align(self.align);
        }

        if self.text != old.text
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
            || self.color != old.color
        {
            state.buffer.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: self.font_family.clone(),
                    stretch: self.font_stretch,
                    weight: self.font_weight,
                    style: self.font_style,
                    color: self.color,
                },
            );

            cx.request_layout();
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

        if let Event::WindowResized(e) = event {
            state.window_size = e.size();
            cx.request_layout();
        }

        if let Event::PointerMoved(e) = event {
            state.timer = 0.0;

            if cx.is_hot() || cx.has_hot() {
                state.position = e.position;
                cx.animate();
            }
        }

        if let Event::Animate(dt) = event {
            if cx.is_hot() || cx.has_hot() && state.timer < 1.0 {
                state.timer += dt / self.delay;
                cx.animate();
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
        let window_size = state.window_size - self.padding.size();
        state.buffer.set_bounds(cx.fonts(), window_size);
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

        // we need to try to move the tooltip so it fits on the screen
        let window_rect = Rect::min_size(Point::ZERO, state.window_size);

        let size = state.buffer.size() + self.padding.size();
        let mut offset = Vector::new(-size.width / 2.0, 20.0);

        let rect = Rect::min_size(state.position + offset, size);

        let tl_delta = window_rect.top_left() - rect.top_left();
        let br_delta = rect.bottom_right() - window_rect.bottom_right();

        offset += Vector::max(tl_delta, Vector::ZERO);
        offset -= Vector::max(br_delta, Vector::ZERO);

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
