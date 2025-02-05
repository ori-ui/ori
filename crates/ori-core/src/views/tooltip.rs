use ori_macro::example;
use smol_str::SmolStr;

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{pt, Padding, Point, Rect, Size, Space, Vector},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    text::{
        FontAttributes, FontFamily, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign,
        TextWrap,
    },
    view::{Pod, PodState, View},
};

/// Create a new [`Tooltip`] view.
pub fn tooltip<V>(view: V, text: impl Into<SmolStr>) -> Tooltip<V> {
    Tooltip::new(view, text)
}

/// The style of a [`Tooltip`].
#[derive(Clone, Rebuild)]
pub struct TooltipStyle {
    /// The delay before the tooltip is displayed.
    pub delay: f32,

    /// The padding of the text.
    #[rebuild(layout)]
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

    /// The color of text.
    #[rebuild(draw)]
    pub color: Color,

    /// The horizontal alignment of the text.
    pub align: TextAlign,

    /// The line height of the text.
    pub line_height: f32,

    /// The text wrap of the text.
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

impl Style for TooltipStyle {
    fn builder() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| Self {
            delay: 0.2,
            padding: Padding::all(4.0),
            font_size: pt(10.0),
            font_family: FontFamily::default(),
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: theme.contrast,
            align: TextAlign::Left,
            line_height: 1.2,
            wrap: TextWrap::None,
            background: theme.surface(2),
            border_radius: BorderRadius::all(4.0),
            border_width: BorderWidth::all(1.0),
            border_color: theme.outline,
        })
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
    #[style(default = 0.2)]
    pub delay: Option<f32>,

    /// The padding of the text.
    #[rebuild(layout)]
    #[style(default = Padding::all(4.0))]
    pub padding: Option<Padding>,

    /// The font size of the text.
    #[style(default = pt(10.0))]
    pub font_size: Option<f32>,

    /// The font family of the text.
    #[style(default)]
    pub font_family: Option<FontFamily>,

    /// The font weight of the text.
    #[style(default)]
    pub font_weight: Option<FontWeight>,

    /// The font stretch of the text.
    #[style(default)]
    pub font_stretch: Option<FontStretch>,

    /// The font style of the text.
    #[style(default)]
    pub font_style: Option<FontStyle>,

    /// The color of text.
    #[rebuild(draw)]
    #[style(default -> Theme::CONTRAST or Color::BLACK)]
    pub color: Option<Color>,

    /// The horizontal alignment of the text.
    #[style(default)]
    pub align: Option<TextAlign>,

    /// The line height of the text.
    #[style(default = 1.2)]
    pub line_height: Option<f32>,

    /// The text wrap of the text.
    #[style(default)]
    pub wrap: Option<TextWrap>,

    /// The background color of the text.
    #[rebuild(draw)]
    #[style(default -> Theme::SURFACE_HIGHER or Color::WHITE)]
    pub background: Option<Color>,

    /// The border radius of the text.
    #[rebuild(draw)]
    #[style(default = BorderRadius::all(4.0))]
    pub border_radius: Option<BorderRadius>,

    /// The border width of the text.
    #[rebuild(draw)]
    #[style(default = BorderWidth::all(1.0))]
    pub border_width: Option<BorderWidth>,

    /// The border color of the text.
    #[rebuild(draw)]
    #[style(default -> Theme::OUTLINE or Color::BLACK)]
    pub border_color: Option<Color>,
}

impl<V> Tooltip<V> {
    /// Create a new tooltip view.
    pub fn new(content: V, text: impl Into<SmolStr>) -> Self {
        Self {
            content: Pod::new(content),
            text: text.into(),
            delay: None,
            padding: None,
            font_size: None,
            font_family: None,
            font_weight: None,
            font_stretch: None,
            font_style: None,
            color: None,
            align: None,
            line_height: None,
            wrap: None,
            background: None,
            border_radius: None,
            border_width: None,
            border_color: None,
        }
    }
}

impl<V> Stylable for Tooltip<V> {
    type Style = TooltipStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        TooltipStyle {
            delay: self.delay.unwrap_or(style.delay),
            padding: self.padding.unwrap_or(style.padding),
            font_size: self.font_size.unwrap_or(style.font_size),
            font_family: (self.font_family.clone()).unwrap_or(style.font_family.clone()),
            font_weight: self.font_weight.unwrap_or(style.font_weight),
            font_stretch: self.font_stretch.unwrap_or(style.font_stretch),
            font_style: self.font_style.unwrap_or(style.font_style),
            color: self.color.unwrap_or(style.color),
            align: self.align.unwrap_or(style.align),
            line_height: self.line_height.unwrap_or(style.line_height),
            wrap: self.wrap.unwrap_or(style.wrap),
            background: self.background.unwrap_or(style.background),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            border_width: self.border_width.unwrap_or(style.border_width),
            border_color: self.border_color.unwrap_or(style.border_color),
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
    type State = (TooltipState, PodState<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let style = self.style(cx.style());

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
        self.rebuild_style(cx, &mut state.style);

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
                let old_timer = state.timer;

                if content.has_hovered() && state.timer < 1.0 {
                    state.timer += dt / state.style.delay;
                    cx.animate();
                }

                if let Some(pointer) = cx.window().pointers().first() {
                    state.position = pointer.position;
                }

                state.timer = f32::clamp(state.timer, 0.0, 1.0);

                if state.timer >= 0.9 && state.timer != old_timer {
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
