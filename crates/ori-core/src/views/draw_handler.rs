use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::View,
};

/// Create a new [`DrawHandler`].
pub fn on_draw<T, V>(
    content: V,
    on_draw: impl FnMut(&mut DrawCx, &mut T) + 'static,
) -> DrawHandler<T, V> {
    DrawHandler::new(content).on_draw(on_draw)
}

/// A view that hooks into the draw cycle.
pub struct DrawHandler<T, V> {
    /// The content.
    pub content: V,
    /// The draw callback.
    #[allow(clippy::type_complexity)]
    pub on_draw: Option<Box<dyn FnMut(&mut DrawCx, &mut T) + 'static>>,
}

impl<T, V> DrawHandler<T, V> {
    /// Create a new [`DrawHandler`].
    pub fn new(content: V) -> Self {
        Self {
            content,
            on_draw: Option::None,
        }
    }

    /// Set the draw callback.
    pub fn on_draw(mut self, on_draw: impl FnMut(&mut DrawCx, &mut T) + 'static) -> Self {
        self.on_draw = Some(Box::new(on_draw));
        self
    }
}

impl<T, V: View<T>> View<T> for DrawHandler<T, V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
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

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);

        if let Some(ref mut on_draw) = self.on_draw {
            on_draw(cx, data);
        }
    }
}
