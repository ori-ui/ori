use ike::{AnyWidgetId, BorderWidth, BuildCx, Color, CornerRadius, Padding};
use ori::ProviderContext;

use crate::{Context, Palette, View};

pub fn container<V>(content: V) -> Container<V> {
    Container::new(content)
}

#[derive(Clone, Debug)]
pub struct ContainerTheme {
    pub padding:          Padding,
    pub border_width:     BorderWidth,
    pub corner_radius:    CornerRadius,
    pub background_color: Option<Color>,
    pub border_color:     Option<Color>,
}

impl Default for ContainerTheme {
    fn default() -> Self {
        Self {
            padding:          Padding::all(8.0),
            border_width:     BorderWidth::all(1.0),
            corner_radius:    CornerRadius::all(8.0),
            background_color: None,
            border_color:     None,
        }
    }
}

pub struct Container<V> {
    content: V,

    padding:          Option<Padding>,
    border_width:     Option<BorderWidth>,
    corner_radius:    Option<CornerRadius>,
    background_color: Option<Color>,
    border_color:     Option<Color>,
}

impl<V> Container<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,

            padding: None,
            border_width: None,
            corner_radius: None,
            background_color: None,
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

    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    fn get_padding(&self, theme: &ContainerTheme) -> Padding {
        self.padding.unwrap_or(theme.padding)
    }

    fn get_border_width(&self, theme: &ContainerTheme) -> BorderWidth {
        self.border_width.unwrap_or(theme.border_width)
    }

    fn get_corner_radius(&self, theme: &ContainerTheme) -> CornerRadius {
        self.corner_radius.unwrap_or(theme.corner_radius)
    }

    fn get_background_color(&self, theme: &ContainerTheme, palette: &Palette) -> Color {
        self.background_color
            .unwrap_or_else(|| theme.background_color.unwrap_or(palette.surface))
    }

    fn get_border_color(&self, theme: &ContainerTheme, palette: &Palette) -> Color {
        self.border_color
            .unwrap_or_else(|| theme.border_color.unwrap_or(palette.outline))
    }
}

impl<V> ori::ViewMarker for Container<V> {}
impl<T, V> ori::View<Context, T> for Container<V>
where
    V: View<T>,
{
    type Element = ike::WidgetId<ike::widgets::Container>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (content, state) = self.content.build(cx, data);

        let element = ike::widgets::Container::new(cx, content.upcast());

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx
            .get_context::<ContainerTheme>()
            .cloned()
            .unwrap_or_default();

        let padding = Self::get_padding(self, &theme);
        let border_width = Self::get_border_width(self, &theme);
        let corner_radius = Self::get_corner_radius(self, &theme);
        let background = Self::get_background_color(self, &theme, &palette);
        let border_color = Self::get_border_color(self, &theme, &palette);

        ike::widgets::Container::set_padding(cx, element, padding);
        ike::widgets::Container::set_border_width(cx, element, border_width);
        ike::widgets::Container::set_corner_radius(cx, element, corner_radius);
        ike::widgets::Container::set_background(cx, element, background);
        ike::widgets::Container::set_border_color(cx, element, border_color);

        (element, (content, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (content, state): &mut Self::State,
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

        if !cx.is_parent(*element, *content) {
            ike::widgets::Container::set_child(cx, *element, *content);
        }

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx
            .get_context::<ContainerTheme>()
            .cloned()
            .unwrap_or_default();

        if self.padding != old.padding {
            let padding = Self::get_padding(self, &theme);
            ike::widgets::Container::set_padding(cx, *element, padding);
        }

        if self.border_width != old.border_width {
            let border_width = Self::get_border_width(self, &theme);
            ike::widgets::Container::set_border_width(cx, *element, border_width);
        }

        if self.corner_radius != old.corner_radius {
            let corner_radius = Self::get_corner_radius(self, &theme);
            ike::widgets::Container::set_corner_radius(cx, *element, corner_radius);
        }

        if self.background_color != old.background_color {
            let background = Self::get_background_color(self, &theme, &palette);
            ike::widgets::Container::set_background(cx, *element, background);
        }

        if self.border_color != old.border_color {
            let border_color = Self::get_border_color(self, &theme, &palette);
            ike::widgets::Container::set_border_color(cx, *element, border_color);
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (content, state): Self::State,
        cx: &mut Context,
        data: &mut T,
    ) {
        self.content.teardown(content, state, cx, data);
        cx.remove(element);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (content, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        let action = self.content.event(content, state, cx, data, event);

        if !cx.is_parent(*element, *content) {
            ike::widgets::Container::set_child(cx, *element, *content);
        }

        action
    }
}
