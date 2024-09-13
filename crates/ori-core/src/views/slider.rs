use std::ops::RangeInclusive;

use ori_macro::{Build, Styled};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Axis, Rect, Size, Space},
    rebuild::Rebuild,
    style::{Styled, Theme},
    view::View,
};

/// Create a new [`Slider`].
pub fn slider<T>(value: f32) -> Slider<T> {
    Slider::new(value)
}

/// A slider.
///
/// Can be styled with a [`SliderStyle`].
#[derive(Styled, Build, Rebuild)]
pub struct Slider<T> {
    /// The value of the slider.
    #[rebuild(draw)]
    pub value: f32,

    /// The range of the slider.
    #[rebuild(draw)]
    pub range: RangeInclusive<f32>,

    /// The callback for when the value changes.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_input: Option<Box<dyn FnMut(&mut EventCx, &mut T, f32) + 'static>>,

    /// The axis of the slider.
    #[rebuild(layout)]
    pub axis: Axis,

    /// The width of the slider.
    #[rebuild(layout)]
    #[styled(default = 10.0)]
    pub width: Styled<f32>,

    /// The length of the slider.
    #[rebuild(layout)]
    #[styled(default = 100.0)]
    pub length: Styled<f32>,

    /// The foreground color of the slider.
    #[rebuild(draw)]
    #[styled(default -> Theme::PRIMARY or Color::BLUE)]
    pub color: Styled<Color>,

    /// The background color of the slider.
    #[rebuild(draw)]
    #[styled(default -> Theme::SURFACE_HIGH or Color::grayscale(0.9))]
    pub background: Styled<Color>,

    /// The border radius of the slider.
    #[rebuild(draw)]
    #[styled(default = BorderRadius::all(5.0))]
    pub border_radius: Styled<BorderRadius>,

    /// The border width of the slider.
    #[rebuild(draw)]
    #[styled(default = BorderWidth::all(0.0))]
    pub border_width: Styled<BorderWidth>,

    /// The border color of the slider.
    #[rebuild(draw)]
    #[styled(default -> Theme::OUTLINE or Color::BLACK)]
    pub border_color: Styled<Color>,
}

impl<T> Slider<T> {
    /// Create a new [`Slider`].
    pub fn new(value: f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            on_input: None,
            axis: Axis::Horizontal,
            width: SliderStyle::WIDTH.into(),
            length: SliderStyle::LENGTH.into(),
            color: SliderStyle::COLOR.into(),
            background: SliderStyle::BACKGROUND.into(),
            border_radius: SliderStyle::BORDER_RADIUS.into(),
            border_width: SliderStyle::BORDER_WIDTH.into(),
            border_color: SliderStyle::BORDER_COLOR.into(),
        }
    }

    /// Set the callback for when the value changes.
    pub fn on_input(mut self, on_input: impl FnMut(&mut EventCx, &mut T, f32) + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }
}

fn normalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
    let value = value.clamp(*range.start(), *range.end());
    (value - range.start()) / (range.end() - range.start())
}

fn denormalize(value: f32, range: &RangeInclusive<f32>) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * (range.end() - range.start()) + range.start()
}

impl<T> View<T> for Slider<T> {
    type State = SliderStyle;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        SliderStyle::styled(self, cx.styles())
    }

    fn rebuild(&mut self, style: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
        style.rebuild(self, cx);
    }

    fn event(&mut self, style: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        match event {
            Event::PointerPressed(e) => {
                let local = cx.local(e.position);

                if cx.is_hovered() {
                    let value = self.axis.unpack(local).0 / style.length;
                    let value = denormalize(value, &self.range);

                    if let Some(on_input) = &mut self.on_input {
                        on_input(cx, data, value);
                    }

                    cx.set_active(true);
                }
            }
            Event::PointerMoved(e) => {
                let local = cx.local(e.position);

                if cx.is_active() {
                    let value = self.axis.unpack(local).0 / style.length;
                    let value = denormalize(value, &self.range);

                    if let Some(on_input) = &mut self.on_input {
                        on_input(cx, data, value);
                    }
                }
            }
            Event::PointerReleased(_) => {
                if cx.is_active() {
                    cx.set_active(false);
                }
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        style: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        let size = self.axis.pack(style.length, style.width);
        space.fit(size)
    }

    fn draw(&mut self, style: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.hoverable(|cx| {
            cx.quad(
                cx.rect(),
                style.background,
                style.border_radius,
                style.border_width,
                style.border_color,
            );

            let (length, width) = self.axis.unpack(cx.size());
            let value = normalize(self.value, &self.range);

            let min_length = style.border_radius.max_element() * 2.0;
            let length = f32::max(length * value, min_length);
            let size = self.axis.pack(length, width);

            cx.quad(
                Rect::min_size(cx.rect().min, size),
                style.color,
                style.border_radius,
                style.border_width,
                style.border_color,
            );
        });
    }
}
