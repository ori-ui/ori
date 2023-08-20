use std::any::Any;

use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
};

use super::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View};

/// The state of a [`BoxedView`].
pub type AnyState = Box<dyn Any>;
/// A boxed dynamic view.
pub type BoxedView<T> = Box<dyn AnyView<T>>;

/// Create a new [`BoxedView`].
///
/// This is useful for when you need to create a view that needs
/// to change its type dynamically.
pub fn any<T>(view: impl AnyView<T> + 'static) -> BoxedView<T> {
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
    fn dyn_event(&mut self, state: &mut AnyState, cx: &mut EventCx, data: &mut T, event: &Event);

    /// Calculate the layout.
    fn dyn_layout(
        &mut self,
        state: &mut AnyState,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size;

    /// Draw the view.
    fn dyn_draw(
        &mut self,
        state: &mut AnyState,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    );
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
            if let Some(state) = state.downcast_mut::<V::State>() {
                self.rebuild(state, cx, data, old);
            } else {
                eprintln!("Failed to downcast state");
            }
        } else {
            *state = self.dyn_build(&mut cx.build_cx(), data);
            *cx.view_state = Default::default();
        }
    }

    fn dyn_event(&mut self, state: &mut AnyState, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some(state) = state.downcast_mut::<V::State>() {
            self.event(state, cx, data, event);
        } else {
            eprintln!("Failed to downcast state");
        }
    }

    fn dyn_layout(
        &mut self,
        state: &mut AnyState,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if let Some(state) = state.downcast_mut::<V::State>() {
            self.layout(state, cx, data, space)
        } else {
            eprintln!("Failed to downcast state");
            Size::ZERO
        }
    }

    fn dyn_draw(
        &mut self,
        state: &mut AnyState,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some(state) = state.downcast_mut::<V::State>() {
            self.draw(state, cx, data, canvas);
        } else {
            eprintln!("Failed to downcast state");
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

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.as_mut().dyn_event(state, cx, data, event);
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

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.as_mut().dyn_draw(state, cx, data, canvas);
    }
}
