use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, State, View, ViewContent},
};

/// Create a new [`Flex`].
pub fn flex<T, V: View<T>>(flex: f32, content: V) -> Flex<V> {
    Flex::new(flex, content)
}

/// A flexible view.
///
/// When used in a stack, will shrink or grow to fill the remaining space.
pub struct Flex<V> {
    /// The content.
    pub content: V,
    /// The flex.
    pub flex: f32,
}

impl<V> Flex<V> {
    /// Create a new [`Flex`].
    pub fn new(flex: f32, content: V) -> Self {
        Self { content, flex }
    }
}

impl<T, V: View<T>> View<T> for Flex<V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build_content(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        if self.flex != old.flex {
            cx.request_layout();
        }

        self.content.rebuild_content(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.content.event_content(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        cx.set_flex(self.flex);

        self.content.layout_content(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw_content(state, cx, data, canvas);
    }
}
