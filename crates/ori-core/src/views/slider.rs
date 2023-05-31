use glam::Vec2;
use ori_graphics::{Quad, Rect};
use ori_macro::Build;
use ori_reactive::{Event, OwnedSignal};
use ori_style::Style;

use crate::{
    AvailableSpace, Axis, Context, DrawContext, EventContext, LayoutContext, PointerEvent, View,
};

/// A slider view.
#[derive(Clone, Debug, Build)]
pub struct Slider {
    /// The value of the slider.
    #[bind]
    #[prop]
    pub value: OwnedSignal<f32>,
    /// Whether the slider is draggable.
    #[prop]
    pub draggable: bool,
    /// The minimum value of the slider.
    #[prop]
    pub min: f32,
    /// The maximum value of the slider.
    #[prop]
    pub max: f32,
    /// The step of the slider.
    #[prop]
    pub step: f32,
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            value: OwnedSignal::new(0.0),
            draggable: true,
            min: 0.0,
            max: 1.0,
            step: 0.0,
        }
    }
}

impl Slider {
    /// Creates a new slider.
    pub fn new() -> Self {
        Self::default()
    }

    fn track_rect(cx: &mut impl Context) -> Rect {
        let axis = cx.style::<Axis>("direction");

        let length = axis.major(cx.rect().size());
        let size = cx.style_range("track-size", 0.0..length);

        Rect::center_size(cx.rect().center(), axis.pack(length, size))
    }

    fn fill_rect(&self, cx: &mut impl Context) -> Rect {
        let axis = cx.style::<Axis>("direction");

        let value = self.value.get();
        let range = self.max - self.min;
        let t = f32::clamp((value - self.min) / range, 0.0, 1.0);

        let track_rect = Self::track_rect(cx);
        let track_size = axis.minor(track_rect.size());
        let knob_size = cx.style_range("knob-size", 0.0..track_size);

        let padding = f32::max(knob_size - track_size, 0.0);
        let length = (axis.major(cx.rect().size()) - padding * 2.0) * t + padding;

        Rect::min_size(
            track_rect.min,
            axis.pack(length, axis.minor(track_rect.size())),
        )
    }

    fn knob_rect(&self, cx: &mut impl Context) -> Rect {
        let axis = cx.style::<Axis>("direction");

        let track_rect = Self::track_rect(cx);
        let fill_rect = self.fill_rect(cx);

        let size = cx.style_range("knob-size", 0.0..axis.minor(track_rect.size()));
        let size = axis.pack(size, size);

        let center = axis.pack(axis.major(fill_rect.max), axis.minor(track_rect.center()));

        Rect::center_size(center, size)
    }
}

impl View for Slider {
    type State = ();

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("slider")
    }

    fn event(&self, _: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if pointer_event.is_press() && cx.hovered() && self.draggable {
                cx.activate();
            } else if pointer_event.is_release() && cx.active() {
                cx.deactivate();
            }

            if cx.active() {
                let axis = cx.style::<Axis>("direction");
                let track_rect = Self::track_rect(cx);

                let track_size = axis.minor(track_rect.size());
                let knob_size = cx.style_range("knob-size", 0.0..track_size);
                let padding = f32::max(knob_size - track_size, 0.0);

                let length = axis.major(track_rect.size()) - padding * 2.0;
                let point = axis.major(pointer_event.position - track_rect.min) - padding;
                let t = f32::clamp(point / length, 0.0, 1.0);

                let value = self.min + t * (self.max - self.min);

                let value = if self.step > 0.0 {
                    (value / self.step).round() * self.step
                } else {
                    value
                };

                if value != self.value.get() {
                    self.value.set(value);
                    cx.request_redraw();
                }
            }
        }
    }

    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        let axis = cx.style::<Axis>("direction");
        let track_size = cx.style_range("track-size", 0.0..axis.minor(space.max));
        let knob_size = cx.style_range("knob-size", 0.0..axis.minor(space.max));
        let length = cx.style_range("length", 0.0..axis.major(space.max));

        let size = f32::max(track_size, knob_size);
        space.constrain(axis.pack(length, size))
    }

    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        let range = 0.0..cx.rect().size().min_element();

        let tl = "border-top-left-radius";
        let tr = "border-top-right-radius";
        let br = "border-bottom-right-radius";
        let bl = "border-bottom-left-radius";

        let tl = cx.style_range_group(&[tl, "border-radius"], range.clone());
        let tr = cx.style_range_group(&[tr, "border-radius"], range.clone());
        let br = cx.style_range_group(&[br, "border-radius"], range.clone());
        let bl = cx.style_range_group(&[bl, "border-radius"], range.clone());

        let track_quad = Quad {
            rect: Self::track_rect(cx),
            background: cx.style("background-color"),
            border_radius: [tl, tr, br, bl],
            border_width: cx.style_range("border-width", range.clone()),
            border_color: cx.style("border-color"),
        };
        cx.draw(track_quad);

        let fill_quad = Quad {
            rect: self.fill_rect(cx),
            background: cx.style("color"),
            border_radius: [tl, tr, br, bl],
            border_width: cx.style_range("border-width", range.clone()),
            border_color: cx.style("border-color"),
        };
        cx.draw(fill_quad);

        let tl = "knob-border-top-left-radius";
        let tr = "knob-border-top-right-radius";
        let br = "knob-border-bottom-right-radius";
        let bl = "knob-border-bottom-left-radius";

        let tl = cx.style_range_group(&[tl, "knob-border-radius"], range.clone());
        let tr = cx.style_range_group(&[tr, "knob-border-radius"], range.clone());
        let br = cx.style_range_group(&[br, "knob-border-radius"], range.clone());
        let bl = cx.style_range_group(&[bl, "knob-border-radius"], range.clone());

        let knob_rect = self.knob_rect(cx);
        let knob_quad = Quad {
            rect: knob_rect,
            background: cx.style_group(&["color", "knob-color"]),
            border_radius: [tl, tr, br, bl],
            border_width: cx.style_range("knob-border-width", range),
            border_color: cx.style("knob-border-color"),
        };
        cx.draw(knob_quad);
    }
}
