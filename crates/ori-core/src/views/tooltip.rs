use ori_macro::{example, Styled};
use smol_str::SmolStr;

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{pt, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Styled, Theme},
    text::{
        FontAttributes, FontFamily, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign,
        TextWrap,
    },
    view::{Pod, State, View},
};

/// Create a new [`Tooltip`] view.
pub fn tooltip<V>(view: V, text: impl Into<SmolStr>) -> Tooltip<V> {
    Tooltip::new(view, text)
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
    #[styled(default -> Theme::CONTRAST or Color::BLACK)]
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
    #[styled(default -> Theme::SURFACE_HIGHER or Color::WHITE)]
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
    #[styled(default -> Theme::OUTLINE or Color::BLACK)]
    pub border_color: Styled<Color>,
}

impl<V> Tooltip<V> {
    /// Create a new tooltip view.
    pub fn new(content: V, text: impl Into<SmolStr>) -> Self {
        Self {
            content: Pod::new(content),
            text: text.into(),
            delay: Styled::style("tooltip.delay"),
            padding: Styled::style("tooltip.padding"),
            font_size: Styled::style("tooltip.font-size"),
            font_family: Styled::style("tooltip.font-family"),
            font_weight: Styled::style("tooltip.font-weight"),
            font_stretch: Styled::style("tooltip.font-stretch"),
            font_style: Styled::style("tooltip.font-style"),
            color: Styled::style("tooltip.color"),
            align: Styled::style("tooltip.align"),
            line_height: Styled::style("tooltip.line-height"),
            wrap: Styled::style("tooltip.wrap"),
            background: Styled::style("tooltip.background"),
            border_radius: Styled::style("tooltip.border-radius"),
            border_width: Styled::style("tooltip.border-width"),
            border_color: Styled::style("tooltip.border-color"),
        }
    }
}

#[doc(hidden)]
pub struct TooltipState {
    pub paragraph: Paragraph,
    pub timer: f32,
    pub position: Point,
    pub style: TooltipStyle,
}

impl<T, V: View<T>> View<T> for Tooltip<V> {
    type State = (TooltipState, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_class("tooltip");

        let style = TooltipStyle::styled(self, cx.styles());

        let mut state = TooltipState {
            paragraph: Paragraph::new(style.line_height, style.align, style.wrap),
            timer: 0.0,
            position: Point::ZERO,
            style,
        };

        state.paragraph.set_text(
            &self.text,
            FontAttributes {
                size: state.style.font_size,
                family: state.style.font_family.clone(),
                stretch: state.style.font_stretch,
                weight: state.style.font_weight,
                style: state.style.font_style,
                ligatures: true,
                color: state.style.color,
            },
        );

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
        state.style.rebuild(self, cx);

        state.paragraph.line_height = state.style.line_height;
        state.paragraph.align = state.style.align;
        state.paragraph.wrap = state.style.wrap;

        state.paragraph.set_text(
            &self.text,
            FontAttributes {
                size: state.style.font_size,
                family: state.style.font_family.clone(),
                stretch: state.style.font_stretch,
                weight: state.style.font_weight,
                style: state.style.font_style,
                ligatures: true,
                color: state.style.color,
            },
        );

        (self.content).rebuild(content, cx, data, &old.content);
    }

    fn event(
        &mut self,
        (state, content): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        let handled = self.content.event(content, cx, data, event);

        if !content.has_hovered() && state.timer > 0.0 {
            state.timer = 0.0;
            cx.draw();
        }

        match event {
            Event::WindowResized(_) => {
                cx.layout();

                handled
            }
            Event::PointerMoved(e) => {
                if state.timer > 0.0 {
                    state.timer = 0.0;
                    cx.draw();
                }

                if content.has_hovered() {
                    state.position = e.position;
                    cx.animate();
                }

                handled
            }
            Event::Animate(dt) => {
                if content.has_hovered() && state.timer < 1.0 {
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

                handled
            }
            _ => handled,
        }
    }

    fn layout(
        &mut self,
        (_state, content): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
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
        let text_size = cx.fonts().measure(&state.paragraph, window_rect.width());

        let size = text_size + state.style.padding.size();
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

                cx.paragraph(
                    &state.paragraph,
                    Rect::min_size(state.style.padding.offset().to_point(), text_size),
                );
            });
        });
    }
}
