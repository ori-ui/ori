use crate::{Action, EffectSeq, Event, Mut, View, ViewMarker};

/// [`View`] that attaches an [`Effect`](crate::Effect) to a [`View`].
pub const fn with_effect<V, W>(contents: V, with: W) -> WithEffect<V, W> {
    WithEffect::new(contents, with)
}

/// [`View`] that attaches an [`Effect`](crate::Effect) to a [`View`].
pub struct WithEffect<V, W> {
    contents: V,
    effect:   W,
}

impl<V, W> WithEffect<V, W> {
    /// Create a [`WithEffect`].
    pub const fn new(contents: V, with: W) -> Self {
        Self {
            contents,
            effect: with,
        }
    }
}

impl<V, W> ViewMarker for WithEffect<V, W> {}
impl<C, T, V, W> View<C, T> for WithEffect<V, W>
where
    V: View<C, T>,
    W: EffectSeq<C, T>,
{
    type Element = V::Element;
    type State = (V::State, W::State);

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, contents) = self.contents.build(cx, data);
        let with = self.effect.seq_build(&mut (), cx, data);

        (element, (contents, with))
    }

    fn rebuild(
        self,
        element: Mut<'_, Self::Element>,
        (contents, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.contents.rebuild(element, contents, cx, data);
        self.effect.seq_rebuild(&mut (), with, cx, data);
    }

    fn event(
        element: Mut<'_, Self::Element>,
        (contents, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let contents_action = V::event(element, contents, cx, data, event);
        let effect_action = W::seq_event(&mut (), with, cx, data, event);

        contents_action | effect_action
    }

    fn teardown(element: Self::Element, (contents, with): Self::State, cx: &mut C) {
        V::teardown(element, contents, cx);
        W::seq_teardown(&mut (), with, cx);
    }
}

/// Sequence of [`Effect`](crate::Effect)s.
pub const fn effects<V>(contents: V) -> Effects<V> {
    Effects::new(contents)
}

/// Sequence of [`Effect`](crate::Effect)s.
pub struct Effects<V> {
    contents: V,
}

impl<V> Effects<V> {
    /// Create new [`Effects`].
    pub const fn new(contents: V) -> Self {
        Self { contents }
    }
}

impl<V> ViewMarker for Effects<V> {}
impl<C, T, V> View<C, T> for Effects<V>
where
    V: EffectSeq<C, T>,
{
    type Element = ();
    type State = V::State;

    fn build(self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let states = self.contents.seq_build(&mut (), cx, data);
        ((), states)
    }

    fn rebuild(
        self,
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.contents.seq_rebuild(&mut (), state, cx, data);
    }

    fn event(
        _element: Mut<'_, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        V::seq_event(&mut (), state, cx, data, event)
    }

    fn teardown(_element: Self::Element, state: Self::State, cx: &mut C) {
        V::seq_teardown(&mut (), state, cx);
    }
}
