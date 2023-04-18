use glam::Vec2;
use ily_graphics::{Rect, TextSection};

use crate::{
    Bindable, BoxConstraints, DrawContext, Event, EventContext, LayoutContext, Properties, Scope,
    SharedSignal, Signal, Style, View,
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

    pub fn bind_text<'a>(self, cx: Scope<'a>, text: &'a Signal<String>) -> Self {
        let signal = cx.alloc(self.text.clone());
        cx.bind(text, &signal);
        self
    }
}

const _: () = {
    pub struct TextInputProperties<'a> {
        text_input: &'a mut TextInput,
    }

    impl<'a> TextInputProperties<'a> {
        pub fn text(&mut self, text: impl Into<String>) {
            self.text_input.text.set(text.into());
        }
    }

    impl Properties for TextInput {
        type Setter<'a> = TextInputProperties<'a>;

        fn setter(&mut self) -> Self::Setter<'_> {
            TextInputProperties { text_input: self }
        }
    }

    pub struct TextInputBinding<'a> {
        text_input: &'a mut TextInput,
    }

    impl<'a> TextInputBinding<'a> {
        pub fn text<'b>(&self, cx: Scope<'b>, text: &'b Signal<String>) {
            let signal = cx.alloc(self.text_input.text.clone());
            cx.bind(text, &signal);
        }
    }

    impl Bindable for TextInput {
        type Setter<'a> = TextInputBinding<'a>;

        fn setter(&mut self) -> Self::Setter<'_> {
            TextInputBinding { text_input: self }
        }
    }
};

#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    font_size: f32,
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
        let font_size = cx.style_range("font-size", 0.0..bc.max.y);

        state.font_size = font_size;

        let section = TextSection {
            rect: Rect::min_size(Vec2::ZERO, bc.max),
            scale: font_size,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.text.cloned(),
            font: cx.style("font"),
            color: cx.style("color"),
        };

        let bounds = cx.text_bounds(&section).unwrap_or_default();
        bounds.size()
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        let section = TextSection {
            rect: cx.rect(),
            scale: state.font_size,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.text.cloned(),
            font: cx.style("font"),
            color: cx.style("color"),
        };

        cx.draw_primitive(section);
    }
}
