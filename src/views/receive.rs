use crate::{Action, Effect, Message, Mut, View, ViewMarker};

/// [`View`] that receives message.
pub fn receive_any<C, T>(
    on_message: impl FnMut(&mut T, &mut Message) -> Action,
) -> impl Effect<C, T> {
    Receive::new(on_message)
}

/// [`View`] that receives messages.
pub fn receive<C, T, E, A>(mut on_message: impl FnMut(&mut T, E) -> A) -> impl Effect<C, T>
where
    E: Send + 'static,
    A: Into<Action>,
{
    receive_any(move |data, event| {
        if let Some(message) = event.take() {
            on_message(data, message).into()
        } else {
            Action::new()
        }
    })
}

/// [`View`] that receives messages.
#[must_use]
pub struct Receive<E> {
    on_message: E,
}

impl<E> Receive<E> {
    /// Create new [`Receive`].
    pub const fn new(on_message: E) -> Self {
        Receive { on_message }
    }
}

impl<F> ViewMarker for Receive<F> {}
impl<C, T, E> View<C, T> for Receive<E>
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
