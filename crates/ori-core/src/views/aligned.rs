use crate::{
    canvas::Canvas,
    event::Event,
    layout::{Alignment, Size, Space},
    rebuild::Rebuild,
    view::{BuildCx, Content, DrawCx, EventCx, LayoutCx, RebuildCx, State, View},
};

/// A view that aligns its content.
#[derive(Rebuild)]
pub struct Aligned<V> {
    /// The content to align.
    pub content: Content<V>,
    /// The alignment.
    #[rebuild(layout)]
    pub alignment: Alignment,
}

impl<V> Aligned<V> {
    /// Create a new aligned view.
    pub fn new(alignment: Alignment, content: V) -> Self {
        Self {
            content: Content::new(content),
            alignment,
        }
    }
}

impl<T, V: View<T>> View<T> for Aligned<V> {
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
        let content_space = space.loosen();
        let content_size = self.content.layout(state, cx, data, content_space);

        let space = space.constrain(Space::FILL);
        let size = space.fit(content_size);

        let align = self.alignment.align(content_size, size);
        state.translate(align);

        size
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

/// Create a new [`Aligned`] view.
pub fn align<V>(alignment: impl Into<Alignment>, content: V) -> Aligned<V> {
    Aligned::new(alignment.into(), content)
}

/// Create a new [`Aligned`] view that aligns its content to the center.
pub fn center<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::CENTER, content)
}

/// Create a new [`Aligned`] view that aligns its content to the top left.
pub fn top_left<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::TOP_LEFT, content)
}

/// Create a new [`Aligned`] view that aligns its content to the top.
pub fn top<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::TOP, content)
}

/// Create a new [`Aligned`] view that aligns its content to the top right.
pub fn top_right<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::TOP_RIGHT, content)
}

/// Create a new [`Aligned`] view that aligns its content to the left.
pub fn left<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::LEFT, content)
}

/// Create a new [`Aligned`] view that aligns its content to the right.
pub fn right<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::RIGHT, content)
}

/// Create a new [`Aligned`] view that aligns its content to the bottom left.
pub fn bottom_left<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::BOTTOM_LEFT, content)
}

/// Create a new [`Aligned`] view that aligns its content to the bottom.
pub fn bottom<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::BOTTOM, content)
}

/// Create a new [`Aligned`] view that aligns its content to the bottom right.
pub fn bottom_right<V>(content: V) -> Aligned<V> {
    Aligned::new(Alignment::BOTTOM_RIGHT, content)
}
