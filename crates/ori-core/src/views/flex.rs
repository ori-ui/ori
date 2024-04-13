use ori_macro::example;

use crate::{
    canvas::Canvas,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    rebuild::Rebuild,
    view::View,
};

/// Create a new [`Flex`] view.
pub fn flex<V>(content: V) -> Flex<V> {
    Flex::new(1.0, false, content)
}

/// Create a new expanded [`Flex`] view.
pub fn expand<V>(content: V) -> Flex<V> {
    Flex::new(1.0, true, content)
}

/// A flexible view.
#[example(name = "flex", width = 400, height = 300)]
#[derive(Rebuild)]
pub struct Flex<V> {
    /// The content of the view.
    pub content: V,
    /// The flex value of the view.
    pub flex: f32,
    /// Whether the view is tight.
    pub tight: bool,
}

impl<V> Flex<V> {
    /// Create a new flexible view.
    pub fn new(flex: f32, tight: bool, content: V) -> Self {
        Self {
            content,
            flex,
            tight,
        }
    }

    /// Set the flex value of the view.
    pub fn factor(mut self, flex: f32) -> Self {
        self.flex = flex;
        self
    }
}

impl<T, V: View<T>> View<T> for Flex<V> {
    type State = V::State;

    fn build(&mut self, cx: &mut BuildCx, data: &mut T) -> Self::State {
        let state = self.content.build(cx, data);

        cx.set_flex(self.flex);
        cx.set_tight(self.tight);

        state
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        self.content.rebuild(state, cx, data, &old.content);

        cx.set_flex(self.flex);
        cx.set_tight(self.tight);
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
