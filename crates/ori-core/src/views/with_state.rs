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
    styles: Styles,
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
            styles: Styles::snapshot(),
        }
    }
}

impl<T, U, V: View<(T, U)>> View<T> for WithState<T, U, V> {
    type State = (Pod<V>, U, State<(T, U), V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let (mut view, mut state) = self.styles.as_context(|| {
            let mut state = (self.build)();
            let view = Pod::new((self.view)(data, &mut state));
            (view, state)
        });

        let content = with_data_state(data, &mut state, |data| view.build(cx, data));

        (view, state, content)
    }

    fn rebuild(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        let mut new_view = self.styles.as_context(|| {
            // we need apply the styles here
            Pod::new((self.view)(data, data_state))
        });

        with_data_state(data, data_state, |data| {
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
        with_data_state(data, data_state, |data| view.event(state, cx, data, event));
    }

    fn layout(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        with_data_state(data, data_state, |data| view.layout(state, cx, data, space))
    }

    fn draw(&mut self, (view, data_state, state): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        with_data_state(data, data_state, |data| view.draw(state, cx, data));
    }
}

fn with_data_state<D, S, O>(data: &mut D, state: &mut S, f: impl FnOnce(&mut (D, S)) -> O) -> O {
    unsafe {
        let state_ptr = state as *mut S;
        let data_ptr = data as *mut D;

        let mut data_state = DataState {
            data_ptr,
            state_ptr,
            data_state: ManuallyDrop::new((ptr::read(data_ptr), ptr::read(state_ptr))),
        };

        f(&mut data_state.data_state)
    }
}

struct DataState<D, S> {
    data_ptr: *mut D,
    state_ptr: *mut S,
    data_state: ManuallyDrop<(D, S)>,
}

impl<D, S> Drop for DataState<D, S> {
    fn drop(&mut self) {
        unsafe {
            let (data, state) = ptr::read(&*self.data_state);
            ptr::write(self.data_ptr, data);
            ptr::write(self.state_ptr, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{panic, rc::Rc};

    use super::*;

    /// Test that `with_data_state` correctly reads the data and state.
    #[test]
    fn writes() {
        let mut data = Some(Box::new(42));
        let mut state = 69;

        with_data_state(&mut data, &mut state, |(data, state)| {
            assert_eq!(*data, Some(Box::new(42)));
            assert_eq!(*state, 69);

            *data = None;
            *state = 0;
        });

        assert_eq!(data, None);
        assert_eq!(state, 0);
    }

    /// Test that `with_data_state` correctly updates the data and state when the closure panics.
    #[test]
    fn writes_on_panic() {
        let data = Rc::new(());
        let state = Rc::new(());

        let _ = panic::catch_unwind({
            let mut data = Some(data.clone());
            let mut state = Some(state.clone());

            move || {
                with_data_state(&mut data, &mut state, |(data, state)| {
                    *data = None;
                    *state = None;
                    panic!();
                });
            }
        });

        assert_eq!(Rc::strong_count(&data), 1);
        assert_eq!(Rc::strong_count(&state), 1);
    }
}
