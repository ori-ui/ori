use std::ops::{Deref, DerefMut};

use ori_macro::example;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Padding, Size, Space},
    rebuild::Rebuild,
    view::{Pod, PodState, View},
};

/// Create a new [`Pad`] view.
pub fn pad<V>(padding: impl Into<Padding>, view: V) -> Pad<V> {
    Pad::new(padding.into(), view)
}

/// Create a new [`Pad`] view adding padding to the top.
pub fn pad_top<V>(padding: f32, view: V) -> Pad<V> {
    Pad::new([padding, 0.0, 0.0, 0.0], view)
}

/// Create a new [`Pad`] view adding padding to the right.
pub fn pad_right<V>(padding: f32, view: V) -> Pad<V> {
    Pad::new([0.0, padding, 0.0, 0.0], view)
}

/// Create a new [`Pad`] view adding padding to the bottom.
pub fn pad_bottom<V>(padding: f32, view: V) -> Pad<V> {
    Pad::new([0.0, 0.0, padding, 0.0], view)
}

/// Create a new [`Pad`] view adding padding to the left.
pub fn pad_left<V>(padding: f32, view: V) -> Pad<V> {
    Pad::new([0.0, 0.0, 0.0, padding], view)
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

impl<V> Deref for Pad<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<V> DerefMut for Pad<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<T, V: View<T>> View<T> for Pad<V> {
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
        let content_space = space.shrink(self.padding.size());
        let content_size = self.content.layout(state, cx, data, content_space);

        state.translate(self.padding.offset());

        space.fit(content_size + self.padding.size())
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        self.content.draw(state, cx, data);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        layout::{Rect, Space},
        views::{
            pad, size,
            testing::{save_layout, test_layout},
        },
    };

    #[test]
    fn layout() {
        let inner = save_layout("inner", size(9.0, ()));
        let mut view = save_layout("pad", pad([3.0, 4.0, 5.0, 6.0], inner));

        let layouts = test_layout(&mut view, &mut (), Space::UNBOUNDED);

        assert_eq!(layouts["pad"], Rect::from([0.0, 0.0, 19.0, 17.0]));
        assert_eq!(layouts["inner"], Rect::from([6.0, 3.0, 15.0, 12.0]));
    }
}
