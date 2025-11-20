use ike::AnyWidgetId;

use crate::{Context, View};

pub fn window<V>(content: V) -> Window<V> {
    Window { content }
}

pub struct Window<V> {
    content: V,
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

        let window = cx.app.create_window(content.upcast());

        let id = window.id();
        (ori::NoElement, (id, content, state))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        (_, content, state): &mut Self::State,
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
