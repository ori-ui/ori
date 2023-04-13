use std::any::{self, Any};

use glam::Vec2;

use crate::{
    BoxConstraints, DrawContext, Event, EventContext, LayoutContext, SharedSignal, StyleAttributes,
    StyleClasses,
};

pub struct ViewState<T> {
    pub state: T,
    pub element: Option<&'static str>,
    pub classes: StyleClasses,
    pub attributes: StyleAttributes,
}

impl<T> ViewState<T> {
    pub fn new(state: T, element: Option<&'static str>) -> Self {
        Self {
            state,
            element,
            classes: StyleClasses::new(),
            attributes: StyleAttributes::new(),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ViewState<U> {
        ViewState {
            state: f(self.state),
            element: self.element,
            classes: self.classes,
            attributes: self.attributes,
        }
    }

    pub fn with_element(mut self, element: &'static str) -> Self {
        self.element = Some(element);
        self
    }
}

/// A [`View`] is a component that can be rendered to the screen.
#[allow(unused_variables)]
pub trait View: 'static {
    /// The state of the view.
    type State: 'static;

    /// Builds the state of the view.
    fn build(&self) -> ViewState<Self::State>;

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
    fn build(&self) -> ViewState<Box<dyn Any>>;

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event);

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2;

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext);
}

impl<T: View> AnyView for T {
    fn build(&self) -> ViewState<Box<dyn Any>> {
        self.build().map(|s| Box::new(s) as _)
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

    fn build(&self) -> ViewState<Self::State> {
        self.get_untracked().build()
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
