use std::f32::consts::PI;

use glam::Vec2;
use ily_graphics::{Curve, Quad, Rect};
use ily_macro::Build;

use crate::{
    BoxConstraints, Context, DrawContext, Event, EventContext, LayoutContext, PointerEvent,
    SharedSignal, Style, View,
};

#[derive(Clone, Debug, Build)]
pub struct Knob {
    #[bind]
    value: SharedSignal<f32>,
    #[prop]
    max: f32,
    #[prop]
    min: f32,
    #[prop]
    label: String,
}

impl Default for Knob {
    fn default() -> Self {
        Self {
            value: SharedSignal::new(0.0),
            max: 1.0,
            min: 0.0,
            label: String::new(),
        }
    }
}

impl View for Knob {
    type State = Option<f32>;

    fn build(&self) -> Self::State {
        None
    }

    fn style(&self) -> Style {
        Style::new("knob")
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            if pointer_event.is_press() && cx.hovered() {
                cx.activate();
            } else if pointer_event.is_release() && cx.active() {
                *state = None;
                cx.deactivate();
            }

            if cx.active() {
                if let Some(prev_position) = *state {
                    let delta = pointer_event.position.y - prev_position;
                    let range = self.max - self.min;
                    let delta = delta / cx.rect().size().y * range * 0.25;
                    let value = self.value.get();
                    let new_value = f32::clamp(*value - delta, self.min, self.max);
                    self.value.set(new_value);
                }

                *state = Some(pointer_event.position.y);
                cx.request_redraw();
            }
        }
    }

    fn layout(&self, _state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let size = cx.style_range("size", bc.min.max_element()..bc.max.min_element());
        bc.constrain(Vec2::splat(size))
    }

    fn draw(&self, _state: &mut Self::State, cx: &mut DrawContext) {
        let size = cx.rect().size().min_element();

        let diameter = size * 0.8;

        let circle = Quad {
            rect: Rect::center_size(cx.rect().center(), Vec2::splat(diameter)),
            background: cx.style("background-color"),
            border_radius: [diameter * 0.5; 4],
            border_width: cx.style_range("border-width", 0.0..diameter * 0.5),
            border_color: cx.style("border-color"),
        };

        cx.draw_primitive(circle);

        let curve = Curve::arc(cx.rect().center(), diameter * 0.65, -PI * 1.25, PI * 0.25);
        let mesh = curve.rounded_mesh(diameter * 0.075, cx.style("background-color"));
        cx.draw_primitive(mesh);

        let range = self.max - self.min;
        let value = self.value.get();
        let angle = (*value - self.min) / range;
        let angle = -PI * 1.25 + angle * PI * 1.5;

        let curve = Curve::arc(cx.rect().center(), diameter * 0.65, -PI * 1.25, angle);
        let mesh = curve.rounded_mesh(diameter * 0.075, cx.style("color"));
        cx.draw_primitive(mesh);

        let mut arm = Curve::new();
        arm.add_point(cx.rect().center());
        arm.add_point(cx.rect().center() + Vec2::new(angle.cos(), angle.sin()) * diameter * 0.65);

        let mesh = arm.rounded_mesh(diameter * 0.075, cx.style("color"));
        cx.draw_primitive(mesh);
    }
}
