use std::any::{self, Any};

use glam::Vec2;

use crate::{
    BoxConstraints, DrawContext, Event, EventContext, LayoutContext, SharedSignal, StyleClasses,
};

/// A [`View`] is a component that can be rendered to the screen.
#[allow(unused_variables)]
pub trait View: 'static {
    /// The state of the view.
    type State: 'static;

    /// Builds the state of the view.
    fn build(&self) -> Self::State;

    /// Returns the element name of the view.
    fn element(&self) -> Option<&'static str> {
        None
    }

    /// Returns the classes of the view.
    fn classes(&self) -> StyleClasses {
        StyleClasses::new()
    }

    /// Handles an event.
    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {}

    /// Handle layout and returns the size of the view.
    ///
    /// This method should return a size that fits the [`BoxConstraints`].
    ///
    /// The default implementation returns the minimum size.
    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        bc.min
    }

    /// Draws the view.
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {}
}

/// A [`View`] that with an unknown state.
///
/// This is used to store a [`View`] in a [`Node`](crate::Node).
pub trait AnyView {
    fn build(&self) -> Box<dyn Any>;

    fn element(&self) -> Option<&'static str>;

    fn classes(&self) -> StyleClasses;

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event);

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2;

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext);
}

impl<T: View> AnyView for T {
    fn build(&self) -> Box<dyn Any> {
        Box::new(self.build())
    }

    fn element(&self) -> Option<&'static str> {
        self.element()
    }

    fn classes(&self) -> StyleClasses {
        self.classes()
    }

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event) {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.event(state, cx, event);
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
        }
    }

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.layout(state, cx, bc)
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
            bc.min
        }
    }

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext) {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.draw(state, cx);
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
        }
    }
}

/// When a view is wrapped in a signal, the view will be redrawn when the signal
/// changes.
impl<V: View> View for SharedSignal<V> {
    type State = V::State;

    fn build(&self) -> Self::State {
        self.get_untracked().build()
    }

    fn element(&self) -> Option<&'static str> {
        self.get().element()
    }

    fn classes(&self) -> StyleClasses {
        self.get().classes()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.get().event(state, cx, event);
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.get().layout(state, cx, bc)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        // redraw when the signal changes
        self.emitter().subscribe_weak(cx.request_redraw.clone());
        self.get().draw(state, cx);
    }
}
