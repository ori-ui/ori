use std::f32::consts::PI;

use glam::Vec2;
use ori_graphics::{Curve, Quad, Rect};
use ori_macro::Build;
use ori_reactive::{Event, OwnedSignal};

use crate::{
    AvailableSpace, Context, DrawContext, EventContext, LayoutContext, PointerEvent, Style, View,
};

/// A knob.
#[derive(Clone, Debug, Build)]
pub struct Knob {
    #[bind]
    pub value: OwnedSignal<f32>,
    #[prop]
    pub min: f32,
    #[prop]
    pub max: f32,
}

impl Default for Knob {
    fn default() -> Self {
        Self {
            value: OwnedSignal::new(0.0),
            min: 0.0,
            max: 1.0,
        }
    }
}

impl View for Knob {
    type State = Option<Vec2>;

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
                    let delta = pointer_event.position - prev_position;
                    let delta = delta.x - delta.y;
                    let range = self.max - self.min;
                    let delta = delta / cx.rect().width() * range * 0.15;
                    let value = self.value.get();
                    let new_value = f32::clamp(value + delta, self.min, self.max);
                    if new_value != value {
                        self.value.set(new_value);
                    }
                }

                *state = Some(pointer_event.position);
                cx.request_redraw();
            }
        }
    }

    fn layout(&self, _: &mut Self::State, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        let size = cx.style_range("size", space.min.max_element()..space.max.min_element());
        space.constrain(Vec2::splat(size))
    }

    fn draw(&self, _: &mut Self::State, cx: &mut DrawContext) {
        let size = cx.rect().size().min_element();

        let diameter = size * 0.7;

        let center = Quad {
            rect: Rect::center_size(cx.rect().center(), Vec2::splat(diameter)),
            background: cx.style("background-color"),
            border_radius: [diameter * 0.5; 4],
            border_width: cx.style_range("border-width", 0.0..diameter * 0.5),
            border_color: cx.style("border-color"),
        };
        cx.draw(center);

        let ring_track =
            Curve::arc_center_angle(cx.rect().center(), diameter * 0.65, -PI * 1.25, PI * 0.25);
        let mesh = ring_track.stroke(diameter * 0.075, cx.style("background-color"));
        cx.draw(mesh);

        let range = self.max - self.min;
        let value = self.value.get();
        let t = f32::clamp((value - self.min) / range, 0.0, 1.0);
        let angle = -PI * 1.25 + t * PI * 1.5;

        let ring = Curve::arc_center_angle(cx.rect().center(), diameter * 0.65, -PI * 1.25, angle);
        let mesh = ring.stroke(diameter * 0.075, cx.style("color"));
        cx.draw(mesh);

        let mut arm = Curve::new();
        arm.add_point(cx.rect().center());
        arm.add_point(cx.rect().center() + Vec2::new(angle.cos(), angle.sin()) * diameter * 0.65);

        let mesh = arm.stroke(diameter * 0.075, cx.style("color"));
        cx.draw(mesh);
    }
}
