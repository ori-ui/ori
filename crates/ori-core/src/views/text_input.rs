use ori_macro::{example, Build, Styled};

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Capitalize, Event, Ime, Key},
    layout::{Point, Rect, Size, Space},
    style::{Styled, Theme},
    text::{
        FontAttributes, FontFamily, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign,
        TextLayoutLine, TextWrap,
    },
    view::View,
    window::Cursor,
};

/// Create a new [`TextInput`].
pub fn text_input<T>() -> TextInput<T> {
    TextInput::new()
}

/// A text input.
///
/// Can be styled using the [`TextInputStyle`].
#[example(name = "text_input", width = 400, height = 300)]
#[derive(Styled, Build)]
pub struct TextInput<T> {
    /// The text.
    #[build(ignore)]
    pub text: Option<String>,

    /// A callback that is called when an input is received.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_input: Option<Box<dyn FnMut(&mut EventCx, &mut T, String)>>,

    /// A callback that is called when the input is submitted.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_submit: Option<Box<dyn FnMut(&mut EventCx, &mut T, String)>>,

    /// Placeholder text to display when the input is empty.
    pub placeholder: String,

    /// Whether the input is multi-line.
    ///
    /// When disabled (the default), the input will only accept a single line of text.
    pub multiline: bool,

    /// How the text should be capitalized.
    ///
    /// This only affects text input from IMEs, eg. on-screen keyboards like the ones on mobile
    /// devices.
    pub capitalize: Capitalize,

    /// The font size of the text.
    #[styled(default = 16.0)]
    #[rebuild(layout)]
    pub font_size: Styled<f32>,

    /// The font family of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_family: Styled<FontFamily>,

    /// The font weight of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_weight: Styled<FontWeight>,

    /// The font stretch of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_stretch: Styled<FontStretch>,

    /// The font.into of the text.
    #[styled(default)]
    #[rebuild(layout)]
    pub font_style: Styled<FontStyle>,

    /// The color of the text.
    #[styled(default -> Theme::CONTRAST or Color::BLACK)]
    #[rebuild(draw)]
    pub color: Styled<Color>,

    /// The color of the placeholder text.
    #[styled(default -> Theme::CONTRAST_LOW or Color::grayscale(0.9))]
    #[rebuild(draw)]
    pub placeholder_color: Styled<Color>,

    /// The vertical alignment of the text.
    #[styled(default)]
    pub align: Styled<TextAlign>,

    /// The line height of the text.
    #[styled(default = 1.2)]
    pub line_height: Styled<f32>,

    /// The text wrap of the text.
    #[styled(default)]
    pub wrap: Styled<TextWrap>,
}

impl<T> Default for TextInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextInput<T> {
    /// Create a new text input view.
    pub fn new() -> Self {
        Self {
            text: None,
            on_input: None,
            on_submit: None,
            placeholder: String::from("..."),
            multiline: false,
            capitalize: Capitalize::Sentences,
            font_size: TextInputStyle::FONT_SIZE.into(),
            font_family: TextInputStyle::FONT_FAMILY.into(),
            font_weight: TextInputStyle::FONT_WEIGHT.into(),
            font_stretch: TextInputStyle::FONT_STRETCH.into(),
            font_style: TextInputStyle::FONT_STYLE.into(),
            color: TextInputStyle::COLOR.into(),
            placeholder_color: TextInputStyle::PLACEHOLDER_COLOR.into(),
            align: TextInputStyle::ALIGN.into(),
            line_height: TextInputStyle::LINE_HEIGHT.into(),
            wrap: TextInputStyle::WRAP.into(),
        }
    }

    /// Set the text of the input.
    pub fn text(mut self, text: impl ToString) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Set the callback that is called when an input is received.
    ///
    /// Note that this doesn't trigger a rebuild automatically.
    pub fn on_input(
        mut self,
        on_change: impl FnMut(&mut EventCx, &mut T, String) + 'static,
    ) -> Self {
        self.on_input = Some(Box::new(on_change));
        self
    }

    /// Set the callback that is called when the input is submitted.
    pub fn on_submit(
        mut self,
        on_submit: impl FnMut(&mut EventCx, &mut T, String) + 'static,
    ) -> Self {
        self.on_submit = Some(Box::new(on_submit));
        self
    }
}

#[doc(hidden)]
pub struct TextInputState {
    // the style of the text input
    style: TextInputStyle,

    // the current text of the input
    text: String,
    paragraph: Paragraph,
    lines: Vec<TextLayoutLine>,

    dragging: bool,
    move_offset: Option<f32>,

    blink: f32,
    cursor: usize,
    selection: Option<usize>,
}

