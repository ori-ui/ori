use glam::Vec2;

use crate::{
    BoxConstraints, Div, DrawContext, Event, EventContext, Events, IntoChildren, IntoNode,
    LayoutContext, Parent, Style, View,
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

impl Button {
    pub fn new(view: impl View) -> Self {
        Self::default().with_child(view)
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

    fn add_child<I: IntoIterator, U: ?Sized>(&mut self, child: impl IntoChildren<I>)
    where
        I::Item: IntoNode<Self::Child, U>,
    {
        self.content.add_child(child)
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

    #[tracing::instrument(name = "Button", skip(self, state, cx, bc))]
    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        self.content.layout(state, cx, bc)
    }

    #[tracing::instrument(name = "Button", skip(self, state, cx))]
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        self.content.draw(state, cx);
    }
}
