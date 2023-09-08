use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, Pod, RebuildCx, State, View},
};

/// Create a new [`EventHandler`], with a before callback.
pub fn on_event_before<T, V>(
    content: V,
    handler: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> EventHandler<T, V> {
    EventHandler::new(content).before(handler)
}

/// Create a new [`EventHandler`], with an after callback.
pub fn on_event_after<T, V>(
    content: V,
    handler: impl FnMut(&mut EventCx, &mut T) + 'static,
) -> EventHandler<T, V> {
    EventHandler::new(content).after(handler)
}

/// A view that handles events.
#[derive(Rebuild)]
pub struct EventHandler<T, V> {
    /// The content.
    pub content: Pod<V>,
    /// The callback before an event is propagated.
    #[allow(clippy::type_complexity)]
    pub before: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,
    /// The callback after an event is propagated.
    #[allow(clippy::type_complexity)]
    pub after: Option<Box<dyn FnMut(&mut EventCx, &mut T) + 'static>>,
}

impl<T, V> EventHandler<T, V> {
    /// Create a new [`EventHandler`].
    pub fn new(content: V) -> Self {
        Self {
            content: Pod::new(content),
            before: None,
            after: None,
        }
    }

    /// Set the callback for before an event is emitted.
    pub fn before(mut self, before: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        self.before = Some(Box::new(before));
        self
    }

    /// Set the callback for when an event is emitted.
    pub fn after(mut self, after: impl FnMut(&mut EventCx, &mut T) + 'static) -> Self {
        self.after = Some(Box::new(after));
        self
    }
}

impl<T, V: View<T>> View<T> for EventHandler<T, V> {
    type State = State<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some(before) = &mut self.before {
            before(cx, data);
        }

        self.content.event(state, cx, data, event);

        if let Some(after) = &mut self.after {
            after(cx, data);
        }
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

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(state, cx, data, canvas);
    }
}
