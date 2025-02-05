use std::ops::RangeInclusive;

use ori_macro::Build;

use crate::{
    canvas::{BorderRadius, BorderWidth, Color},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Axis, Rect, Size, Space},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    view::View,
};

/// Create a new [`Slider`].
pub fn slider<T>(value: f32) -> Slider<T> {
    Slider::new(value)
}

/// The style of a [`Slider`].
#[derive(Clone, Rebuild)]
pub struct SliderStyle {
    /// The length of the slider.
    pub length: f32,

    /// The width of the slider.
    pub width: f32,

    /// The background color of the slider.
    pub background: Color,

    /// The foreground color of the slider.
    pub color: Color,

    /// The border radius of the slider.
    pub border_radius: BorderRadius,

    /// The border width of the slider.
    pub border_width: BorderWidth,

    /// The border color of the slider.
    pub border_color: Color,
}

impl Style for SliderStyle {
    fn builder() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| Self {
            length: 100.0,
            width: 10.0,
            background: theme.surface(1),
            color: theme.primary,
            border_radius: BorderRadius::all(5.0),
            border_width: BorderWidth::all(0.0),
            border_color: theme.outline,
        })
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
    pub width: Option<f32>,

    /// The length of the slider.
    pub length: Option<f32>,

    /// The foreground color of the slider.
    pub color: Option<Color>,

    /// The background color of the slider.
    pub background: Option<Color>,

    /// The border radius of the slider.
    pub border_radius: Option<BorderRadius>,

    /// The border width of the slider.
    pub border_width: Option<BorderWidth>,

    /// The border color of the slider.
    pub border_color: Option<Color>,
}

impl<T> Slider<T> {
    /// Create a new [`Slider`].
    pub fn new(value: f32) -> Self {
        Self {
            value,
            range: 0.0..=1.0,
            on_input: None,
            axis: Axis::Horizontal,
            width: None,
            length: None,
            color: None,
            background: None,
            border_radius: None,
            border_width: None,
            border_color: None,
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

impl<T> Stylable for Slider<T> {
    type Style = SliderStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        SliderStyle {
            length: self.length.unwrap_or(style.length),
            width: self.width.unwrap_or(style.width),
            background: self.background.unwrap_or(style.background),
            color: self.color.unwrap_or(style.color),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            border_width: self.border_width.unwrap_or(style.border_width),
            border_color: self.border_color.unwrap_or(style.border_color),
        }
    }
}

impl<T> View<T> for Slider<T> {
    type State = SliderStyle;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        self.style(cx.style())
    }

    fn rebuild(&mut self, style: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
        self.rebuild_style(cx, style);
    }

    fn event(
        &mut self,
        style: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        match event {
            Event::PointerPressed(e) if cx.is_hovered() => {
                let local = cx.local(e.position);

                let value = self.axis.unpack(local).0 / style.length;
                let value = denormalize(value, &self.range);

                if let Some(on_input) = &mut self.on_input {
                    on_input(cx, data, value);
                }

                cx.set_active(true);

                true
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

                false
            }
            Event::PointerReleased(_) if cx.is_active() => {
                cx.set_active(false);

                true
            }
            _ => false,
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
