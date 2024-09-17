use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// Create a new [`Trigger`] view.
pub fn trigger<V>(view: V) -> Trigger<V> {
    Trigger::new(view)
}

/// A view that creates a trigger around the content.
pub struct Trigger<V> {
    /// The content.
    pub content: V,
}

impl<V> Trigger<V> {
    /// Create a new [`Trigger`].
    pub fn new(content: V) -> Self {
        Self { content }
    }
}

impl<T, V: View<T>> View<T> for Trigger<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.hoverable(|cx| {
            self.content.draw(state, cx, data);
        });
    }
}
