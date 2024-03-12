use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Flex`] view.
pub fn flex<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, false, content)
}

/// Create a new expanded [`Flex`] view.
pub fn expand<V>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, true, content)
}

/// A flexible view.
#[derive(Rebuild)]
pub struct Flex<V> {
    /// The content of the view.
    pub content: V,
    /// The flex value of the view.
    pub flex: f32,
    /// Whether the view is tight.
    pub tight: bool,
}

impl<V> Flex<V> {
    /// Create a new flexible view.
    pub fn new(flex: f32, tight: bool, content: V) -> Self {
        Self {
            content,
            flex,
            tight,
        }
    }
}

impl<T, V: View<T>> View<T> for Flex<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_flex(self.flex);
        cx.set_tight(self.tight);

        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        cx.set_flex(self.flex);
        cx.set_tight(self.tight);

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
