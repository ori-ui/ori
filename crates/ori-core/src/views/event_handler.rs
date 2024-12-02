use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// Create a new [`EventHandler`], with a before callback.
pub fn on_event_before<T, V>(
    content: V,
    handler: impl FnMut(&mut EventCx, &mut T, &Event) -> bool + 'static,
) -> EventHandler<T, V> {
    EventHandler::new(content).before(handler)
}

/// Create a new [`EventHandler`], with an after callback.
pub fn on_event<T, V>(
    content: V,
    handler: impl FnMut(&mut EventCx, &mut T, &Event) -> bool + 'static,
) -> EventHandler<T, V> {
    EventHandler::new(content).after(handler)
}

/// A view that handles events.
pub struct EventHandler<T, V> {
    /// The content.
    pub content: V,

    /// The callback before an event is propagated.
    #[allow(clippy::type_complexity)]
    pub before: Option<Box<dyn FnMut(&mut EventCx, &mut T, &Event) -> bool + 'static>>,

    /// The callback after an event is propagated.
    #[allow(clippy::type_complexity)]
    pub after: Option<Box<dyn FnMut(&mut EventCx, &mut T, &Event) -> bool + 'static>>,
}

impl<T, V> EventHandler<T, V> {
    /// Create a new [`EventHandler`].
    pub fn new(content: V) -> Self {
        Self {
            content,
            before: None,
            after: None,
        }
    }

    /// Set the callback for before an event is emitted.
    pub fn before(
        mut self,
        before: impl FnMut(&mut EventCx, &mut T, &Event) -> bool + 'static,
    ) -> Self {
        self.before = Some(Box::new(before));
        self
    }

    /// Set the callback for when an event is emitted.
    pub fn after(
        mut self,
        after: impl FnMut(&mut EventCx, &mut T, &Event) -> bool + 'static,
    ) -> Self {
        self.after = Some(Box::new(after));
        self
    }
}

impl<T, V: View<T>> View<T> for EventHandler<T, V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        let mut handled = false;

        if let Some(ref mut before) = self.before {
            handled = before(cx, data, event);
        }

        match handled {
            true => _ = self.content.event(state, cx, data, &Event::Notify),
            false => handled = self.content.event(state, cx, data, event),
        }

        if let Some(ref mut after) = self.after {
            if !handled {
                handled = after(cx, data, event);
            }
        }

        handled
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}
