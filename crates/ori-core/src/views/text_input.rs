use glam::Vec2;

use crate::{
    builtin::text_input, style, BuildCx, Canvas, Code, Color, DrawCx, Event, EventCx, FontFamily,
    FontStretch, FontStyle, FontWeight, Fragment, Glyph, Glyphs, KeyboardEvent, LayoutCx,
    Modifiers, PointerEvent, Primitive, Quad, Rebuild, RebuildCx, Rect, Size, Space, TextAlign,
    TextSection, TextWrap, View,
};

pub fn text_input<T>(text: impl FnMut(&mut T) -> &mut String + 'static) -> TextInput<T> {
    TextInput::new(text)
}

/// A text input.
#[derive(Rebuild)]
pub struct TextInput<T> {
    pub text: Box<dyn FnMut(&mut T) -> &mut String>,
    /// Placeholder text to display when the input is empty.
    #[rebuild(layout)]
    pub placeholder: String,
    /// Whether the input is multi-line.
    ///
    /// When disabled (the default), the input will only accept a single line of text.
    #[rebuild(layout)]
    pub multiline: bool,
    /// The font size of the text.
    #[rebuild(layout)]
    pub font_size: f32,
    /// The font family of the text.
    #[rebuild(layout)]
    pub font_family: FontFamily,
    /// The font weight of the text.
    #[rebuild(layout)]
    pub font_weight: FontWeight,
    /// The font stretch of the text.
    #[rebuild(layout)]
    pub font_stretch: FontStretch,
    /// The font.into of the text.
    #[rebuild(layout)]
    pub font_style: FontStyle,
    /// The color of the text.
    #[rebuild(layout)]
    pub color: Color,
    /// The vertical alignment of the text.
    #[rebuild(layout)]
    pub v_align: TextAlign,
    /// The horizontal alignment of the text.
    #[rebuild(layout)]
    pub h_align: TextAlign,
    /// The line height of the text.
    #[rebuild(layout)]
    pub line_height: f32,
    /// The text wrap of the text.
    #[rebuild(layout)]
    pub wrap: TextWrap,
}

