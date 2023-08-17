use crate::{
    Canvas, Color, DrawCx, Event, EventCx, FontFamily, FontStretch, FontStyle, FontWeight, Glyphs,
    LayoutCx, RebuildCx, Size, Space, TextAlign, TextSection, TextWrap, View,
};

pub fn text(text: impl ToString) -> Text {
    Text {
        text: text.to_string(),
    }
}

pub struct Text {
    pub text: String,
}

impl<T> View<T> for Text {
    type State = Option<Glyphs>;

    fn build(&self) -> Self::State {
        None
    }

    fn rebuild(&mut self, cx: &mut RebuildCx, old: &Self, _state: &mut Self::State) {
        if self.text != old.text {
            cx.request_layout();
        }
    }

    fn event(
        &mut self,
        _cx: &mut EventCx,
        _state: &mut Self::State,
        _data: &mut T,
        _event: &Event,
    ) {
    }

    fn layout(&mut self, cx: &mut LayoutCx, state: &mut Self::State, space: Space) -> Size {
        let section = TextSection {
            text: &self.text,
            font_size: 16.0,
            font_family: FontFamily::SansSerif,
            font_weight: FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style: FontStyle::Normal,
            color: Color::rgb(0.0, 0.0, 0.0),
            v_align: TextAlign::Top,
            h_align: TextAlign::Left,
            line_height: 1.0,
            wrap: TextWrap::Word,
            bounds: space.max,
        };

        *state = cx.layout_text(&section);

        state.as_ref().map(|g| g.size()).unwrap_or(space.max)
    }

    fn draw(&mut self, cx: &mut DrawCx, state: &mut Self::State, canvas: &mut Canvas) {
        if let Some(glyphs) = state {
            if let Some(mesh) = cx.text_mesh(glyphs, cx.rect()) {
                canvas.draw(mesh);
            }
        }
    }
}
