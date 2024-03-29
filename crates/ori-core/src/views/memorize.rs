use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    style::Style,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Memo`].
pub fn memo<T, V, D: PartialEq>(
    data: impl FnOnce(&mut T) -> D + 'static,
    build: impl FnOnce(&mut T) -> V + 'static,
) -> Memo<T, V, D> {
    Memo::new(data, build)
}

/// A view that only builds the inner view when certain data changes.
pub struct Memo<T, V, D> {
    #[allow(clippy::type_complexity)]
    data: Option<Box<dyn FnOnce(&mut T) -> D>>,
    #[allow(clippy::type_complexity)]
    build: Option<Box<dyn FnOnce(&mut T) -> V>>,
    theme: Style,
}

impl<T, V, D: PartialEq> Memo<T, V, D> {
    /// Create a new [`Memo`].
    pub fn new(
        data: impl FnOnce(&mut T) -> D + 'static,
        build: impl FnOnce(&mut T) -> V + 'static,
    ) -> Self {
        Self {
            data: Some(Box::new(data)),
            build: Some(Box::new(build)),
            theme: Style::snapshot(),
        }
    }

    fn data(&mut self, data: &mut T) -> D {
        (self.theme).as_context(|| (self.data.take().expect("Memo::data called twice"))(data))
    }

    fn build(&mut self, data: &mut T) -> V {
        (self.theme).as_context(|| (self.build.take().expect("Memo::build called twice"))(data))
    }
}

#[doc(hidden)]
pub struct MemoState<T, V: View<T>, D> {
    view: V,
    state: V::State,
    data: D,
}

impl<T, V: View<T>, D: PartialEq> View<T> for Memo<T, V, D> {
    type State = MemoState<T, V, D>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut view = self.build(data);
        let state = view.build(cx, data);
        let data = self.data(data);

        MemoState { view, state, data }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, _old: &Self) {
        let new_data = self.data(data);
        if new_data != state.data {
            let mut view = self.build(data);
            view.rebuild(&mut state.state, cx, data, &state.view);

            state.view = view;
            state.data = new_data;
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        state.view.event(&mut state.state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        state.view.layout(&mut state.state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        state.view.draw(&mut state.state, cx, data, canvas);
    }
}
