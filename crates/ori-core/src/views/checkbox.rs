use ori_macro::{example, Build};

use crate::{
    canvas::{BorderRadius, BorderWidth, Color, Curve},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Point, Size, Space},
    rebuild::Rebuild,
    style::{Stylable, Style, StyleBuilder, Theme},
    transition::Transition,
    view::View,
};

/// Create a new [`Checkbox`].
pub fn checkbox(checked: bool) -> Checkbox {
    Checkbox::new(checked)
}

/// The style of a checkbox.
#[derive(Clone, Rebuild)]
pub struct CheckboxStyle {
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

impl Style for CheckboxStyle {
    fn default_style() -> StyleBuilder<Self> {
        StyleBuilder::new(|theme: &Theme| CheckboxStyle {
            transition: Transition::ease(0.1),
            size: 24.0,
            color: theme.primary,
            stroke: 2.0,
            background: Color::TRANSPARENT,
            border_radius: BorderRadius::all(6.0),
            border_width: BorderWidth::all(2.0),
            border_color: theme.outline,
        })
    }
}

/// A checkbox.
///
/// Can be styled using the [`CheckboxStyle`].
#[example(name = "checkbox", width = 400, height = 300)]
#[derive(Build, Rebuild)]
pub struct Checkbox {
    /// Whether the checkbox is checked.
    #[rebuild(draw)]
    pub checked: bool,

    /// The transition of the checkbox.
    pub transition: Option<Transition>,

    /// The size of the checkbox.
    pub size: Option<f32>,

    /// The color of the checkbox.
    pub color: Option<Color>,

    /// The stroke width of the checkbox.
    pub stroke: Option<f32>,

    /// The background color.
    pub background: Option<Color>,

    /// The border radius.
    pub border_radius: Option<BorderRadius>,

    /// The border width.
    pub border_width: Option<BorderWidth>,

    /// The border color.
    pub border_color: Option<Color>,
}

impl Checkbox {
    /// Create a new [`Checkbox`].
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            transition: None,
            size: None,
            color: None,
            stroke: None,
            background: None,
            border_radius: None,
            border_width: None,
            border_color: None,
        }
    }
}

impl Stylable for Checkbox {
    type Style = CheckboxStyle;

    fn style(&self, style: &Self::Style) -> Self::Style {
        CheckboxStyle {
            transition: self.transition.unwrap_or(style.transition),
            size: self.size.unwrap_or(style.size),
            color: self.color.unwrap_or(style.color),
            stroke: self.stroke.unwrap_or(style.stroke),
            background: self.background.unwrap_or(style.background),
            border_radius: self.border_radius.unwrap_or(style.border_radius),
            border_width: self.border_width.unwrap_or(style.border_width),
            border_color: self.border_color.unwrap_or(style.border_color),
        }
    }
}

impl<T> View<T> for Checkbox {
    type State = (CheckboxStyle, f32);

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.set_focusable(true);

        (self.style(cx.style()), 0.0)
    }

    fn rebuild(
        &mut self,
        (style, _): &mut Self::State,
        cx: &mut RebuildCx,
        _data: &mut T,
        old: &Self,
    ) {
        Rebuild::rebuild(self, cx, old);
        self.rebuild_style(cx, style);
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
                cx.draw();
            }
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
                true => cx.theme().info,
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
