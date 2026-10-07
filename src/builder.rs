use std::mem::ManuallyDrop;

use crate::{Action, Element, Is, Message, Mut, View, ViewMarker};

/// Marker view for types implementing [`Builder`].
pub trait BuilderMarker {}

/// Helper trait for implementing the builder pattern for [`View`]s.
pub trait Builder<C, T>: BuilderMarker {
    /// The [`Element`] of the built [`View`].
    type Element: Element;

    /// Build the [`View`] of this builder.
    fn build(self) -> impl View<C, T, Element: Is<C, Self::Element>>;
}

// this implementation is truly sinful, and has to be this way. `Builder::build` returns an `impl
// View` type which makes it impossible to reference, thus we have to avoid referencing it by using
// some black magic.

impl<V> ViewMarker for V where V: BuilderMarker {}
impl<C, T, B> View<C, T> for B
where
    B: Builder<C, T>,
{
    type Element = B::Element;
    type State = BuilderState<C, T, B>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view = self.build();
        BuilderState::<C, T, B>::new(view, cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let view = self.build();
        unsafe { BuilderState::<C, T, B>::rebuild(view, element, state.state, cx, data) };
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        unsafe { (state.message)(element, state.state, cx, data, message) }
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        let state = ManuallyDrop::new(state);
        unsafe { (state.teardown)(element, state.state, cx) };
    }
}

pub struct BuilderState<C, T, B>
where
    B: Builder<C, T>,
{
    state:    *mut u8,
    message:  unsafe fn(Mut<'_, B::Element>, *mut u8, &mut C, &mut T, &mut Message) -> Action,
    teardown: unsafe fn(B::Element, *mut u8, &mut C),
    drop:     unsafe fn(*mut u8),
}

impl<C, T, B> Drop for BuilderState<C, T, B>
where
    B: Builder<C, T>,
{
    fn drop(&mut self) {
        unsafe { (self.drop)(self.state) };
    }
}

impl<C, T, B> BuilderState<C, T, B>
where
    B: Builder<C, T>,
{
    fn new<V>(view: V, cx: &mut C, data: &mut T) -> (B::Element, Self)
    where
        V: View<C, T>,
        V::Element: Is<C, B::Element>,
    {
        let (element, state) = view.build(cx, data);

        let state = Self {
            state:    Box::into_raw(Box::new(state)).cast(),
            message:  |element, state, cx, data, message| {
                let state = unsafe { &mut *state.cast::<V::State>() };

                match V::Element::downcast_mut(element) {
                    Ok(element) => V::message(element, state, cx, data, message),
                    Err(_) => Action::new(),
                }
            },
            teardown: |element, state, cx| {
                let state = unsafe { Box::from_raw(state.cast()) };

                if let Ok(element) = V::Element::downcast(element) {
                    V::teardown(element, *state, cx)
                }
            },
            drop:     |ptr| unsafe {
                let _ = Box::from_raw(ptr.cast::<V::State>());
            },
        };

        (Is::upcast(cx, element), state)
    }

    unsafe fn rebuild<V>(
        view: V,
        element: Mut<'_, B::Element>,
        state: *mut u8,
        cx: &mut C,
        data: &mut T,
    ) where
        V: View<C, T>,
        V::Element: Is<C, B::Element>,
    {
        let state = unsafe { &mut *state.cast() };

        if let Ok(element) = V::Element::downcast_mut(element) {
            view.rebuild(element, state, cx, data);
        }
    }
}
