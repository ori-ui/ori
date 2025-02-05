use std::{mem::ManuallyDrop, ptr};

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::{Pod, PodState, View},
};

use super::focus;

/// Create a new [`WithState`].
///
/// # Example
/// ```rust
/// # use ori_core::{view::View, views::{button, on_click, text, with_state}};
/// struct Data {
///     // ...
/// }
///
/// fn ui() -> impl View<Data> {
///     with_state(
///         || 0,
///         |count, _data| {
///             on_click(
///                 button(text!("Clicked {} time(s)", count)),
///                 |cx, (count, _data)| {
///                     *count += 1;
///                     cx.rebuild();
///                 },
///             )
///         }
///     )
/// }
/// ```
pub fn with_state<S, T, V>(
    build: impl FnOnce() -> S + 'static,
    view: impl FnMut(&mut S, &mut T) -> V + 'static,
) -> WithState<S, T, V>
where
    V: View<(S, T)>,
{
    WithState::new(build, view)
}

/// Create a new [`WithState`] using `S::default()`.
///
/// # Example
/// ```rust
/// # use ori_core::{view::View, views::{button, on_click, text, with_state_default}};
/// struct Data {
///     // ...
/// }
///
/// fn ui() -> impl View<Data> {
///     with_state_default(
///         |count: &mut i32, _data| {
///             on_click(
///                 button(text!("Clicked {} time(s)", count)),
///                 |cx, (count, _data)| {
///                     *count += 1;
///                     cx.rebuild();
///                 },
///             )
///         }
///     )
/// }
/// ```
pub fn with_state_default<S, T, V>(
    view: impl FnMut(&mut S, &mut T) -> V + 'static,
) -> WithState<S, T, V>
where
    S: Default + 'static,
    V: View<(S, T)>,
{
    with_state(Default::default, view)
}

/// Create a new [`WithState`] that replaces the data with the state.
///
/// # Example
/// ```rust
/// # use ori_core::{view::View, views::{button, on_click, text, with_data}};
/// struct Data {
///     // ...
/// }
///
/// fn ui() -> impl View<Data> {
///     with_data(
///         || 0,
///         |count| {
///             on_click(
///                 button(text!("Clicked {} time(s)", count)),
///                 |cx, count| {
///                     *count += 1;
///                     cx.rebuild();
///                 },
///             )
///         }
///     )
/// }
/// ```
pub fn with_data<S, T, V>(
    build: impl FnOnce() -> S + 'static,
    mut view: impl FnMut(&mut S) -> V + 'static,
) -> impl View<T>
where
    V: View<S>,
{
    with_state(build, move |state, _| without_data(view(state)))
}

/// Create a new [`WithState`] that replaces the data with the state using `S::default()`.
///
/// # Example
/// ```rust
/// # use ori_core::{view::View, views::{button, on_click, text, with_data_default}};
/// struct Data {
///     // ...
/// }
///
/// fn ui() -> impl View<Data> {
///     with_data_default(
///         |count: &mut i32| {
///             on_click(
///                 button(text!("Clicked {} time(s)", count)),
///                 |cx, count| {
///                     *count += 1;
///                     cx.rebuild();
///                 },
///             )
///         }
///     )
/// }
/// ```
pub fn with_data_default<S, T, V>(view: impl Fn(&mut S) -> V + 'static) -> impl View<T>
where
    S: Default + 'static,
    V: View<S>,
{
    with_data(Default::default, view)
}

/// Create a new view unwrapping some state from the data.
///
/// This is equivalent to `focus(|(data, _state), lens| lens(data), view)`.
pub fn without_state<S, T, V>(view: V) -> impl View<(S, T)>
where
    V: View<T>,
{
    focus(view, |(_state, data), lens| lens(data))
}

/// Create a new view unwrapping some data from the state.
///
/// This is equivalent to `focus(|(_data, state), lens| lens(state), view)`.
pub fn without_data<S, T, V>(view: V) -> impl View<(S, T)>
where
    V: View<S>,
{
    focus(view, |(state, _data), lens| lens(state))
}

/// A view that stores some additional data.
pub struct WithState<S, T, V> {
    build: Option<Box<dyn FnOnce() -> S>>,
    #[allow(clippy::type_complexity)]
    view: Box<dyn FnMut(&mut S, &mut T) -> V>,
}

impl<S, T, V> WithState<S, T, V> {
    /// Create a new [`WithState`].
    pub fn new(
        build: impl FnOnce() -> S + 'static,
        view: impl FnMut(&mut S, &mut T) -> V + 'static,
    ) -> Self {
        Self {
            build: Some(Box::new(build)),
            view: Box::new(view),
        }
    }
}

impl<S, T, V: View<(S, T)>> View<T> for WithState<S, T, V> {
    type State = (Pod<V>, S, PodState<(S, T), V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let build = self.build.take().expect("Build should only be called once");
        let mut state = build();
        let mut view = Pod::new((self.view)(&mut state, data));

        let content = with_data_state(&mut state, data, |data| view.build(cx, data));

        (view, state, content)
    }

    fn rebuild(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        let mut new_view = Pod::new((self.view)(data_state, data));

        with_data_state(data_state, data, |data| {
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
    ) -> bool {
        with_data_state(data_state, data, |data| view.event(state, cx, data, event))
    }

    fn layout(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        with_data_state(data_state, data, |data| view.layout(state, cx, data, space))
    }

    fn draw(&mut self, (view, data_state, state): &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        with_data_state(data_state, data, |data| view.draw(state, cx, data));
    }
}

fn with_data_state<S, D, O>(state: &mut S, data: &mut D, f: impl FnOnce(&mut (S, D)) -> O) -> O {
    unsafe {
        let data_ptr = data as *mut D;
        let state_ptr = state as *mut S;

        let mut data_state = DataState {
            data_ptr,
            state_ptr,
            data_state: ManuallyDrop::new((ptr::read(state_ptr), ptr::read(data_ptr))),
        };

        f(&mut data_state.data_state)
    }
}

struct DataState<S, D> {
    state_ptr: *mut S,
    data_ptr: *mut D,
    data_state: ManuallyDrop<(S, D)>,
}

impl<S, D> Drop for DataState<S, D> {
    fn drop(&mut self) {
        unsafe {
            let (state, data) = ptr::read(&*self.data_state);
            ptr::write(self.state_ptr, state);
            ptr::write(self.data_ptr, data);
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
