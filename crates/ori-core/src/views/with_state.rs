use std::{mem::ManuallyDrop, ptr};

use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// Create a new [`WithState`].
pub fn with_state<T, U, V>(
    build: impl Fn() -> U + 'static,
    view: impl FnMut(&mut T, &mut U) -> V + 'static,
) -> WithState<T, U, V>
where
    V: View<(T, U)>,
{
    WithState::new(build, view)
}

/// A view that stores some additional data.
pub struct WithState<T, U, V> {
    build: Box<dyn Fn() -> U>,
    #[allow(clippy::type_complexity)]
    view: Box<dyn FnMut(&mut T, &mut U) -> V>,
}

impl<T, U, V> WithState<T, U, V> {
    /// Create a new [`WithState`].
    pub fn new(
        build: impl Fn() -> U + 'static,
        view: impl FnMut(&mut T, &mut U) -> V + 'static,
    ) -> Self {
        Self {
            build: Box::new(build),
            view: Box::new(view),
        }
    }

    // NOTE: this seems incredibly dodgy and i don't know if it's safe
    fn data<O>(state: &mut U, data: &mut T, f: impl FnOnce(&mut (T, U)) -> O) -> O {
        let mut data_state = unsafe { ManuallyDrop::new((ptr::read(data), ptr::read(state))) };

        let result = f(&mut data_state);

        let (data_inner, state_inner) = ManuallyDrop::into_inner(data_state);

        unsafe {
            ptr::write(data, data_inner);
            ptr::write(state, state_inner);
        }

        result
    }
}

impl<T, U, V: View<(T, U)>> View<T> for WithState<T, U, V> {
    type State = (Content<V>, U, State<(T, U), V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let mut state = (self.build)();
        let mut view = Content::new((self.view)(data, &mut state));

        let view_state = Self::data(&mut state, data, |data| view.build(cx, data));

        (view, state, view_state)
    }

    fn rebuild(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        let mut new_view = Content::new((self.view)(data, data_state));

        Self::data(data_state, data, |data| {
            new_view.rebuild(state, cx, data, view)
        });

        *view = new_view;
    }

    fn event(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) {
        Self::data(data_state, data, |data| view.event(state, cx, data, event))
    }

    fn layout(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        Self::data(data_state, data, |data| view.layout(state, cx, data, space))
    }

    fn draw(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        Self::data(data_state, data, |data| view.draw(state, cx, data, canvas))
    }
}
