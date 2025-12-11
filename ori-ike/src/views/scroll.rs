use ike::{Axis, BorderWidth, BuildCx, CornerRadius, Padding, Paint, Transition};
use ori::ProviderContext;

use crate::{Context, Palette, View};

pub fn vscroll<V>(contents: V) -> Scroll<V> {
    Scroll::new(Axis::Vertical, contents)
}

pub fn hscroll<V>(contents: V) -> Scroll<V> {
    Scroll::new(Axis::Horizontal, contents)
}

#[derive(Clone, Debug)]
pub struct ScrollTheme {
    pub bar_width:          f32,
    pub bar_padding:        Padding,
    pub bar_border_width:   BorderWidth,
    pub bar_corner_radius:  CornerRadius,
    pub knob_corner_radius: CornerRadius,
    pub transition:         Transition,
    pub bar_border_paint:   Option<Paint>,
    pub bar_paint:          Option<Paint>,
    pub knob_paint:         Option<Paint>,
}

impl Default for ScrollTheme {
    fn default() -> Self {
        Self {
            bar_width:          16.0,
            bar_padding:        Padding::all(4.0),
            bar_border_width:   BorderWidth::all(1.0),
            bar_corner_radius:  CornerRadius::all(0.0),
            knob_corner_radius: CornerRadius::all(4.0),
            transition:         Transition::INSTANT,
            bar_border_paint:   None,
            bar_paint:          None,
            knob_paint:         None,
        }
    }
}

pub struct Scroll<V> {
    contents: V,
    axis:     Axis,

    bar_width:          Option<f32>,
    bar_padding:        Option<Padding>,
    bar_border_width:   Option<BorderWidth>,
    bar_corner_radius:  Option<CornerRadius>,
    knob_corner_radius: Option<CornerRadius>,
    transition:         Option<Transition>,
    bar_border_paint:   Option<Paint>,
    bar_paint:          Option<Paint>,
    knob_paint:         Option<Paint>,
}

impl<V> Scroll<V> {
    pub fn new(axis: Axis, contents: V) -> Self {
        Self {
            contents,
            axis,

            bar_width: None,
            bar_padding: None,
            bar_border_width: None,
            bar_corner_radius: None,
            knob_corner_radius: None,
            transition: None,
            bar_border_paint: None,
            bar_paint: None,
            knob_paint: None,
        }
    }

    pub fn bar_width(mut self, width: f32) -> Self {
        self.bar_width = Some(width);
        self
    }

    pub fn bar_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.bar_padding = Some(padding.into());
        self
    }

    pub fn bar_border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.bar_border_width = Some(border_width.into());
        self
    }

    pub fn bar_corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.bar_corner_radius = Some(corner_radius.into());
        self
    }

    pub fn knob_corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.knob_corner_radius = Some(corner_radius.into());
        self
    }

    pub fn bar_border_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.bar_border_paint = Some(paint.into());
        self
    }

    pub fn bar_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.bar_paint = Some(paint.into());
        self
    }

    pub fn knob_paint(mut self, paint: impl Into<Paint>) -> Self {
        self.knob_paint = Some(paint.into());
        self
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = Some(transition);
        self
    }

    fn get_bar_width(&self, theme: &ScrollTheme) -> f32 {
        self.bar_width.unwrap_or(theme.bar_width)
    }

    fn get_bar_padding(&self, theme: &ScrollTheme) -> Padding {
        self.bar_padding.unwrap_or(theme.bar_padding)
    }

    fn get_bar_border_width(&self, theme: &ScrollTheme) -> BorderWidth {
        self.bar_border_width.unwrap_or(theme.bar_border_width)
    }

    fn get_bar_corner_radius(&self, theme: &ScrollTheme) -> CornerRadius {
        self.bar_corner_radius.unwrap_or(theme.bar_corner_radius)
    }

    fn get_transition(&self, theme: &ScrollTheme) -> Transition {
        self.transition.unwrap_or(theme.transition)
    }

    fn get_bar_border_paint(&self, theme: &ScrollTheme, palette: &Palette) -> Paint {
        self.bar_border_paint.clone().unwrap_or_else(|| {
            theme
                .bar_border_paint
                .clone()
                .unwrap_or_else(|| Paint::from(palette.outline))
        })
    }

    fn get_bar_paint(&self, theme: &ScrollTheme, palette: &Palette) -> Paint {
        self.bar_paint.clone().unwrap_or_else(|| {
            theme
                .bar_paint
                .clone()
                .unwrap_or_else(|| Paint::from(palette.surface))
        })
    }

    fn get_knob_paint(&self, theme: &ScrollTheme, palette: &Palette) -> Paint {
        self.knob_paint.clone().unwrap_or_else(|| {
            theme
                .knob_paint
                .clone()
                .unwrap_or_else(|| Paint::from(palette.contrast))
        })
    }

    fn get_knob_corner_radius(&self, theme: &ScrollTheme) -> CornerRadius {
        self.knob_corner_radius.unwrap_or(theme.knob_corner_radius)
    }
}

