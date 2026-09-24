use crate::{Action, Effect, Message, Mut, Tracker, View, ViewId, ViewMarker};

/// [`View`] that receives message.
pub fn receive_all<C, T>(
    view_id: impl Into<Option<ViewId>>,
    on_message: impl FnMut(&mut T, &mut Message) -> Action,
) -> impl Effect<C, T>
where
    C: Tracker,
{
    Receive::new(view_id.into(), on_message)
}

/// [`View`] that receives messages.
pub fn receive<C, T, E, A>(
    view_id: impl Into<Option<ViewId>>,
    mut on_message: impl FnMut(&mut T, E) -> A,
) -> impl Effect<C, T>
where
    C: Tracker,
    E: Send + 'static,
    A: Into<Action>,
{
    let view_id = view_id.into();

    receive_all(view_id, move |data, event| {
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
    view_id:    Option<ViewId>,
    on_message: F,
}

impl<F> Receive<F> {
    /// Create new [`Receive`].
    pub const fn new(view_id: Option<ViewId>, on_message: F) -> Self {
        Receive {
            view_id,
            on_message,
        }
    }
}

impl<F> ViewMarker for Receive<F> {}
impl<C, T, F> View<C, T> for Receive<F>
where
    C: Tracker,
    F: FnMut(&mut T, &mut Message) -> Action,
{
    type Element = ();
    type State = Self;

    fn build(self, cx: &mut C, _data: &mut T) -> (Self::Element, Self::State) {
        if let Some(id) = self.view_id {
            cx.register(id);
        }

        ((), self)
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        _data: &mut T,
    ) {
        if state.view_id != self.view_id {
            if let Some(id) = state.view_id {
                cx.unregister(id);
            }

            if let Some(id) = self.view_id {
                cx.register(id);
            }
        }

        *state = self;
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        _cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        (state.on_message)(data, message)
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut C) {
        if let Some(id) = state.view_id {
            cx.unregister(id);
        }
    }
}
