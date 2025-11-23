use ike::{AnyWidgetId, Color, Size, WindowSizing};
use ori::ProviderContext;

use crate::{Context, Palette, View};

pub fn window<V>(content: V) -> Window<V> {
    Window::new(content)
}

pub struct Window<V> {
    content:   V,
    title:     String,
    sizing:    WindowSizing,
    visible:   bool,
    decorated: bool,
    color:     Option<Color>,
}

impl<V> Window<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            title: String::new(),
            sizing: WindowSizing::Resizable {
                default_size: Size::new(800.0, 600.0),
                min_size:     Size::all(0.0),
                max_size:     Size::all(f32::INFINITY),
            },
            visible: true,
            decorated: true,
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn sizing(mut self, sizing: WindowSizing) -> Self {
        self.sizing = sizing;
        self
    }

    pub fn fit_content(mut self) -> Self {
        self.sizing = WindowSizing::FitContent;
        self
    }
}

impl<V> ori::ViewMarker for Window<V> {}
impl<T, V> ori::View<Context, T> for Window<V>
where
    V: View<T>,
{
    type Element = ori::NoElement;
    type State = (ike::WindowId, V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (content, state) = self.content.build(cx, data);

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let window = cx.app.create_window(content.upcast());

        window.title = self.title.clone();
        window.sizing = self.sizing;
        window.visible = self.visible;
        window.decorated = self.decorated;
        window.color = self.color.unwrap_or(palette.background);

        let id = window.id();
        (ori::NoElement, (id, content, state))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        (id, content, state): &mut Self::State,
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

        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();

        if let Some(window) = cx.app.get_window_mut(*id) {
            window.title = self.title.clone();
            window.sizing = self.sizing;
            window.visible = self.visible;
            window.decorated = self.decorated;
            window.color = self.color.unwrap_or(palette.background);
        }
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (window, _, _): Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        cx.app.remove_window(window);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (_, content, state): &mut Self::State,
        cx: &mut Context,
        data: &mut T,
        event: &mut ori::Event,
    ) -> ori::Action {
        self.content.event(content, state, cx, data, event)
    }
}
