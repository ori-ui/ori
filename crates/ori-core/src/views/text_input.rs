use cosmic_text::{
    Action, Attrs, AttrsList, Buffer, BufferLine, BufferRef, Edit, Editor, LineEnding, Metrics,
    Motion, Shaping,
};
use ori_macro::{example, Build};

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, Key, KeyPressed},
    layout::{Point, Rect, Size, Space, Vector},
    style::{style, Palette, Style, Styles},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    view::View,
    window::Cursor,
};

use super::TextStyle;

/// Create a new [`TextInput`].
pub fn text_input<T>() -> TextInput<T> {
    TextInput::new()
}

/// The style of a text input.
#[derive(Clone, Debug)]
pub struct TextInputStyle {
    /// The font size of the text.
    pub font_size: f32,

    /// The font family of the text.
    pub font_family: FontFamily,

    /// The font weight of the text.
    pub font_weight: FontWeight,

    /// The font stretch of the text.
    pub font_stretch: FontStretch,

    /// The font style of the text.
    pub font_style: FontStyle,

    /// The color of the text.
    pub color: Color,

    /// The color of the placeholder text.
    pub placeholder_color: Color,

    /// The horizontal alignment of the text.
    pub align: TextAlign,

    /// The line height of the text.
    pub line_height: f32,

    /// The text wrap of the text.
    pub wrap: TextWrap,
}

impl Style for TextInputStyle {
    fn styled(style: &Styles) -> Self {
        let text_style = style.get::<TextStyle>();
        let palette = style.get::<Palette>();

        Self {
            font_size: text_style.font_size,
            font_family: text_style.font_family,
            font_weight: text_style.font_weight,
            font_stretch: text_style.font_stretch,
            font_style: text_style.font_style,
            color: text_style.color,
            placeholder_color: palette.surface_highest,
            align: text_style.align,
            line_height: text_style.line_height,
            wrap: text_style.wrap,
        }
    }
}

/// A text input.
///
/// Can be styled using the [`TextInputStyle`].
#[example(name = "text_input", width = 400, height = 300)]
#[derive(Build)]
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

    /// The font size of the text.
    pub font_size: f32,

    /// The font family of the text.
    pub font_family: FontFamily,

    /// The font weight of the text.
    pub font_weight: FontWeight,

    /// The font stretch of the text.
    pub font_stretch: FontStretch,

    /// The font.into of the text.
    pub font_style: FontStyle,

    /// The color of the text.
    pub color: Color,

    /// The color of the placeholder text.
    pub placeholder_color: Color,

    /// The vertical alignment of the text.
    pub align: TextAlign,

    /// The line height of the text.
    pub line_height: f32,

    /// The text wrap of the text.
    pub wrap: TextWrap,
}

impl<T> Default for TextInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TextInput<T> {
    /// Create a new text input view.
    pub fn new() -> Self {
        Self::styled(style())
    }

    /// Create a new text input view with a style.
    pub fn styled(style: TextInputStyle) -> Self {
        Self {
            text: None,
            on_input: None,
            on_submit: None,
            placeholder: String::from("..."),
            multiline: false,
            font_size: style.font_size,
            font_family: style.font_family,
            font_weight: style.font_weight,
            font_stretch: style.font_stretch,
            font_style: style.font_style,
            color: style.color,
            placeholder_color: style.placeholder_color,
            align: style.align,
            line_height: style.line_height,
            wrap: style.wrap,
        }
    }

    /// Set the text of the input.
    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.text = Some(text.as_ref().to_string());
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

