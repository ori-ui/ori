use ori_macro::{example, Build, Styled};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color, Curve},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Point, Size, Space},
    rebuild::Rebuild,
    style::{Styled, Theme},
    transition::Transition,
    view::View,
};

/// Create a new [`Checkbox`].
pub fn checkbox(checked: bool) -> Checkbox {
    Checkbox::new(checked)
}

/// A checkbox.
///
/// Can be styled using the [`CheckboxStyle`].
#[example(name = "checkbox", width = 400, height = 300)]
#[derive(Styled, Build, Rebuild)]
pub struct Checkbox {
    /// Whether the checkbox is checked.
    #[rebuild(draw)]
    pub checked: bool,

    /// The transition of the checkbox.
    #[rebuild(draw)]
    #[styled(default = Transition::ease(0.1))]
    pub transition: Styled<Transition>,

    /// The size of the checkbox.
    #[rebuild(layout)]
    #[styled(default = 24.0)]
    pub size: Styled<f32>,

    /// The color of the checkbox.
    #[rebuild(draw)]
    #[styled(default -> Theme::PRIMARY or Color::BLUE)]
    pub color: Styled<Color>,

    /// The stroke width of the checkbox.
    #[rebuild(draw)]
    #[styled(default = 2.0)]
    pub stroke: Styled<f32>,

    /// The background color.
    #[rebuild(draw)]
    #[styled(default = Color::TRANSPARENT)]
    pub background: Styled<Color>,

    /// The border radius.
    #[rebuild(draw)]
    #[styled(default = BorderRadius::all(6.0))]
    pub border_radius: Styled<BorderRadius>,

    /// The border width.
    #[rebuild(draw)]
    #[styled(default = BorderWidth::all(2.0))]
    pub border_width: Styled<BorderWidth>,

    /// The border color.
    #[rebuild(draw)]
    #[styled(default -> Theme::OUTLINE or Color::BLACK)]
    pub border_color: Styled<Color>,
}

impl Checkbox {
    /// Create a new [`Checkbox`].
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            transition: Styled::style("checkbox.transition"),
            size: Styled::style("checkbox.size"),
            color: Styled::style("checkbox.color"),
            stroke: Styled::style("checkbox.stroke"),
            background: Styled::style("checkbox.background"),
            border_radius: Styled::style("checkbox.border-radius"),
            border_width: Styled::style("checkbox.border-width"),
            border_color: Styled::style("checkbox.border-color"),
        }
    }
}

impl<T> View<T> for Checkbox {
    type State = (CheckboxStyle, f32);

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.set_class("checkbox");
        cx.set_focusable(true);

        let style = CheckboxStyle::styled(self, cx.styles());
        (style, 0.0)
    }

    fn rebuild(
        &mut self,
        (style, _): &mut Self::State,
        cx: &mut RebuildCx,
        _data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        style.rebuild(self, cx);
    }

    fn event(
        &mut self,
        (style, t): &mut Self::State,
        cx: &mut EventCx,
        _data: &mut T,
        event: &Event,
    ) -> bool {
        if cx.focused_changed() {
            cx.draw();
        }

        if cx.hovered_changed() {
            cx.animate();
        }

        if let Event::Animate(dt) = event {
            let on = cx.is_hovered() && !cx.is_active();
            if style.transition.step(t, on, *dt) {
                cx.animate();
            }

            cx.draw();
        }

        false
    }

    fn layout(
        &mut self,
        (style, _t): &mut Self::State,
        _cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        space.fit(Size::all(style.size))
    }

    fn draw(&mut self, (style, t): &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.hoverable(|cx| {
            let bright = style.border_color.lighten(0.2);

            let border_color = match cx.is_focused() {
                true => cx.styles().get_or(Color::BLUE, &Theme::INFO),
                false => style.border_color.mix(bright, style.transition.get(*t)),
            };

            cx.quad(
                cx.rect(),
                style.background,
                style.border_radius,
                style.border_width,
                border_color,
            );

            if self.checked {
                let mut curve = Curve::new();
                curve.move_to(Point::new(0.2, 0.5) * cx.size());
                curve.line_to(Point::new(0.4, 0.7) * cx.size());
                curve.line_to(Point::new(0.8, 0.3) * cx.size());

                cx.stroke(curve, style.stroke, style.color);
            }
        });
    }
}
