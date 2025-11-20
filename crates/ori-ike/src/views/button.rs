use ike::{AnyWidgetId, BorderWidth, BuildCx, Color, CornerRadius, Padding};
use ori::{AsyncContext, ProviderContext, Proxy};

use crate::{Context, Palette, View};

pub fn button<T, A, V>(content: V, on_click: impl FnMut(&mut T) -> A + 'static) -> Button<T, V>
where
    A: ori::IntoAction,
    V: View<T>,
{
    Button::new(content, on_click)
}

#[derive(Clone, Debug)]
pub struct ButtonTheme {
    pub padding:       Padding,
    pub border_width:  BorderWidth,
    pub corner_radius: CornerRadius,
    pub color:         Option<Color>,
    pub hovered_color: Option<Color>,
    pub active_color:  Option<Color>,
    pub border_color:  Option<Color>,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            padding:       Padding::all(8.0),
            border_width:  BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            color:         None,
            hovered_color: None,
            active_color:  None,
            border_color:  None,
        }
    }
}

pub struct Button<T, V> {
    content:  V,
    on_click: Box<dyn FnMut(&mut T) -> ori::Action>,

    padding:       Option<Padding>,
    border_width:  Option<BorderWidth>,
    corner_radius: Option<CornerRadius>,
    color:         Option<Color>,
    hovered_color: Option<Color>,
    active_color:  Option<Color>,
    border_color:  Option<Color>,
}

impl<T, V> Button<T, V> {
    pub fn new<A>(content: V, mut on_click: impl FnMut(&mut T) -> A + 'static) -> Self
    where
        A: ori::IntoAction,
    {
        Button {
            content,
            on_click: Box::new(move |data| on_click(data).into_action()),
            padding: None,
            border_width: None,
            corner_radius: None,
            color: None,
            hovered_color: None,
            active_color: None,
            border_color: None,
        }
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn border_width(mut self, border_width: BorderWidth) -> Self {
        self.border_width = Some(border_width);
        self
    }

    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.corner_radius = Some(corner_radius);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
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

    fn get_padding(&self, theme: &ButtonTheme) -> Padding {
        self.padding.unwrap_or(theme.padding)
    }

    fn get_border_width(&self, theme: &ButtonTheme) -> BorderWidth {
        self.border_width.unwrap_or(theme.border_width)
    }

    fn get_corner_radius(&self, theme: &ButtonTheme) -> CornerRadius {
        self.corner_radius.unwrap_or(theme.corner_radius)
    }

    fn get_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.color
            .unwrap_or_else(|| theme.color.unwrap_or(palette.primary))
    }

    fn get_hovered_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.hovered_color.unwrap_or_else(|| {
            theme
                .hovered_color
                .unwrap_or_else(|| palette.primary.darken(0.05))
        })
    }

    fn get_active_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.active_color.unwrap_or_else(|| {
            theme
                .active_color
                .unwrap_or_else(|| palette.primary.darken(0.1))
        })
    }

    fn get_border_color(&self, theme: &ButtonTheme, palette: &Palette) -> Color {
        self.border_color
            .unwrap_or_else(|| theme.border_color.unwrap_or(palette.outline))
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
    type State = (ori::Key, V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (content, state) = self.content.build(cx, data);

        let element = ike::widgets::Button::new(cx, content.upcast());

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<ButtonTheme>().cloned().unwrap_or_default();

        let padding = Self::get_padding(self, &theme);
        let border_width = Self::get_border_width(self, &theme);
        let corner_radius = Self::get_corner_radius(self, &theme);
        let color = Self::get_color(self, &theme, &palette);
        let hovered_color = Self::get_hovered_color(self, &theme, &palette);
        let active_color = Self::get_active_color(self, &theme, &palette);
        let border_color = Self::get_border_color(self, &theme, &palette);

        ike::widgets::Button::set_padding(cx, element, padding);
        ike::widgets::Button::set_border_width(cx, element, border_width);
        ike::widgets::Button::set_corner_radius(cx, element, corner_radius);
        ike::widgets::Button::set_color(cx, element, color);
        ike::widgets::Button::set_hovered_color(cx, element, hovered_color);
        ike::widgets::Button::set_active_color(cx, element, active_color);
        ike::widgets::Button::set_border_color(cx, element, border_color);

        let key = ori::Key::next();
        let proxy = cx.proxy();

        ike::widgets::Button::set_on_click(cx, element, move |_| {
            proxy.event(ori::Event::new(
                ButtonEvent::Clicked,
                key,
            ));
        });

        (element, (key, content, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (_key, content, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            content,
            state,
            cx,
            data,
            &mut old.content,
        );

        if !cx.is_parent(&element, &content) {
            ike::widgets::Button::set_child(cx, *element, content);
        }

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<ButtonTheme>().cloned().unwrap_or_default();

        if self.padding != old.padding {
            let padding = Self::get_padding(self, &theme);
            ike::widgets::Button::set_padding(cx, *element, padding);
        }

        if self.border_width != old.border_width {
            let border_width = Self::get_border_width(self, &theme);
            ike::widgets::Button::set_border_width(cx, *element, border_width);
        }

        if self.corner_radius != old.corner_radius {
            let corner_radius = Self::get_corner_radius(self, &theme);
            ike::widgets::Button::set_corner_radius(cx, *element, corner_radius);
        }

        if self.color != old.color {
            let color = Self::get_color(self, &theme, &palette);
            ike::widgets::Button::set_color(cx, *element, color);
        }

        if self.hovered_color != old.hovered_color {
            let hovered_color = Self::get_hovered_color(self, &theme, &palette);
            ike::widgets::Button::set_hovered_color(cx, *element, hovered_color);
        }

        if self.active_color != old.active_color {
            let active_color = Self::get_active_color(self, &theme, &palette);
            ike::widgets::Button::set_active_color(cx, *element, active_color);
        }

        if self.border_color != old.border_color {
            let border_color = Self::get_border_color(self, &theme, &palette);
            ike::widgets::Button::set_border_color(cx, *element, border_color);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (_key, content, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(content, state, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (key, content, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.content.event(content, state, cx, data, event);

        if !cx.is_parent(&element, &content) {
            ike::widgets::Button::set_child(cx, *element, content);
        }

        match event.get_targeted(*key) {
            Some(ButtonEvent::Clicked) => action | (self.on_click)(data),
            None => action,
        }
    }
}
