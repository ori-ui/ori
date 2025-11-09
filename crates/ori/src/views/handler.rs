use crate::{Action, Event, IntoAction, NoElement, View};

/// Create a new [`Handler`].
pub fn handler() -> Handler<()> {
    Handler::new()
}

/// [`View`] that handles events.
pub fn on_event<T, A>(
    handler: impl FnMut(&mut T, &mut Event) -> A,
) -> Handler<impl EventHandler<T>>
where
    A: IntoAction,
{
    Handler::new().on_event(handler)
}

/// [`View`] that handles events.
#[must_use]
pub struct Handler<E> {
    event_handler: E,
}

impl Default for Handler<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler<()> {
    /// Create a new [`Handler`].
    pub const fn new() -> Self {
        Self { event_handler: () }
    }

    /// Add an event handler.
    pub const fn on_event<T>(
        self,
        handler: impl EventHandler<T>,
    ) -> Handler<impl EventHandler<T>> {
        Handler {
            event_handler: handler,
        }
    }
}

impl<C, T, E> View<C, T> for Handler<E>
where
    E: EventHandler<T>,
{
    type Element = NoElement;
    type State = ();

    fn build(
        &mut self,
        _cx: &mut C,
        _data: &mut T,
    ) -> (Self::Element, Self::State) {
        (NoElement, ())
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _state: Self::State,
        _cx: &mut C,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        self.event_handler.on_event(data, event)
    }
}

/// A handler for events, see [`Handler`] for more information.
pub trait EventHandler<T> {
    /// Handle an event.
    fn on_event(&mut self, data: &mut T, event: &mut Event) -> Action;
}

impl<T> EventHandler<T> for () {
    fn on_event(&mut self, _data: &mut T, _event: &mut Event) -> Action {
        Action::new()
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