impl<T> TextInput<T> {
    /// Create a new text input view.
    pub fn new(text: impl FnMut(&mut T) -> &mut String + 'static) -> Self {
        Self {
            text: Box::new(text),
            placeholder: String::from("Text..."),
            multiline: false,
            font_size: style(text_input::FONT_SIZE),
            font_family: style(text_input::FONT_FAMILY),
            font_weight: style(text_input::FONT_WEIGHT),
            font_stretch: style(text_input::FONT_STRETCH),
            font_style: style(text_input::FONT_STYLE),
            color: style(text_input::COLOR),
            v_align: style(text_input::V_ALIGN),
            h_align: style(text_input::H_ALIGN),
            line_height: style(text_input::LINE_HEIGHT),
            wrap: style(text_input::WRAP),
        }
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set whether the input is multi-line.
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// Set the font size.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Set the font family.
    pub fn font_family(mut self, font_family: impl Into<FontFamily>) -> Self {
        self.font_family = font_family.into();
        self
    }

    /// Set the font weight.
    pub fn font_weight(mut self, font_weight: impl Into<FontWeight>) -> Self {
        self.font_weight = font_weight.into();
        self
    }

    /// Set the font stretch.
    pub fn font_stretch(mut self, font_stretch: impl Into<FontStretch>) -> Self {
        self.font_stretch = font_stretch.into();
        self
    }

    /// Set the font.into.
    pub fn font_style(mut self, font_style: impl Into<FontStyle>) -> Self {
        self.font_style = font_style.into();
        self
    }

    /// Set the color.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Set the vertical alignment.
    pub fn v_align(mut self, v_align: impl Into<TextAlign>) -> Self {
        self.v_align = v_align.into();
        self
    }

    /// Set the horizontal alignment.
    pub fn h_align(mut self, h_align: impl Into<TextAlign>) -> Self {
        self.h_align = h_align.into();
        self
    }

    /// Set the line height.
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    /// Set the text wrap.
    pub fn wrap(mut self, wrap: impl Into<TextWrap>) -> Self {
        self.wrap = wrap.into();
        self
    }

    fn cursor_select(&self, state: &mut TextInputState, cx: &mut EventCx, text: &str, local: Vec2) {
        if text.is_empty() {
            state.cursor_index = 0;
            return;
        }

        let mut line = None;
        let mut dist = f32::MAX;

        for glyph in state.glyphs.iter().flatten() {
            let delta = local - glyph.rect.center();

            if glyph.rect.contains(local) {
                state.cursor_index = glyph.byte_offset;

                if delta.x > 0.0 {
                    state.cursor_index += glyph.code.len_utf8();
                }

                break;
            }

            if line != Some(glyph.line) && line.is_some() {
                continue;
            }

            let line_top = glyph.baseline - glyph.line_descent;
            let line_bottom = glyph.baseline - glyph.line_ascent;

            if local.y < line_bottom || local.y > line_top {
                continue;
            }

            if delta.length_squared() < dist {
                line = Some(glyph.line);
                dist = delta.length_squared();

                state.cursor_index = glyph.byte_offset;
            }
        }

        state.cursor_blink = 0.0;
        cx.request_draw();
    }

    fn handle_pointer_event(
        &self,
        state: &mut TextInputState,
        cx: &mut EventCx,
        text: &str,
        event: &PointerEvent,
    ) -> bool {
        let local = cx.local(event.position);

        let hovered = cx.rect().contains(local);

        if event.is_press() && hovered {
            cx.set_active(true);
            self.cursor_select(state, cx, text, local);
            cx.request_draw();
            return true;
        }

        if event.is_press() && !hovered {
            cx.set_active(false);
            return false;
        }

        false
    }

    fn prev_char(&self, state: &TextInputState, text: &str) -> Option<char> {
        for i in 1..=4 {
            if state.cursor_index < i {
                continue;
            }

            if text.is_char_boundary(state.cursor_index - i) {
                return text[state.cursor_index - i..].chars().next();
            }
        }

        None
    }

    fn next_char(&self, state: &TextInputState, text: &str) -> Option<char> {
        text[state.cursor_index..].chars().next()
    }

    fn input_text(
        &self,
        state: &mut TextInputState,
        cx: &mut EventCx,
        text: &mut String,
        input: &str,
    ) {
        let mut input = input.replace('\x08', "");

        if !self.multiline {
            input = input.replace(['\n', '\r'], "");
        }

        text.insert_str(state.cursor_index, &input);
        state.cursor_index += input.len();
        state.cursor_blink = 0.0;

        cx.request_layout();
    }

    fn input_key(
        &mut self,
        state: &mut TextInputState,
        cx: &mut EventCx,
        text: &mut String,
        modifiers: Modifiers,
        key: Code,
    ) -> bool {
        match key {
            Code::Escape => {
                cx.set_active(false);
            }
            Code::Backspace => self.input_backspace(state, cx, text),
            Code::Enter => self.input_enter(state, text, modifiers),
            Code::Left => self.input_left(state, cx, text),
            Code::Right => self.input_right(state, cx, text),
            _ => return false,
        }

        true
    }

    fn input_backspace(&mut self, state: &mut TextInputState, cx: &mut EventCx, text: &mut String) {
        let Some(prev_char) = self.prev_char(state, text) else { return };
        text.remove(state.cursor_index - prev_char.len_utf8());
        state.cursor_index -= prev_char.len_utf8();
        state.cursor_blink = 0.0;

        cx.request_layout();
    }

    fn input_enter(&mut self, _state: &mut TextInputState, _text: &mut str, _modifiers: Modifiers) {
        todo!()
    }

    fn input_left(&mut self, state: &mut TextInputState, _cx: &mut EventCx, text: &str) {
        if let Some(prev_char) = self.prev_char(state, text) {
            state.cursor_index -= prev_char.len_utf8();
            state.cursor_blink = 0.0;
        }
    }

    fn input_right(&mut self, state: &mut TextInputState, _cx: &mut EventCx, text: &str) {
        if let Some(next_char) = self.next_char(state, text) {
            state.cursor_index += next_char.len_utf8();
            state.cursor_blink = 0.0;
        }
    }

    fn handle_keyboard_event(
        &mut self,
        state: &mut TextInputState,
        cx: &mut EventCx,
        text: &mut String,
        event: &KeyboardEvent,
    ) -> bool {
        if !cx.is_active() {
            return false;
        }

        if let Some(ref input) = event.text {
            self.input_text(state, cx, text, input);
            return true;
        }

        if let Some(key) = event.key {
            if event.is_press() {
                return self.input_key(state, cx, text, event.modifiers, key);
            }
        }

        true
    }

    fn find_glyph(&self, state: &TextInputState) -> Option<Glyph> {
        let glyphs = state.glyphs.as_ref()?;

        glyphs
            .iter()
            .find(|glyph| glyph.byte_offset == state.cursor_index)
            .copied()
    }

    fn cursor_position(&self, state: &TextInputState) -> Option<Vec2> {
        let glyph = self.find_glyph(state)?;

        Some(Vec2::new(
            glyph.rect.min.x,
            glyph.baseline - (glyph.line_ascent + glyph.line_descent) / 2.0,
        ))
    }
}

#[doc(hidden)]
#[derive(Default, Debug)]
pub struct TextInputState {
    glyphs: Option<Glyphs>,
    cursor_blink: f32,
    cursor_index: usize,
    old_text: String,
}

impl<T> View<T> for TextInput<T> {
    type State = TextInputState;

