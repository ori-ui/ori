use ori_macro::example;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Padding, Size, Space},
    rebuild::Rebuild,
    view::{Pod, State, View},
};

/// Create a new [`Pad`] view.
pub fn pad<V>(padding: impl Into<Padding>, content: V) -> Pad<V> {
    Pad::new(padding, content)
}

/// Create a new [`Pad`] view adding padding to the top.
pub fn pad_top<V>(padding: f32, content: V) -> Pad<V> {
    Pad::new([padding, 0.0, 0.0, 0.0], content)
}

/// Create a new [`Pad`] view adding padding to the right.
pub fn pad_right<V>(padding: f32, content: V) -> Pad<V> {
    Pad::new([0.0, padding, 0.0, 0.0], content)
}

/// Create a new [`Pad`] view adding padding to the bottom.
pub fn pad_bottom<V>(padding: f32, content: V) -> Pad<V> {
    Pad::new([0.0, 0.0, padding, 0.0], content)
}

/// Create a new [`Pad`] view adding padding to the left.
pub fn pad_left<V>(padding: f32, content: V) -> Pad<V> {
    Pad::new([0.0, 0.0, 0.0, padding], content)
}

/// A view that adds padding to its content.
#[example(name = "pad", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Pad<V> {
    /// The content.
    pub content: Pod<V>,
    /// The padding.
    #[rebuild(layout)]
    pub padding: Padding,
}

impl<V> Pad<V> {
    /// Create a new [`Pad`] view.
    pub fn new(padding: impl Into<Padding>, content: V) -> Self {
        Self {
            content: Pod::new(content),
            padding: padding.into(),
        }
    }
}

impl<T, V: View<T>> View<T> for Pad<V> {
    type State = State<T, V>;

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
        let content_space = space.shrink(self.padding.size());
        let content_size = self.content.layout(state, cx, data, content_space);

        state.translate(self.padding.offset());

        space.fit(content_size + self.padding.size())
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}
