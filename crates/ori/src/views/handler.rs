use crate::{Action, Event, IntoAction, View};

/// Create a new [`Handler`].
pub fn handler<V>(content: V) -> Handler<V, ()> {
    Handler::new(content)
}

/// [`View`] that handles events.
pub fn on_event<V, T>(
    content: V,
    handler: impl EventHandler<T>,
) -> Handler<V, impl EventHandler<T>> {
    Handler::new(content).on_event(handler)
}

/// [`View`] that handles events.
#[must_use]
pub struct Handler<V, E> {
    content: V,
    event_handler: E,
}

impl<V> Handler<V, ()> {
    /// Create a new [`Handler`].
    pub fn new(content: V) -> Self {
        Self {
            content,
            event_handler: (),
        }
    }

    /// Add an event handler.
    pub fn on_event<T>(
        self,
        handler: impl EventHandler<T>,
    ) -> Handler<V, impl EventHandler<T>> {
        Handler {
            content: self.content,
            event_handler: handler,
        }
    }
}

impl<C, T, V, E> View<C, T> for Handler<V, E>
where
    V: View<C, T>,
    E: EventHandler<T>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        self.content.build(cx, data)
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            element,
            state,
            cx,
            data,
            &mut old.content,
        );
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        state: Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.content.teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let action = self.event_handler.on_event(data, event);
        action | self.content.event(element, state, cx, data, event)
    }
}

pub trait EventHandler<T> {
    fn on_event(&mut self, data: &mut T, event: &mut Event) -> Action;
}

impl<T> EventHandler<T> for () {
    fn on_event(&mut self, _data: &mut T, _event: &mut Event) -> Action {
        Action::none()
    }
}

impl<T, F, A> EventHandler<T> for F
where
    F: FnMut(&mut T, &mut Event) -> A,
    A: IntoAction,
{
    fn on_event(&mut self, data: &mut T, event: &mut Event) -> Action {
        self(data, event).into_action()
    }
}
