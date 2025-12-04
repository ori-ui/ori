use ike::{AnyWidgetId, BorderWidth, BuildCx, Color, CornerRadius, Padding, Transition};
use ori::{AsyncContext, ProviderContext, Proxy};

use crate::{Context, Palette, View};

pub fn button<T, A, V>(contents: V, on_click: impl FnMut(&mut T) -> A + 'static) -> Button<T, V>
where
    A: ori::IntoAction,
    V: View<T>,
{
    Button::new(contents, on_click)
}

#[derive(Clone, Debug)]
pub struct ButtonTheme {
    pub padding:       Padding,
    pub border_width:  BorderWidth,
    pub corner_radius: CornerRadius,
    pub idle_color:    Option<Color>,
    pub hovered_color: Option<Color>,
    pub active_color:  Option<Color>,
    pub border_color:  Option<Color>,
    pub focus_color:   Option<Color>,
    pub transition:    Transition,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            padding:       Padding::all(8.0),
            border_width:  BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            idle_color:    None,
            hovered_color: None,
            active_color:  None,
            border_color:  None,
            focus_color:   None,
            transition:    Transition::ease(0.1),
        }
    }
}

pub struct Button<T, V> {
    contents: V,
    on_click: Box<dyn FnMut(&mut T) -> ori::Action>,

    padding:       Option<Padding>,
    border_width:  Option<BorderWidth>,
    corner_radius: Option<CornerRadius>,
    idle_color:    Option<Color>,
    hovered_color: Option<Color>,
    active_color:  Option<Color>,
    border_color:  Option<Color>,
    focus_color:   Option<Color>,
    transition:    Option<Transition>,
}

impl<T, V> Button<T, V> {
    pub fn new<A>(contents: V, mut on_click: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        Button {
            contents,
            on_click: Box::new(move |data| on_click(data).into_action()),
            padding: None,
            border_width: None,
            corner_radius: None,
            idle_color: None,
            hovered_color: None,
            active_color: None,
            border_color: None,
            focus_color: None,
            transition: None,
        }
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn border_width(mut self, border_width: impl Into<BorderWidth>) -> Self {
        self.border_width = Some(border_width.into());
        self
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(corner_radius.into());
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.idle_color = Some(color);
        self
    }

    pub fn hovered_color(mut self, color: Color) -> Self {
        self.hovered_color = Some(color);
        self
    }

    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn focus_color(mut self, color: Color) -> Self {
        self.focus_color = Some(color);
        self
    }

    fn get_padding(&self, theme: &ButtonTheme) -> Padding {
        self.padding.unwrap_or(theme.padding)
    }

    fn get_border_width(&self, theme: &ButtonTheme) -> BorderWidth {
        self.border_width.unwrap_or(theme.border_width)
    }

    fn get_corner_radius(&self, theme: &ButtonTheme) -> CornerRadius {
        self.corner_radius.unwrap_or(theme.corner_radius)
    }

    fn get_idle_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.idle_color
            .unwrap_or_else(|| theme.idle_color.unwrap_or_else(|| palette.surface(1)))
    }

    fn get_hovered_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.hovered_color
            .unwrap_or_else(|| theme.hovered_color.unwrap_or_else(|| palette.surface(0)))
    }

    fn get_active_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.active_color
            .unwrap_or_else(|| theme.active_color.unwrap_or_else(|| palette.surface(-1)))
    }

    fn get_border_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.border_color
            .unwrap_or_else(|| theme.border_color.unwrap_or(palette.outline))
    }

    fn get_focus_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.focus_color
            .unwrap_or_else(|| theme.focus_color.unwrap_or(palette.info))
    }

    fn get_transition(&self, theme: &ButtonTheme) -> Transition {
        self.transition.unwrap_or(theme.transition)
    }
}

enum ButtonEvent {
    Clicked,
}

