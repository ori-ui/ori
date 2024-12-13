use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// A view for creating custom window decorations.
pub fn decorate<F, V, T>(
    content: V,
    builder: impl FnOnce(&mut T, V) -> F + 'static,
) -> Decorate<F, V, T> {
    Decorate::new(content, builder)
}

/// A view for creating custom window decorations.
pub struct Decorate<F, V, T> {
    #[allow(clippy::type_complexity)]
    content: Option<(V, Box<dyn FnOnce(&mut T, V) -> F>)>,
}

impl<F, V, T> Decorate<F, V, T> {
    /// Create a new `Decorate` view.
    pub fn new(content: V, builder: impl FnOnce(&mut T, V) -> F + 'static) -> Self {
        Self {
            content: Some((content, Box::new(builder))),
        }
    }
}

#[doc(hidden)]
pub enum DecorateState<F, V, T>
where
    F: View<T>,
    V: View<T>,
{
    Content(V, V::State),
    Frame(F, F::State),
}

impl<F, V, T> View<T> for Decorate<F, V, T>
where
    F: View<T>,
    V: View<T>,
{
    type State = DecorateState<F, V, T>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let (mut content, builder) = self.content.take().expect("Frame content not set");

        match cx.window().decorated {
            true => {
                let state = content.build(cx, data);
                DecorateState::Content(content, state)
            }
            false => {
                let mut frame = builder(data, content);
                let state = frame.build(cx, data);
                DecorateState::Frame(frame, state)
            }
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, _old: &Self) {
        let is_frame = matches!(state, DecorateState::Frame(..));

        if is_frame != cx.window().decorated {
            *state = self.build(&mut cx.as_build_cx(), data);
            cx.layout();
            return;
        }

        let (mut content, builder) = self.content.take().expect("Frame content not set");

        match state {
            DecorateState::Content(old_view, state) => {
                content.rebuild(state, cx, data, old_view);
                *old_view = content;
            }
            DecorateState::Frame(old_view, state) => {
                let mut frame = builder(data, content);
                frame.rebuild(state, cx, data, old_view);
                *old_view = frame;
            }
        }
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        match state {
            DecorateState::Content(view, state) => view.event(state, cx, data, event),
            DecorateState::Frame(view, state) => view.event(state, cx, data, event),
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        match state {
            DecorateState::Content(view, state) => view.layout(state, cx, data, space),
            DecorateState::Frame(view, state) => view.layout(state, cx, data, space),
        }
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        match state {
            DecorateState::Content(view, state) => view.draw(state, cx, data),
            DecorateState::Frame(view, state) => view.draw(state, cx, data),
        }
    }
}
