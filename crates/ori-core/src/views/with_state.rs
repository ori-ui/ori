use std::{mem::ManuallyDrop, ptr};

use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    theme::Theme,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

use super::focus;

/// Create a new [`WithState`].
pub fn with_state<T, S, V: View<(T, S)>>(
    build: impl Fn() -> S + 'static,
    view: impl FnMut(&mut T, &mut S) -> V + 'static,
) -> WithState<T, S, V> {
    WithState::new(build, view)
}

/// Create a new [`WithState`] using `S::default()`.
pub fn with_state_default<T, S: Default + 'static, V: View<(T, S)>>(
    view: impl FnMut(&mut T, &mut S) -> V + 'static,
) -> WithState<T, S, V> {
    with_state(Default::default, view)
}

/// Create a new view unwrapping some state from the data.
///
/// This is equivalent to `focus(|(data, _state), lens| lens(data), view)`.
pub fn without_state<T, S, V: View<T>>(view: V) -> impl View<(T, S)> {
    focus(|(data, _state), lens| lens(data), view)
}

/// Create a new view unwrapping some data from the state.
///
/// This is equivalent to `focus(|(_data, state), lens| lens(state), view)`.
pub fn without_data<T, S, V: View<S>>(view: V) -> impl View<(T, S)> {
    focus(|(_data, state), lens| lens(state), view)
}

/// A view that stores some additional data.
pub struct WithState<T, S, V> {
    build: Box<dyn Fn() -> S>,
    #[allow(clippy::type_complexity)]
    view: Box<dyn FnMut(&mut T, &mut S) -> V>,
    theme: Theme,
}

impl<T, S, V> WithState<T, S, V> {
    /// Create a new [`WithState`].
    pub fn new(
        build: impl Fn() -> S + 'static,
        view: impl FnMut(&mut T, &mut S) -> V + 'static,
    ) -> Self {
        Self {
            build: Box::new(build),
            view: Box::new(view),
            theme: Theme::snapshot(),
        }
    }

    // NOTE: this seems incredibly dodgy and i don't know if it's safe
    fn data<O>(state: &mut S, data: &mut T, f: impl FnOnce(&mut (T, S)) -> O) -> O {
        let mut data_state: ManuallyDrop<(T, S)> =
            unsafe { ManuallyDrop::new((ptr::read(data), ptr::read(state))) };

        let result = f(&mut data_state);

        unsafe {
            // note that we don't drop the data and state here
            // see ptr::write.
            let (data_inner, state_inner): (T, S) = ManuallyDrop::into_inner(data_state);
            ptr::write(data, data_inner);
            ptr::write(state, state_inner);
        }

        result
    }
}

impl<T, U, V: View<(T, U)>> View<T> for WithState<T, U, V> {
    type State = (Pod<V>, U, State<(T, U), V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let (mut view, mut state) = Theme::as_global(&mut self.theme, || {
            let mut state = (self.build)();
            let view = Pod::new((self.view)(data, &mut state));
            (view, state)
        });

        let content = Self::data(&mut state, data, |data| view.build(cx, data));

        (view, state, content)
    }

    fn rebuild(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        let mut new_view = Theme::as_global(&mut self.theme, || {
            // we need apply the global theme here
            Pod::new((self.view)(data, data_state))
        });

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
        Self::data(data_state, data, |data| view.event(state, cx, data, event));
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
        Self::data(data_state, data, |data| view.draw(state, cx, data, canvas));
    }
}
