use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{Action, Event, View, ViewMarker};

/// [`View`] that is only rebuilt when `data` changes.
pub fn memo<C, T, V, F, D>(data: D, build: F) -> Memo<F, D>
where
    V: View<C, T>,
    F: FnOnce(&mut T) -> V,
    D: PartialEq,
{
    Memo::new(data, build)
}

/// [`View`] that is only rebuilt when the hash of `data` changes.
pub fn hash_memo<C, T, V, F, D>(data: &D, build: F) -> Memo<F, u64>
where
    V: View<C, T>,
    F: FnOnce(&mut T) -> V,
    D: Hash + ?Sized,
{
    let mut hasher = DefaultHasher::new();

    data.hash(&mut hasher);

    memo(hasher.finish(), build)
}

/// [`View`] that is only rebuilt when `data` changes.
#[must_use]
pub struct Memo<F, D> {
    data:  D,
    build: Option<F>,
}

impl<F, D> Memo<F, D> {
    /// Crate new [`Memo`].
    pub fn new<C, T, V>(data: D, build: F) -> Self
    where
        V: View<C, T>,
        F: FnOnce(&mut T) -> V,
        D: PartialEq,
    {
        Self {
            data,
            build: Some(build),
        }
    }
}

impl<F, D> ViewMarker for Memo<F, D> {}
impl<C, T, V, F, D> View<C, T> for Memo<F, D>
where
    V: View<C, T>,
    F: FnOnce(&mut T) -> V,
    D: PartialEq,
{
    type Element = V::Element;
    type State = (V, V::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let build = self.build.take().expect("build should only be called once");
        let mut view = build(data);
        let (element, state) = view.build(cx, data);
        (element, (view, state))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        if self.data != old.data
            && let Some(build) = self.build.take()
        {
            let mut new_view = build(data);
            new_view.rebuild(element, state, cx, data, view);
            *view = new_view;
        }
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (mut view, state): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        view.teardown(element, state, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (view, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        view.event(element, state, cx, data, event)
    }
}
