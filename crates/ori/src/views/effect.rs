use crate::{Action, Effect, EffectSeq, Event, NoElement, View};

/// [`View`] that attaches an [`Effect`] to a [`View`].
pub const fn with_effect<V, W>(content: V, with: W) -> WithEffect<V, W> {
    WithEffect::new(content, with)
}

/// [`View`] that attaches an [`Effect`] to a [`View`].
pub struct WithEffect<V, W> {
    content: V,
    with: W,
}

impl<V, W> WithEffect<V, W> {
    /// Create a [`WithEffect`].
    pub const fn new(content: V, with: W) -> Self {
        Self { content, with }
    }
}

impl<C, T, V, W> View<C, T> for WithEffect<V, W>
where
    V: View<C, T>,
    W: Effect<C, T>,
{
    type Element = V::Element;
    type State = (V::State, W::State);

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        let (element, content) = self.content.build(cx, data);
        let (_, with) = self.with.build(cx, data);

        (element, (content, with))
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        (content, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        old: &mut Self,
    ) -> bool {
        let element_changed = self.content.rebuild(
            element,
            content,
            cx,
            data,
            &mut old.content,
        );

        self.with.rebuild(
            &mut NoElement,
            with,
            cx,
            data,
            &mut old.with,
        );

        element_changed
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        (content, with): Self::State,
        cx: &mut C,
        data: &mut T,
    ) {
        self.content.teardown(element, content, cx, data);
        self.with.teardown(NoElement, with, cx, data);
    }

    fn event(
        &mut self,
        element: &mut Self::Element,
        (content, with): &mut Self::State,
        cx: &mut C,
        data: &mut T,
        event: &mut Event,
    ) -> (bool, Action) {
        let (element_changed, content_action) =
            self.with.event(&mut NoElement, with, cx, data, event);

        let (_, effect_action) = self.content.event(element, content, cx, data, event);

        (
            element_changed,
            content_action | effect_action,
        )
    }
}

pub use crate::effects;

/// Sequence of [`Effect`].
#[macro_export]
macro_rules! effects {
    [$($effect:expr),* $(,)?] => (
        $crate::views::effects(($($effect,)*))
    )
}

/// Sequence of [`Effect`]s.
pub const fn effects<V>(content: V) -> Effects<V> {
    Effects::new(content)
}

/// Sequence of [`Effect`]s.
pub struct Effects<V> {
    content: V,
}

impl<V> Effects<V> {
    /// Create new [`Effects`].
    pub const fn new(content: V) -> Self {
        Self { content }
    }
}

impl<C, T, V> View<C, T> for Effects<V>
where
    V: EffectSeq<C, T>,
{
    type Element = NoElement;
    type State = (Vec<NoElement>, V::SeqState);

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
    ) -> bool {
        self.content.seq_rebuild(
            children,
            state,
            cx,
            data,
            &mut old.content,
        );

        false
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
    ) -> (bool, Action) {
        self.content.seq_event(children, state, cx, data, event)
    }
}
