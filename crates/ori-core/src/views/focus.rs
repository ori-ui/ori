use std::marker::PhantomData;

use crate::{
    BuildCx, Canvas, DrawCx, Event, EventCx, LayoutCx, Pod, PodState, RebuildCx, Size, Space, View,
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

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let data = (self.focus)(data);
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        let data = (self.focus)(data);
        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        let data = (self.focus)(data);
        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let data = (self.focus)(data);
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T, scene: &mut Canvas) {
        let data = (self.focus)(data);
        self.content.draw(state, cx, data, scene);
    }
}
