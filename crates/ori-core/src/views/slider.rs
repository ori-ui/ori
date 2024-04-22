use ori_macro::Build;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color},
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
    fn style(style: &Styles) -> Self {
        let palette = style.palette();

        Self {
            axis: Axis::Horizontal,
            width: 10.0,
            length: 100.0,
            color: palette.primary,
            background: palette.surface_high,
            border_radius: BorderRadius::all(5.0),
            border_width: BorderWidth::all(0.0),
            border_color: palette.outline_variant,
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
    pub fn on_input(mut self, on_change: impl FnMut(&mut EventCx, &mut T, f32) + 'static) -> Self {
        self.on_input = Some(Box::new(on_change));
        self
    }
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
                    let value = value.clamp(0.0, 1.0);

                    if let Some(on_input) = &mut self.on_input {
                        on_input(cx, data, value);
                        cx.request_rebuild();
                    }

                    cx.set_active(true);
                }
            }
            Event::PointerMoved(e) => {
                if cx.is_active() {
                    let local = cx.local(e.position);
                    let value = self.axis.unpack(local).0 / self.length;
                    let value = value.clamp(0.0, 1.0);

                    if let Some(on_input) = &mut self.on_input {
                        on_input(cx, data, value);
                        cx.request_rebuild();
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

    fn draw(
        &mut self,
        _state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.set_hoverable(cx.id());

        canvas.draw_quad(
            cx.rect(),
            self.background,
            self.border_radius,
            self.border_width,
            self.border_color,
        );

        let (length, width) = self.axis.unpack(cx.size());
        let value = self.value.clamp(0.0, 1.0);

        let min_length = self.border_radius.max_element() * 2.0;
        let length = f32::max(length * value, min_length);
        let size = self.axis.pack(length, width);

        canvas.draw_quad(
            Rect::min_size(cx.rect().min, size),
            self.color,
            self.border_radius,
            self.border_width,
            self.border_color,
        );
    }
}
