use crate::{Action, Effect, Message, Mut, View, ViewId, ViewMarker};

/// [`View`] that receives message.
pub fn receive_all<C, T>(
    on_message: impl FnMut(&mut T, &mut Message) -> Action,
) -> impl Effect<C, T> {
    Receive::new(on_message)
}

/// [`View`] that receives messages.
pub fn receive<C, T, E, A>(
    view_id: impl Into<Option<ViewId>>,
    mut on_message: impl FnMut(&mut T, E) -> A,
) -> impl Effect<C, T>
where
    E: Send + 'static,
    A: Into<Action>,
{
    let view_id = view_id.into();

    receive_all(move |data, event| {
        let message = match view_id {
            Some(id) => event.take(id),
            None => event.take_untargeted(),
        };

        match message {
            Some(message) => on_message(data, message).into(),
            None => Action::new(),
        }
    })
}

/// [`View`] that receives messages.
#[must_use]
pub struct Receive<F> {
    on_message: F,
}

impl<F> Receive<F> {
    /// Create new [`Receive`].
    pub const fn new(on_message: F) -> Self {
        Receive { on_message }
    }
}

impl<F> ViewMarker for Receive<F> {}
impl<C, T, F> View<C, T> for Receive<F>
where
    F: FnMut(&mut T, &mut Message) -> Action,
{
    type Element = ();
    type State = F;

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
