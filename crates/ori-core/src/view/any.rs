use std::any::Any;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
};

use super::View;

/// The state of a [`BoxedView`].
pub type AnyState = Box<dyn Any>;
/// A boxed dynamic view.
pub type BoxedView<T> = Box<dyn AnyView<T>>;

/// Create a new [`BoxedView`].
///
/// This is useful for when you need to create a view that needs
/// to change its type dynamically.
///
/// ```compile_fail
/// # use ori_core::{views::*, view::View};
/// // this will fail to compile because the type of each branch is different
/// fn ui(data: &mut bool) -> impl View<bool> {
///     if *data {
///         button(text("True"))
///     } else {
///         text("False")
///     }
/// }
/// ```
///
/// ```no_run
/// # use ori_core::{views::*, view::{View, any}};
/// // whereas this will compile using `any`
/// fn ui(data: &mut bool) -> impl View<bool> {
///     if *data {
///         any(button(text("True")))
///     } else {
///         any(text("False"))
///     }
/// }
/// ```
pub fn any<'a, T>(view: impl AnyView<T> + 'a) -> Box<dyn AnyView<T> + 'a> {
    Box::new(view)
}

/// A view that supports dynamic dispatch.
pub trait AnyView<T> {
    /// Get a reference to the underlying [`Any`] object.
    fn as_any(&self) -> &dyn Any;

    /// Build the view.
    fn dyn_build(&mut self, cx: &mut BuildCx, data: &mut T) -> Box<dyn Any>;

    /// Rebuild the view.
    fn dyn_rebuild(
        &mut self,
        state: &mut AnyState,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &dyn AnyView<T>,
    );

    /// Handle an event.
    fn dyn_event(
        &mut self,
        state: &mut AnyState,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool;

    /// Calculate the layout.
    fn dyn_layout(
        &mut self,
        state: &mut AnyState,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size;

    /// Draw the view.
    fn dyn_draw(&mut self, state: &mut AnyState, cx: &mut DrawCx, data: &mut T);
}

impl<T, V> AnyView<T> for V
where
    V: View<T> + Any,
    V::State: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&mut self, cx: &mut BuildCx, data: &mut T) -> Box<dyn Any> {
        Box::new(self.build(cx, data))
    }

    fn dyn_rebuild(
        &mut self,
        state: &mut AnyState,
        cx: &mut RebuildCx,
        data: &mut T,
        old: &dyn AnyView<T>,
    ) {
        if let Some(old) = old.as_any().downcast_ref::<V>() {
            match state.downcast_mut::<V::State>() {
                Some(state) => self.rebuild(state, cx, data, old),
                None => eprintln!("Failed to downcast state"),
            }
        } else {
            *cx.view_state = Default::default();
            *state = self.dyn_build(&mut cx.as_build_cx(), data);
        }
    }

    fn dyn_event(
        &mut self,
        state: &mut AnyState,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        match state.downcast_mut::<V::State>() {
            Some(state) => self.event(state, cx, data, event),
            None => false,
        }
    }

    fn dyn_layout(
        &mut self,
        state: &mut AnyState,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        match state.downcast_mut::<V::State>() {
            Some(state) => self.layout(state, cx, data, space),
            None => space.min,
        }
    }

    fn dyn_draw(&mut self, state: &mut AnyState, cx: &mut DrawCx, data: &mut T) {
        match state.downcast_mut::<V::State>() {
            Some(state) => self.draw(state, cx, data),
            None => eprintln!("Failed to downcast state"),
        }
    }
}

impl<T> View<T> for BoxedView<T> {
    type State = AnyState;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.as_mut().dyn_build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        self.as_mut().dyn_rebuild(state, cx, data, old.as_ref());
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        self.as_mut().dyn_event(state, cx, data, event)
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.as_mut().dyn_layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.as_mut().dyn_draw(state, cx, data);
    }
}