impl TextInputState {
    fn set_cursor(&mut self, cursor: usize, select: bool) {
        if !select {
            self.selection = None;
        } else if self.selection.is_none() {
            self.selection = Some(self.cursor);
        }

        self.cursor = cursor;
        self.blink = 0.0;
        self.move_offset = None;
    }

    fn remove_selection(&mut self) {
        if let Some(selection) = self.selection {
            let start = usize::min(self.cursor, selection);
            let end = usize::max(self.cursor, selection);

            self.text.drain(start..end);
            self.set_cursor(start, false);

            self.selection = None;
        }
    }

    fn move_right(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            // if the selection is active, clear it

            if self.cursor < self.selection.unwrap() {
                self.cursor = self.selection.unwrap();
            }

            self.selection = None;
            return;
        }

        if self.cursor >= self.text.len() {
            // if the cursor is at the end of the text, do nothing
            return;
        }

        let next_char = self.text[self.cursor..].chars().next().unwrap();
        self.set_cursor(self.cursor + next_char.len_utf8(), select);
    }

    fn move_left(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            // if the selection is active, clear it

            if self.cursor > self.selection.unwrap() {
                self.cursor = self.selection.unwrap();
            }

            self.selection = None;
            return;
        }

        if self.cursor == 0 {
            // if the cursor is at the start of the text, do nothing
            return;
        }

        // FIXME: this might be slow
        let prev_char = self.text[..self.cursor].chars().next_back().unwrap();
        self.set_cursor(self.cursor - prev_char.len_utf8(), select);
    }

    fn move_up(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            // if the selection is active, clear it

            self.selection = None;
            return;
        }

        let line = self.current_line_number();
        let next_line = line.saturating_sub(1);

        if self.move_offset.is_none() {
            self.move_offset = Some(self.get_cursor_offset());
        }

        if !select {
            self.selection = None;
        } else if self.selection.is_none() {
            self.selection = Some(self.cursor);
        }

        let move_offset = self.move_offset.unwrap();
        self.cursor = self.select_point_in_line(next_line, move_offset);
        self.blink = 0.0;
    }

    fn move_down(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            // if the selection is active, clear it

            self.selection = None;
            return;
        }

        let line = self.current_line_number();
        let next_line = usize::min(line + 1, self.lines.len() - 1);

        if self.move_offset.is_none() {
            self.move_offset = Some(self.get_cursor_offset());
        }

        if !select {
            self.selection = None;
        } else if self.selection.is_none() {
            self.selection = Some(self.cursor);
        }

        let move_offset = self.move_offset.unwrap();
        self.cursor = self.select_point_in_line(next_line, move_offset);
        self.blink = 0.0;
    }

    fn get_cursor_offset(&self) -> f32 {
        if self.lines.is_empty() {
            return 0.0;
        }

        let line_index = self.current_line_number();

        for glyph in &self.lines[line_index].glyphs {
            if glyph.range.start == self.cursor {
                return glyph.bounds.left();
            }
        }

        let line = &self.lines[line_index];

        if let Some(glyph) = line.glyphs.last() {
            glyph.bounds.right()
        } else {
            line.right()
        }
    }

    fn current_line_number(&self) -> usize {
        for (i, line) in self.lines.iter().enumerate() {
            if self.cursor < line.range.end + 1 {
                return i;
            }
        }

        self.lines.len() - 1
    }

    fn select_point_in_line(&self, line_index: usize, offset: f32) -> usize {
        let line = &self.lines[line_index];

        for glyph in &line.glyphs {
            if offset < glyph.bounds.center().x {
                return glyph.range.start;
            }
        }

        line.range.end
    }

    fn select_point(&self, point: Point) -> usize {
        for (i, line) in self.lines.iter().enumerate() {
            if point.y <= line.bottom() {
                return self.select_point_in_line(i, point.x);
            }
        }

        0
    }
}

impl<T> View<T> for TextInput<T> {
    type State = TextInputState;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        cx.set_focusable(true);

        let style = TextInputStyle::styled(self, cx.styles());

        let mut paragraph = Paragraph::new(style.line_height, style.align, style.wrap);

        paragraph.set_text(
            self.text.as_deref().unwrap_or_default(),
            FontAttributes {
                size: style.font_size,
                family: style.font_family.clone(),
                weight: style.font_weight,
                stretch: style.font_stretch,
                style: style.font_style,
                ligatures: false,
                color: style.color,
            },
        );

        let text = self.text.clone().unwrap_or_default();
        let cursor = text.len();

