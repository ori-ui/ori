use crate::{Action, AsyncContext, Event, IntoAction, NoElement, View};

/// [`View`] that acts when built.
pub fn actor<T, A>(
    act: impl FnOnce(&mut T) -> A,
) -> Actor<impl FnOnce(&mut T) -> Action>
where
    A: IntoAction,
{
    Actor::new(act)
}

/// [`View`] that spawns a task when built.
pub fn task<C, T>(
    task: impl Future<Output: IntoAction> + Send + 'static,
) -> impl View<C, T, Element = NoElement>
where
    C: AsyncContext,
{
    Actor::new(move |_| Action::spawn(async { task.await.into_action() }))
}

/// [`View`] that acts with it's built.
pub struct Actor<F> {
    act: Option<F>,
}

impl Actor<()> {
    /// Create an [`Actor`].
    pub fn new<T, A>(
        act: impl FnOnce(&mut T) -> A,
    ) -> Actor<impl FnOnce(&mut T) -> Action>
    where
        A: IntoAction,
    {
        Actor {
            act: Some(move |data: &mut T| act(data).into_action()),
        }
    }
}

impl<C, T, F> View<C, T> for Actor<F>
where
    C: AsyncContext,
    F: FnOnce(&mut T) -> Action,
{
    type Element = NoElement;
    type State = ();

    fn build(
        &mut self,
        cx: &mut C,
        data: &mut T,
    ) -> (Self::Element, Self::State) {
        let act = self.act.take().unwrap();
        cx.send_action(act(data));

        (NoElement, ())
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn teardown(
        &mut self,
        _element: Self::Element,
        _state: Self::State,
        _cx: &mut C,
        _data: &mut T,
    ) {
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut C,
        _data: &mut T,
        _event: &mut Event,
    ) -> Action {
        Action::none()
    }
}
