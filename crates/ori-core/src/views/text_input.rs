use cosmic_text::{Action, Buffer, Edit, Editor, Metrics, Shaping};
use ori_macro::Build;

use crate::{
    canvas::{Background, BorderRadius, BorderWidth, Canvas, Color, Quad},
    event::{
        AnimationFrame, Code, Event, KeyPressed, PointerMoved, PointerPressed, PointerReleased,
        RequestFocus,
    },
    layout::{Point, Rect, Size, Space},
    rebuild::Rebuild,
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    theme::{style, text_input},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
    window::Cursor,
};

/// Create a new [`TextInput`].
pub fn text_input<T>() -> TextInput<T> {
    TextInput::new()
}

/// A text input.
#[derive(Rebuild, Build)]
pub struct TextInput<T> {
    /// The text.
    #[rebuild(layout)]
    #[build(ignore)]
    pub text: Option<String>,
    /// A function that returns the text to display.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_change: Option<Box<dyn FnMut(&mut EventCx, &mut T, String)>>,
    /// A function that is called when the input is submitted.
    #[build(ignore)]
    #[allow(clippy::type_complexity)]
    pub on_submit: Option<Box<dyn FnMut(&mut EventCx, &mut T, String)>>,

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
    pub align: TextAlign,
    /// The line height of the text.
    #[rebuild(layout)]
    pub line_height: f32,
    /// The text wrap of the text.
    #[rebuild(layout)]
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
        Self {
            text: None,
            on_change: None,
            on_submit: None,
            placeholder: String::from("Placeholder"),
            multiline: false,
            font_size: style(text_input::FONT_SIZE),
            font_family: style(text_input::FONT_FAMILY),
            font_weight: style(text_input::FONT_WEIGHT),
            font_stretch: style(text_input::FONT_STRETCH),
            font_style: style(text_input::FONT_STYLE),
            color: style(text_input::COLOR),
            align: style(text_input::ALIGN),
            line_height: style(text_input::LINE_HEIGHT),
            wrap: style(text_input::WRAP),
        }
    }

    /// Set the text of the input.
    pub fn text(mut self, text: impl AsRef<str>) -> Self {
        self.text = Some(text.as_ref().to_string());
        self
    }

    /// Set the callback that is called when the input changes.
    pub fn on_change(
        mut self,
        on_change: impl FnMut(&mut EventCx, &mut T, String) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(on_change));
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
            color: self.color,
        };
        let placeholder_attrs = TextAttributes {
            family: self.font_family.clone(),
            stretch: self.font_stretch,
            weight: self.font_weight,
            style: self.font_style,
            color: self.color.lighten(0.3),
        };
        let metrics = Metrics {
            font_size: self.font_size,
            line_height: self.line_height * self.font_size,
        };

        /* editor */
        let buffer = state.editor.buffer_mut();
        buffer.set_wrap(&mut fonts.font_system, self.wrap.to_cosmic_text());
        buffer.set_metrics(&mut fonts.font_system, metrics);

        let mut text = self.get_text(state);

        if text.ends_with('\n') {
            text.push('\n');
        }

        state.editor.buffer_mut().set_text(
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

    fn get_text(&self, state: &TextInputState) -> String {
        if let Some(ref text) = self.text {
            text.clone()
        } else {
            state.text()
        }
    }
}

#[doc(hidden)]
pub struct TextInputState {
    editor: Editor,
    placeholder: TextBuffer,
    dragging: bool,
    blink: f32,
}

impl TextInputState {
    fn text(&self) -> String {
        let mut text = String::new();

        for (i, line) in self.editor.buffer().lines.iter().enumerate() {
            if i > 0 {
                text.push('\n');
            }

            text.push_str(line.text());
        }

        text
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
        Rebuild::rebuild(self, cx, old);
        self.set_attributes(cx.fonts(), state);
    }