    fn build(&mut self, _cx: &mut BuildCx, data: &mut T) -> Self::State {
        let text = (self.text)(data);
        TextInputState {
            old_text: text.clone(),
            ..Default::default()
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);

        let text = (self.text)(data);
        if state.old_text != *text {
            state.old_text = text.clone();
            cx.request_layout();
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if let Some(pointer_event) = event.get::<PointerEvent>() {
            let text = (self.text)(data);
            if self.handle_pointer_event(state, cx, text, pointer_event) {
                event.handle();
            }
        }

        if let Some(keyboard_event) = event.get::<KeyboardEvent>() {
            let text = (self.text)(data);
            if self.handle_keyboard_event(state, cx, text, keyboard_event) {
                event.handle();
            }
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        data: &mut T,
        space: Space,
    ) -> Size {
        let mut color = self.color;

        let text = (self.text)(data);
        if text.is_empty() {
            color = color.brighten(0.3);
        }

        let mut text = if text.is_empty() {
            self.placeholder.clone()
        } else {
            text.clone()
        };

        text.push(' ');

        let section = TextSection {
            text: &text,
            font_size: self.font_size,
            font_family: self.font_family.clone(),
            font_weight: self.font_weight,
            font_stretch: self.font_stretch,
            font_style: self.font_style,
            color,
            v_align: self.v_align,
            h_align: self.h_align,
            line_height: self.line_height,
            wrap: self.wrap,
            bounds: space.max,
        };

        state.glyphs = cx.layout_text(&section);
        state.glyphs.as_ref().map_or(Size::ZERO, Glyphs::size)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        let Some(ref glyphs) = state.glyphs else { return };

        if let Some(mesh) = cx.text_mesh(glyphs, cx.rect()) {
            canvas.draw(mesh);
        }

        if !cx.is_active() {
            return;
        }

        cx.request_draw();

        let offset = glyphs.offset(cx.rect());
        let cursor_center = match self.cursor_position(state) {
            Some(position) => position + offset,
            None => cx.rect().left(),
        };

        let cursor_size = Size::new(1.0, self.font_size);
        let cursor_min = cursor_center - cursor_size / 2.0;

        state.cursor_blink += cx.dt() * 10.0;

        let mut color = self.color;
        color.a = state.cursor_blink.sin() * 0.5 + 0.5;

        let quad = Quad {
            rect: Rect::min_size(cursor_min.round(), cursor_size),
            color,
            border_radius: [0.0; 4],
            border_width: [0.0; 4],
            border_color: Color::TRANSPARENT,
        };

        canvas.draw_fragment(Fragment {
            primitive: Primitive::Quad(quad),
            transform: canvas.transform.round(),
            depth: canvas.depth,
            clip: canvas.clip,
        });
    }
}
