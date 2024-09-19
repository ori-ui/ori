use ori_macro::example;

use crate::{
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    view::{PodSeq, SeqState, View, ViewSeq},
};

pub use crate::zstack;

/// Create a new [`ZStack`] view.
#[macro_export]
macro_rules! zstack {
    ($($child:expr),* $(,)?) => {
        $crate::views::zstack(($($child,)*))
    };
}

/// Create a new [`ZStack`] view.
pub fn zstack<V>(view: V) -> ZStack<V> {
    ZStack::new(view)
}

/// A view that overlays its content on top of each other.
#[example(name = "zstack", width = 400, height = 300)]
pub struct ZStack<V> {
    /// The content to overlay.
    pub content: PodSeq<V>,
}

impl<V> ZStack<V> {
    /// Create a new overlay view.
    pub fn new(content: V) -> Self {
        Self {
            content: PodSeq::new(content),
        }
    }
}

impl<T, V: ViewSeq<T>> View<T> for ZStack<V> {
    type State = SeqState<T, V>;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        self.content.build(cx, data)
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        (self.content).rebuild(state, &mut cx.as_build_cx(), data, &old.content);

        for i in 0..self.content.len() {
            self.content.rebuild_nth(i, state, cx, data, &old.content);
        }
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
        let size = space.fit(self.content.layout_nth(0, state, cx, data, space));

        for i in 1..self.content.len() {
            let content_space = Space::new(space.min, size);
            self.content.layout_nth(i, state, cx, data, content_space);
        }

        size
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, data: &mut T) {
        for i in 0..self.content.len() {
            self.content.draw_nth(i, state, cx, data);
        }
    }
}
