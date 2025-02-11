use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
    window::Cursor,
};

/// Create a new [`CursorSetter`].
pub fn cursor<V>(cursor: Cursor, content: V) -> CursorSetter<V> {
    CursorSetter::new(content, cursor)
}

/// A view that sets the cursor when hovered.
pub struct CursorSetter<V> {
    /// The content view.
    pub content: V,

    /// The cursor to set.
    pub cursor: Cursor,
}

impl<V> CursorSetter<V> {
    /// Create a new [`CursorSetter`].
    pub fn new(content: V, cursor: Cursor) -> Self {
        Self { content, cursor }
    }
}

impl<T, V: View<T>> View<T> for CursorSetter<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        if cx.is_hovered() {
            cx.set_cursor(Some(self.cursor));
        } else {
            cx.set_cursor(None);
        }

        self.content.event(state, cx, data, event)
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
        self.content.draw(state, cx, data)
    }
}