    fn event(&mut self, state: &mut Self::State, cx: &mut EventCx, data: &mut T, event: &Event) {
        if event.is::<RequestFocus>() {
            cx.set_focused(true);
            cx.request_animation_frame();
            cx.request_draw();
            event.handle();
        }

        if let Some(e) = event.get::<KeyPressed>() {
            if !cx.is_focused() {
                return;
            }

            let editor = &mut state.editor;

            let mut changed = false;
            let mut submit = false;

            if let Some(ref text) = e.text {
                for c in text.chars() {
                    editor.action(&mut cx.fonts().font_system, Action::Insert(c));
                }

                cx.request_layout();
                state.blink = 0.0;
                changed = true;
            }

            if e.is(Code::Backspace) {
                editor.action(&mut cx.fonts().font_system, Action::Backspace);
                cx.request_layout();
                state.blink = 0.0;
                changed = true;
            }

            if e.is(Code::Delete) {
                editor.action(&mut cx.fonts().font_system, Action::Delete);
                cx.request_layout();
                state.blink = 0.0;
                changed = true;
            }

            if e.is(Code::Escape) {
                editor.action(&mut cx.fonts().font_system, Action::Escape);
            }

            if e.is(Code::Enter) && self.multiline {
                editor.action(&mut cx.fonts().font_system, Action::Enter);
                cx.request_layout();
                state.blink = 0.0;
                changed = true;
            }

            if e.is(Code::Enter) && !self.multiline {
                cx.set_focused(false);
                submit = true;
            }

            if e.is(Code::Left) && !e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::Left);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::Right) && !e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::Right);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::Up) && !e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::Up);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::Down) && !e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::Down);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::Left) && e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::LeftWord);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::Right) && e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::RightWord);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::Home) && !e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::Home);
                cx.request_layout();
                state.blink = 0.0;
            }

            if e.is(Code::End) && !e.modifiers.ctrl {
                editor.action(&mut cx.fonts().font_system, Action::End);
                cx.request_layout();
                state.blink = 0.0;
            }

            if changed || submit {
                let text = state.text();

                if changed {
                    if let Some(ref mut on_change) = self.on_change {
                        on_change(cx, data, text.clone());
                        cx.request_rebuild();
                    }
                }

                if submit {
                    if let Some(ref mut on_submit) = self.on_submit {
                        on_submit(cx, data, text);
                        cx.request_rebuild();

                        if self.text.is_none() {
                            state.editor.buffer_mut().lines.clear();
                            state.editor.set_cursor(cosmic_text::Cursor::default());
                        }
                    }
                }
            }
        }

        if let Some(e) = event.get::<PointerPressed>() {
            if !cx.is_hot() {
                if cx.is_focused() {
                    cx.set_focused(false);
                    cx.request_draw();
                }

                return;
            }

            cx.set_focused(true);
            cx.request_animation_frame();

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

        if event.is::<PointerReleased>() {
            state.dragging = false;
        }

        if let Some(e) = event.get::<PointerMoved>() {
            let local = cx.local(e.position);

            if state.dragging {
                state.editor.action(
                    &mut cx.fonts().font_system,
                    Action::Drag {
                        x: local.x as i32,
                        y: local.y as i32,
                    },
                );

                cx.request_draw();
            }

            if cx.is_hot() {
                cx.set_cursor(Cursor::Text);
            } else {
                cx.set_cursor(None);
            }
        }

        if let Some(AnimationFrame(dt)) = event.get() {
            if cx.is_focused() {
                cx.request_animation_frame();
                cx.request_draw();

                state.blink += *dt * 10.0;
            }
        }
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        state.editor.buffer_mut().set_size(
            &mut cx.fonts().font_system,
            space.max.width,
            space.max.height,
        );
        state.placeholder.set_bounds(cx.fonts(), space.max);

        // FIXME: this is bad
        state.editor.shape_as_needed(&mut cx.fonts().font_system);

        // if the text is empty, we need to layout the placeholder
        let mut size = if !self.get_text(state).is_empty() {
            Fonts::buffer_size(state.editor.buffer())
        } else {
            state.placeholder.size()
        };

        size.height = f32::max(size.height, self.font_size);
        space.fit(size)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        canvas.trigger(cx.id(), cx.rect());

        // FIXME: this is bad
        state.editor.shape_as_needed(&mut cx.fonts().font_system);

        let cursor = state.editor.cursor();

        /* draw the highlights and the cursor */
        for (i, run) in state.editor.buffer().layout_runs().enumerate() {
            if let Some(select) = state.editor.select_opt() {
                let start = cursor.min(select);
                let end = cursor.max(select);

                if let Some((start, width)) = run.highlight(start, end) {
                    let min = Point::new(cx.rect().min.x + start, cx.rect().min.y + run.line_top);
                    let size = Size::new(width, self.font_size * self.line_height);

                    let highlight = Rect::min_size(min, size);

                    canvas.draw_pixel_perfect(Quad {
                        rect: highlight,
                        background: Background::new(Color::hex("#25d0ea80")),
                        border_radius: BorderRadius::ZERO,
                        border_width: BorderWidth::ZERO,
                        border_color: Color::TRANSPARENT,
                    });
                }
            }

            if i == cursor.line && cx.is_focused() {
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

                let cursor = Rect::min_size(min, size);

                let blink = state.blink.cos() * 0.5 + 0.5;
                canvas.draw_pixel_perfect(Quad {
                    rect: cursor,
                    background: Background::new(self.color * blink),
                    border_radius: BorderRadius::ZERO,
                    border_width: BorderWidth::ZERO,
                    border_color: Color::TRANSPARENT,
                });
            }
        }

        /* draw the text */
        let mesh = if !self.get_text(state).is_empty() {
            cx.rasterize_text_raw(state.editor.buffer(), cx.rect())
        } else {
            cx.rasterize_text(&state.placeholder, cx.rect())
        };

        canvas.draw_pixel_perfect(mesh);
    }
}
