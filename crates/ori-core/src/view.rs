use std::{
    any::{self, Any},
    sync::Arc,
};

use glam::Vec2;

use crate::{
    BoxConstraints, Callback, DrawContext, Event, EventContext, LayoutContext, RequestRedrawEvent,
    SendSync, SharedSignal, Style,
};

pub trait IntoView {
    type View: View;

    fn into_view(self) -> Self::View;
}

impl<T: View> IntoView for T {
    type View = T;

    fn into_view(self) -> Self::View {
        self
    }
}

/// A [`View`] is a component that can be rendered to the screen.
#[allow(unused_variables)]
pub trait View: SendSync + 'static {
    /// The state of the view.
    type State: SendSync + 'static;

    /// Builds the state of the view.
    fn build(&self) -> Self::State;

    /// Returns the style of the view.
    fn style(&self) -> Style {
        Style::default()
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

#[cfg(feature = "multi-thread")]
type AnyViewState = Box<dyn Any + Send + Sync>;
#[cfg(not(feature = "multi-thread"))]
type AnyViewState = Box<dyn Any>;

/// A [`View`] that with an unknown state.
///
/// This is used to store a [`View`] in a [`Node`](crate::Node).
pub trait AnyView: SendSync {
    fn build(&self) -> AnyViewState;

    fn style(&self) -> Style;

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event);

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2;

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext);
}

impl<T: View> AnyView for T {
    fn build(&self) -> AnyViewState {
        Box::new(self.build())
    }

    fn style(&self) -> Style {
        self.style()
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

impl dyn AnyView {
    pub fn downcast_ref<T: AnyView>(&self) -> Option<&T> {
        if any::type_name::<T>() == any::type_name::<Self>() {
            // SAFETY: `T` and `Self` are the same type
            unsafe { Some(&*(self as *const dyn AnyView as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: AnyView>(&mut self) -> Option<&mut T> {
        if any::type_name::<T>() == any::type_name::<Self>() {
            // SAFETY: `T` and `Self` are the same type
            unsafe { Some(&mut *(self as *mut dyn AnyView as *mut T)) }
        } else {
            None
        }
    }
}

impl View for Box<dyn AnyView> {
    type State = AnyViewState;

    fn build(&self) -> Self::State {
        self.as_ref().build()
    }

    fn style(&self) -> Style {
        self.as_ref().style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.as_ref().event(state.as_mut(), cx, event);
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.as_ref().layout(state.as_mut(), cx, bc)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.as_ref().draw(state.as_mut(), cx);
    }
}

impl View for Arc<dyn AnyView> {
    type State = AnyViewState;

    fn build(&self) -> Self::State {
        self.as_ref().build()
    }

    fn style(&self) -> Style {
        self.as_ref().style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.as_ref().event(state.as_mut(), cx, event);
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.as_ref().layout(state.as_mut(), cx, bc)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.as_ref().draw(state.as_mut(), cx);
    }
}

/// When a view is wrapped in a signal, the view will be redrawn when the signal
/// changes.
impl<V: View + SendSync> View for SharedSignal<V> {
    type State = (Callback<'static, ()>, V::State);

    fn build(&self) -> Self::State {
        (Callback::default(), self.get_untracked().build())
    }

    fn style(&self) -> Style {
        self.get_untracked().style()
    }

    fn event(&self, (_, state): &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.get().event(state, cx, event);
    }

    fn layout(
        &self,
        (_, state): &mut Self::State,
        cx: &mut LayoutContext,
        bc: BoxConstraints,
    ) -> Vec2 {
        self.get().layout(state, cx, bc)
    }

    fn draw(&self, (callback, state): &mut Self::State, cx: &mut DrawContext) {
        // redraw when the signal changes
        let event_sink = cx.event_sink.clone();
        let recreated = cx.state.recreated.clone();
        *callback = Callback::new(move |&()| {
            event_sink.emit(RequestRedrawEvent);
            recreated.set(true);
        });

        self.emitter().subscribe_weak(callback.downgrade());
        self.get().draw(state, cx);
    }
}

impl View for () {
    type State = ();

    fn build(&self) -> Self::State {}

    fn event(&self, _state: &mut Self::State, _cx: &mut EventContext, _event: &Event) {}

    fn layout(
        &self,
        _state: &mut Self::State,
        _cx: &mut LayoutContext,
        bc: BoxConstraints,
    ) -> Vec2 {
        bc.min
    }

    fn draw(&self, _state: &mut Self::State, _cx: &mut DrawContext) {}
}
