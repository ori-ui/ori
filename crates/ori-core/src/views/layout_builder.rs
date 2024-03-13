use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    theme::Theme,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`LayoutBuilder`] view.
pub fn layout_builder<T, V>(
    builder: impl FnMut(&mut LayoutCx, &mut T, Space) -> V + 'static,
) -> LayoutBuilder<T, V> {
    LayoutBuilder {
        content: Box::new(builder),
    }
}

/// A view that builds its content based on the layout constraints.
///
/// Note that the content is only built on layout.
pub struct LayoutBuilder<T, V> {
    /// The builder function.
    #[allow(clippy::type_complexity)]
    pub content: Box<dyn FnMut(&mut LayoutCx, &mut T, Space) -> V>,
}

impl<T, V> LayoutBuilder<T, V> {
    /// Create a new [`LayoutBuilder`] view.
    pub fn new(mut builder: impl FnMut(&mut LayoutCx, &mut T, Space) -> V + 'static) -> Self {
        let mut snapshot = Theme::snapshot();

        Self {
            content: Box::new(move |cx, data, space| {
                snapshot.as_global(|| builder(cx, data, space))
            }),
        }
    }
}

impl<T, V: View<T>> View<T> for LayoutBuilder<T, V> {
    type State = Option<(V, V::State)>;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.request_layout();

        None
    }

    fn rebuild(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut RebuildCx,
        _data: &mut T,
        _old: &Self,
    ) {
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some((ref mut view, ref mut state)) = state {
            view.event(state, cx, data, event);
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let mut new_view = (self.content)(cx, data, space);

        if let Some((ref mut view, ref mut state)) = state {
            new_view.rebuild(state, &mut cx.rebuild_cx(), data, view);
            *view = new_view;
        } else {
            let view_state = new_view.build(&mut cx.build_cx(), data);
            *state = Some((new_view, view_state));
        }

        let (view, state) = state.as_mut().expect("state was set earlier");

        view.layout(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some((ref mut view, ref mut state)) = state {
            view.draw(state, cx, data, canvas);
        }
    }
}