        TextInputState {
            style,
            text,
            paragraph,
            lines: Vec::new(),
            dragging: false,
            move_offset: None,
            blink: 0.0,
            cursor,
            selection: None,
        }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, _old: &Self) {
        state.style.rebuild(self, cx);

        if let Some(text) = &self.text {
            if state.cursor >= state.text.len() {
                state.cursor = text.len();
            }

            state.text = text.clone();
            state.lines.clear();

            cx.layout();
        }

        if state.paragraph.line_height != state.style.line_height
            || state.paragraph.align != state.style.align
            || state.paragraph.wrap != state.style.wrap
        {
            state.paragraph.line_height = state.style.line_height;
            state.paragraph.align = state.style.align;
            state.paragraph.wrap = state.style.wrap;

            cx.layout();
        }

        state.paragraph.set_text(
            &state.text,
            FontAttributes {
                size: state.style.font_size,
                family: state.style.font_family.clone(),
                weight: state.style.font_weight,
                stretch: state.style.font_stretch,
                style: state.style.font_style,
                ligatures: false,
                color: state.style.color,
            },
        );
    }

    fn event(
        &mut self,
        state: &mut Self::State,
        cx: &mut EventCx,
        data: &mut T,
        event: &Event,
    ) -> bool {
        if cx.focused_changed() {
            if cx.is_focused() {
                state.blink = 0.0;
                state.selection = None;
            } else {
                state.selection = None;
            }

            cx.draw();
        }

        if cx.is_hovered() {
            cx.set_cursor(Some(Cursor::Text));
        } else {
            cx.set_cursor(None);
        }

        if cx.is_focused() {
            let selection = state.selection.unwrap_or(state.cursor);

            let min = usize::min(state.cursor, selection);
            let max = usize::max(state.cursor, selection);

            cx.set_ime(Some(Ime {
                text: state.text.clone(),
                selection: min..max,
                compose: None,
                multiline: self.multiline,
                capitalize: self.capitalize,
            }));

            cx.animate();
        } else {
            cx.set_ime(None);
        }

        match event {
            Event::PointerPressed(e) if cx.is_hovered() => {
                let local = cx.local(e.position);
                state.set_cursor(state.select_point(local), false);
                state.dragging = true;

                cx.focus();

                true
            }

            Event::PointerMoved(e) if state.dragging => {
                let local = cx.local(e.position);
                let local = cx.rect().contain(local);
                let cursor = state.select_point(local);
                state.set_cursor(cursor, true);

                cx.draw();

                true
            }

            Event::PointerReleased(_) if state.dragging => {
                state.dragging = false;

                true
            }

            Event::KeyPressed(e) if cx.is_focused() => {
                let mut text_changed = false;
                let mut text_submitted = false;

                if let Some(ref text) = e.text {
                    if !text.chars().any(char::is_control) && !e.modifiers.ctrl {
                        state.remove_selection();
                        state.text.insert_str(state.cursor, text);
                        state.set_cursor(state.cursor + text.len(), false);

                        text_changed = true;
                    }
                }

                if e.is_key('v') && e.modifiers.ctrl {
                    state.remove_selection();

                    let text = cx.clipboard().get();

                    state.text.insert_str(state.cursor, &text);
                    state.set_cursor(state.cursor + text.len(), false);

                    text_changed = true;
                }

                if e.is_key('c') && e.modifiers.ctrl {
                    if let Some(selection) = state.selection {
                        let start = usize::min(state.cursor, selection);
                        let end = usize::max(state.cursor, selection);

                        let text = state.text[start..end].to_string();
                        cx.clipboard().set(text);
                    }
                }

                if e.is_key('x') && e.modifiers.ctrl {
                    if let Some(selection) = state.selection {
                        let start = usize::min(state.cursor, selection);
                        let end = usize::max(state.cursor, selection);

                        let text = state.text.drain(start..end).collect::<String>();
                        cx.clipboard().set(text);

                        state.set_cursor(start, false);

                        text_changed = true;
                    }
                }

                if e.is_key(Key::Escape) {
                    if state.selection.is_some() {
                        state.selection = None;
                    } else {
                        cx.set_focused(false);
                    }
                }

                if e.is_key(Key::Enter) && self.multiline {
                    state.remove_selection();
                    state.text.insert(state.cursor, '\n');
                    state.set_cursor(state.cursor + 1, false);

                    text_changed = true;
                }

                if e.is_key(Key::Enter) && !self.multiline {
                    cx.focus_next();

                    text_changed = true;
                    text_submitted = true;
                }

                if e.is_key(Key::Backspace) {
                    if state.selection.is_some() {
                        state.remove_selection();
                        text_changed = true;
                    } else if state.cursor > 0 {
                        state.move_left(false);
                        state.text.remove(state.cursor);
                        text_changed = true;
                    }
                }

                if e.is_key(Key::Right) {
                    state.move_right(e.modifiers.shift);
                    cx.draw();
                }

                if e.is_key(Key::Left) {
                    state.move_left(e.modifiers.shift);
                    cx.draw();
                }

                if e.is_key(Key::Up) {
                    state.move_up(e.modifiers.shift);
                    cx.draw();
                }

                if e.is_key(Key::Down) {
                    state.move_down(e.modifiers.shift);
                    cx.draw();
                }

                if text_changed {
                    if let Some(on_input) = &mut self.on_input {
                        on_input(cx, data, state.text.clone());
                    }

                    state.paragraph.set_text(
                        &state.text,
                        FontAttributes {
                            size: state.style.font_size,
                            family: state.style.font_family.clone(),
                            weight: state.style.font_weight,
                            stretch: state.style.font_stretch,
                            style: state.style.font_style,
                            ligatures: false,
                            color: state.style.color,
                        },
                    );

                    state.lines.clear();

                    cx.layout();
                }

                if text_submitted {
                    if let Some(on_submit) = &mut self.on_submit {
                        on_submit(cx, data, state.text.clone());
                    }
                }

                true
            }

            Event::Animate(dt) => {
                state.blink += *dt;

                cx.draw();

                false
            }
            _ => false,
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        if state.text.is_empty() {
            state.lines.clear();

            let mut placeholder = Paragraph::new(
                // please, don't make this ugly rustfmt
                state.style.line_height,
                state.style.align,
                state.style.wrap,
            );

            placeholder.set_text(
                &self.placeholder,
                FontAttributes {
                    size: state.style.font_size,
                    family: state.style.font_family.clone(),
                    weight: state.style.font_weight,
                    stretch: state.style.font_stretch,
                    style: state.style.font_style,
                    ligatures: false,
                    color: state.style.placeholder_color,
                },
            );

            let size = cx.measure_paragraph(&placeholder, space.max.width);
            return space.fit(size);
        }

        state.lines = cx.layout_paragraph(&state.paragraph, space.max.width);
        let size = cx.measure_paragraph(&state.paragraph, space.max.width);
        space.fit(size)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.trigger(cx.rect());

        if !state.text.is_empty() {
            cx.paragraph(&state.paragraph, cx.rect());
        } else {
            let mut placeholder = Paragraph::new(
                // please, don't make this ugly rustfmt
                state.style.line_height,
                state.style.align,
                state.style.wrap,
            );

            placeholder.set_text(
                &self.placeholder,
                FontAttributes {
                    size: state.style.font_size,
                    family: state.style.font_family.clone(),
                    weight: state.style.font_weight,
                    stretch: state.style.font_stretch,
                    style: state.style.font_style,
                    ligatures: false,
                    color: state.style.placeholder_color,
                },
            );

            cx.paragraph(&placeholder, cx.rect());
        }

        let contrast = cx.styles().get_or(Color::BLACK, Theme::CONTRAST);
        let info = cx.styles().get_or(Color::BLUE, Theme::INFO);

        // draw the cursor
        if cx.is_focused() {
            let color = f32::cos(state.blink * 5.0).abs();

            draw_highlight(state, cx, info.fade(0.5));

            if state.selection.is_none() {
                draw_cursor(state, cx, contrast.fade(color));
            }
        }
    }
}

