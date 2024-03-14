use std::fmt::{self, Write};

use ori_macro::Build;
use smol_str::SmolStr;

use crate::{
    canvas::{Canvas, Color, Mesh},
    event::Event,
    layout::{Size, Space},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    theme::{style, style_or, Key, Palette},
    view::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx, View},
};

use smol_str;

pub use crate::format_text as text;

/// Key for the default font size.
pub const FONT_SIZE: Key<f32> = Key::new("font-size");

/// Key for the default font family.
pub const FONT_FAMILY: Key<FontFamily> = Key::new("font-family");

/// Key for the default font weight.
pub const FONT_WEIGHT: Key<FontWeight> = Key::new("font-weight");

/// Key for the default font stretch.
pub const FONT_STRETCH: Key<FontStretch> = Key::new("font-stretch");

/// Key for the default font style.
pub const FONT_STYLE: Key<FontStyle> = Key::new("font-style");

/// Key for the default text color.
pub const TEXT_ALIGN: Key<TextAlign> = Key::new("text-align");

/// Key for the default text color.
pub const LINE_HEIGHT: Key<f32> = Key::new("line-height");

/// Create a formatted [`Text`].
///
/// This macro is slightly more efficient than using [`format!`] and [`Text::new`].
#[macro_export]
macro_rules! format_text {
    ($($tt:tt)*) => {
        $crate::views::Text::from(::std::format_args!($($tt)*))
    };
}

/// Create a new [`Text`].
pub fn text(text: impl Into<SmolStr>) -> Text {
    Text::new(text)
}

/// A view that displays text.
#[derive(Build)]
pub struct Text {
    /// The text.
    pub text: SmolStr,
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
    /// The horizontal alignment of the text.
    pub align: TextAlign,
    /// The line height of the text.
    pub line_height: f32,
    /// The text wrap of the text.
    pub wrap: TextWrap,
}

impl Text {
    /// Create a new text.
    pub fn new(text: impl Into<SmolStr>) -> Text {
        Text {
            text: text.into(),
            font_size: style_or(FONT_SIZE, 16.0),
            font_family: style_or(FONT_FAMILY, FontFamily::SansSerif),
            font_weight: style_or(FONT_WEIGHT, FontWeight::NORMAL),
            font_stretch: style_or(FONT_STRETCH, FontStretch::Normal),
            font_style: style_or(FONT_STYLE, FontStyle::Normal),
            color: style(Palette::TEXT),
            align: style_or(TEXT_ALIGN, TextAlign::Start),
            line_height: style_or(LINE_HEIGHT, 1.2),
            wrap: TextWrap::Word,
        }
    }

    fn set_attributes(&self, fonts: &mut Fonts, buffer: &mut TextBuffer) {
        buffer.set_wrap(fonts, self.wrap);
        buffer.set_align(self.align);
        buffer.set_text(
            fonts,
            &self.text,
            TextAttributes {
                family: self.font_family.clone(),
                stretch: self.font_stretch,
                weight: self.font_weight,
                style: self.font_style,
                color: self.color,
            },
        );
    }
}

#[doc(hidden)]
pub struct TextState {
    buffer: TextBuffer,
    mesh: Option<Mesh>,
}

impl<T> View<T> for Text {
    type State = TextState;

    fn build(&mut self, cx: &mut BuildCx, _data: &mut T) -> Self::State {
        let mut buffer = TextBuffer::new(cx.fonts(), self.font_size, self.line_height);
        self.set_attributes(cx.fonts(), &mut buffer);

        TextState { buffer, mesh: None }
    }

    fn rebuild(&mut self, state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        if self.wrap != old.wrap {
            state.buffer.set_wrap(cx.fonts(), self.wrap);

            state.mesh = Some(cx.rasterize_text(&state.buffer));
            cx.request_draw();
        }

        if self.align != old.align {
            state.buffer.set_align(self.align);

            state.mesh = Some(cx.rasterize_text(&state.buffer));
            cx.request_draw();
        }

        if self.text != old.text
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
            || self.color != old.color
        {
            state.buffer.set_text(
                cx.fonts(),
                &self.text,
                TextAttributes {
                    family: self.font_family.clone(),
                    stretch: self.font_stretch,
                    weight: self.font_weight,
                    style: self.font_style,
                    color: self.color,
                },
            );

            state.mesh = Some(cx.rasterize_text(&state.buffer));
            cx.request_layout();
        }
    }

    fn event(
        &mut self,
        _state: &mut Self::State,
        _cx: &mut EventCx,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(
        &mut self,
        state: &mut Self::State,
        cx: &mut LayoutCx,
        _data: &mut T,
        space: Space,
    ) -> Size {
        if state.mesh.is_none() || state.buffer.bounds() != space.max {
            state.buffer.set_bounds(cx.fonts(), space.max);
            state.mesh = Some(cx.rasterize_text(&state.buffer));
        }

        space.fit(state.buffer.size())
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        let offset = cx.rect().center() - state.buffer.rect().center();

        if let Some(ref mesh) = state.mesh {
            canvas.translate(offset);
            canvas.draw_pixel_perfect(mesh.clone());
        }
    }
}

impl From<fmt::Arguments<'_>> for Text {
    fn from(args: fmt::Arguments<'_>) -> Text {
        let mut w = smol_str::Writer::new();
        let _ = w.write_fmt(args);
        Text::new(w)
    }
}
