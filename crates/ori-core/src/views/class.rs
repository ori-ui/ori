use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::Styles,
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
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        cx.context_mut::<Styles>().push_class(&self.name);
        self.content.rebuild(state, cx, data, &old.content);
        cx.context_mut::<Styles>().pop_class();
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        cx.context_mut::<Styles>().push_class(&self.name);
        let handled = self.content.event(state, cx, data, event);
        cx.context_mut::<Styles>().pop_class();
        handled
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        cx.context_mut::<Styles>().push_class(&self.name);
        let size = self.content.layout(state, cx, data, space);
        cx.context_mut::<Styles>().pop_class();
        size
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        cx.context_mut::<Styles>().push_class(&self.name);
        self.content.draw(state, cx, data);
        cx.context_mut::<Styles>().pop_class();
    }
}
