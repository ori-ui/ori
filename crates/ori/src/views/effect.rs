use crate::{Action, Event, NoElement, View, ViewSeq};

pub use crate::effects;

/// Sequence of [`Effect`](crate::Effect).
#[macro_export]
macro_rules! effects {
    [$($effect:expr),* $(,)?] => (
        $crate::views::effects(($($effect,)*))
    )
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
    /// Create new [`Effects`](crate::Effect).
    pub const fn new(content: V) -> Self {
        Self { content }
    }
}

impl<C, T, V> View<C, T> for Effects<V>
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
