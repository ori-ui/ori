use glam::Vec2;

use crate::{
    canvas::{BorderRadius, BorderWidth, Canvas, Color, Curve},
    event::{Event, PointerEvent},
    layout::{Size, Space},
    rebuild::Rebuild,
    style::{checkbox, style},
    transition::Transition,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Checkbox`].
pub fn checkbox<T>(
    checked: bool,
    on_press: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> Checkbox<T> {
    Checkbox::new(checked, on_press)
}

/// A checkbox.
#[derive(Rebuild)]
pub struct Checkbox<T> {
    /// Whether the checkbox is checked.
    #[rebuild(draw)]
    pub checked: bool,
    /// The callback for when the checkbox is pressed.
    #[allow(clippy::type_complexity)]
    pub on_press: Box<dyn FnMut(&mut EventCx, &mut T)>,
    /// The transition of the checkbox.
    #[rebuild(draw)]
    pub transition: Transition,
    /// The size of the checkbox.
    #[rebuild(layout)]
    pub size: f32,
    /// The color of the checkbox.
    #[rebuild(draw)]
    pub color: Color,
    /// The stroke width of the checkbox.
    #[rebuild(draw)]
    pub stroke: f32,
    /// The background color.
    #[rebuild(draw)]
    pub background: Color,
    /// The border radius.
    #[rebuild(draw)]
    pub border_radius: BorderRadius,
    /// The border width.
    #[rebuild(draw)]
    pub border_width: BorderWidth,
    /// The border color.
    #[rebuild(draw)]
    pub border_color: Color,
}

impl<T> Checkbox<T> {
    /// Create a new [`Checkbox`].
    pub fn new(checked: bool, on_press: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        Self {
            checked,
            on_press: Box::new(on_press),
            transition: style(checkbox::TRANSITION),
            size: style(checkbox::SIZE),
            color: style(checkbox::COLOR),
            stroke: style(checkbox::STROKE),
            background: style(checkbox::BACKGROUND),
            border_radius: style(checkbox::BORDER_RADIUS),
            border_width: style(checkbox::BORDER_WIDTH),
            border_color: style(checkbox::BORDER_COLOR),
        }
    }

    /// Set the transition of the checkbox.
    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = transition;
        self
    }

    /// Set the size of the checkbox.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Set the color of the checkbox.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the stroke width of the checkbox.
    pub fn stroke(mut self, stroke: f32) -> Self {
        self.stroke = stroke;
        self
    }

    /// Set the background color of the checkbox.
    pub fn background(mut self, background: impl Into<Color>) -> Self {
        self.background = background.into();
        self
    }

    /// Set the border radius of the checkbox.
    pub fn border_radius(mut self, border_radius: impl Into<BorderRadius>) -> Self {
        self.border_radius = border_radius.into();
        self
    }

    /// Set the border width of the checkbox.
    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = border_width.into();
        self
    }

    /// Set the border color of the checkbox.
    pub fn border_color(mut self, border_color: impl Into<Color>) -> Self {
        self.border_color = border_color.into();
        self
    }
}

impl<T> View<T> for Checkbox<T> {
    type State = f32;

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {
        0.0
    }

    fn rebuild(&mut self, _t: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
    }

    fn event(&mut self, _t: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if event.is_handled() {
            return;
        }

        if let Some(pointer) = event.get::<PointerEvent>() {
            let local = cx.local(pointer.position);
            let over = cx.rect().contains(local) && !pointer.left;

            if cx.set_hot(over) {
                cx.request_draw();
            }

            if over && pointer.is_press() {
                (self.on_press)(cx, data);

                cx.set_active(true);
                cx.request_rebuild();
                cx.request_draw();

                event.handle();
            } else if cx.is_active() && pointer.is_release() {
                cx.set_active(false);
                cx.request_draw();

                event.handle();
            }
        }
    }

    fn layout(
        &mut self,
        _t: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(Size::splat(self.size))
    }

    fn draw(&mut self, t: &mut Self::State, cx: &mut DrawCx, _data: &mut T, canvas: &mut Canvas) {
        if (self.transition).step(t, cx.is_hot() || self.checked, cx.dt()) {
            cx.request_draw();
        }

        let bright = self.border_color.brighten(0.2);

        canvas.draw_quad(
            cx.rect(),
            self.background,
            self.border_radius,
            self.border_width,
            self.border_color.mix(bright, self.transition.on(*t)),
        );

        if self.checked {
            let mut curve = Curve::new();
            curve.add_point(Vec2::new(0.2, 0.5) * cx.size());
            curve.add_point(Vec2::new(0.4, 0.7) * cx.size());
            curve.add_point(Vec2::new(0.8, 0.3) * cx.size());

            canvas.draw(curve.stroke(self.stroke, self.color));
        }
    }
}
