use crate::{Action, Message, Mut, View, ViewMarker};

/// [`View`] that is built from a callback.
pub fn build<C, T, V>(build: impl FnOnce(&T) -> V) -> impl View<C, T, Element = V::Element>
where
    V: View<C, T>,
{
    build_with_context(move |_, data| build(data))
}

/// [`View`] that is built from a callback with access to the context.
pub fn build_with_context<C, T, V>(
    build: impl FnOnce(&C, &T) -> V,
) -> impl View<C, T, Element = V::Element>
where
    V: View<C, T>,
{
    Builder::new(build)
}

/// [`View`] that is built from a callback.
pub struct Builder<F> {
    build: F,
}

impl<F> Builder<F> {
    /// Create a [`Builder`].
    pub fn new<C, T, V>(build: F) -> Self
    where
        F: FnOnce(&C, &T) -> V,
        V: View<C, T>,
    {
        Self { build }
    }
}

impl<F> ViewMarker for Builder<F> {}
impl<C, T, V, F> View<C, T> for Builder<F>
where
    F: FnOnce(&C, &T) -> V,
    V: View<C, T>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view = (self.build)(cx, data);
        view.build(cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        let view = (self.build)(cx, data);
        view.rebuild(element, state, cx, data);
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        V::message(element, state, cx, data, message)
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        V::teardown(element, state, cx);
    }
}
