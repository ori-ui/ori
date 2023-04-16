use glam::Vec2;

use crate::{
    BoxConstraints, DrawContext, Event, EventContext, LayoutContext, SharedSignal, Style, View,
};

#[derive(Clone, Debug, Default)]
pub struct TextInput {
    text: SharedSignal<String>,
}

impl TextInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: SharedSignal::new(text.into()),
        }
    }

    pub fn with_text(self, text: impl Into<String>) -> Self {
        self.text.set(text.into());
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    cursor: Option<usize>,
}

impl View for TextInput {
    type State = TextInputState;

    fn build(&self) -> Self::State {
        TextInputState::default()
    }

    fn style(&self) -> Style {
        Style::new("text-input")
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {}

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        bc.min
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {}
}