impl<V> ori::ViewMarker for Scroll<V> {}
impl<T, V> ori::View<Context, T> for Scroll<V>
where
    V: View<T>,
{
    type Element = ike::WidgetId<ike::widgets::Scroll>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.contents.build(cx, data);

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<ScrollTheme>().cloned().unwrap_or_default();

        let mut widget = ike::widgets::Scroll::new(cx, element);

        let bar_width = self.get_bar_width(&theme);
        let bar_padding = self.get_bar_padding(&theme);
        let bar_border_width = self.get_bar_border_width(&theme);
        let bar_corner_radius = self.get_bar_corner_radius(&theme);
        let knob_corner_radius = self.get_knob_corner_radius(&theme);
        let transition = self.get_transition(&theme);
        let bar_border_paint = self.get_bar_border_paint(&theme, &palette);
        let bar_paint = self.get_bar_paint(&theme, &palette);
        let knob_paint = self.get_knob_paint(&theme, &palette);

        ike::widgets::Scroll::set_axis(&mut widget, self.axis);
        ike::widgets::Scroll::set_bar_width(&mut widget, bar_width);
        ike::widgets::Scroll::set_bar_padding(&mut widget, bar_padding);
        ike::widgets::Scroll::set_bar_border_width(&mut widget, bar_border_width);
        ike::widgets::Scroll::set_bar_corner_radius(&mut widget, bar_corner_radius);
        ike::widgets::Scroll::set_knob_corner_radius(&mut widget, knob_corner_radius);
        ike::widgets::Scroll::set_transition(&mut widget, transition);
        ike::widgets::Scroll::set_bar_border_paint(&mut widget, bar_border_paint);
        ike::widgets::Scroll::set_bar_paint(&mut widget, bar_paint);
        ike::widgets::Scroll::set_knob_paint(&mut widget, knob_paint);

        (widget.id(), (element, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            contents,
            state,
            cx,
            data,
            &mut old.contents,
        );

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<ScrollTheme>().cloned().unwrap_or_default();
        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Scroll::set_child(&mut widget, *contents);
        }

        if self.axis != old.axis {
            ike::widgets::Scroll::set_axis(&mut widget, self.axis);
        }

        if self.bar_width != old.bar_width {
            let bar_width = self.get_bar_width(&theme);
            ike::widgets::Scroll::set_bar_width(&mut widget, bar_width);
        }

        if self.bar_padding != old.bar_padding {
            let bar_padding = self.get_bar_padding(&theme);
            ike::widgets::Scroll::set_bar_padding(&mut widget, bar_padding);
        }

        if self.bar_border_width != old.bar_border_width {
            let bar_border_width = self.get_bar_border_width(&theme);
            ike::widgets::Scroll::set_bar_border_width(&mut widget, bar_border_width);
        }

        if self.bar_corner_radius != old.bar_corner_radius {
            let bar_corner_radius = self.get_bar_corner_radius(&theme);
            ike::widgets::Scroll::set_bar_corner_radius(&mut widget, bar_corner_radius);
        }

        if self.knob_corner_radius != old.knob_corner_radius {
            let knob_corner_radius = self.get_knob_corner_radius(&theme);
            ike::widgets::Scroll::set_knob_corner_radius(&mut widget, knob_corner_radius);
        }

        if self.transition != old.transition {
            let transition = self.get_transition(&theme);
            ike::widgets::Scroll::set_transition(&mut widget, transition);
        }

        if self.bar_border_paint != old.bar_border_paint {
            let bar_border_paint = self.get_bar_border_paint(&theme, &palette);
            ike::widgets::Scroll::set_bar_border_paint(&mut widget, bar_border_paint);
        }

        if self.bar_paint != old.bar_paint {
            let bar_paint = self.get_bar_paint(&theme, &palette);
            ike::widgets::Scroll::set_bar_paint(&mut widget, bar_paint);
        }

        if self.knob_paint != old.knob_paint {
            let knob_paint = self.get_knob_paint(&theme, &palette);
            ike::widgets::Scroll::set_knob_paint(&mut widget, knob_paint);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (contents, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(contents, state, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(contents, state, cx, data, event);

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Scroll::set_child(&mut widget, *contents);
        }

        action
    }
}
