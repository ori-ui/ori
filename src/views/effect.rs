use crate::{Action, EffectSeq, Event, Mut, NoElement, NoElements, View, ViewMarker};

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

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, contents) = self.contents.build(cx, data);
        let with = self.effect.seq_build(&mut NoElements, cx, data);

        (element, (contents, with))
    }

    fn rebuild(
        &mut self,
        element: Mut<C, Self::Element>,
        (contents, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.rebuild(
            element,
            contents,
            cx,
            data,
            &mut old.contents,
        );

        self.effect.seq_rebuild(
            &mut NoElements,
            with,
            cx,
            data,
            &mut old.effect,
        );
    }

    fn event(
        &mut self,
        element: Mut<C, Self::Element>,
        (contents, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let contents_action = self
            .effect
            .seq_event(&mut NoElements, with, cx, data, event);
        let effect_action = self.contents.event(element, contents, cx, data, event);

        contents_action | effect_action
    }

    fn teardown(&mut self, element: Self::Element, (contents, with): Self::State, cx: &mut C) {
        self.contents.teardown(element, contents, cx);
        self.effect.seq_teardown(&mut NoElements, with, cx);
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
    type Element = NoElement;
    type State = V::State;

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let states = self.contents.seq_build(&mut NoElements, cx, data);
        (NoElement, states)
    }

    fn rebuild(
        &mut self,
        _element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.contents.seq_rebuild(
            &mut NoElements,
            state,
            cx,
            data,
            &mut old.contents,
        );
    }

    fn event(
        &mut self,
        _element: Mut<C, Self::Element>,
        state: &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        (self.contents).seq_event(&mut NoElements, state, cx, data, event)
    }

    fn teardown(&mut self, _element: Self::Element, state: Self::State, cx: &mut C) {
        self.contents.seq_teardown(&mut NoElements, state, cx);
    }
}
