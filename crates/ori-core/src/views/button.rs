use glam::Vec2;
use ori_reactive::Event;
use ori_style::Style;

use crate::{
    AvailableSpace, Div, DrawContext, EventContext, Events, IntoNode, LayoutContext, Node, Parent,
    View,
};

/// A button view.
#[derive(Default)]
pub struct Button {
    /// The content of the button.
    pub content: Div,
}

impl Button {
    /// Create a new button.
    pub fn new(child: impl IntoNode) -> Self {
        Self {
            content: Div::new().with_child(child),
        }
    }
}

impl Events for Button {
    type Setter<'a> = <Div as Events>::Setter<'a>;

    fn setter(&mut self) -> Self::Setter<'_> {
        Events::setter(&mut self.content)
    }
}

impl Parent for Button {
    type Child = <Div as Parent>::Child;

    fn clear_children(&mut self) {
        self.content.clear_children();
    }

    fn add_children(&mut self, child: impl Iterator<Item = Node<Self::Child>>) -> usize {
        self.content.add_children(child)
    }

    fn set_children(&mut self, slot: usize, child: impl Iterator<Item = Node<Self::Child>>) {
        self.content.set_children(slot, child)
    }
}

impl View for Button {
    type State = <Div as View>::State;

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("button")
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.content.event(state, cx, event);
    }

    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        self.content.layout(state, cx, space)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.content.draw(state, cx);
    }
}
