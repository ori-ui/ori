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
pub fn with_state<T, U, V>(
    build: impl Fn() -> U + 'static,
    view: impl FnMut(&mut T, &mut U) -> V + 'static,
) -> WithState<T, U, V>
where
    V: View<(T, U)>,
{
    WithState::new(build, view)
}

/// Create a new view unwrapping some state from the data.
///
/// This is equivalent to `focus(|(data, _state), lens| lens(data), view)`.
pub fn without_state<T, U, V: View<T>>(view: V) -> impl View<(T, U)> {
    focus(|(data, _state), lens| lens(data), view)
}

/// A view that stores some additional data.
pub struct WithState<T, U, V> {
    build: Box<dyn Fn() -> U>,
    #[allow(clippy::type_complexity)]
    view: Box<dyn FnMut(&mut T, &mut U) -> V>,
    theme: Theme,
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
            theme: Theme::snapshot(),
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
    type State = (Pod<V>, U, State<(T, U), V>);

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        Theme::with_global(&mut self.theme, || {
            let mut state = (self.build)();
            let mut view = Pod::new((self.view)(data, &mut state));

            let content = Self::data(&mut state, data, |data| view.build(cx, data));

            (view, state, content)
        })
    }

    fn rebuild(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut RebuildCx,
        data: &mut T,
        _old: &Self,
    ) {
        let mut new_view = Pod::new((self.view)(data, data_state));

        Theme::with_global(&mut self.theme, || {
            Self::data(data_state, data, |data| {
                new_view.rebuild(state, cx, data, view)
            });
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
        Theme::with_global(&mut self.theme, || {
            Self::data(data_state, data, |data| view.event(state, cx, data, event));
        });
    }

    fn layout(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        Theme::with_global(&mut self.theme, || {
            Self::data(data_state, data, |data| view.layout(state, cx, data, space))
        })
    }

    fn draw(
        &mut self,
        (view, data_state, state): &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        Theme::with_global(&mut self.theme, || {
            Self::data(data_state, data, |data| view.draw(state, cx, data, canvas));
        });
    }
}
