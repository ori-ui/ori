use crate::{Action, Element, Elements, Message, Mut, View, ViewMarker, ViewSeq};

/// [`View`] that may choose to not rebuild itself.
///
/// This is an advanced [`View`] and should be used with care, and will `panic` if used
/// incorrectly. `contents` must always be guaranteed to be [`Some`] when [`Maybe`] is built.
///
/// # Panics
/// - If `contents` is [`None`] when [`Maybe`] is built.
#[track_caller]
pub fn maybe<V>(contents: Option<V>) -> Maybe<V> {
    Maybe::new(contents)
}

/// [`ViewSeq`] that may choose to not rebuild itself.
///
/// This is an advanced [`ViewSeq`] and should be used with care, and will `panic` if used
/// incorrectly. `contents` must always be guaranteed to be [`Some`] when [`MaybeSeq`] is built.
///
/// # Panics
/// - If `contents` is [`None`] when [`MaybeSeq`] is built.
#[track_caller]
pub fn maybe_seq<V>(contents: Option<V>) -> MaybeSeq<V> {
    MaybeSeq::new(contents)
}

/// [`View`] that may choose to not rebuild itself.
///
/// This is an advanced [`View`] and should be used with care, and will `panic` if used
/// incorrectly. `contents` must always be guaranteed to be [`Some`] when [`Maybe`] is built.
///
/// # Panics
/// - If `contents` is [`None`] when [`Maybe`] is built.
pub struct Maybe<V> {
    contents: Option<V>,
    #[cfg(debug_assertions)]
    location: &'static std::panic::Location<'static>,
}

impl<V> Maybe<V> {
    /// Create new [`Maybe`].
    #[track_caller]
    pub fn new(contents: Option<V>) -> Self {
        Self {
            contents,
            #[cfg(debug_assertions)]
            location: std::panic::Location::caller(),
        }
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
        let contents = self.contents.unwrap_or_else(|| {
            #[cfg(debug_assertions)]
            panic!(
                "contents of `maybe` must not be `None` during build\n\n`maybe` called here: {}",
                self.location
            );

            #[cfg(not(debug_assertions))]
            panic!("contents of `maybe` must not be `None` during build");
        });

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

/// [`ViewSeq`] that may choose to not rebuild itself.
///
/// This is an advanced [`ViewSeq`] and should be used with care, and will `panic` if used
/// incorrectly. `contents` must always be guaranteed to be [`Some`] when [`MaybeSeq`] is built.
///
/// # Panics
/// - If `contents` is [`None`] when [`MaybeSeq`] is built.
pub struct MaybeSeq<V> {
    contents: Option<V>,
    #[cfg(debug_assertions)]
    location: &'static std::panic::Location<'static>,
}

impl<V> MaybeSeq<V> {
    /// Create new [`MaybeSeq`].
    #[track_caller]
    pub fn new(contents: Option<V>) -> Self {
        Self {
            contents,
            #[cfg(debug_assertions)]
            location: std::panic::Location::caller(),
        }
    }
}

impl<C, T, E, V> ViewSeq<C, T, E> for MaybeSeq<V>
where
    E: Element,
    V: ViewSeq<C, T, E>,
{
    type State = (V::State, usize);

    fn seq_build(
        self,
        elements: &mut impl Elements<C, E>,
        cx: &mut C,
        data: &mut T,
    ) -> Self::State {
        let contents = self.contents.unwrap_or_else(|| {
            #[cfg(debug_assertions)]
            panic!(
                "contents of `maybe` must not be `None` during build\n\n`maybe` called here: {}",
                self.location
            );

            #[cfg(not(debug_assertions))]
            panic!("contents of `maybe` must not be `None` during build");
        });

        let start_index = elements.index();
        let state = contents.seq_build(elements, cx, data);
        let count = elements.index() - start_index;

        (state, count)
    }

    fn seq_rebuild(
        self,
        elements: &mut impl Elements<C, E>,
        (state, count): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        match self.contents {
            Some(seq) => {
                let start_index = elements.index();
                seq.seq_rebuild(elements, state, cx, data);
                *count = elements.index() - start_index;
            }

            None => {
                for _ in 0..*count {
                    elements.next(cx);
                }
            }
        }
    }

    fn seq_message(
        elements: &mut impl Elements<C, E>,
        (state, _count): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        V::seq_message(elements, state, cx, data, message)
    }

    fn seq_teardown(elements: &mut impl Elements<C, E>, (state, _count): Self::State, cx: &mut C) {
        V::seq_teardown(elements, state, cx);
    }
}
