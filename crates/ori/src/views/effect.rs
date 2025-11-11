use crate::{Action, EffectSeq, Event, NoElement, View, ViewMarker};

/// [`View`] that attaches an [`Effect`](crate::Effect) to a [`View`].
pub const fn with_effect<V, W>(content: V, with: W) -> WithEffect<V, W> {
    WithEffect::new(content, with)
}

/// [`View`] that attaches an [`Effect`](crate::Effect) to a [`View`].
pub struct WithEffect<V, W> {
    content: V,
    effect: W,
}

impl<V, W> WithEffect<V, W> {
    /// Create a [`WithEffect`].
    pub const fn new(content: V, with: W) -> Self {
        Self {
            content,
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
    type State = (V::State, W::Elements, W::States);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, content) = self.content.build(cx, data);
        let (elements, with) = self.effect.seq_build(cx, data);

        (element, (content, elements, with))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (content, elements, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.rebuild(
            element,
            content,
            cx,
            data,
            &mut old.content,
        );

        self.effect.seq_rebuild(
            elements,
            with,
            cx,
            data,
            &mut old.effect,
        );
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (content, elements, with): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.content.teardown(element, content, cx, data);
        self.effect.seq_teardown(elements, with, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (content, elements, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        let content_action = self.effect.seq_event(elements, with, cx, data, event);
        let effect_action = self.content.event(element, content, cx, data, event);

        content_action | effect_action
    }
}

/// Sequence of [`Effect`](crate::Effect)s.
pub const fn effects<V>(content: V) -> Effects<V> {
    Effects::new(content)
}

/// Sequence of [`Effect`](crate::Effect)s.
pub struct Effects<V> {
    content: V,
}

impl<V> Effects<V> {
    /// Create new [`Effects`].
    pub const fn new(content: V) -> Self {
        Self { content }
    }
}

impl<V> ViewMarker for Effects<V> {}
impl<C, T, V> View<C, T> for Effects<V>
where
    V: EffectSeq<C, T>,
{
    type Element = NoElement;
    type State = (V::Elements, V::States);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (children, states) = self.content.seq_build(cx, data);
        (NoElement, (children, states))
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) {
        self.content.seq_rebuild(
            children,
            state,
            cx,
            data,
            &mut old.content,
        );
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        (children, state): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.content.seq_teardown(children, state, cx, data);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        (children, state): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> Action {
        self.content.seq_event(children, state, cx, data, event)
    }
}
