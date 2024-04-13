use ori_macro::example;

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Canvas, Color, Curve},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Point, Size, Space},
    rebuild::Rebuild,
    style::{style, Style, Styles},
    transition::Transition,
    view::View,
};

/// Create a new [`Checkbox`].
pub fn checkbox(checked: bool) -> Checkbox {
    Checkbox::new(checked)
}

/// The style of a [`Checkbox`].
#[derive(Clone, Debug)]
pub struct CheckboxStyle {
    /// The transition of the checkbox.
    pub transition: Transition,
    /// The size of the checkbox.
    pub size: f32,
    /// The color of the checkbox.
    pub color: Color,
    /// The stroke width of the checkbox.
    pub stroke: f32,
    /// The background color.
    pub background: Background,
    /// The border radius.
    pub border_radius: BorderRadius,
    /// The border width.
    pub border_width: BorderWidth,
    /// The border color.
    pub border_color: Color,
}

impl Style for CheckboxStyle {
    fn style(style: &Styles) -> Self {
        Self {
            transition: Transition::ease(0.1),
            size: 24.0,
            color: style.palette().accent(),
            stroke: 2.0,
            background: Background::color(Color::TRANSPARENT),
            border_radius: BorderRadius::all(6.0),
            border_width: BorderWidth::all(2.0),
            border_color: style.palette().text_dark(),
        }
    }
}

/// A checkbox.
#[example(name = "checkbox", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Checkbox {
    /// Whether the checkbox is checked.
    #[rebuild(draw)]
    pub checked: bool,
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
    pub background: Background,
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

impl Checkbox {
    /// Create a new [`Checkbox`].
    pub fn new(checked: bool) -> Self {
        let style = style::<CheckboxStyle>();

        Self {
            checked,
            transition: style.transition,
            size: style.size,
            color: style.color,
            stroke: style.stroke,
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
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
    pub fn background(mut self, background: impl Into<Background>) -> Self {
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

impl<T> View<T> for Checkbox {
    type State = f32;

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {
        0.0
    }

    fn rebuild(&mut self, _t: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
    }

    fn event(&mut self, t: &mut Self::State, cx: &mut EventCx, _data: &mut T, event: &Event) {
        if cx.hot_changed() {
            cx.animate();
        }

        if let Event::Animate(dt) = event {
            let on = cx.is_hot() && !cx.is_active();
            if self.transition.step(t, on, *dt) {
                cx.animate();
            }

            cx.request_draw();
        }
    }

    fn layout(
        &mut self,
        _t: &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(Size::all(self.size))
    }

    fn draw(&mut self, t: &mut Self::State, cx: &mut DrawCx, _data: &mut T, canvas: &mut Canvas) {
        let bright = self.border_color.lighten(0.2);

        canvas.draw_quad(
            cx.rect(),
            self.background.clone(),
            self.border_radius,
            self.border_width,
            self.border_color.mix(bright, self.transition.get(*t)),
        );

        if self.checked {
            let mut curve = Curve::new();
            curve.push(Point::new(0.2, 0.5) * cx.size());
            curve.push(Point::new(0.4, 0.7) * cx.size());
            curve.push(Point::new(0.8, 0.3) * cx.size());

            canvas.draw(curve.stroke(self.stroke, self.color));
        }
    }
}
