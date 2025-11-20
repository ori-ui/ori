use ike::{
    BuildCx, Color, FontStretch, FontStyle, FontWeight, Paragraph, TextAlign, TextStyle, TextWrap,
};
use ori::ProviderContext;

use crate::{Context, Palette, views::TextTheme};

pub fn label(text: impl ToString) -> Label {
    Label {
        text:         text.to_string(),
        font_size:    None,
        font_family:  None,
        font_weight:  None,
        font_stretch: None,
        font_style:   None,
        line_height:  None,
        align:        None,
        wrap:         None,
        color:        None,
    }
}

pub struct Label {
    text:         String,
    font_size:    Option<f32>,
    font_family:  Option<String>,
    font_weight:  Option<FontWeight>,
    font_stretch: Option<FontStretch>,
    font_style:   Option<FontStyle>,
    line_height:  Option<f32>,
    align:        Option<TextAlign>,
    wrap:         Option<TextWrap>,
    color:        Option<Color>,
}

impl Label {
    fn build_paragraph(&self, palette: &Palette, theme: &TextTheme) -> Paragraph {
        let style = TextStyle {
            font_size:    self.font_size.unwrap_or(theme.font_size),
            font_weight:  self.font_weight.unwrap_or(theme.font_weight),
            font_stretch: self.font_stretch.unwrap_or(theme.font_stretch),
            font_style:   self.font_style.unwrap_or(theme.font_style),

            font_family: self
                .font_family
                .clone()
                .unwrap_or_else(|| theme.font_family.clone().into_owned()),

            color: self
                .color
                .unwrap_or_else(|| theme.color.unwrap_or(palette.contrast)),
        };

        let mut paragraph = Paragraph::new(
            self.line_height.unwrap_or(theme.line_height),
            self.align.unwrap_or(theme.align),
            self.wrap.unwrap_or(theme.wrap),
        );

        paragraph.push(&self.text, style);
        paragraph
    }
}

impl ori::ViewMarker for Label {}
impl<T> ori::View<Context, T> for Label {
    type Element = ike::WidgetId<ike::widgets::Text>;
    type State = ();

    fn build(&mut self, cx: &mut Context, _data: &mut T) -> (Self::Element, Self::State) {
        let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
        let theme = cx.get_context::<TextTheme>().cloned().unwrap_or_default();

        let paragraph = self.build_paragraph(&palette, &theme);
        let element = ike::widgets::Text::new(cx, paragraph);

        (element, ())
    }

    fn rebuild(
        &mut self,
        element: &mut Self::Element,
        _state: &mut Self::State,
        cx: &mut Context,
        _data: &mut T,
        old: &mut Self,
    ) {
        if self.text != old.text
            || self.font_size != old.font_size
            || self.font_family != old.font_family
            || self.font_weight != old.font_weight
            || self.font_stretch != old.font_stretch
            || self.font_style != old.font_style
            || self.line_height != old.line_height
            || self.align != old.align
            || self.wrap != old.wrap
            || self.color != old.color
        {
            let palette = cx.get_context::<Palette>().cloned().unwrap_or_default();
            let theme = cx.get_context::<TextTheme>().cloned().unwrap_or_default();

            let paragraph = self.build_paragraph(&palette, &theme);
            ike::widgets::Text::set_paragraph(cx, *element, paragraph);
        }
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
