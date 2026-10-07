use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Action, Message, Mut, View, ViewMarker};

/// [`View`] that is only rebuilt when `data` changes.
pub fn memo<T, V, F, K>(key: K, build: F) -> Memo<F, K>
where
    F: FnOnce(&T) -> V,
    K: PartialEq,
{
    Memo::new(key, build)
}

/// [`View`] that is only rebuilt when the hash of `data` changes.
pub fn memo_hashed<T, V, F, K>(key: &K, build: F) -> Memo<F, u64>
where
    F: FnOnce(&T) -> V,
    K: Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();

    key.hash(&mut hasher);

    memo(hasher.finish(), build)
}

/// [`View`] that is only rebuilt when `data` changes.
#[must_use]
pub struct Memo<F, K> {
    key:   K,
    build: F,
}

impl<F, K> Memo<F, K> {
    /// Crate new [`Memo`].
    pub fn new<T, V>(key: K, build: F) -> Self
    where
        F: FnOnce(&T) -> V,
        K: PartialEq,
    {
        Self { key, build }
    }
}

impl<F, K> ViewMarker for Memo<F, K> {}
impl<C, T, V, F, K> View<C, T> for Memo<F, K>
where
    V: View<C, T>,
    F: FnOnce(&T) -> V,
    K: PartialEq,
{
    type Element = V::Element;
    type State = (K, V::State);

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
