use std::{
    any::{self, Any},
    sync::Arc,
};

use glam::Vec2;

use crate::{
    BoxConstraints, Callback, Context, DrawContext, Event, EventContext, Guard, LayoutContext,
    Lock, Lockable, OwnedSignal, RequestRedrawEvent, SendSync, Shared, Style,
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

pub struct ViewRef<V: View> {
    view: Shared<Lock<V>>,
    is_dirty: OwnedSignal<bool>,
}

impl<V: View> Clone for ViewRef<V> {
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            is_dirty: self.is_dirty.clone(),
        }
    }
}

impl<V: View> ViewRef<V> {
    pub fn new(view: V) -> Self {
        Self {
            view: Shared::new(Lock::new(view)),
            is_dirty: OwnedSignal::new(false),
        }
    }

    pub fn lock(&self) -> Guard<'_, V> {
        self.set_dirty();
        self.view.lock_mut()
    }

    pub fn lock_untracked(&self) -> Guard<'_, V> {
        self.view.lock_mut()
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }

    pub fn set_dirty(&self) {
        self.is_dirty.set(true);
    }

    pub fn clear_dirty(&self) {
        self.is_dirty.set(false);
    }
}

impl<V: View> View for ViewRef<V> {
    type State = V::State;

    fn build(&self) -> Self::State {
        self.lock_untracked().build()
    }

    fn style(&self) -> Style {
        self.lock_untracked().style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if self.is_dirty.get_untracked() {
            self.clear_dirty();
            cx.request_layout();
            cx.request_redraw();
        }

        self.lock_untracked().event(state, cx, event);
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        if self.is_dirty.get_untracked() {
            self.clear_dirty();
            cx.request_layout();
            cx.request_redraw();
        }

        self.lock_untracked().layout(state, cx, bc)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        if self.is_dirty.get_untracked() {
            self.clear_dirty();
            cx.request_layout();
            cx.request_redraw();
        }

        self.lock_untracked().draw(state, cx);
    }
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
impl<V: View + SendSync> View for OwnedSignal<Arc<V>> {
    type State = (Callback<'static, ()>, V::State);

    fn build(&self) -> Self::State {
        (Callback::default(), V::build(&self.get_untracked()))
    }

    fn style(&self) -> Style {
        V::style(&self.get_untracked())
    }

    fn event(&self, (_, state): &mut Self::State, cx: &mut EventContext, event: &Event) {
        V::event(&self.get_untracked(), state, cx, event);
    }

    fn layout(
        &self,
        (_, state): &mut Self::State,
        cx: &mut LayoutContext,
        bc: BoxConstraints,
    ) -> Vec2 {
        V::layout(&self.get_untracked(), state, cx, bc)
    }

    fn draw(&self, (callback, state): &mut Self::State, cx: &mut DrawContext) {
        // redraw when the signal changes
        let event_sink = cx.event_sink.clone();
        let recreated = cx.state.recreated.clone();
        *callback = Callback::new(move |&()| {
            event_sink.emit(RequestRedrawEvent);
            recreated.set(true);
        });

        if let Some(emitter) = self.emitter.get() {
            emitter.subscribe_weak(callback.downgrade());
        }

        V::draw(&self.get_untracked(), state, cx);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyView;

impl View for EmptyView {
    type State = ();

    fn build(&self) -> Self::State {}
}
