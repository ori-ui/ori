use crate::{Action, Event, NoElement, SideEffect, View};

/// [`View`] that attaches a [`SideEffect`].
///
/// This is useful in conjunction with:
///  - [`Handler`](crate::views::Handler), [`handler`](crate::views::handler).
///  - [`Actor`](crate::views::Actor), [`actor`](crate::views::actor), [`task`](crate::views::task).
pub fn with<V, W>(content: V, with: W) -> With<V, W> {
    With::new(content, with)
}

/// [`View`] that attaches a [`SideEffect`].
///
/// This is useful in conjunction with:
///  - [`Handler`](crate::views::Handler), [`handler`](crate::views::handler).
///  - [`Actor`](crate::views::Actor), [`actor`](crate::views::actor), [`task`](crate::views::task).
pub struct With<V, W> {
    content: V,
    with: W,
}

impl<V, W> With<V, W> {
    /// Create a new [`With`].
    pub fn new(content: V, with: W) -> Self {
        Self { content, with }
    }
}

impl<C, T, V, W> View<C, T> for With<V, W>
where
    V: View<C, T>,
    W: SideEffect<C, T>,
{
    type Element = V::Element;
    type State = (V::State, W::State);

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
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
    ) {
        self.content.rebuild(
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
    ) -> Action {
        self.with.event(&mut NoElement, with, cx, data, event)
            | self.content.event(element, content, cx, data, event)
    }
}
