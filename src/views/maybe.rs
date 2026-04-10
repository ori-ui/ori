use crate::{Action, Message, Mut, View, ViewMarker};

/// [`View`] that may choose to not rebuild itself.
///
/// This is an advanced [`View`] and should be used with care, and will `panic` if used
/// incorrectly. `contents` must always be guaranteed to be [`Some`] when [`Maybe`] is built.
pub fn maybe<V>(contents: Option<V>) -> Maybe<V> {
    Maybe::new(contents)
}

/// [`View`] that may choose to not rebuild itself.
///
/// This is an advanced [`View`] and should be used with care, and will `panic` if used
/// incorrectly. `contents` must always be guaranteed to be [`Some`] when [`Maybe`] is built.
pub struct Maybe<V> {
    contents: Option<V>,
}

impl<V> Maybe<V> {
    /// Create new [`Maybe`].
    pub fn new(contents: Option<V>) -> Self {
        Self { contents }
    }
}

impl<V> ViewMarker for Maybe<V> {}
impl<C, T, V> View<C, T> for Maybe<V>
where
    V: View<C, T>,
{
    type Element = V::Element;
    type State = V::State;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let contents = self
            .contents
            .expect("contents of `maybe` must not be `None` during build");

        contents.build(cx, data)
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        if let Some(contents) = self.contents {
            contents.rebuild(element, state, cx, data);
        }
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