impl<T, V> ori::ViewMarker for Button<T, V> {}
impl<T, V> ori::View<Context, T> for Button<T, V>
where
    V: View<T>,
{
    type Element = ike::WidgetId<ike::widgets::Button>;
    type State = (ori::ViewId, V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<ButtonTheme>().cloned().unwrap_or_default();
        let proxy = cx.proxy();
        let id = ori::ViewId::next();

        let mut widget = ike::widgets::Button::new(cx, contents.upcast());

        let padding = self.get_padding(&theme);
        let border_width = self.get_border_width(&theme);
        let corner_radius = self.get_corner_radius(&theme);
        let idle_color = self.get_idle_color(&theme, &palette);
        let hovered_color = self.get_hovered_color(&theme, &palette);
        let active_color = self.get_active_color(&theme, &palette);
        let border_color = self.get_border_color(&theme, &palette);
        let focus_color = self.get_focus_color(&theme, &palette);
        let transition = self.get_transition(&theme);

        ike::widgets::Button::set_padding(&mut widget, padding);
        ike::widgets::Button::set_border_width(&mut widget, border_width);
        ike::widgets::Button::set_corner_radius(&mut widget, corner_radius);
        ike::widgets::Button::set_idle_color(&mut widget, idle_color);
        ike::widgets::Button::set_hovered_color(&mut widget, hovered_color);
        ike::widgets::Button::set_active_color(&mut widget, active_color);
        ike::widgets::Button::set_border_color(&mut widget, border_color);
        ike::widgets::Button::set_focus_color(&mut widget, focus_color);
        ike::widgets::Button::set_transition(&mut widget, transition);

        ike::widgets::Button::set_on_click(&mut widget, move || {
            proxy.event(ori::Event::new(
                ButtonEvent::Clicked,
                id,
            ));
        });

        (widget.id(), (id, contents, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (_id, contents, state): &mut Self::State,
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
        let theme = cx.get_context::<ButtonTheme>().cloned().unwrap_or_default();

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Button::set_child(&mut widget, *contents);
        }

        if self.padding != old.padding {
            let padding = self.get_padding(&theme);
            ike::widgets::Button::set_padding(&mut widget, padding);
        }

        if self.border_width != old.border_width {
            let border_width = self.get_border_width(&theme);
            ike::widgets::Button::set_border_width(&mut widget, border_width);
        }

        if self.corner_radius != old.corner_radius {
            let corner_radius = self.get_corner_radius(&theme);
            ike::widgets::Button::set_corner_radius(&mut widget, corner_radius);
        }

        if self.idle_color != old.idle_color {
            let idle_color = self.get_idle_color(&theme, &palette);
            ike::widgets::Button::set_idle_color(&mut widget, idle_color);
        }

        if self.hovered_color != old.hovered_color {
            let hovered_color = self.get_hovered_color(&theme, &palette);
            ike::widgets::Button::set_hovered_color(&mut widget, hovered_color);
        }

        if self.active_color != old.active_color {
            let active_color = self.get_active_color(&theme, &palette);
            ike::widgets::Button::set_active_color(&mut widget, active_color);
        }

        if self.border_color != old.border_color {
            let border_color = self.get_border_color(&theme, &palette);
            ike::widgets::Button::set_border_color(&mut widget, border_color);
        }

        if self.focus_color != old.focus_color {
            let focus_color = self.get_focus_color(&theme, &palette);
            ike::widgets::Button::set_focus_color(&mut widget, focus_color);
        }

        if self.transition != old.transition {
            let transition = self.get_transition(&theme);
            ike::widgets::Button::set_transition(&mut widget, transition);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (_id, contents, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.contents.teardown(contents, state, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (id, contents, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.contents.event(contents, state, cx, data, event);

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Button::set_child(&mut widget, *contents);
        }

        match event.get_targeted(*id) {
            Some(ButtonEvent::Clicked) => action | (self.on_click)(data),
            None => action,
        }
    }
}
