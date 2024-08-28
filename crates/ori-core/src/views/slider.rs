use std::ops::RangeInclusive;

use ori_macro::Build;

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Axis, Rect, Size, Space},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    view::View,
};

/// Create a new [`Slider`].
pub fn slider<T>(value: f32) -> Slider<T> {
    Slider::new(value)
}

/// The style of a slider.
#[derive(Clone, Debug)]
pub struct SliderStyle {
    /// The axis of the slider.
    pub axis: Axis,

    /// The width of the slider.
    pub width: f32,

    /// The length of the slider.
    pub length: f32,

    /// The foreground color of the slider.
    pub color: Color,

    /// The background color of the slider.
    pub background: Color,

    /// The border radius of the slider.
    pub border_radius: BorderRadius,

    /// The border width of the slider.
    pub border_width: BorderWidth,

    /// The border color of the slider.
    pub border_color: Color,
}

impl Style for SliderStyle {
    fn styled(style: &Styles) -> Self {
        let palette = style.palette();

        Self {
            axis: Axis::Horizontal,
            width: 10.0,
            length: 100.0,
            color: palette.primary,
            background: palette.surface_high,
            border_radius: BorderRadius::all(5.0),
            border_width: BorderWidth::all(0.0),
            border_color: palette.outline,
        }
    }
}

/// A slider.
///
/// Can be styled with a [`SliderStyle`].
#[derive(Build, Rebuild)]
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
    pub width: f32,

    /// The length of the slider.
    #[rebuild(layout)]
    pub length: f32,

    /// The foreground color of the slider.
    #[rebuild(draw)]
    pub color: Color,

    /// The background color of the slider.
    #[rebuild(draw)]
    pub background: Color,

    /// The border radius of the slider.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,

    /// The border width of the slider.
    #[rebuild(draw)]
    pub border_width: BorderWidth,

    /// The border color of the slider.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<T> Slider<T> {
    /// Create a new [`Slider`].
    pub fn new(value: f32) -> Self {
        Self::styled(value, style())
    }

    /// Create a new [`Slider`] with a style.
    pub fn styled(value: f32, style: SliderStyle) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            on_input: None,
            axis: style.axis,
            width: style.width,
            length: style.length,
            color: style.color,
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
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
    type State = ();

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {}

    fn rebuild(&mut self, _state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
    }

    fn event(&mut self, _state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        match event {
            Event::PointerPressed(e) => {
                let local = cx.local(e.position);

                if cx.is_hot() {
                    let value = self.axis.unpack(local).0 / self.length;
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
                    let value = self.axis.unpack(local).0 / self.length;
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
        _state: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        let size = self.axis.pack(self.length, self.width);
        space.fit(size)
    }

    fn draw(&mut self, _state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.hoverable(|cx| {
            cx.quad(
                cx.rect(),
                self.background,
                self.border_radius,
                self.border_width,
                self.border_color,
            );

            let (length, width) = self.axis.unpack(cx.size());
            let value = normalize(self.value, &self.range);

            let min_length = self.border_radius.max_element() * 2.0;
            let length = f32::max(length * value, min_length);
            let size = self.axis.pack(length, width);

            cx.quad(
                Rect::min_size(cx.rect().min, size),
                self.color,
                self.border_radius,
                self.border_width,
                self.border_color,
            );
        });
    }
}
