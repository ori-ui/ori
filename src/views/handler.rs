use crate::{Action, Effect, Message, Mut, View, ViewMarker};

/// [`View`] that handles message.
pub fn on_any_message<C, T>(
    on_message: impl FnMut(&mut T, &mut Message) -> Action,
) -> impl Effect<C, T> {
    Handler::new().on_message(on_message)
}

/// [`View`] that handles messages.
pub fn on_message<C, T, E, A>(mut on_message: impl FnMut(&mut T, E) -> A) -> impl Effect<C, T>
where
    E: Send + 'static,
    A: Into<Action>,
{
    on_any_message(move |data, event| {
        if let Some(message) = event.take() {
            on_message(data, message).into()
        } else {
            Action::new()
        }
    })
}

/// [`View`] that handles messages.
#[must_use]
pub struct Handler<E> {
    on_message: E,
}

impl Default for Handler<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler<()> {
    /// Create a new [`Handler`].
    pub const fn new() -> Self {
        Self { on_message: () }
    }

    /// Add an message handler.
    pub const fn on_message<T>(
        self,
        on_message: impl FnMut(&mut T, &mut Message) -> Action,
    ) -> Handler<impl FnMut(&mut T, &mut Message) -> Action> {
        Handler { on_message }
    }
}

impl<F> ViewMarker for Handler<F> {}
impl<C, T, E> View<C, T> for Handler<E>
where
    E: FnMut(&mut T, &mut Message) -> Action,
{
    type Element = ();
    type State = E;

    fn build(self, _cx: &mut C, _data: &mut T) -> (Self::Element, Self::State) {
        ((), self.on_message)
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
    ) {
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        on_message: &mut Self::State,
        _cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        on_message(data, message)
    }

    fn teardown(_element: Self::Element, _state: Self::State, _cx: &mut C) {}
}
