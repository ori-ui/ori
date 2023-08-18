use crate::{
    builtin::text, style, BuildCx, Canvas, Color, DrawCx, Event, EventCx, FontFamily, FontStretch,
    FontStyle, FontWeight, Glyphs, LayoutCx, Rebuild, RebuildCx, Size, Space, TextAlign,
    TextSection, TextWrap, View,
};

/// Create a new [`Text`].
pub fn text(text: impl ToString) -> Text {
    Text::new(text)
}

/// A view that displays text.
#[derive(Rebuild)]
pub struct Text {
    /// The text.
    #[rebuild(layout)]
    pub text: String,
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

impl Text {
    /// Creates a new text.
    pub fn new(text: impl ToString) -> Text {
        Text {
            text: text.to_string(),
            font_size: style(text::FONT_SIZE),
            font_family: style(text::FONT_FAMILY),
            font_weight: style(text::FONT_WEIGHT),
            font_stretch: style(text::FONT_STRETCH),
            font_style: style(text::FONT_STYLE),
            color: style(text::COLOR),
            v_align: style(text::V_ALIGN),
            h_align: style(text::H_ALIGN),
            line_height: style(text::LINE_HEIGHT),
            wrap: style(text::WRAP),
        }
    }

    /// Sets the font size.
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    /// Sets the font family.
    pub fn font_family(mut self, font_family: impl Into<FontFamily>) -> Self {
        self.font_family = font_family.into();
        self
    }

    /// Sets the font weight.
    pub fn font_weight(mut self, font_weight: impl Into<FontWeight>) -> Self {
        self.font_weight = font_weight.into();
        self
    }

    /// Sets the font stretch.
    pub fn font_stretch(mut self, font_stretch: impl Into<FontStretch>) -> Self {
        self.font_stretch = font_stretch.into();
        self
    }

    /// Sets the font style.
    pub fn font_style(mut self, font_style: impl Into<FontStyle>) -> Self {
        self.font_style = font_style.into();
        self
    }

    /// Sets the color.
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Sets the vertical alignment.
    pub fn v_align(mut self, v_align: impl Into<TextAlign>) -> Self {
        self.v_align = v_align.into();
        self
    }

    /// Sets the horizontal alignment.
    pub fn h_align(mut self, h_align: impl Into<TextAlign>) -> Self {
        self.h_align = h_align.into();
        self
    }

    /// Sets the line height.
    pub fn line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    /// Sets the text wrap.
    pub fn wrap(mut self, wrap: impl Into<TextWrap>) -> Self {
        self.wrap = wrap.into();
        self
    }
}

impl<T> View<T> for Text {
    type State = Option<Glyphs>;

    fn build(&mut self, _cx: &mut BuildCx, _data: &mut T) -> Self::State {
        None
    }

    fn rebuild(&mut self, _state: &mut Self::State, cx: &mut RebuildCx, _data: &mut T, old: &Self) {
        Rebuild::rebuild(self, cx, old);
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
        let section = TextSection {
            text: &self.text,
            font_size: self.font_size,
            font_family: self.font_family.clone(),
            font_weight: self.font_weight,
            font_stretch: self.font_stretch,
            font_style: self.font_style,
            color: self.color,
            v_align: self.v_align,
            h_align: self.h_align,
            line_height: self.line_height,
            wrap: self.wrap,
            bounds: space.max,
        };

        *state = cx.layout_text(&section);

        let text_size = state.as_ref().map(|g| g.size()).unwrap_or(space.max);
        space.fit(text_size)
    }

    fn draw(
        &mut self,
        state: &mut Self::State,
        cx: &mut DrawCx,
        _data: &mut T,
        canvas: &mut Canvas,
    ) {
        if let Some(glyphs) = state {
            if let Some(mesh) = cx.text_mesh(glyphs, cx.rect()) {
                canvas.draw_pixel_perfect(mesh);
            }
        }
    }
}
