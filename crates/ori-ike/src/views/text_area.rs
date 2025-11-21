use ike::{
    BuildCx, Color, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign, TextStyle, TextWrap,
};

use crate::Context;

pub fn text_area() -> TextArea {
    TextArea {}
}

pub struct TextArea {}

impl ori::ViewMarker for TextArea {}
impl<T> ori::View<Context, T> for TextArea {
    type Element = ike::WidgetId<ike::widgets::TextArea>;
    type State = ();

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let mut paragraph = Paragraph::new(1.0, TextAlign::Start, TextWrap::Word);
        paragraph.push(
            "test text",
            TextStyle {
                font_size:    16.0,
                font_family:  String::from("Ubuntu Light"),
                font_weight:  FontWeight::NORMAL,
                font_stretch: FontStretch::Normal,
                font_style:   FontStyle::Normal,
                color:        Color::WHITE,
            },
        );

        let element = ike::widgets::TextArea::new(cx, paragraph);

        (element, ())
    }

    fn rebuild(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _old: &mut Self,
    ) {
    }

    fn teardown(
        &mut self,
        element: Self::Element,
        _state: Self::State,
        cx: &mut Context,
        _data: &mut T,
    ) {
        cx.remove(element);
    }

    fn event(
        &mut self,
        _element: &mut Self::Element,
        _state: &mut Self::State,
        _cx: &mut Context,
        _data: &mut T,
        _event: &mut ori::Event,
    ) -> ori::Action {
        ori::Action::new()
    }
}
