use ike::{AnyWidgetId, BuildCx};

use crate::{Context, View};

pub fn align<V>(x: f32, y: f32, content: V) -> Aligned<V> {
    Aligned { content, x, y }
}

pub fn center<V>(content: V) -> Aligned<V> {
    align(0.5, 0.5, content)
}

pub struct Aligned<V> {
    content: V,
    x:       f32,
    y:       f32,
}

impl<V> ori::ViewMarker for Aligned<V> {}
impl<T, V> ori::View<Context, T> for Aligned<V>
where
    V: View<T>,
{
    type Element = ike::WidgetId<ike::widgets::Aligned>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (content, state) = self.content.build(cx, data);

        let element = ike::widgets::Aligned::new(cx, self.x, self.y, content.upcast());

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
            ike::widgets::Aligned::set_child(cx, *element, *content);
        }

        if self.x != old.x || self.y != old.y {
            ike::widgets::Aligned::set_alignment(cx, *element, self.x, self.y);
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
            ike::widgets::Aligned::set_child(cx, *element, *content);
        }

        action
    }
}