    fn set_attributes(&self, fonts: &mut Fonts, state: &mut TextInputState) {
        let attrs = TextAttributes {
            family: self.font_family.clone(),
            stretch: self.font_stretch,
            weight: self.font_weight,
            style: self.font_style,
        };
        let placeholder_attrs = TextAttributes {
            family: self.font_family.clone(),
            stretch: self.font_stretch,
            weight: self.font_weight,
            style: self.font_style,
        };
        let metrics = Metrics {
            font_size: self.font_size,
            line_height: self.line_height * self.font_size,
        };

        /* editor */
        let buffer = state.buffer_mut();
        buffer.set_wrap(&mut fonts.font_system, self.wrap.to_cosmic_text());
        buffer.set_metrics(&mut fonts.font_system, metrics);

        let mut text = state.text();

        if text.ends_with('\n') {
            text.push('\n');
        }

        state.buffer_mut().set_text(
            &mut fonts.font_system,
            &text,
            attrs.to_cosmic_text(),
            Shaping::Advanced,
        );

        /* placeholder */
        state.placeholder.set_wrap(fonts, self.wrap);
        (state.placeholder).set_metrics(fonts, self.font_size, self.line_height);
        (state.placeholder).set_text(fonts, &self.placeholder, placeholder_attrs);
    }

    fn set_attrs_list(&self, buffer: &mut Buffer) {
        let attrs = TextAttributes {
            family: self.font_family.clone(),
            stretch: self.font_stretch,
            weight: self.font_weight,
            style: self.font_style,
        };

        let attrs_list = AttrsList::new(attrs.to_cosmic_text());

        for line in buffer.lines.iter_mut() {
            line.set_attrs_list(attrs_list.clone());
        }
    }
}

#[doc(hidden)]
pub struct TextInputState {
    editor: Editor<'static>,
    placeholder: TextBuffer,
    dragging: bool,
    blink: f32,
}

impl TextInputState {
    fn buffer(&self) -> &Buffer {
        match self.editor.buffer_ref() {
            BufferRef::Owned(buffer) => buffer,
            _ => unreachable!(),
        }
    }

    fn buffer_mut(&mut self) -> &mut Buffer {
        match self.editor.buffer_ref_mut() {
            BufferRef::Owned(buffer) => buffer,
            _ => unreachable!(),
        }
    }

    fn text(&self) -> String {
        let mut text = String::new();

        for (i, line) in self.buffer().lines.iter().enumerate() {
            if i > 0 {
                text.push('\n');
            }

            text.push_str(line.text());
        }

        text
    }

    fn clear_text(&mut self) {
        self.buffer_mut().lines = vec![BufferLine::new(
            "",
            LineEnding::None,
            AttrsList::new(Attrs {
                cache_key_flags: cosmic_text::CacheKeyFlags::empty(),
                color_opt: None,
                family: cosmic_text::Family::SansSerif,
                stretch: cosmic_text::Stretch::Normal,
                style: cosmic_text::Style::Normal,
                weight: cosmic_text::Weight::NORMAL,
                metadata: 0,
                metrics_opt: None,
            }),
            Shaping::Advanced,
        )];
    }
}

fn move_key(e: &KeyPressed) -> Option<Motion> {
    match e.key {
        Key::Left if e.modifiers.ctrl => Some(Motion::LeftWord),
        Key::Right if e.modifiers.ctrl => Some(Motion::RightWord),
        Key::Left => Some(Motion::Left),
        Key::Right => Some(Motion::Right),
        Key::Up => Some(Motion::Up),
        Key::Down => Some(Motion::Down),
        _ => None,
    }
}

fn delete_key(e: &KeyPressed) -> Option<Action> {
    match e.key {
        Key::Backspace => Some(Action::Backspace),
        Key::Delete => Some(Action::Delete),
        _ => None,
    }
}

impl<T> View<T> for TextInput<T> {
    type State = TextInputState;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let editor = Editor::new(Buffer::new(
            &mut cx.fonts().font_system,
            Metrics {
                font_size: self.font_size,
                line_height: self.line_height * self.font_size,
            },
        ));

        let placeholder = TextBuffer::new(cx.fonts(), self.font_size, self.line_height);

        let mut state = TextInputState {
            editor,
            placeholder,
            dragging: false,
            blink: 0.0,
        };

        self.set_attributes(cx.fonts(), &mut state);

        state
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        if self.font_size != old.font_size || self.line_height != old.line_height {
            state.buffer_mut().set_metrics(
                &mut cx.fonts().font_system,
                Metrics {
                    font_size: self.font_size,
                    line_height: self.line_height * self.font_size,
                },
            );

            (state.placeholder).set_metrics(cx.fonts(), self.font_size, self.line_height);

            cx.layout();
        }

