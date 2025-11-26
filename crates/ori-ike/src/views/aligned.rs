use ike::{AnyWidgetId, BuildCx};

use crate::{Context, View};

pub fn align<V>(x: f32, y: f32, contents: V) -> Aligned<V> {
    Aligned { contents, x, y }
}

pub fn top_left<V>(contents: V) -> Aligned<V> {
    align(0.0, 0.0, contents)
}

pub fn top<V>(contents: V) -> Aligned<V> {
    align(0.5, 0.0, contents)
}

pub fn top_right<V>(contents: V) -> Aligned<V> {
    align(1.0, 0.0, contents)
}

pub fn left<V>(contents: V) -> Aligned<V> {
    align(0.0, 0.5, contents)
}

pub fn center<V>(contents: V) -> Aligned<V> {
    align(0.5, 0.5, contents)
}

pub fn right<V>(contents: V) -> Aligned<V> {
    align(1.0, 0.5, contents)
}

pub fn bottom_left<V>(contents: V) -> Aligned<V> {
    align(0.0, 1.0, contents)
}

pub fn bottom<V>(contents: V) -> Aligned<V> {
    align(0.5, 1.0, contents)
}

pub fn bottom_right<V>(contents: V) -> Aligned<V> {
    align(1.0, 1.0, contents)
}

pub struct Aligned<V> {
    contents: V,
    x:        f32,
    y:        f32,
}

impl<V> ori::ViewMarker for Aligned<V> {}
impl<T, V> ori::View<Context, T> for Aligned<V>
where
    V: View<T>,
{
    type Element = ike::WidgetId<ike::widgets::Aligned>;
    type State = (V::Element, V::State);

    fn build(&mut self, cx: &mut Context, data: &mut T) -> (Self::Element, Self::State) {
        let (contents, state) = self.contents.build(cx, data);

        let element = ike::widgets::Aligned::new(cx, self.x, self.y, contents.upcast());

        (element.id(), (contents, state))
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

        let mut widget = cx.get_mut(*element);

        if !widget.is_child(*contents) {
            ike::widgets::Aligned::set_child(&mut widget, *contents);
        }

        if self.x != old.x || self.y != old.y {
            ike::widgets::Aligned::set_alignment(&mut widget, self.x, self.y);
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
            ike::widgets::Aligned::set_child(&mut widget, *contents);
        }

        action
    }
}