fn draw_highlight(state: &mut TextInputState, cx: &mut DrawCx, color: Color) {
    if let Some(selection) = state.selection {
        let start = usize::min(state.cursor, selection);
        let end = usize::max(state.cursor, selection);

        for line in &state.lines {
            let top = line.top();
            let bottom = line.bottom();

            if line.glyphs.is_empty() && line.range.start >= start && line.range.end <= end {
                let rect = Rect::new(
                    Point::new(line.left(), top),
                    Point::new(line.left() + 2.0, bottom),
                );

                cx.fill_rect(rect, color);

                continue;
            }

            let mut left = line.right();
            let mut right = line.left();

            for glyph in &line.glyphs {
                if start <= glyph.range.start && end >= glyph.range.start {
                    left = f32::min(left, glyph.bounds.left());
                }

                if start <= glyph.range.end && end >= glyph.range.end {
                    right = f32::max(right, glyph.bounds.right());
                }
            }

            if left >= right {
                continue;
            }

            let rect = Rect::new(Point::new(left, top), Point::new(right, bottom));
            cx.fill_rect(rect, color);
        }
    }
}

fn draw_cursor(state: &mut TextInputState, cx: &mut DrawCx, color: Color) {
    if state.lines.is_empty() {
        // if there are no lines, just draw the cursor at the start

        let size = Size::new(1.0, cx.size().height);
        let rect = Rect::min_size(cx.rect().top_left(), size);

        cx.fill_rect(rect, color);

        return;
    }

    let line = &state.lines[state.current_line_number()];
    let offset = state.get_cursor_offset();

    let size = Size::new(1.0, line.height());
    let rect = Rect::min_size(Point::new(offset, line.top()), size);

    cx.fill_rect(rect, color);
}
