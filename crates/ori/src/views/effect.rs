use crate::{Action, Event, NoElement, View, ViewSeq};

pub use crate::side_effects;

/// Sequence of [`SideEffect`](crate::SideEffect).
#[macro_export]
macro_rules! side_effects {
    [$($side_effect:expr),* $(,)?] => (
        $crate::views::side_effects(($($side_effect,)*))
    )
}

/// Sequence of [`SideEffect`](crate::SideEffect)s.
pub const fn side_effects<V>(content: V) -> SideEffects<V> {
    SideEffects::new(content)
}

/// Sequence of [`SideEffect`](crate::SideEffect)s.
pub struct SideEffects<V> {
    content: V,
}

impl<V> SideEffects<V> {
    /// Create new [`SideEffects`](crate::SideEffect).
    pub const fn new(content: V) -> Self {
        Self { content }
    }
}

impl<C, T, V> View<C, T> for SideEffects<V>
where
    V: ViewSeq<C, NoElement, T>,
{
    type Element = NoElement;
    type State = (Vec<NoElement>, V::SeqState);

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
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
