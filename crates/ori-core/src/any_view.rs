use std::any::Any;

use crate::{Canvas, DrawCx, Event, EventCx, LayoutCx, RebuildCx, Size, Space, View};

pub type AnyState = Box<dyn Any>;
pub type BoxedView<T> = Box<dyn AnyView<T>>;

pub fn any<T>(view: impl AnyView<T> + 'static) -> BoxedView<T> {
    Box::new(view)
}

pub trait AnyView<T> {
    fn as_any(&self) -> &dyn Any;

    fn dyn_build(&self) -> Box<dyn Any>;

    fn dyn_rebuild(&mut self, cx: &mut RebuildCx, old: &dyn AnyView<T>, state: &mut AnyState);

    fn dyn_event(&mut self, cx: &mut EventCx, state: &mut AnyState, data: &mut T, event: &Event);

    fn dyn_layout(&mut self, cx: &mut LayoutCx, state: &mut AnyState, space: Space) -> Size;

    fn dyn_draw(&mut self, cx: &mut DrawCx, state: &mut AnyState, canvas: &mut Canvas);
}

impl<T, V> AnyView<T> for V
where
    V: View<T> + Any,
    V::State: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_build(&self) -> Box<dyn Any> {
        Box::new(self.build())
    }

    fn dyn_rebuild(&mut self, cx: &mut RebuildCx, old: &dyn AnyView<T>, state: &mut AnyState) {
        if let Some(old) = old.as_any().downcast_ref::<V>() {
            if let Some(state) = state.downcast_mut::<V::State>() {
                self.rebuild(cx, old, state);
            } else {
                eprintln!("Failed to downcast state");
            }
        } else {
            *state = self.dyn_build();
            cx.request_layout();
            cx.request_draw();
        }
    }

    fn dyn_event(&mut self, cx: &mut EventCx, state: &mut AnyState, data: &mut T, event: &Event) {
        if let Some(state) = state.downcast_mut::<V::State>() {
            self.event(cx, state, data, event);
        } else {
            eprintln!("Failed to downcast state");
        }
    }

    fn dyn_layout(&mut self, cx: &mut LayoutCx, state: &mut AnyState, space: Space) -> Size {
        if let Some(state) = state.downcast_mut::<V::State>() {
            self.layout(cx, state, space)
        } else {
            eprintln!("Failed to downcast state");
            Size::ZERO
        }
    }

    fn dyn_draw(&mut self, cx: &mut DrawCx, state: &mut AnyState, canvas: &mut Canvas) {
        if let Some(state) = state.downcast_mut::<V::State>() {
            self.draw(cx, state, canvas);
        } else {
            eprintln!("Failed to downcast state");
        }
    }
}

impl<T> View<T> for BoxedView<T> {
    type State = AnyState;

    fn build(&self) -> Self::State {
        self.as_ref().dyn_build()
    }

    fn rebuild(&mut self, cx: &mut RebuildCx, old: &Self, state: &mut Self::State) {
        self.as_mut().dyn_rebuild(cx, old.as_ref(), state);
    }

    fn event(&mut self, cx: &mut EventCx, state: &mut Self::State, data: &mut T, event: &Event) {
        self.as_mut().dyn_event(cx, state, data, event);
    }

    fn layout(&mut self, cx: &mut LayoutCx, state: &mut Self::State, space: Space) -> Size {
        self.as_mut().dyn_layout(cx, state, space)
    }

    fn draw(&mut self, cx: &mut DrawCx, state: &mut Self::State, canvas: &mut Canvas) {
        self.as_mut().dyn_draw(cx, state, canvas);
    }
}
