use crate::{
    canvas::Canvas,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// Create a new [`RebuildHandler`].
pub fn on_rebuild<T, V>(
    content: V,
    rebuild: impl FnMut(&mut RebuildCx, &mut T) + 'static,
) -> RebuildHandler<T, V> {
    RebuildHandler::new(content, rebuild)
}

/// A view that handles rebuilds.
///
/// The [`Self::rebuild`] callback is called when a rebuild is requested.
pub struct RebuildHandler<T, V> {
    /// The content.
    pub content: V,
    /// The callback for when a rebuild is requested.
    #[allow(clippy::type_complexity)]
    pub rebuild: Box<dyn FnMut(&mut RebuildCx, &mut T)>,
}

impl<T, V> RebuildHandler<T, V> {
    /// Create a new [`RebuildHandler`].
    pub fn new(content: V, rebuild: impl FnMut(&mut RebuildCx, &mut T) + 'static) -> Self {
        Self {
            content,
            rebuild: Box::new(rebuild),
        }
    }
}

impl<T, V: View<T>> View<T> for RebuildHandler<T, V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        (self.rebuild)(cx, data);
        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        self.content.event(state, cx, data, event);
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        self.content.layout(state, cx, data, space)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        data: &mut T,
        canvas: &mut Canvas,
    ) {
        self.content.draw(state, cx, data, canvas);
    }
}
