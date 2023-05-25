use glam::Vec2;
use ori_reactive::Event;

use crate::{
    AvailableSpace, Div, DrawContext, Element, EventContext, Events, LayoutContext, Parent, Style,
    View,
};

pub struct Button {
    pub content: Div,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            content: Div::new(),
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

    fn add_children(&mut self, child: impl Iterator<Item = Element<Self::Child>>) -> usize {
        self.content.add_children(child)
    }

    fn set_children(&mut self, slot: usize, child: impl Iterator<Item = Element<Self::Child>>) {
        self.content.set_children(slot, child)
    }
}

impl View for Button {
    type State = <Div as View>::State;

    fn build(&self) -> Self::State {}

    fn style(&self) -> Style {
        Style::new("button")
    }

    #[tracing::instrument(name = "Button", skip(self, state, cx, event))]
    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        self.content.event(state, cx, event);
    }

    #[tracing::instrument(name = "Button", skip(self, state, cx, space))]
    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        self.content.layout(state, cx, space)
    }

    #[tracing::instrument(name = "Button", skip(self, state, cx))]
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.content.draw(state, cx);
    }
}
