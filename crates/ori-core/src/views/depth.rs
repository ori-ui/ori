use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

/// Create a new [`Depth`] view.
pub fn depth<V>(depth: f32, content: V) -> Depth<V> {
    Depth::new(depth, content)
}

/// A view with a depth value.
#[derive(Rebuild)]
pub struct Depth<V> {
    /// The content.
    pub content: V,
    /// The depth value.
    #[rebuild(draw)]
    pub depth: f32,
}

impl<V> Depth<V> {
    /// Create a new [`Depth`] view.
    pub fn new(depth: f32, content: V) -> Self {
        Self { content, depth }
    }
}

impl<T, V: View<T>> View<T> for Depth<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

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
        let mut layer = canvas.layer();
        layer.depth = self.depth;
        self.content.draw(state, cx, data, &mut layer);
    }
}
