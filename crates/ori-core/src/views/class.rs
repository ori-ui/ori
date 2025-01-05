use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// Wrap a view in a class.
pub fn class<V>(name: impl ToString, view: V) -> Class<V> {
    Class::new(name, view)
}

/// A view styled as a class.
pub struct Class<V> {
    /// The content.
    pub content: V,

    /// The name of the class.
    pub name: String,
}

impl<V> Class<V> {
    /// Create a new [`Class`].
    pub fn new(name: impl ToString, content: V) -> Self {
        Self {
            content,
            name: name.to_string(),
        }
    }
}

impl<T, V: View<T>> View<T> for Class<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        cx.set_class(&self.name);
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
        self.content.draw(state, cx, data);
    }
}
