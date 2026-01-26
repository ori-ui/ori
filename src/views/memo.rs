use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Action, Message, Mut, View, ViewMarker};

/// [`View`] that is only rebuilt when `data` changes.
pub fn memo<C, T, V, F, D>(data: D, build: F) -> Memo<F, D>
where
    V: View<C, T>,
    F: FnOnce(&T) -> V,
    D: PartialEq,
{
    Memo::new(data, build)
}

/// [`View`] that is only rebuilt when the hash of `data` changes.
pub fn memo_hashed<C, T, V, F, D>(data: &D, build: F) -> Memo<F, u64>
where
    V: View<C, T>,
    F: FnOnce(&T) -> V,
    D: Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();

    data.hash(&mut hasher);

    memo(hasher.finish(), build)
}

/// [`View`] that is only rebuilt when `data` changes.
#[must_use]
pub struct Memo<F, D> {
    key:   D,
    build: F,
}

impl<F, D> Memo<F, D> {
    /// Crate new [`Memo`].
    pub fn new<C, T, V>(data: D, build: F) -> Self
    where
        V: View<C, T>,
        F: FnOnce(&T) -> V,
        D: PartialEq,
    {
        Self { key: data, build }
    }
}

impl<F, D> ViewMarker for Memo<F, D> {}
impl<C, T, V, F, D> View<C, T> for Memo<F, D>
where
    V: View<C, T>,
    F: FnOnce(&T) -> V,
    D: PartialEq,
{
    type Element = V::Element;
    type State = (D, V::State);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let view = (self.build)(data);
        let (element, state) = view.build(cx, data);
        (element, (self.key, state))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (key, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        if self.key != *key {
            let view = (self.build)(data);
            view.rebuild(element, state, cx, data);
            *key = self.key;
        }
    }

    fn message(
        element: Mut<'_, Self::Element>,
        (_, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        message: &mut Message,
    ) -> Action {
        V::message(element, state, cx, data, message)
    }

    fn teardown(element: Self::Element, (_, state): Self::State, cx: &mut C) {
        V::teardown(element, state, cx);
    }
}
