use glam::Vec2;
use ily_graphics::{Color, Quad, Rect, TextAlign, TextSection};
use ily_macro::Build;

use crate::{
    BoxConstraints, Context, DrawContext, Event, EventContext, EventSignal, Key, KeyboardEvent,
    LayoutContext, PointerEvent, Scope, SharedSignal, Signal, Style, View,
};

#[derive(Clone, Debug, Build)]
pub struct TextInput {
    #[prop]
    placeholder: String,
    #[bind]
    text: SharedSignal<String>,
    #[event]
    on_input: Option<EventSignal<KeyboardEvent>>,
    #[event]
    on_submit: Option<EventSignal<String>>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            placeholder: String::from("Enter text..."),
            text: SharedSignal::new(String::new()),
            on_input: None,
            on_submit: None,
        }
    }
}

impl TextInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: SharedSignal::new(text.into()),
            ..Default::default()
        }
    }

    pub fn with_text(self, text: impl Into<String>) -> Self {
        self.text.set(text.into());
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn bind_text<'a>(self, cx: Scope<'a>, text: &'a Signal<String>) -> Self {
        let signal = cx.alloc(self.text.clone());
        cx.bind(text, &signal);
        self
    }

    fn display_text(&self) -> String {
        if self.text.get().is_empty() {
            self.placeholder.clone()
        } else {
            self.text.cloned()
        }
    }

    fn display_section(&self, state: &TextInputState, cx: &mut impl Context) -> TextSection {
        let color = if self.text.get().is_empty() {
            cx.style("placeholder-color")
        } else {
            cx.style("color")
        };

        TextSection {
            rect: cx.rect().translate(Vec2::new(state.padding, 0.0)),
            scale: state.font_size,
            h_align: TextAlign::Start,
            v_align: TextAlign::Center,
            wrap: false,
            text: self.display_text(),
            font: cx.style("font"),
            color,
        }
    }

    fn section(&self, state: &TextInputState, cx: &mut impl Context) -> TextSection {
        TextSection {
            rect: cx.rect().translate(Vec2::new(state.padding, 0.0)),
            scale: state.font_size,
            h_align: TextAlign::Start,
            v_align: TextAlign::Center,
            wrap: false,
            text: self.text.cloned(),
            font: cx.style("font"),
            ..Default::default()
        }
    }

    fn handle_pointer_event(
        &self,
        state: &mut TextInputState,
        cx: &mut EventContext,
        event: &PointerEvent,
    ) {
        if event.is_press() && cx.hovered() {
            let section = self.section(state, cx);
            let hit = cx.renderer.hit_text(&section, event.position);

            if let Some(hit) = hit {
                if hit.delta.x > 0.0 {
                    state.cursor = Some(hit.index + 1);
                    cx.focus();
                } else {
                    state.cursor = Some(hit.index);
                    cx.focus()
                }
            } else {
                state.cursor = Some(0);
                cx.focus();
            }
        } else if event.is_press() && !cx.hovered() {
            state.cursor = None;
            cx.unfocus();
        }
    }

    fn handle_keyboard_input(
        &self,
        state: &mut TextInputState,
        cx: &mut EventContext,
        event: &KeyboardEvent,
    ) {
        if event.is_press() {
            if let Some(on_input) = &self.on_input {
                on_input.emit(event.clone());
            }

            self.handle_key(state, cx, event.key.unwrap());
        }

        if let Some(c) = event.text {
            if !c.is_control() {
                if let Some(on_input) = &self.on_input {
                    on_input.emit(event.clone());
                }

                let mut text = self.text.modify();
                if let Some(cursor) = state.cursor {
                    if cursor < text.len() {
                        text.insert(cursor, c);
                    } else {
                        text.push(c);
                    }

                    let new_cursor = cursor + c.len_utf8();
                    state.cursor = Some(new_cursor);
                }

                cx.request_redraw();
            }
        }
    }

    fn handle_key(&self, state: &mut TextInputState, cx: &mut EventContext, key: Key) {
        match key {
            Key::Right => {
                state.cursor_right(&self.text.get());
                cx.request_redraw();
            }
            Key::Left => {
                state.cursor_left();
                cx.request_redraw();
            }
            Key::Backspace => {
                if let Some(cursor) = state.cursor {
                    if cursor > 0 {
                        let mut text = self.text.modify();
                        text.remove(cursor - 1);
                        state.cursor_left();
                        cx.request_redraw();
                    }
                }
            }
            Key::Escape => {
                state.cursor = None;
                cx.unfocus();
            }
            Key::Enter => {
                if let Some(on_submit) = &self.on_submit {
                    on_submit.emit(self.text.cloned());
                    state.cursor = None;
                    cx.unfocus();
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextInputState {
    font_size: f32,
    blink: f32,
    padding: f32,
    cursor: Option<usize>,
}

impl TextInputState {
    fn reset_blink(&mut self) {
        self.blink = 0.0;
    }

    fn cursor_right(&mut self, text: &str) {
        if let Some(cursor) = self.cursor {
            if cursor < text.len() {
                self.cursor = Some(cursor + 1);
            }
        }

        self.reset_blink();
    }

    fn cursor_left(&mut self) {
        if let Some(cursor) = self.cursor {
            if cursor > 0 {
                self.cursor = Some(cursor - 1);
            }
        }

        self.reset_blink();
    }
}

impl View for TextInput {
    type State = TextInputState;

    fn build(&self) -> Self::State {
        TextInputState::default()
    }

    fn style(&self) -> Style {
        Style::new("text-input")
    }

    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            self.handle_pointer_event(state, cx, pointer_event);
        }

        if let Some(keyboard_event) = event.get::<KeyboardEvent>() {
            self.handle_keyboard_input(state, cx, keyboard_event);
        }
    }

    fn layout(&self, state: &mut Self::State, cx: &mut LayoutContext, bc: BoxConstraints) -> Vec2 {
        let font_size = cx.style_range("font-size", 0.0..bc.max.y);

        state.font_size = font_size;

        let padding = cx.style_range("padding", 0.0..bc.max.min_element() / 2.0);
        state.padding = padding;

        let min_width = cx.style_range_group("width", "min-width", bc.width());
        let max_width = cx.style_range_group("width", "max-width", bc.width());

        let mut min_height = cx.style_range_group("height", "min-height", bc.height());
        let max_height = cx.style_range_group("height", "max-height", bc.height());

        min_height = min_height.max(font_size + padding * 2.0);

        let min_size = bc.constrain(Vec2::new(min_width, min_height));
        let max_size = bc.constrain(Vec2::new(max_width, max_height));

        let section = self.display_section(state, cx);
        let mut size = cx.messure_text(&section).unwrap_or_default().size();
        size += padding * 2.0;
        size.clamp(min_size, max_size)
    }

    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        let section = self.display_section(state, cx);
        cx.draw(section);

        if let Some(cursor) = state.cursor {
            state.blink += cx.state.delta() * 10.0;
            cx.request_redraw();

            let section = TextSection {
                text: self.text.get()[..cursor].into(),
                ..self.section(state, cx)
            };

            let bounds = cx.renderer.messure_text(&section).unwrap_or_default();
            let cursor = f32::max(bounds.max.x, cx.rect().min.x + state.padding);

            let quad = Quad {
                rect: Rect::min_size(
                    Vec2::new(cursor, cx.rect().min.y + state.padding),
                    Vec2::new(1.0, cx.rect().height() - state.padding * 2.0),
                )
                .rounded(),
                background: cx.style::<Color>("color") * state.blink.cos(),
                ..Quad::default()
            };

            cx.draw(quad);
        }
    }
}
