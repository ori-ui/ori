use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`BuildHandler`].
pub fn on_build<T, V>(
    content: V,
    after: impl FnMut(&mut BuildCx, &mut T) + 'static,
) -> BuildHandler<T, V> {
    BuildHandler::new(content).after(after)
}

/// A view that hooks into the build cycle.
pub struct BuildHandler<T, V> {
    /// The content.
    pub content: V,
    /// The build callback.
    #[allow(clippy::type_complexity)]
    pub after: Option<Box<dyn FnMut(&mut BuildCx, &mut T) + 'static>>,
}

impl<T, V> BuildHandler<T, V> {
    /// Create a new [`BuildHandler`].
    pub fn new(content: V) -> Self {
        Self {
            content,
            after: None,
        }
    }

    /// Set the build callback for after the `content` is built.
    pub fn after(mut self, after: impl FnMut(&mut BuildCx, &mut T) + 'static) -> Self {
        self.after = Some(Box::new(after));
        self
    }
}

impl<T, V: View<T>> View<T> for BuildHandler<T, V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = self.content.build(cx, data);

        if let Some(after) = &mut self.after {
            after(cx, data);
        }

        state
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
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
