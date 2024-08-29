use ori_macro::example;
use smol_str::SmolStr;

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{pt, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    view::{Pod, State, View},
};

use super::TextStyle;

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
    fn styled(style: &Styles) -> Self {
        let text_style = style.get::<TextStyle>();
        let palette = style.palette();

        Self {
            delay: 0.2,
            padding: Padding::from([8.0, 4.0]),
            font_size: pt(10.0),
            font_family: text_style.font_family.clone(),
            font_weight: text_style.font_weight,
            font_stretch: text_style.font_stretch,
            font_style: text_style.font_style,
            color: text_style.color,
            align: text_style.align,
            line_height: text_style.line_height,
            wrap: text_style.wrap,
            background: palette.surface_higher,
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(1.0),
            border_color: palette.outline,
        }
    }
}

/// A view that displays some text when the content is hovered.
///
/// Can be styled using the [`TooltipStyle`].
#[example(name = "tooltip", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Tooltip<V> {
    /// The content to display.
    pub content: Pod<V>,

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
            content: Pod::new(content),
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
    type State = (TooltipState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut state = TooltipState {
            buffer: TextBuffer::new(cx.fonts(), self.font_size, 1.0),
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

        if self.font_size != old.font_size || self.line_height != old.line_height {
            (state.buffer).set_metrics(cx.fonts(), self.font_size, self.line_height);
        }

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
        {
            state.buffer.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: self.font_family.clone(),
                    stretch: self.font_stretch,
                    weight: self.font_weight,
                    style: self.font_style,
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

        match event {
            Event::WindowResized(_) => {
                cx.request_layout();
            }
            Event::PointerMoved(e) => {
                state.timer = 0.0;

                if content.has_hot() {
                    state.position = e.position;
                    cx.animate();
                }
            }
            Event::Animate(dt) => {
                if content.has_hot() && state.timer < 1.0 {
                    state.timer += dt / self.delay;
                    cx.animate();
                }

                state.timer = f32::clamp(state.timer, 0.0, 1.0);

                if state.timer >= 0.9 {
                    cx.request_draw();
                }
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let window_size = cx.window().size - self.padding.size();
        state.buffer.set_bounds(cx.fonts(), window_size);
        self.content.layout(content, cx, data, space)
    }

    fn draw(&mut self, (state, content): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        // make sure the tooltip is hoverable
        cx.canvas().trigger(content.rect(), content.id());

        // we need to set the view to be enable hit testing
        self.content.draw(content, cx, data);

        let alpha = f32::clamp(state.timer * 10.0 - 9.0, 0.0, 1.0);

        if alpha <= 0.0 {
            return;
        }

        // we need to try to move the tooltip so it fits on the screen
        let window_rect = Rect::min_size(Point::ZERO, cx.window().size);

        let size = state.buffer.size() + self.padding.size();
        let mut offset = Vector::new(-size.width / 2.0, 20.0);

        let rect = Rect::min_size(state.position + offset, size);

        let tl_delta = window_rect.top_left() - rect.top_left();
        let br_delta = rect.bottom_right() - window_rect.bottom_right();

        offset += Vector::max(tl_delta, Vector::ZERO);
        offset -= Vector::max(br_delta, Vector::ZERO);

        cx.overlay(0, |cx| {
            cx.translate(Vector::from(state.position + offset), |cx| {
                cx.quad(
                    Rect::min_size(Point::ZERO, size),
                    self.background.fade(alpha),
                    self.border_radius,
                    self.border_width,
                    self.border_color.fade(alpha),
                );

                cx.text(&state.buffer, self.color, self.padding.offset());
            });
        });
    }
}
