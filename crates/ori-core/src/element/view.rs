use std::any::Any;

use glam::Vec2;
use ori_reactive::Event;

use crate::{AnyView, AvailableSpace, DrawContext, EventContext, LayoutContext, Style, View};

/// A view that can be used as an element.
///
/// This exists because specialization doesn't.
pub trait ElementView: Send + Sync + 'static {
    /// The state of the element.
    type State: Send + Sync + 'static;

    /// Build the state for the element.
    fn build(&self) -> Self::State;

    /// Get the style for the element.
    fn style(&self) -> Style;

    /// Handle an event for the element.
    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event);

    /// Layout the element.
    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2;

    /// Draw the element.
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext);
}

impl<T: View> ElementView for T {
    type State = T::State;

    fn build(&self) -> Self::State {
        self.build()
    }

    fn style(&self) -> Style {
        self.style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.event(state, cx, event)
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        self.layout(state, cx, space)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.draw(state, cx)
    }
}

impl ElementView for Box<dyn AnyView> {
    type State = Box<dyn Any + Send + Sync>;

    fn build(&self) -> Self::State {
        self.as_ref().build()
    }

    fn style(&self) -> Style {
        self.as_ref().style()
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.as_ref().event(state.as_mut(), cx, event)
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        self.as_ref().layout(state.as_mut(), cx, space)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.as_ref().draw(state.as_mut(), cx)
    }
}
