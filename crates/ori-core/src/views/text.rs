use std::fmt::{self, Write};

use ori_macro::{example, Build};
use smol_str::SmolStr;

use crate::{
    canvas::{Canvas, Color, Mesh},
    context::{BuildCx, DrawCx, EventCx, LayoutCx, RebuildCx},
    event::Event,
    layout::{Size, Space},
    style::{style, Style, Styles},
    text::{
        FontFamily, FontStretch, FontStyle, FontWeight, Fonts, TextAlign, TextAttributes,
        TextBuffer, TextWrap,
    },
    view::View,
};

use smol_str;

pub use crate::format_text as text;

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

/// The style of a text.
#[derive(Clone, Debug)]
pub struct TextStyle {
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
    /// The horizontal alignment of the text.
    pub align: TextAlign,
    /// The line height of the text.
    pub line_height: f32,
    /// The text wrap of the text.
    pub wrap: TextWrap,
}

impl Style for TextStyle {
    fn style(style: &Styles) -> Self {
        Self {
            font_size: 16.0,
            font_family: FontFamily::SansSerif,
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: style.palette().text,
            align: TextAlign::Start,
            line_height: 1.3,
            wrap: TextWrap::Word,
        }
    }
}

/// A view that displays text.
///
/// Can be styled using the [`TextStyle`].
#[example(name = "text", width = 400, height = 300)]
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
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self::styled(text, style())
    }

    /// Create a new text with a style.
    pub fn styled(text: impl Into<SmolStr>, style: TextStyle) -> Self {
        Text {
            text: text.into(),
            font_size: style.font_size,
            font_family: style.font_family,
            font_weight: style.font_weight,
            font_stretch: style.font_stretch,
            font_style: style.font_style,
            color: style.color,
            align: style.align,
            line_height: style.line_height,
            wrap: style.wrap,
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

            state.mesh.take();
            cx.request_draw();
        }

        if self.align != old.align {
            state.buffer.set_align(self.align);

            state.mesh.take();
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

            state.mesh.take();
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
            cx.prepare_text(&state.buffer);
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

        if state.mesh.is_none() {
            state.mesh = Some(cx.rasterize_text(&state.buffer));
        }

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
