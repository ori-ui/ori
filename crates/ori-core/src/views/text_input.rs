use glam::Vec2;
use ori_graphics::{Color, Quad, Rect, TextSection};
use ori_macro::Build;
use ori_reactive::{CallbackEmitter, Event, OwnedSignal, Signal};

use crate::{
    AvailableSpace, Context, DrawContext, EventContext, Key, KeyboardEvent, LayoutContext,
    Modifiers, PointerEvent, Style, View,
};

#[derive(Clone, Debug, Build)]
pub struct TextInput {
    #[prop]
    placeholder: String,
    #[bind]
    text: OwnedSignal<String>,
    #[event]
    on_input: CallbackEmitter<KeyboardEvent>,
    #[event]
    on_submit: CallbackEmitter<String>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            placeholder: String::from("Enter text..."),
            text: OwnedSignal::new(String::new()),
            on_input: CallbackEmitter::new(),
            on_submit: CallbackEmitter::new(),
        }
    }
}

impl TextInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: OwnedSignal::new(text.into()),
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

    pub fn bind_text(mut self, text: Signal<String>) -> Self {
        self.text.bind(text);
        self
    }

    fn display_text(&self) -> String {
        if self.text.get().is_empty() {
            self.placeholder.clone()
        } else {
            self.text.get()
        }
    }

    fn display_section(
        &self,
        state: &TextInputState,
        space: Option<AvailableSpace>,
        cx: &mut impl Context,
    ) -> TextSection {
        let color = if self.text.get().is_empty() {
            cx.style("placeholder-color")
        } else {
            cx.style("color")
        };

        let rect = if let Some(space) = space {
            Rect::min_size(Vec2::ZERO, space.max - state.padding * 2.0)
        } else {
            cx.rect().shrink(state.padding - 1.0)
        };

        TextSection {
            rect,
            scale: state.font_size,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.display_text(),
            font_family: cx.style("font-family"),
            color,
        }
    }

    fn section(&self, state: &TextInputState, cx: &mut impl Context) -> TextSection {
        TextSection {
            rect: cx.rect().shrink(state.padding - 1.0),
            scale: state.font_size,
            h_align: cx.style("text-align"),
            v_align: cx.style("text-valign"),
            wrap: cx.style("text-wrap"),
            text: self.text.get(),
            font_family: cx.style("font-family"),
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
                    state.cursor = Some(hit.index);
                    state.cursor_right(&self.text.get());
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
            self.on_input.emit(event);

            self.handle_key(state, cx, event.key.unwrap(), event.modifiers);
        }

        if let Some(c) = event.text {
            if !c.is_control() {
                self.input_char(state, cx, c);
            }
        }
    }

    fn input_char(&self, state: &mut TextInputState, cx: &mut EventContext, c: char) {
        let mut text = self.text.get();
        if let Some(cursor) = state.cursor {
            if cursor < text.len() {
                text.insert(cursor, c);
            } else {
                text.push(c);
            }

            let new_cursor = cursor + c.len_utf8();
            state.cursor = Some(new_cursor);
        }

        self.text.set(text);
        cx.request_redraw();
    }

    fn prev_char_index(&self, cursor: usize) -> Option<usize> {
        if cursor == 0 {
            return None;
        }

        let text = self.text.get();

        let mut index = cursor - 1;
        while !text.is_char_boundary(index) {
            index -= 1;
        }

        Some(index)
    }

    fn handle_key(
        &self,
        state: &mut TextInputState,
        cx: &mut EventContext,
        key: Key,
        modifiers: Modifiers,
    ) {
        match key {
            Key::Right => {
                state.cursor_right(&self.text.get());
                cx.request_redraw();
            }
            Key::Left => {
                state.cursor_left(&self.text.get());
                cx.request_redraw();
            }
            Key::Backspace => {
                if let Some(cursor) = state.cursor {
                    if let Some(prev) = self.prev_char_index(cursor) {
                        let mut text = self.text.get();

                        state.cursor_left(&text);
                        text.remove(prev);

                        self.text.set(text);
                        cx.request_redraw();
                    }
                }
            }
            Key::Escape => {
                state.cursor = None;
                cx.unfocus();
            }
            Key::Enter => {
                let wrap = cx.style("text-wrap");

                if !self.on_submit.is_empty() && !modifiers.shift && wrap {
                    self.on_submit.emit(&self.text.get());
                    state.cursor = None;
                    cx.unfocus();
                } else if wrap {
                    self.input_char(state, cx, '\n');
                }
            }
            _ => {}
        }
    }

    fn cursor_rect(&self, state: &mut TextInputState, cx: &mut DrawContext) -> Rect {
        if state.cursor.is_none() || state.cursor == Some(0) {
            return Rect::min_size(
                cx.rect().min + state.padding,
                Vec2::new(0.0, state.font_size),
            );
        }

        let cursor = state.cursor.unwrap();

        let section = self.section(state, cx);
        let glyphs = cx.renderer.text_glyphs(&section);
        let index = glyphs.iter().position(|g| g.index == cursor).unwrap() - 1;

        let glyph = glyphs[index];
        if section.text.as_bytes()[cursor - 1] != b'\n' {
            return glyph.rect;
        }

        let mut newlines = 0;

        while section.text.as_bytes()[cursor - 1 - newlines] == b'\n' {
            newlines += 1;
        }

        Rect::min_size(
            Vec2::new(
                cx.rect().min.x + state.padding,
                glyph.rect.min.y + newlines as f32 * state.font_size,
            ),
            Vec2::new(0.0, state.font_size),
        )
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
        if let Some(ref mut cursor) = self.cursor {
            if *cursor < text.len() {
                *cursor += 1;

                while !text.is_char_boundary(*cursor) {
                    *cursor += 1;
                }
            }
        }

        self.reset_blink();
    }

    fn cursor_left(&mut self, text: &str) {
        if let Some(ref mut cursor) = self.cursor {
            if *cursor > 0 {
                *cursor -= 1;

                while !text.is_char_boundary(*cursor) {
                    *cursor -= 1;
                }
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

    #[tracing::instrument(name = "TextInput", skip(self, state, cx, event))]
    fn event(&self, state: &mut Self::State, cx: &mut EventContext, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            self.handle_pointer_event(state, cx, pointer_event);
        }

        if let Some(keyboard_event) = event.get::<KeyboardEvent>() {
            if cx.focused() {
                self.handle_keyboard_input(state, cx, keyboard_event);
            }
        }
    }

    #[tracing::instrument(name = "TextInput", skip(self, state, cx, space))]
    fn layout(
        &self,
        state: &mut Self::State,
        cx: &mut LayoutContext,
        space: AvailableSpace,
    ) -> Vec2 {
        let font_size = cx.style_range("font-size", 0.0..space.max.y);

        state.font_size = font_size;

        let padding = cx.style_range("padding", 0.0..space.max.min_element() / 2.0);
        state.padding = padding;

        let section = self.display_section(state, Some(space), cx);
        let mut size = cx.messure_text(&section).unwrap_or_default().size();
        size += padding * 2.0;
        space.constrain(size)
    }

    #[tracing::instrument(name = "TextInput", skip(self, state, cx))]
    fn draw(&self, state: &mut Self::State, cx: &mut DrawContext) {
        cx.draw_quad();

        let section = self.display_section(state, None, cx);
        cx.draw(section.clone());

        if state.cursor.is_some() {
            state.blink += cx.state.delta() * 10.0;
            cx.request_redraw();

            let rect = self.cursor_rect(state, cx);

            let cursor_width = 1.0;
            let cursor_height = state.font_size;

            let quad = Quad {
                rect: Rect::min_size(
                    rect.right_center() - Vec2::new(0.0, cursor_height / 2.0),
                    Vec2::new(cursor_width, cursor_height),
                )
                .round(),
                background: cx.style::<Color>("color") * state.blink.cos(),
                ..Quad::default()
            };

            cx.draw_layer(|cx| {
                cx.draw(quad);
            });
        }
    }
}
