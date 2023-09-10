use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Flex`] view.
pub fn flex<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, 1.0, content)
}

/// Create a new [`Flex`] view with a flexible grow value.
pub fn flex_grow<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, 0.0, content)
}

/// Create a new [`Flex`] view with a flexible shrink value.
pub fn flex_shrink<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(0.0, flex, content)
}

/// A flexible view.
#[derive(Rebuild)]
pub struct Flex<V> {
    /// The content of the view.
    pub content: V,
    /// The flex grow value of the view.
    pub grow: f32,
    /// The flex shrink value of the view.
    pub shrink: f32,
}

impl<V> Flex<V> {
    /// Create a new flexible view.
    pub fn new(grow: f32, shrink: f32, content: V) -> Self {
        Self {
            content,
            grow,
            shrink,
        }
    }
}

impl<T, V: View<T>> View<T> for Flex<V> {
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
        cx.set_flex_grow(self.grow);
        cx.set_flex_shrink(self.shrink);
        self.content.layout(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(state, cx, data, canvas);
    }
}
