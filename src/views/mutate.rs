use crate::{Action, Effect, Message, Mut, View, ViewMarker};

/// An [`Effect`] that mutates state when built.
pub fn mutate<C, T>(mutate: impl FnOnce(&mut T)) -> impl Effect<C, T> {
    Mutate::new(mutate)
}

/// An [`Effect`] that mutates state when built.
pub struct Mutate<F> {
    mutate: F,
}

impl<F> Mutate<F> {
    /// Create new [`Mutate`].
    pub fn new(mutate: F) -> Self {
        Self { mutate }
    }
}

impl<F> ViewMarker for Mutate<F> {}
impl<C, T, F> View<C, T> for Mutate<F>
where
    F: FnOnce(&mut T),
{
    type Element = ();
    type State = ();

    fn build(self, _cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        (self.mutate)(data);
        ((), ())
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut C,
        data: &mut T,
    ) {
        (self.mutate)(data);
    }

    fn message(
        _element: Mut<'_, Self::Element>,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _message: &mut Message,
    ) -> Action {
        Action::new()
    }

    fn teardown(_element: Self::Element, _state: Self::State, _cx: &mut C) {}
}
