use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::{Pod, State, View},
};

/// Build a view from a closure.
///
/// This allows you to access the [`BuildCx`] while building the view.
pub fn build<T, V, F>(builder: F) -> impl View<T>
where
    V: View<T>,
    F: FnOnce(&mut BuildCx, &mut T) -> V,
{
    Builder::new(builder)
}

/// A view that is built from a closure.
pub struct Builder<F> {
    builder: Option<F>,
}

impl<F> Builder<F> {
    /// Create a new builder view.
    pub fn new(builder: F) -> Self {
        Self {
            builder: Some(builder),
        }
    }
}

impl<T, V, F> View<T> for Builder<F>
where
    V: View<T>,
    F: FnOnce(&mut BuildCx, &mut T) -> V,
{
    type State = (Pod<V>, State<T, V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let builder = self.builder.take().unwrap();

        let mut view = Pod::new(builder(cx, data));
        let state = view.build(cx, data);

        (view, state)
    }

    fn rebuild(
        &mut self,
        (view, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        if let Some(builder) = self.builder.take() {
            let new_view = builder(&mut cx.as_build_cx(), data);
            let mut new_view = Pod::new(new_view);
            new_view.rebuild(state, cx, data, view);
        }
    }

    fn event(
        &mut self,
        (view, state): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        view.event(state, cx, data, event)
    }

    fn layout(
        &mut self,
        (view, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        view.layout(state, cx, data, space)
    }

    fn draw(&mut self, (view, state): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        view.draw(state, cx, data)
    }
}
