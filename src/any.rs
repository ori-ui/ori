use std::{
    any::{Any, TypeId},
    mem,
};

use crate::{Action, Element, Is, Message, Mut, View, ViewMarker};

/// Type erased [`View`].
pub trait AnyView<C, T, E>
where
    E: Element,
{
    /// Build type erased state.
    fn build(self: Box<Self>, cx: &mut C, data: &mut T) -> (E, AnyState<C, T, E>);

    /// Rebuild type erased state.
    fn rebuild(
        self: Box<Self>,
        element: E::Mut<'_>,
        state: &mut AnyState<C, T, E>,
        cx: &mut C,
        data: &mut T,
    );
}

impl<C, T, E, V> AnyView<C, T, E> for V
where
    E: Element,
    V: View<C, T>,
    V::State: 'static,
    V::Element: Is<C, E>,
{
    fn build(self: Box<Self>, cx: &mut C, data: &mut T) -> (E, AnyState<C, T, E>) {
        let (element, state) = V::build(*self, cx, data);
        let element = V::Element::upcast(cx, element);
        let state = AnyState::new::<V>(state);

        (element, state)
    }

    fn rebuild(
        self: Box<Self>,
        element: E::Mut<'_>,
        state: &mut AnyState<C, T, E>,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(view_state) = state.downcast_mut::<V::State>() {
            if let Ok(element) = V::Element::downcast_mut(element) {
                V::rebuild(*self, element, view_state, cx, data);
            }
        } else {
            let (new_element, new_state) = V::build(*self, cx, data);
            let old_element = V::Element::replace(cx, element, new_element);
            let old_state = mem::replace(state, AnyState::new::<V>(new_state));
            (state.teardown)(old_element, old_state, cx);
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct AnyState<C, T, E>
where
    E: Element,
{
    type_id:   TypeId,
    type_name: &'static str,
    state:     Box<dyn Any>,
    message:   fn(Mut<'_, E>, &mut Self, &mut C, &mut T, &mut Message) -> Action,
    teardown:  fn(E, Self, &mut C),
}

impl<C, T, E> AnyState<C, T, E>
where
    E: Element,
{
    fn new<V>(state: V::State) -> Self
    where
        V: View<C, T>,
        V::State: 'static,
        V::Element: Is<C, E>,
    {
        Self {
            type_id:   TypeId::of::<V::State>(),
            type_name: std::any::type_name::<V::State>(),
            state:     Box::new(state),
            message:   AnyState::<C, T, E>::message::<V>,
            teardown:  AnyState::<C, T, E>::teardown::<V>,
        }
    }

    fn message<V>(
        element: Mut<'_, E>,
        state: &mut Self,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action
    where
        V: View<C, T>,
        V::State: 'static,
        V::Element: Is<C, E>,
    {
        if let Some(state) = state.downcast_mut()
            && let Ok(element) = V::Element::downcast_mut(element)
        {
            V::message(element, state, cx, data, message)
        } else {
            Action::new()
        }
    }

    fn teardown<V>(element: E, state: Self, cx: &mut C)
    where
        V: View<C, T>,
        V::State: 'static,
        V::Element: Is<C, E>,
    {
        if let Ok(state) = state.downcast()
            && let Ok(element) = V::Element::downcast(element)
        {
            V::teardown(element, *state, cx);
        }
    }

    fn is<U>(&self) -> bool
    where
        U: 'static,
    {
        #[cfg(debug_assertions)]
        if crate::get_relaxed_type_check() {
            return self.type_name == std::any::type_name::<U>();
        }

        self.type_id == TypeId::of::<U>()
    }

    fn downcast<U>(self) -> Result<Box<U>, Self>
    where
        U: 'static,
    {
        if self.is::<U>() {
            let ptr = Box::into_raw(self.state) as *mut _ as *mut U;
            Ok(unsafe { Box::from_raw(ptr) })
        } else {
            Err(self)
        }
    }

    fn downcast_mut<U>(&mut self) -> Option<&mut U>
    where
        U: 'static,
    {
        if self.is::<U>() {
            let ptr = self.state.as_mut() as *mut _ as *mut U;
            Some(unsafe { &mut *ptr })
        } else {
            None
        }
    }
}

impl<'a, C, T, E> ViewMarker for Box<dyn AnyView<C, T, E> + 'a> {}
impl<'a, C, T, E> View<C, T> for Box<dyn AnyView<C, T, E> + 'a>
where
    E: Element,
{
    type Element = E;
    type State = AnyState<C, T, E>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        AnyView::build(self, cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        AnyView::rebuild(self, element, state, cx, data)
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        (state.message)(element, state, cx, data, message)
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        (state.teardown)(element, state, cx)
    }
}
