use ori_macro::example;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Alignment, Size, Space},
    rebuild::Rebuild,
    view::{Pod, PodState, View},
};

/// Create a new [`Aligned`] view.
pub fn align<V>(alignment: impl Into<Alignment>, view: V) -> Aligned<V> {
    Aligned::new(alignment.into(), view)
}

/// Create a new [`Aligned`] view that aligns its content to the center.
pub fn center<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::CENTER, view)
}

/// Create a new [`Aligned`] view that aligns its content to the top left.
pub fn top_left<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::TOP_LEFT, view)
}

/// Create a new [`Aligned`] view that aligns its content to the top.
pub fn top<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::TOP, view)
}

/// Create a new [`Aligned`] view that aligns its content to the top right.
pub fn top_right<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::TOP_RIGHT, view)
}

/// Create a new [`Aligned`] view that aligns its content to the left.
pub fn left<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::LEFT, view)
}

/// Create a new [`Aligned`] view that aligns its content to the right.
pub fn right<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::RIGHT, view)
}

/// Create a new [`Aligned`] view that aligns its content to the bottom left.
pub fn bottom_left<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::BOTTOM_LEFT, view)
}

/// Create a new [`Aligned`] view that aligns its content to the bottom.
pub fn bottom<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::BOTTOM, view)
}

/// Create a new [`Aligned`] view that aligns its content to the bottom right.
pub fn bottom_right<V>(view: V) -> Aligned<V> {
    Aligned::new(Alignment::BOTTOM_RIGHT, view)
}

/// A view that aligns its content.
#[example(name = "align", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Aligned<V> {
    /// The content to align.
    pub content: Pod<V>,
    /// The alignment.
    #[rebuild(layout)]
    pub alignment: Alignment,
}

impl<V> Aligned<V> {
    /// Create a new aligned view.
    pub fn new(alignment: Alignment, content: V) -> Self {
        Self {
            content: Pod::new(content),
            alignment,
        }
    }
}

impl<T, V: View<T>> View<T> for Aligned<V> {
    type State = PodState<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        self.content.rebuild(state, cx, data, &old.content);
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        self.content.event(state, cx, data, event)
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

        let size = content_size
            .max(space.min.finite_or_zero())
            .max(space.max.finite_or_zero());

        let align = self.alignment.align(content_size, size);
        state.translate(align);

        size
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}
