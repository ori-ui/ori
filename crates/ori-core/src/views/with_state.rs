use std::{mem::ManuallyDrop, ptr};

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::Styles,
    view::{Pod, State, View},
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
    theme: Styles,
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
            theme: Styles::snapshot(),
        }
    }
}

impl<T, U, V: View<(T, U)>> View<T> for WithState<T, U, V> {
    type State = (Pod<V>, U, State<(T, U), V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let (mut view, mut state) = self.theme.as_context(|| {
            let mut state = (self.build)();
            let view = Pod::new((self.view)(data, &mut state));
            (view, state)
        });

        let content = with_data(&mut state, data, |data| view.build(cx, data));

        (view, state, content)
    }

    fn rebuild(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        let mut new_view = self.theme.as_context(|| {
            // we need apply the global theme here
            Pod::new((self.view)(data, data_state))
        });

        with_data(data_state, data, |data| {
            new_view.rebuild(state, cx, data, view);
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
        with_data(data_state, data, |data| view.event(state, cx, data, event));
    }

    fn layout(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        with_data(data_state, data, |data| view.layout(state, cx, data, space))
    }

    fn draw(&mut self, (view, data_state, state): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        with_data(data_state, data, |data| view.draw(state, cx, data));
    }
}

fn with_data<S, T, O>(state: &mut S, data: &mut T, f: impl FnOnce(&mut (T, S)) -> O) -> O {
    // convert the state and data to raw pointers
    let state_ptr = state as *mut S;
    let data_ptr = data as *mut T;

    // SAFETY: data_ptr and state_ptr are created from mutable references and are thus
    // - valid for reads
    // - aligned
    // - point to initialized values
    //
    // data_state is wrapped in ManuallyDrop to prevent it from being dropped since we don't own
    // the values pointed to by data_ptr and state_ptr. the ManuallyDrop is necessary to prevent
    // the case where `f` panics, which would lead to a double drop.
    let mut data_state = unsafe { ManuallyDrop::new((ptr::read(data_ptr), ptr::read(state_ptr))) };

    // here a mutable reference to data_state is created, as stated above, no other mutable
    // references to the values pointed to by data_ptr and state_ptr exist.
    let result = f(&mut data_state);

    // SAFETY: valid for the same reasons as above
    //
    // ptr::write moves the values and prevents them from being dropped
    unsafe {
        let (data_inner, state_inner): (T, S) = ManuallyDrop::into_inner(data_state);
        ptr::write(data_ptr, data_inner);
        ptr::write(state_ptr, state_inner);
    }

    result
}
