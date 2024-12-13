use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// A view for creating views based on the available space.
pub fn layout<V, T>(builder: impl FnMut(&mut T, Space) -> V + 'static) -> Layout<V, T> {
    Layout::new(builder)
}

/// A view for creating views based on the available space.
pub struct Layout<V, T> {
    #[allow(clippy::type_complexity)]
    builder: Box<dyn FnMut(&mut T, Space) -> V>,
}

impl<V, T> Layout<V, T> {
    /// Create a new `Layout` view.
    pub fn new(builder: impl FnMut(&mut T, Space) -> V + 'static) -> Self {
        Self {
            builder: Box::new(builder),
        }
    }
}

#[doc(hidden)]
pub struct LayoutState<V: View<T>, T> {
    view: Option<(V, V::State)>,
    space: Option<Space>,
}

impl<V: View<T>, T> View<T> for Layout<V, T> {
    type State = LayoutState<V, T>;

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {
        LayoutState {
            view: None,
            space: None,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, _old: &Self) {
        let Some(space) = state.space else {
            return;
        };

        let mut view = (self.builder)(data, space);

        match state.view {
            Some((ref mut old_view, ref mut state)) => {
                view.rebuild(state, cx, data, old_view);
                *old_view = view;
            }
            None => {
                let view_state = view.build(&mut cx.as_build_cx(), data);
                state.view = Some((view, view_state));
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
        match state.view {
            Some((ref mut view, ref mut state)) => view.event(state, cx, data, event),
            None => false,
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if state.space != Some(space) {
            let mut view = (self.builder)(data, space);

            if let Some((ref mut old_view, ref mut state)) = state.view {
                let mut rebuild_cx = RebuildCx::new(cx.base, cx.view_state);
                view.rebuild(state, &mut rebuild_cx, data, old_view);
                *old_view = view;
            } else {
                let mut build_cx = BuildCx::new(cx.base, cx.view_state);
                let view_state = view.build(&mut build_cx, data);
                state.view = Some((view, view_state));
            }

            state.space = Some(space);
        }

        let (view, state) = state.view.as_mut().unwrap();
        view.layout(state, cx, data, space)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        if let Some((view, state)) = &mut state.view {
            view.draw(state, cx, data);
        }
    }
}
