use crate::{
    Action, AsyncContext, Effect, Event, IntoAction, NoElement, View, ViewMarker,
    views::builder::build_with_context,
};

/// [`View`] that acts when built.
pub fn on_build<C, T>(on_build: impl FnOnce(&mut T) -> Action) -> impl Effect<C, T>
where
    C: AsyncContext,
{
    AsyncHandler::new().on_build(on_build)
}

/// [`View`] that spawns a task when built.
pub fn task<C, T, I>(task: impl Future<Output: IntoAction<I>> + Send + 'static) -> impl Effect<C, T>
where
    C: AsyncContext,
{
    on_build(move |_| Action::spawn(async { task.await.into_action() }))
}

/// [`View`] that spawns a task with a proxy when built.
pub fn task_with_proxy<C, T, F, I>(task: impl FnOnce(C::Proxy) -> F) -> impl Effect<C, T>
where
    C: AsyncContext,
    F: Future + Send + 'static,
    F::Output: IntoAction<I>,
{
    build_with_context(|cx: &mut C, _| self::task(task(cx.proxy())))
}

/// [`View`] that acts with it's built.
pub struct AsyncHandler<F> {
    on_build: Option<F>,
}

impl Default for AsyncHandler<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncHandler<()> {
    /// Create an [`AsyncHandler`].
    pub fn new() -> Self {
        Self { on_build: None }
    }
}

impl<F> AsyncHandler<F> {
    /// Add a callback trigged when the [`View`] is built.
    pub fn on_build<T>(
        self,
        on_build: impl FnOnce(&mut T) -> Action,
    ) -> AsyncHandler<impl FnOnce(&mut T) -> Action> {
        AsyncHandler {
            on_build: Some(move |data: &mut T| on_build(data).into_action()),
        }
    }
}

impl<F> ViewMarker for AsyncHandler<F> {}
impl<C, T, F> View<C, T> for AsyncHandler<F>
where
    C: AsyncContext,
    F: FnOnce(&mut T) -> Action,
{
    type Element = NoElement;
    type State = ();

    fn build(&mut self, cx: &mut C, data: &mut T) -> (Self::Element, Self::State) {
        if let Some(on_build) = self.on_build.take() {
            cx.send_action(on_build(data));
        }

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
        Action::new()
    }
}
