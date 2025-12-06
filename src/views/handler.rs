use crate::{Action, Effect, Event, IntoAction, NoElement, View, ViewMarker};

/// Create a new [`Handler`].
pub fn handler() -> Handler<()> {
    Handler::new()
}

/// [`View`] that handles events.
pub fn on_any_event<C, T>(on_event: impl FnMut(&mut T, &mut Event) -> Action) -> impl Effect<C, T> {
    Handler::new().on_event(on_event)
}

/// [`View`] that handles events.
pub fn on_event<C, T, E, A, I>(mut on_event: impl FnMut(&mut T, E) -> A) -> impl Effect<C, T>
where
    E: Send + 'static,
    A: IntoAction<I>,
{
    on_any_event(move |data, event| {
        if let Some(event) = event.take() {
            on_event(data, event).into_action()
        } else {
            Action::new()
        }
    })
}

/// [`View`] that handles events.
#[must_use]
pub struct Handler<E> {
    on_event: E,
}

impl Default for Handler<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler<()> {
    /// Create a new [`Handler`].
    pub const fn new() -> Self {
        Self { on_event: () }
    }

    /// Add an event handler.
    pub const fn on_event<T>(
        self,
        on_event: impl FnMut(&mut T, &mut Event) -> Action,
    ) -> Handler<impl FnMut(&mut T, &mut Event) -> Action> {
        Handler { on_event }
    }
}

impl<F> ViewMarker for Handler<F> {}
impl<C, T, E> View<C, T> for Handler<E>
where
    E: FnMut(&mut T, &mut Event) -> Action,
{
    type Element = NoElement;
    type State = ();

    fn build(&mut self, _cx: &mut C, _data: &mut T) -> (Self::Element, Self::State) {
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
        (self.on_event)(data, event)
    }
}
