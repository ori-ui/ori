use std::marker::PhantomData;

use crate::{
    Canvas, DrawCx, Event, EventCx, LayoutCx, Pod, PodState, RebuildCx, Size, Space, View,
};

pub fn focus<T, U, V: View<U>>(
    focus: impl FnMut(&mut T) -> &mut U + 'static,
    content: V,
) -> Focus<T, U, V> {
    Focus::new(content, focus)
}

pub struct Focus<T, U, V> {
    content: Pod<U, V>,
    focus: Box<dyn FnMut(&mut T) -> &mut U>,
    marker: PhantomData<fn() -> T>,
}

impl<T, U, V> Focus<T, U, V> {
    pub fn new(content: V, focus: impl FnMut(&mut T) -> &mut U + 'static) -> Self {
        Self {
            content: Pod::new(content),
            focus: Box::new(focus),
            marker: PhantomData,
        }
    }
}

impl<T, U, V: View<U>> View<T> for Focus<T, U, V> {
    type State = PodState<U, V>;

    fn build(&self) -> Self::State {
        self.content.build()
    }

    fn rebuild(&mut self, cx: &mut RebuildCx, old: &Self, state: &mut Self::State) {
        self.content.rebuild(cx, &old.content, state);
    }

    fn event(&mut self, cx: &mut EventCx, state: &mut Self::State, data: &mut T, event: &Event) {
        let data = (self.focus)(data);
        self.content.event(cx, state, data, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx, state: &mut Self::State, space: Space) -> Size {
        self.content.layout(cx, state, space)
    }

    fn draw(&mut self, cx: &mut DrawCx, state: &mut Self::State, scene: &mut Canvas) {
        self.content.draw(cx, state, scene);
    }
}
