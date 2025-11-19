use ike::Windows as _;

use crate::{Context, View};

pub struct Window<V> {
    content: V,
}

impl<V> ori::ViewMarker for Window<V> {}
impl<T, V> ori::View<Context, T> for Window<V>
where
    V: View<T>,
{
    type Element = ori::NoElement;
    type State = (ike::WindowId, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (element, state) = self.content.build(cx, data);

        let window = cx.windows.create();
        window.set_content(element);

        let id = window.id();
        (ori::NoElement, (id, state))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _state: Self::State,
        _cx: &mut Context,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> ori::Action {
        ori::Action::new()
    }
}