        if self.wrap != old.wrap {
            (state.buffer_mut()).set_wrap(&mut cx.fonts().font_system, self.wrap.to_cosmic_text());
            state.placeholder.set_wrap(cx.fonts(), self.wrap);

            cx.layout();
        }

        if self.align != old.align {
            for line in state.buffer_mut().lines.iter_mut() {
                line.set_align(Some(self.align.to_cosmic_text()));
            }

            state.placeholder.set_align(self.align);

            cx.layout();
        }

        let attrs_changed = self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style;

        if self.text != Some(state.text()) && self.text.is_some() {
            if let Some(mut text) = self.text.clone() {
                let attrs = TextAttributes {
                    family: self.font_family.clone(),
                    stretch: self.font_stretch,
                    weight: self.font_weight,
                    style: self.font_style,
                };

                if text.ends_with('\n') {
                    text.push('\n');
                }

                state.buffer_mut().set_text(
                    &mut cx.fonts().font_system,
                    &text,
                    attrs.to_cosmic_text(),
                    Shaping::Advanced,
                );

                cx.layout();
            }
        } else if attrs_changed {
            self.set_attrs_list(state.buffer_mut());

            cx.layout();
        }

        if self.placeholder != old.placeholder || attrs_changed {
            state.placeholder.set_text(
                cx.fonts(),
                &self.placeholder,
                TextAttributes {
                    family: self.font_family.clone(),
                    stretch: self.font_stretch,
                    weight: self.font_weight,
                    style: self.font_style,
                },
            );

            cx.layout();
        }
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if cx.is_hot() {
            cx.set_cursor(Some(Cursor::Text));
        } else {
            cx.set_cursor(None);
        }

