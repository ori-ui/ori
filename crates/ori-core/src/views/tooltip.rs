use ori_macro::{example, Styled};
use smol_str::SmolStr;

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{pt, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{key, Styled},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    view::{Pod, State, View},
};

/// Create a new [`Tooltip`] view.
pub fn tooltip<V>(content: V, text: impl Into<SmolStr>) -> Tooltip<V> {
    Tooltip::new(content, text)
}

/// A view that displays some text when the content is hovered.
///
/// Can be styled using the [`TooltipStyle`].
#[example(name = "tooltip", width = 400, height = 300)]
#[derive(Styled, Rebuild)]
pub struct Tooltip<V> {
    /// The content to display.
    pub content: Pod<V>,

    /// The text to display.
    #[rebuild(layout)]
    pub text: SmolStr,

    /// The delay before the tooltip is displayed.
    #[styled(default = 0.2)]
    pub delay: Styled<f32>,

    /// The padding of the text.
    #[rebuild(layout)]
    #[styled(default = Padding::all(4.0))]
    pub padding: Styled<Padding>,

    /// The font size of the text.
    #[styled(default = pt(10.0))]
    pub font_size: Styled<f32>,

    /// The font family of the text.
    #[styled(default)]
    pub font_family: Styled<FontFamily>,

    /// The font weight of the text.
    #[styled(default)]
    pub font_weight: Styled<FontWeight>,

    /// The font stretch of the text.
    #[styled(default)]
    pub font_stretch: Styled<FontStretch>,

    /// The font style of the text.
    #[styled(default)]
    pub font_style: Styled<FontStyle>,

    /// The color of text.
    #[rebuild(draw)]
    #[styled(default -> "palette.contrast" or Color::BLACK)]
    pub color: Styled<Color>,

    /// The horizontal alignment of the text.
    #[styled(default)]
    pub align: Styled<TextAlign>,

    /// The line height of the text.
    #[styled(default = 1.2)]
    pub line_height: Styled<f32>,

    /// The text wrap of the text.
    #[styled(default)]
    pub wrap: Styled<TextWrap>,

    /// The background color of the text.
    #[rebuild(draw)]
    #[styled(default -> "palette.surface_higher" or Color::WHITE)]
    pub background: Styled<Color>,

    /// The border radius of the text.
    #[rebuild(draw)]
    #[styled(default = BorderRadius::all(4.0))]
    pub border_radius: Styled<BorderRadius>,

    /// The border width of the text.
    #[rebuild(draw)]
    #[styled(default = BorderWidth::all(1.0))]
    pub border_width: Styled<BorderWidth>,

    /// The border color of the text.
    #[rebuild(draw)]
    #[styled(default -> "palette.outline" or Color::BLACK)]
    pub border_color: Styled<Color>,
}

impl<V> Tooltip<V> {
    /// Create a new tooltip view.
    pub fn new(content: V, text: impl Into<SmolStr>) -> Self {
        Self {
            content: Pod::new(content),
            text: text.into(),
            delay: key("tooltip.delay"),
            padding: key("tooltip.padding"),
            font_size: key("tooltip.font_size"),
            font_family: key("tooltip.font_family"),
            font_weight: key("tooltip.font_weight"),
            font_stretch: key("tooltip.font_stretch"),
            font_style: key("tooltip.font_style"),
            color: key("tooltip.color"),
            align: key("tooltip.align"),
            line_height: key("tooltip.line_height"),
            wrap: key("tooltip.wrap"),
            background: key("tooltip.background"),
            border_radius: key("tooltip.border_radius"),
            border_width: key("tooltip.border_width"),
            border_color: key("tooltip.border_color"),
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer, style: &TooltipStyle) {
        buffer.set_wrap(fonts, style.wrap);
        buffer.set_align(style.align);
        buffer.set_text(
            fonts,
            &self.text,
            TextAttributes {
                family: style.font_family.clone(),
                weight: style.font_weight,
                stretch: style.font_stretch,
                style: style.font_style,
            },
        );
    }
}

#[doc(hidden)]
pub struct TooltipState {
    pub buffer: TextBuffer,
    pub timer: f32,
    pub position: Point,
    pub style: TooltipStyle,
}

impl<T, V: View<T>> View<T> for Tooltip<V> {
    type State = (TooltipState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let style = TooltipStyle::styled(self, cx.styles());

        let mut state = TooltipState {
            buffer: TextBuffer::new(cx.fonts(), style.font_size, 1.0),
            timer: 0.0,
            position: Point::ZERO,
            style,
        };

        self.set_attributes(cx.fonts(), &mut state.buffer, &state.style);

        (state, self.content.build(cx, data))
    }

    fn rebuild(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &Self,
    ) {
        let font_size = state.style.font_size;
        let line_height = state.style.line_height;
        let wrap = state.style.wrap;
        let align = state.style.align;
        let font_family = state.style.font_family.clone();
        let font_stretch = state.style.font_stretch;
        let font_weight = state.style.font_weight;
        let font_style = state.style.font_style;

        Rebuild::rebuild(self, cx, old);
        state.style.rebuild(self, cx);

        if state.style.font_size != font_size || state.style.line_height != line_height {
            (state.buffer).set_metrics(cx.fonts(), state.style.font_size, state.style.line_height);
        }

        if state.style.wrap != wrap {
            state.buffer.set_wrap(cx.fonts(), state.style.wrap);
        }

        if state.style.align != align {
            state.buffer.set_align(state.style.align);
        }

        if self.text != old.text
            || state.style.font_family != font_family
            || state.style.font_stretch != font_stretch
            || state.style.font_weight != font_weight
            || state.style.font_style != font_style
        {
            state.buffer.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: state.style.font_family.clone(),
                    stretch: state.style.font_stretch,
                    weight: state.style.font_weight,
                    style: state.style.font_style,
                },
            );

            cx.layout();
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

        if !content.has_hot() && state.timer > 0.0 {
            state.timer = 0.0;
            cx.draw();
        }

        match event {
            Event::WindowResized(_) => {
                cx.layout();
            }
            Event::PointerMoved(e) => {
                if state.timer > 0.0 {
                    state.timer = 0.0;
                    cx.draw();
                }

                if content.has_hot() {
                    state.position = e.position;
                    cx.animate();
                }
            }
            Event::Animate(dt) => {
                if content.has_hot() && state.timer < 1.0 {
                    state.timer += dt / state.style.delay;
                    cx.animate();
                }

                if let Some(pointer) = cx.window().pointers().first() {
                    state.position = pointer.position;
                }

                state.timer = f32::clamp(state.timer, 0.0, 1.0);

                if state.timer >= 0.9 {
                    cx.draw();
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
        let window_size = cx.window().size - state.style.padding.size();
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

        let size = state.buffer.size() + state.style.padding.size();
        let mut offset = Vector::new(-size.width / 2.0, 20.0);

        let rect = Rect::min_size(state.position + offset, size);

        let tl_delta = window_rect.top_left() - rect.top_left();
        let br_delta = rect.bottom_right() - window_rect.bottom_right();

        offset += Vector::max(tl_delta, Vector::ZERO);
        offset -= Vector::max(br_delta, Vector::ZERO);

        cx.overlay(0, |cx| {
            cx.translated(Vector::from(state.position + offset), |cx| {
                cx.quad(
                    Rect::min_size(Point::ZERO, size),
                    state.style.background.fade(alpha),
                    state.style.border_radius,
                    state.style.border_width,
                    state.style.border_color.fade(alpha),
                );

                cx.text(
                    &state.buffer,
                    state.style.color,
                    state.style.padding.offset(),
                );
            });
        });
    }
}
