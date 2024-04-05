use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    style::Styles,
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
        let mut snapshot = Styles::snapshot();

        Self {
            content: Box::new(move |cx, data, space| {
                snapshot.as_context(|| builder(cx, data, space))
            }),
        }
    }
}

#[doc(hidden)]
pub struct LayoutBuilderState<T, V: View<T>> {
    state: V::State,
    view: V,
    space: Space,
}

impl<T, V: View<T>> View<T> for LayoutBuilder<T, V> {
    type State = Option<LayoutBuilderState<T, V>>;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.request_layout();

        None
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, _old: &Self) {
        if let Some(ref mut state) = state {
            let mut new_view = (self.content)(&mut cx.layout_cx(), data, state.space);
            new_view.rebuild(&mut state.state, cx, data, &state.view);
            state.view = new_view;
        } else {
            cx.request_layout();
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some(ref mut state) = state {
            state.view.event(&mut state.state, cx, data, event);
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        if let Some(ref mut state) = state {
            if state.space != space {
                let mut new_view = (self.content)(cx, data, space);
                new_view.rebuild(&mut state.state, &mut cx.rebuild_cx(), data, &state.view);
                state.view = new_view;
                state.space = space;
            }
        } else {
            let mut new_view = (self.content)(cx, data, space);
            let view_state = new_view.build(&mut cx.build_cx(), data);
            *state = Some(LayoutBuilderState {
                state: view_state,
                view: new_view,
                space,
            });
        }

        let state = state.as_mut().expect("state was set earlier");

        state.view.layout(&mut state.state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some(ref mut state) = state {
            state.view.draw(&mut state.state, cx, data, canvas);
        }
    }
}