        match event {
            Event::KeyPressed(e) => {
                if !cx.is_focused() {
                    return;
                }

                let mut changed = false;
                let mut submit = false;

                if !e.modifiers.ctrl && !e.modifiers.alt && !e.modifiers.meta {
                    if let Some(ref text) = e.text {
                        for c in text.chars() {
                            (state.editor).action(&mut cx.fonts().font_system, Action::Insert(c));
                        }

                        self.set_attrs_list(state.buffer_mut());

                        cx.layout();
                        state.blink = 0.0;
                        changed = true;
                    }
                }

                if let Some(action) = delete_key(e) {
                    state.editor.action(&mut cx.fonts().font_system, action);
                    cx.layout();
                    state.blink = 0.0;
                    changed = true;
                }

                if e.is_key(Key::Escape) {
                    (state.editor).action(&mut cx.fonts().font_system, Action::Escape);
                    cx.set_focused(false);
                    cx.draw();
                }

                if e.is_key(Key::Enter) && self.multiline {
                    (state.editor).action(&mut cx.fonts().font_system, Action::Enter);
                    cx.layout();
                    state.blink = 0.0;
                    changed = true;
                }

                if e.is_key(Key::Enter) && !self.multiline {
                    cx.set_focused(false);
                    submit = true;
                }

                if let Some(motion) = move_key(e) {
                    (state.editor).action(&mut cx.fonts().font_system, Action::Motion(motion));
                    cx.draw();
                    state.blink = 0.0;
                }

                if e.is_key('c') && e.modifiers.ctrl {
                    if let Some(selection) = state.editor.copy_selection() {
                        cx.clipboard().set(selection);
                    }
                }

                if e.is_key('x') && e.modifiers.ctrl {
                    if let Some(selection) = state.editor.copy_selection() {
                        cx.clipboard().set(selection);
                        cx.layout();
                    }

                    state.editor.delete_selection();
                    changed = true;
                }

                if e.is_key('v') && e.modifiers.ctrl {
                    let text = cx.clipboard().get();
                    state.editor.insert_string(&text, None);

                    cx.layout();
                    changed = true;
                }

                if !(changed || submit) {
                    return;
                }

                let text = state.text();

                if changed {
                    if let Some(ref mut on_change) = self.on_input {
                        on_change(cx, data, text.clone());
                    }
                }

                if submit {
                    if let Some(ref mut on_submit) = self.on_submit {
                        on_submit(cx, data, text);

                        if self.text.is_none() {
                            state.clear_text();
                        }

                        state.editor.set_cursor(cosmic_text::Cursor::default());
                    }
                }
            }
            Event::PointerPressed(e) => {
                if !cx.is_hot() {
                    if cx.is_focused() {
                        (state.editor).action(&mut cx.fonts().font_system, Action::Escape);
                        cx.set_focused(false);
                        cx.draw();
                    }

                    return;
                }

                cx.set_focused(true);
                cx.animate();

                state.blink = 0.0;
                state.dragging = true;

                let local = cx.local(e.position);
                state.editor.action(
                    &mut cx.fonts().font_system,
                    Action::Click {
                        x: local.x as i32,
                        y: local.y as i32,
                    },
                );
            }
            Event::PointerReleased(_) => {
                state.dragging = false;
            }
            Event::PointerMoved(e) => {
                let local = cx.local(e.position);

                if state.dragging {
                    state.editor.action(
                        &mut cx.fonts().font_system,
                        Action::Drag {
                            x: local.x as i32,
                            y: local.y as i32,
                        },
                    );

                    cx.draw();
                }
            }
            Event::Animate(dt) => {
                if cx.is_focused() {
                    cx.animate();
                    cx.draw();

                    state.blink += *dt * 10.0;
                }
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        state.buffer_mut().set_size(
            &mut cx.fonts().font_system,
            Some(space.max.width),
            Some(space.max.height),
        );
        state.placeholder.set_bounds(cx.fonts(), space.max);

        // FIXME: this is bad
        (state.editor).shape_as_needed(&mut cx.fonts().font_system, true);

        // if the text is empty, we need to layout the placeholder
        let mut size = if !state.text().is_empty() {
            Fonts::buffer_size(state.buffer())
        } else {
            state.placeholder.size()
        };

        size.height = f32::max(size.height, self.font_size);
        space.fit(size)
    }

    fn draw(&mut self, state: &mut Self::State, cx: &mut DrawCx, _data: &mut T) {
        cx.hoverable(|cx| {
            cx.trigger(cx.rect());

            // FIXME: this is bad
            (state.editor).shape_as_needed(&mut cx.fonts().font_system, true);

            let cursor = state.editor.cursor();

            /* draw the highlights and the cursor */
            // FIXME: this is bad
            for (i, run) in state.buffer().layout_runs().enumerate() {
                if !cx.is_focused() {
                    break;
                }

                if let Some((start, end)) = state.editor.selection_bounds() {
                    if let Some((start, width)) = run.highlight(start, end) {
                        let min =
                            Point::new(cx.rect().min.x + start, cx.rect().min.y + run.line_top);
                        let size = Size::new(width, self.font_size * self.line_height);

                        let highlight = Rect::min_size(min, size);

                        cx.fill_rect(highlight, self.color.fade(0.2));
                    }
                }

                if i == cursor.line {
                    let size = Size::new(1.0, self.font_size * self.line_height);

                    let min = match run.glyphs.get(cursor.index) {
                        Some(glyph) => {
                            let physical = glyph.physical((cx.rect().min.x, cx.rect().min.y), 1.0);
                            Point::new(physical.x as f32, run.line_top + physical.y as f32)
                        }
                        None if cursor.index == 0 => {
                            Point::new(cx.rect().min.x, cx.rect().min.y + run.line_top)
                        }
                        None => {
                            Point::new(cx.rect().min.x + run.line_w, cx.rect().min.y + run.line_top)
                        }
                    };

                    let cursor = Rect::min_size(min.round(), size);

                    let blink = state.blink.cos() * 0.5 + 0.5;
                    cx.fill_rect(cursor, self.color.fade(blink));
                }
            }

            /* draw the text */
            if !state.text().is_empty() {
                cx.text_raw(state.buffer(), self.color, Vector::ZERO)
            } else {
                cx.text(&state.placeholder, self.placeholder_color, Vector::ZERO)
            };
        });
    }
}
