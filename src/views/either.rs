use std::mem;

use crate::{Action, Base, Is, Message, Mut, View, ViewMarker};

/// A [`View`] that is either one of two [`View`]s.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Either<A, B> {
    /// An instance of the `left` view.
    Left(A),

    /// An instance of the `right` view.
    Right(B),
}

impl<A, B> ViewMarker for Either<A, B> {}
impl<C, T, A, B> View<C, T> for Either<A, B>
where
    C: Base,
    A: View<C, T>,
    B: View<C, T>,
    A::Element: Is<C, C::Element>,
    B::Element: Is<C, C::Element>,
{
    type Element = C::Element;
    type State = Either<A::State, B::State>;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        match self {
            Either::Left(left) => {
                let (element, state) = left.build(cx, data);
                let element = A::Element::upcast(cx, element);
                (element, Either::Left(state))
            }

            Either::Right(right) => {
                let (element, state) = right.build(cx, data);
                let element = B::Element::upcast(cx, element);
                (element, Either::Right(state))
            }
        }
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        match (self, state) {
            (Self::Left(view), Either::Left(state)) => {
                if let Ok(element) = A::Element::downcast_mut(element) {
                    view.rebuild(element, state, cx, data);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "element downcast failed, a view was provided the wrong element",
                    );
                }
            }

            (Self::Right(view), Either::Right(state)) => {
                if let Ok(element) = B::Element::downcast_mut(element) {
                    view.rebuild(element, state, cx, data);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "element downcast failed, a view was provided the wrong element",
                    );
                }
            }

            (Self::Left(view), state) => {
                let (new_element, new_state) = view.build(cx, data);
                let element = A::Element::replace(cx, element, new_element);
                let state = mem::replace(state, Either::Left(new_state));
                Self::teardown(element, state, cx);
            }

            (Self::Right(view), state) => {
                let (new_element, new_state) = view.build(cx, data);
                let element = B::Element::replace(cx, element, new_element);
                let state = mem::replace(state, Either::Right(new_state));
                Self::teardown(element, state, cx);
            }
        }
    }

    fn message(
        element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        match state {
            Either::Left(state) => {
                if let Ok(element) = A::Element::downcast_mut(element) {
                    A::message(element, state, cx, data, message)
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "element downcast failed, a view was provided the wrong element",
                    );

                    Action::new()
                }
            }

            Either::Right(state) => {
                if let Ok(element) = B::Element::downcast_mut(element) {
                    B::message(element, state, cx, data, message)
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "element downcast failed, a view was provided the wrong element",
                    );

                    Action::new()
                }
            }
        }
    }

    fn teardown(element: Self::Element, state: Self::State, cx: &mut C) {
        match state {
            Either::Left(state) => {
                if let Ok(element) = A::Element::downcast(element) {
                    A::teardown(element, state, cx);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "element downcast failed, a view was provided the wrong element",
                    );
                }
            }

            Either::Right(state) => {
                if let Ok(element) = B::Element::downcast(element) {
                    B::teardown(element, state, cx);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "element downcast failed, a view was provided the wrong element",
                    );
                }
            }
        }
    }
}
