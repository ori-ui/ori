use crate::{
    builtin::text, style, BuildCx, Canvas, Color, DrawCx, Event, EventCx, FontFamily, FontStretch,
    FontStyle, FontWeight, Glyphs, LayoutCx, Rebuild, RebuildCx, Size, Space, TextAlign,
    TextSection, TextWrap, View,
};

pub fn text(text: impl ToString) -> Text {
    Text::new(text)
}

/// A text.
#[derive(Rebuild)]
pub struct Text {
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

        state.as_ref().map(|g| g.size()).unwrap_or(space.max)
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
                canvas.draw(mesh);
            }
        }
    }
}
