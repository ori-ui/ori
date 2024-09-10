use cosmic_text::{
    Action, Attrs, AttrsList, Buffer, BufferLine, BufferRef, Edit, Editor, LineEnding, Metrics,
    Motion, Shaping,
};
use ori_macro::{example, Build, Styled};

use crate::{
    canvas::Color,
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::{Event, Ime, Key, KeyPressed},
    layout::{Point, Rect, Size, Space, Vector},
    style::{Styled, Theme},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
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

    /// The font size of the text.
    #[styled(default = 16.0)]
    pub font_size: Styled<f32>,

    /// The font family of the text.
    #[styled(default)]
    pub font_family: Styled<FontFamily>,

    /// The font weight of the text.
    #[styled(default)]
    pub font_weight: Styled<FontWeight>,

    /// The font stretch of the text.
    #[styled(default)]
    pub font_stretch: Styled<FontStretch>,

    /// The font.into of the text.
    #[styled(default)]
    pub font_style: Styled<FontStyle>,

    /// The color of the text.
    #[styled(default -> Theme::CONTRAST or Color::BLACK)]
    pub color: Styled<Color>,

    /// The color of the placeholder text.
    #[styled(default -> Theme::CONTRAST_LOW or Color::grayscale(0.9))]
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

    fn set_attributes(&self, fonts: &mut Fonts, state: &mut TextInputState) {
        let attrs = TextAttributes {
            family: state.style.font_family.clone(),
            stretch: state.style.font_stretch,
            weight: state.style.font_weight,
            style: state.style.font_style,
        };
        let placeholder_attrs = TextAttributes {
            family: state.style.font_family.clone(),
            stretch: state.style.font_stretch,
            weight: state.style.font_weight,
            style: state.style.font_style,
        };
        let metrics = Metrics {
            font_size: state.style.font_size,
            line_height: state.style.line_height * state.style.font_size,
        };

        /* editor */
        let wrap = state.style.wrap.to_cosmic_text();
        let buffer = state.buffer_mut();
        buffer.set_wrap(&mut fonts.font_system, wrap);
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
        state.placeholder.set_wrap(fonts, state.style.wrap);
        (state.placeholder).set_metrics(fonts, state.style.font_size, state.style.line_height);
        (state.placeholder).set_text(fonts, &self.placeholder, placeholder_attrs);
    }

    fn set_attrs_list(&self, buffer: &mut Buffer, style: &TextInputStyle) {
        let attrs = TextAttributes {
            family: style.font_family.clone(),
            stretch: style.font_stretch,
            weight: style.font_weight,
            style: style.font_style,
        };

        let attrs_list = AttrsList::new(attrs.to_cosmic_text());

        for line in buffer.lines.iter_mut() {
            line.set_attrs_list(attrs_list.clone());
        }
    }
}

#[doc(hidden)]
pub struct TextInputState {
    style: TextInputStyle,
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
        let style = TextInputStyle::styled(self, cx.styles());

        let editor = Editor::new(Buffer::new(
            &mut cx.fonts().font_system,
            Metrics {
                font_size: style.font_size,
                line_height: style.line_height * style.font_size,
            },
        ));

        let placeholder = TextBuffer::new(cx.fonts(), style.font_size, style.line_height);

        let mut state = TextInputState {
            style,
            editor,
            placeholder,
            dragging: false,
            blink: 0.0,
        };

        if let Some(ref text) = self.text {
            let attrs = TextAttributes {
                family: state.style.font_family.clone(),
                stretch: state.style.font_stretch,
                weight: state.style.font_weight,
                style: state.style.font_style,
            };

            state.buffer_mut().set_text(
                &mut cx.fonts().font_system,
                text,
                attrs.to_cosmic_text(),
                Shaping::Advanced,
            );
        }

        self.set_attributes(cx.fonts(), &mut state);

        state
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        let style = TextInputStyle::styled(self, cx.styles());

        if style.font_size != state.style.font_size || style.line_height != state.style.line_height
        {
            let metrics = Metrics {
                font_size: style.font_size,
                line_height: style.line_height * style.font_size,
            };

            (state.buffer_mut()).set_metrics(&mut cx.fonts().font_system, metrics);
            (state.placeholder).set_metrics(cx.fonts(), style.font_size, style.line_height);

            cx.layout();
        }

        if style.wrap != state.style.wrap {
            let wrap = style.wrap.to_cosmic_text();
            (state.buffer_mut()).set_wrap(&mut cx.fonts().font_system, wrap);
            state.placeholder.set_wrap(cx.fonts(), style.wrap);

            cx.layout();
        }

        if style.align != state.style.align {
            let align = style.align.to_cosmic_text();

            for line in state.buffer_mut().lines.iter_mut() {
                line.set_align(Some(align));
            }

            state.placeholder.set_align(style.align);

            cx.layout();
        }

        let attrs_changed = style.font_family != state.style.font_family
            || style.font_weight != state.style.font_weight
            || style.font_stretch != state.style.font_stretch
            || style.font_style != state.style.font_style;

        if self.text != Some(state.text()) && self.text.is_some() {
            if let Some(mut text) = self.text.clone() {
                let attrs = TextAttributes {
                    family: style.font_family.clone(),
                    stretch: style.font_stretch,
                    weight: style.font_weight,
                    style: style.font_style,
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
            let buffer = match state.editor.buffer_ref_mut() {
                BufferRef::Owned(buffer) => buffer,
                _ => unreachable!(),
            };

            self.set_attrs_list(buffer, &style);

            cx.layout();
        }

        if self.placeholder != old.placeholder || attrs_changed {
            state.placeholder.set_text(
                cx.fonts(),
                &self.placeholder,
                TextAttributes {
                    family: style.font_family.clone(),
                    stretch: style.font_stretch,
                    weight: style.font_weight,
                    style: style.font_style,
                },
            );

            cx.layout();
        }

        state.style = style;
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

                        let buffer = match state.editor.buffer_ref_mut() {
                            BufferRef::Owned(buffer) => buffer,
                            _ => unreachable!(),
                        };

                        self.set_attrs_list(buffer, &state.style);

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
                        cx.set_ime(None);
                        cx.draw();
                    }

                    return;
                }

                cx.set_focused(true);
                cx.set_ime(Some(Ime::default()));
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

        size.height = f32::max(size.height, state.style.font_size);
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
                        let size =
                            Size::new(width, state.style.font_size * state.style.line_height);

                        let highlight = Rect::min_size(min, size);

                        cx.fill_rect(highlight, state.style.color.fade(0.2));
                    }
                }

                if i == cursor.line {
                    let size = Size::new(1.0, state.style.font_size * state.style.line_height);

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
                    cx.fill_rect(cursor, state.style.color.fade(blink));
                }
            }

            /* draw the text */
            if !state.text().is_empty() {
                cx.text_raw(state.buffer(), state.style.color, Vector::ZERO)
            } else {
                cx.text(
                    &state.placeholder,
                    state.style.placeholder_color,
                    Vector::ZERO,
                )
            };
        });
    }
}
