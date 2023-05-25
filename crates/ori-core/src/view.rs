use std::any::{self, Any, TypeId};

use glam::Vec2;
use ori_reactive::Event;

use crate::{AvailableSpace, DrawContext, EventContext, LayoutContext, Style};

/// A [`View`] is a component that can be rendered to the screen.
#[allow(unused_variables)]
pub trait View: Send + Sync + 'static {
    /// The state of the view.
    type State: Send + Sync + 'static;

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
    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        space.min
    }

    /// Draws the view.
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {}
}

/// A [`View`] that with an unknown state.
///
/// This is used to store a [`View`] in a [`Node`](crate::Node).
pub trait AnyView: Any + Send + Sync {
    fn build(&self) -> Box<dyn Any + Send + Sync>;

    fn style(&self) -> Style;

    fn event(&self, state: &mut dyn Any, cx: &mut EventContext, event: &Event);

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2;

    fn draw(&self, state: &mut dyn Any, cx: &mut DrawContext);
}

impl<T: View> AnyView for T {
    fn build(&self) -> Box<dyn Any + Send + Sync> {
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

    fn layout(&self, state: &mut dyn Any, cx: &mut LayoutContext, space: AvailableSpace) -> Vec2 {
        if let Some(state) = state.downcast_mut::<T::State>() {
            self.layout(state, cx, space)
        } else {
            tracing::warn!("invalid state type on {}", any::type_name::<T>());
            space.min
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
        if self.type_id() == TypeId::of::<T>() {
            // SAFETY: `T` and `Self` are the same type
            unsafe { Some(&*(self as *const dyn AnyView as *const T)) }
        } else {
            None
        }
    }

    pub fn downcast_mut<T: AnyView>(&mut self) -> Option<&mut T> {
        if <dyn AnyView>::type_id(self) == TypeId::of::<T>() {
            // SAFETY: `T` and `Self` are the same type
            unsafe { Some(&mut *(self as *mut dyn AnyView as *mut T)) }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EmptyView;

impl View for EmptyView {
    type State = ();

    fn build(&self) -> Self::State {}
}
